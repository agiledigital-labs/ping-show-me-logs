//! A multi-room chat server.

use crate::errors::ShowMeErrors;
use crate::ping_logs::logs::{GenericLog, tail_logs};
use crate::trees::journeys::AuthenticationTreeList;
use crate::{AppMutState, ConnId, JourneyId, Msg, TransactionId, add_to_rolling_buffer};
use actix_web::web::Data;
use rand::Rng as _;
use reqwest::Client;
use std::time::Duration;
use std::{
  collections::{HashMap, HashSet},
  io,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};
use tantivy::schema::{STORED, Schema, TEXT};
use tantivy::{Index, IndexReader, doc};
use tempfile::TempDir;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;

/// A command received by the [`LogsServer`].
enum Command {
  Connect {
    conn_tx: mpsc::UnboundedSender<Msg>,
    res_tx: oneshot::Sender<ConnId>,
  },

  Disconnect {
    conn: ConnId,
  },

  List {
    res_tx: oneshot::Sender<Vec<JourneyId>>,
  },

  Join {
    conn: ConnId,
    room: JourneyId,
    res_tx: oneshot::Sender<()>,
  },

  NewId {
    transaction_id: TransactionId,
    journey_id: JourneyId,
    res_tx: oneshot::Sender<()>,
  },
  Message {
    msg: Msg,
    conn: ConnId,
    res_tx: oneshot::Sender<()>,
  },
}

/// A multi-room chat server.
///
/// Contains the logic of how connections chat with each other plus room management.
///
/// Call and spawn [`run`](Self::run) to start processing commands.
#[derive(Debug)]
pub struct LogsServer {
  sessions: HashMap<ConnId, mpsc::UnboundedSender<Msg>>,
  journey_watch: HashMap<JourneyId, HashSet<ConnId>>,
  user_count: Arc<AtomicUsize>,
  cmd_rx: mpsc::UnboundedReceiver<Command>,
  authentication_tree_list: AuthenticationTreeList,
}

impl LogsServer {
  pub fn new(authentication_tree_list: AuthenticationTreeList) -> (Self, LogsServerHandle) {
    let mut rooms = HashMap::with_capacity(4);

    rooms.insert("main".to_owned(), HashSet::new());

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

    (
      Self {
        sessions: HashMap::new(),
        journey_watch: rooms,
        user_count: Arc::new(AtomicUsize::new(0)),
        cmd_rx,
        authentication_tree_list,
      },
      LogsServerHandle { cmd_tx },
    )
  }

  /// Send message to users in a room.
  ///
  /// `skip` is used to prevent messages triggered by a connection also being received by it.
  async fn send_system_message(&self, room: &str, skip: ConnId, msg: impl Into<Msg>) {
    if let Some(sessions) = self.journey_watch.get(room) {
      let msg = msg.into();

      for conn_id in sessions {
        if *conn_id != skip {
          if let Some(tx) = self.sessions.get(conn_id) {
            // errors if client disconnected abruptly and hasn't been timed-out yet
            let _ = tx.send(msg.clone());
          }
        }
      }
    }
  }

  async fn send_new_transaction_id(&self, journey_id: JourneyId, transaction_id: TransactionId) {
    let msg: Msg = (transaction_id, journey_id.clone()).into();
    println!("{}", journey_id);
    if let Some(sessions) = self.journey_watch.get(&journey_id) {
      for conn_id in sessions {
        if let Some(tx) = self.sessions.get(conn_id) {
          let _ = tx.send(msg.clone());
        }
      }
    }
  }

  /// Send message to all other users in current room.
  ///
  /// `conn` is used to find current room and prevent messages sent by a connection also being
  /// received by it.
  async fn send_message(&self, conn: ConnId, msg: impl Into<Msg>) {
    if let Some(room) = self
      .journey_watch
      .iter()
      .find_map(|(room, participants)| participants.contains(&conn).then_some(room))
    {
      self.send_system_message(room, conn, msg).await;
    };
  }

  /// Register new session and assign unique ID to this session
  async fn connect(&mut self, tx: mpsc::UnboundedSender<Msg>) -> ConnId {
    log::info!("Someone joined");

    // notify all users in same room
    self.send_system_message("main", 0, "Someone joined").await;

    // register session with random connection ID
    let id = rand::rng().random::<ConnId>();
    self.sessions.insert(id, tx);

    // auto join session to main room
    self
      .journey_watch
      .entry("main".to_owned())
      .or_default()
      .insert(id);

    let count = self.user_count.fetch_add(1, Ordering::SeqCst);
    self
      .send_system_message("main", 0, format!("Total visitors {count}"))
      .await;

    // send id back
    id
  }

  /// Unregister connection from room map and broadcast disconnection message.
  async fn disconnect(&mut self, conn_id: ConnId) {
    println!("Someone disconnected");

    let mut rooms: Vec<JourneyId> = Vec::new();

    // remove sender
    if self.sessions.remove(&conn_id).is_some() {
      // remove session from all rooms
      for (name, sessions) in &mut self.journey_watch {
        if sessions.remove(&conn_id) {
          rooms.push(name.to_owned());
        }
      }
    }

    // send message to other users
    for room in rooms {
      self
        .send_system_message(&room, 0, "Someone disconnected")
        .await;
    }
  }

  /// Returns list of created room names.
  fn list_rooms(&mut self) -> Vec<JourneyId> {
    self.authentication_tree_list.get_tree_list()
    // self.journey_watch.keys().cloned().collect()
  }

  /// Join room, send disconnect message to old room send join message to new room.
  async fn join_room(&mut self, conn_id: ConnId, room: JourneyId) {
    if room.eq("all") {
      println!("all rooms");
      for room in self.authentication_tree_list.get_tree_list() {
        self
          .journey_watch
          .entry(room.clone())
          .or_default()
          .insert(conn_id);
      }
      return;
    };

    self
      .journey_watch
      .entry(room.clone())
      .or_default()
      .insert(conn_id);

    self
      .send_system_message(&room, conn_id, "Someone connected")
      .await;
  }

  pub async fn run(mut self) -> io::Result<()> {
    while let Some(cmd) = self.cmd_rx.recv().await {
      match cmd {
        Command::Connect { conn_tx, res_tx } => {
          let conn_id = self.connect(conn_tx).await;
          let _ = res_tx.send(conn_id);
        }

        Command::NewId {
          journey_id,
          transaction_id,
          res_tx,
        } => {
          self
            .send_new_transaction_id(journey_id, transaction_id)
            .await;
          let _ = res_tx.send(());
        }

        Command::Disconnect { conn } => {
          self.disconnect(conn).await;
        }

        Command::List { res_tx } => {
          let _ = res_tx.send(self.list_rooms());
        }

        Command::Join { conn, room, res_tx } => {
          self.join_room(conn, room).await;
          let _ = res_tx.send(());
        }

        Command::Message { conn, msg, res_tx } => {
          self.send_message(conn, msg).await;
          let _ = res_tx.send(());
        }
      }
    }

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct LogsServerHandle {
  cmd_tx: mpsc::UnboundedSender<Command>,
}

impl LogsServerHandle {
  /// Register client message sender and obtain connection ID.
  pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<Msg>) -> ConnId {
    let (res_tx, res_rx) = oneshot::channel();

    // unwrap: chat server should not have been dropped
    self
      .cmd_tx
      .send(Command::Connect { conn_tx, res_tx })
      .unwrap();

    // unwrap: chat server does not drop out response channel
    res_rx.await.unwrap()
  }

  /// List all created rooms.
  pub async fn list_journeys(&self) -> Vec<JourneyId> {
    let (res_tx, res_rx) = oneshot::channel();

    // unwrap: chat server should not have been dropped
    self.cmd_tx.send(Command::List { res_tx }).unwrap();

    // unwrap: chat server does not drop our response channel
    res_rx.await.unwrap()
  }

  /// Join `room`, creating it if it does not exist.
  pub async fn watch_journey(&self, conn: ConnId, room: impl Into<JourneyId>) {
    let (res_tx, res_rx) = oneshot::channel();

    // unwrap: chat server should not have been dropped
    self
      .cmd_tx
      .send(Command::Join {
        conn,
        room: room.into(),
        res_tx,
      })
      .unwrap();

    // unwrap: chat server does not drop our response channel
    res_rx.await.unwrap();
  }

  pub async fn new_transaction_id(&self, journey_id: JourneyId, transaction_id: TransactionId) {
    let (res_tx, res_rx) = oneshot::channel();

    self
      .cmd_tx
      .send(Command::NewId {
        journey_id,
        transaction_id,
        res_tx,
      })
      .unwrap();

    res_rx.await.unwrap()
  }

  /// Broadcast message to current room.
  pub async fn send_message(&self, conn: ConnId, msg: impl Into<Msg>) {
    let (res_tx, res_rx) = oneshot::channel();

    // unwrap: chat server should not have been dropped
    self
      .cmd_tx
      .send(Command::Message {
        msg: msg.into(),
        conn,
        res_tx,
      })
      .unwrap();

    // unwrap: chat server does not drop our response channel
    res_rx.await.unwrap();
  }

  /// Unregister message sender and broadcast disconnection message to current room.
  pub fn disconnect(&self, conn: ConnId) {
    // unwrap: chat server should not have been dropped
    self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
  }
}
