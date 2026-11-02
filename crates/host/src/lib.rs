//! Session host for Quai des Orfèvres 36.
//!
//! [`GameHost`] owns the live [`qdo36_session::GameSession`] behind
//! the MCP command channel: it drains `CommandKind`s, keeps the
//! seat↔token registry and executes the shared `request_move` path.
//! Deliberately GTK-free and headless-testable - the app drains it
//! on the glib main loop, MCP tests drive it directly.

mod game_host;

pub use game_host::GameHost;
