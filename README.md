# Ping, show me logs

Visualize and explore PingOne/ForgeRock Authentication Journeys and related logs in near real time.

This project is a Rust Actix Web server with an embedded React UI. It authenticates using a service account, tails the Ping logs API, indexes recent transaction IDs, and exposes endpoints/WebSocket messages consumed by the UI.

## Stack

- Backend: Rust (edition 2024), Actix Web, Tokio, Reqwest, actix-ws, Serde, Tantivy (for in-memory search)
- Auth/JWT: jsonwebkey, jsonwebtoken
- Build: Cargo; `build.rs` runs `npm ci && npm run build` inside `ui/`
- Frontend: React + Vite + TypeScript + MUI (in `ui/`), built and embedded into the binary
- Package managers: Cargo (Rust), npm (UI)

## Requirements

- Rust toolchain (stable) and Cargo
- Node.js and npm (required at build time by `build.rs` to compile the UI)
- A PingOne/ForgeRock environment with:
  - Service Account ID and JWK credentials
  - Domain (e.g. `https://your-aic-domain.id.forgerock.io`)
  - Logs endpoint and API key/secret for Monitoring Logs API
- Build tools for native Rust crates (OpenSSL vendored is enabled, but a C toolchain is still required)

## Environment variables

Set the following before running. Example values shown for reference only.

```sh
# Service account used to obtain OAuth token for AM/IDM APIs
export SA_ID="your-service-account-id"
export DOM="https://your-aic-domain.id.forgerock.io"
export KEY_FILE="/absolute/path/to/service-account.jwk.json"

# Monitoring Logs API (used for fetching/tailing logs)
# Typically: https://<domain>/monitoring/logs
export LOGS_ENDPOINT="$DOM/monitoring/logs"
export PING_KEY="your-logs-api-key-id"
export PING_SEC="your-logs-api-secret"
```

Notes
- `LOGS_ENDPOINT`, `PING_KEY`, and `PING_SEC` are required for the logs API.
- `SA_ID`, `DOM`, and `KEY_FILE` are required to mint an access token for other API calls.

## Setup

1. Install prerequisites (Rust, Node.js, npm).
2. Set the environment variables above.
3. Build and run the server. The build step will automatically install and build the UI via `build.rs`:

   ```sh
   cargo run
   ```

4. Open the UI in your browser:

   - http://localhost:8081

## Running

- Development run (backend + embedded UI):

  ```sh
  cargo run
  ```

  The server binds to `0.0.0.0:8081`.

- Backend only rebuild without running:

  ```sh
  cargo build
  ```

### UI only (optional local UI workflows)

If you wish to develop the UI separately, you can work inside `ui/` with Vite:

```sh
cd ui
npm ci
npm run dev    # starts Vite dev server
```

Note: the Rust app embeds the production UI build; the Vite dev server is only for frontend development. Integrating the dev server with the Rust backend requires manual proxy configuration (TODO).

## Endpoints and WebSocket

Base URL: `http://localhost:8081`

- Static UI: `GET /` and asset paths served by the backend (built artifacts from `ui/dist`)
- API base: `GET /api/*`
  - WebSocket: `GET /api/ws`
  - Monitoring passthrough:
    - `GET /api/monitoring/am`
    - `GET /api/monitoring/idm`
  - Logs and search (selected):
    - `GET /api/transaction/ids` — returns recent transaction IDs
    - `GET /api/transaction/demo/search` — search PoC
    - `GET /api/{fr_id}/script/{script_id}` — logs for a specific script
  - Journeys:
    - `GET /api/{name}/flow?transaction_id=<id>` — journey graph for a specific transaction (optional parameter)
    - `GET /api/{name}/transactions` — list transactions for a journey
    - `GET /api/{name}/scripts` — scripts linked to a journey
    - `GET /api/scripts` — saved script configs

Notes
- Endpoints are derived from the current code and may evolve. For details, see `src/ping_logs/service.rs` and `src/trees/service.rs`.

## Scripts

Backend (Cargo)
- `cargo run` — builds UI and runs the server
- `cargo build` — builds the binary (also builds UI)

Frontend (npm, in `ui/`)
- `npm run dev` — Vite dev server
- `npm run build` — type-check and production build
- `npm run preview` — preview built UI
- `npm run lint` — ESLint

Utility
- `test.sh` — example script to create a JWT and request a token (uses `jose`, `openssl`, `curl`, 1Password CLI in example). Update paths/secrets as appropriate.

## Tests

- There are currently no automated Rust tests in this repository. TODO: add integration tests for API routes and unit tests for log parsing and search indexing.

## Project structure

```
show-me-logs/
├─ src/
│  ├─ main.rs                 # Actix Web server entrypoint (port 8081)
│  ├─ front.rs                # Serves embedded UI and monitoring endpoints
│  ├─ ping_logs/
│  │  ├─ logs.rs              # Log payload models and log fetch/tail helpers
│  │  └─ service.rs           # Logs-related API endpoints
│  ├─ trees/
│  │  ├─ journeys.rs          # Journey graph structures and helpers
│  │  ├─ nodes.rs             # Node configuration and data types
│  │  └─ service.rs           # Journey-related API endpoints
│  ├─ watcher/                # Journey watcher (WIP)
│  ├─ workers/
│  │  └─ scripts.rs           # Script discovery/config handling
│  ├─ ws_server.rs            # WebSocket server and rooms
│  ├─ ws_handler.rs           # WebSocket session handling
│  ├─ errors.rs               # Error types
│  └─ token.rs                # Service-account token minting and refresh
├─ ui/                        # React + Vite frontend (built and embedded)
├─ build.rs                   # Builds UI with npm at compile time
├─ Cargo.toml                 # Rust crate manifest
├─ Cargo.lock
├─ test.sh                    # Token/JWT helper script (example)
├─ renovate.json              # Dependency update config
├─ rustfmt.toml               # Rust formatting config
├─ vscode-ext-poc/            # VSCode extension POC (not integrated)  # TODO: document or remove
└─ target/                    # Cargo build artifacts
```

## License

No license file was found in the repository. TODO: add a `LICENSE` file and update this section.

## Notes and caveats

- The server requires valid PingOne/ForgeRock credentials and access to the Monitoring Logs API.
- UI assets are embedded at compile time; re-run `cargo run` after UI changes so `build.rs` rebuilds the assets. Alternatively, use the UI dev server for frontend work.
- Tantivy is used in-memory for a small rolling index of transaction IDs (see `MAX_SIZE` in `main.rs`). This is not persisted across restarts.

## Future improvements

- Some nice way to expand or view inner journey flows using the same transaction ID.
- Aggregates of statuses (such as errors) across many transactions.
- More efficient tracking ID flow linking (don't query logs with the same tracking ID twice).
- Colour code nodes in the path rather than/as well as edges.
- Remove the `error` outcome from inner journey nodes.
- The logs API for individual scripts appears to be broken (404).
