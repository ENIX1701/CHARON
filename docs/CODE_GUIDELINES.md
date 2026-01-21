# Code guidelines

## General standards

**Language standard**: `Rust (2024)`
**Indentation**: 4 spaces
**Line endings**: LF (Unix style)
**File encoding**: `UTF-8`

## Naming conventions

Consistency is key. I like keys :3

| Type                      | Convention                    | Example                           |
|---------------------------|-------------------------------|-----------------------------------|
| **Structs/Enums**         | PascalCase                    | `AppState`, `TaskStatus`          |
| **Variables**             | snake_case                    | `sleep_interval`, `ghost_id`      |
| **Functions**             | snake_case                    | `fetch_ghosts`, `render_ui`       |
| **Constants**             | SCREAMING_SNAKE_CASE          | `SHADOW_URL`, `DEFAULT_TICK_RATE` |

## Project structure

Refer to [architecture overview](ARCHITECTURE.md#directory-structure).

## Implementation details

CHARON strictly adheres to the **Model-View-Update** pattern adapted for Tokio. The runtime, not the city.

### Immutability where possible

UI rendering takes an immutable reference to `AppState`. This way makes it impossible to break things (unless you try *very* hard, but why would you...).

### Message passing

Components communicate via `Action` enums sent over [`mpsc`](https://doc.rust-lang.org/std/sync/mpsc/index.html) channels.

### Separation of concerns

Simple rules:
- `ui.rs` must NEVER contain business logic
- `client.rs` must NEVER depend on UI types
- `state.rs` should only contain data, not behavior

### Error handling

#### Result types

Public APIs must return `Result<T, String>` or `Result<T, Error>`

#### Panics

DO NOT PANIC (in the main loop). UI thread must stay alive. Weird things happen if you do panic (you can check those by using the `todo!` macro, but you'll most likely just crash the UI...)

#### Status bar

Errors meant for the operator's eyes should be pushed to `app.status_message`, not printed to `stdout`/`stderr`. Raw output may break the TUI. See [panics](#panics).

## Dependencies

### Ratatui

TUI rendering. Amazing name.

### Tokio

Used for async runtime and task scheduling. Makes non-blocking data refreshing a breeze. 

### reqwest

HTTP reqwests :3

### Serde

JSON serialization and deserialization. I've learned my lesson when creating GHOST. Never manually parsing JSON again.

### crossterm

Raw terminal handling and input events. That's it.
