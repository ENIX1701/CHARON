# Architecture

CHARON is an asynchronous terminal-based user interface written in Rust. It is designed to act as the primary operator console for the AETHER framework.

## High-level overview

CHARON follows the **[The Elm Architecture](https://guide.elm-lang.org/architecture/)** (TEA) that utilizes a unidirectional data flow pattern. This makes state stability manageable and simplifies testing.

1. **State**: Entire application's state is held in [`AppState`](../src/state.rs)
2. **View**: [`UI`](../src/ui.rs) is a pure function (no state-dependent variables; always produces same output for same input) that renders the current `AppState` to the terminal
3. **Action**: User inputs and async events (like network responses) produce [`Action`](../src/action.rs) enums
4. **Update**: Takes the current `AppState` and some `Action`, returns a new state and an optional [`Command`](../src/update.rs)

The entire application runs on `tokio` async runtime. It's able to handle non-blocking network I/O with SHADOW while maintaining a fully responsive UI. I think it's pretty neat :3

## Directory structure

```bash
CHARON/
├── Cargo.lock
├── Cargo.toml
├── charon-ascii.png
├── Dockerfile
├── docs
│   ├── ARCHITECTURE.md
│   ├── images
│   └── MANUAL.md
├── README.md
└── src
    ├── action.rs
    ├── api.rs
    ├── client.rs
    ├── main.rs
    ├── models.rs
    ├── state.rs
    ├── ui.rs
    └── update.rs
```

## Core components

### AppState

**Located at**: `src/state.rs`

Single source of truth about the navigation states (current tab), sub-states (`DashboardState`, `TerminalState`, `ConfigState`, `BuilderState`) and global flags (status messages and such).

### UI renderer

**Located at**: `src/ui.rs`

Built using `ratatui` (peak name btw). Splits the terminal into chunks (header, content and footer) and renders widgets based on the `AppState`. Stateless. Idempotent.

### Network client

**Located at**: `src/client.rs`

`RealClient` struct implements the `C2Client` trait. It uses `reqwest` to perform asynchronous HTTP requests to SHADOW. Errors are shown in footer.

If you need reference as to what these are, head over to the [SHADOW API guide](https://github.com/ENIX1701/SHADOW/blob/main/docs/API_GUIDE.md).

### Builder

**Located at**: `src/update.rs`

CHARON's unique. CHARON's powerful. CHARON can command GHOSTs to life. It can build them.

GHOSTs build orchestrator works as follows on a build trigger:
1. Spawns a blocking task in `tokio` runtime
2. Invokes `cmake` on the `../GHOST` directory, passing all the necessary flags to it
3. Invokes `make` to compile the GHOST
4. Returns the result on screen when the binary is built in `../GHOST/build/bin/Ghost`

## Extension guide

### Adding a new tab

**1. Update state**

Add a new variant to `CurrentScreen` enum in [`state.rs`](../src/state.rs) and a corresponding sub-state struct (like `LogState`).

**2. Update UI**

Add the tab title to `render_header` in [`ui.rs`](../src/ui.rs). Create a `render_<module>` function and call it in the main `draw` match block.

**3. Update logic**

Add navigation handling in [`update.rs`](../src/update.rs) (switching to the new tab, etc.). Handle inputs specific to the new view in `handle_char_input` or navigation helpers.

### Adding a new command

**1. Define action**

Add a variant to `Action` in [`action.rs`](../src/action.rs) (like `Action::PurgeLogs`).

**2. Define command**

Add a variant to `Command` in [`update.rs`](../src/update.rs) if it requires a side effect (like `Command::PurgeRemoteLogs`).

**3. Handle update**

In [`update.rs`](../src/update.rs) match the `Action` and return the `Command`.

**4. Handle side effect**

In [`main.rs`](../src/main.rs) match the `Command` in `handle_command` and execute the async logic.
