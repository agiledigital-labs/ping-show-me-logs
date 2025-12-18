import BluetoothSearching from "@mui/icons-material/BluetoothSearching";
import Search from "@mui/icons-material/Search";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Divider from "@mui/material/Divider";
import Drawer from "@mui/material/Drawer";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import "@xyflow/react/dist/style.css";
import { type ReactNode, useEffect, useReducer, useState } from "react";
import { Link, useSearchParams } from "react-router";
import { AppSharedContext } from "./Contexts.tsx";

const DrawerList = ({
  toggleDrawer,
}: {
  toggleDrawer: (input: boolean) => () => void;
}) => (
  <Box sx={{ width: 250 }} role="presentation" onClick={toggleDrawer(false)}>
    <List>
      {[
        ["Search Logs", "search"],
        ["Watch Logs", "watch"],
        ["Flow", "/"],
      ].map(([text, key], index) => (
        <ListItem key={text} disablePadding>
          <Link to={key}>
            <ListItemButton>
              <ListItemIcon>
                {index % 2 === 0 ? <Search /> : <BluetoothSearching />}
              </ListItemIcon>
              <ListItemText primary={text} />
            </ListItemButton>
          </Link>
        </ListItem>
      ))}
    </List>
    <Divider />
  </Box>
);

type State = { flow: Record<string, object[]>; ws?: WebSocket };

type ActionType = "NewTransactionId" | "SetWebSocket";

type Action<A extends ActionType, T extends object> = { type: A; msg: T };

type WebSocAction = Action<"SetWebSocket", { webSoc: WebSocket }>;

const makeSteWebSocket = (webSoc: WebSocket): WebSocAction => ({
  type: "SetWebSocket",
  msg: { webSoc },
});

type TransactionAction = Action<
  "NewTransactionId",
  { journeyId: string; transactionId: string }
>;

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-expect-error
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const makeNewTransactionIdAction = (
  journeyId: string,
  transactionId: string,
): TransactionAction => ({
  type: "NewTransactionId",
  msg: { journeyId, transactionId },
});

type Actions = WebSocAction | TransactionAction;

const reducer = (state: State, action: Actions) => {
  switch (action.type) {
    case "NewTransactionId": {
      return state;
    }
    case "SetWebSocket": {
      return { ...state, ws: action.msg.webSoc };
    }

    default: {
      return state;
    }
  }
};

const AppNav = ({ children }: { children: ReactNode }) => {
  const [open, setOpen] = useState(false);

  const [state, dispatch] = useReducer(reducer, {
    flow: {},
  });

  useEffect(() => {
    const ws = new WebSocket("ws://localhost:8081/api/ws");
    dispatch(makeSteWebSocket(ws));
    ws.onopen = () => {
      console.info("connected to server");
    };
    ws.onmessage = (event) => {
      console.info(event);
    };
    ws.onclose = () => console.log("Disconnected");
    return () => ws.close();
  }, []);

  useEffect(() => {
    if (state.ws !== undefined && state.ws.readyState === WebSocket.OPEN) {
      try {
        state.ws.send("/join all");
      } catch (e) {
        console.error(e);
      }
    }
  }, [open]);

  const toggleDrawer = (newOpen: boolean) => () => {
    setOpen(newOpen);
  };
  const [urlSearchPrams] = useSearchParams();

  return (
    <AppSharedContext value={{ location: urlSearchPrams.get("view") ?? "web" }}>
      <div style={{ height: "90vh", width: "97vw" }}>
        <Button onClick={toggleDrawer(true)}>Open Side Menu</Button>
        <Drawer open={open} onClose={toggleDrawer(false)}>
          <DrawerList toggleDrawer={toggleDrawer} />
        </Drawer>
        <>{children}</>
        {/* Persistent bottom-left logo on every screen */}
        <img
          src={"/14386_Agile_Logo_2lines_CMYK-transparent-500px.png"}
          alt="Agile Digital logo"
          style={{
            position: "fixed",
            left: 12,
            bottom: 12,
            width: 120,
            height: "auto",
            opacity: 0.9,
            pointerEvents: "none",
            zIndex: 9999,
          }}
        />
      </div>
    </AppSharedContext>
  );
};

export default AppNav;
