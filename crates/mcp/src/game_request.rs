//! `GameRequest` - the command-plus-reply pair crossing from the MCP
//! thread into the app's session owner (chess' channel pattern).

use tokio::sync::oneshot::Sender;

use crate::command::CommandKind;
use crate::command::CommandReply;

/// A command plus the channel back to the waiting tool call.
#[derive(Debug)]
pub struct GameRequest {
    /// What to execute.
    pub command: CommandKind,
    /// Reply channel; carries the typed answer back to the tool.
    pub reply: Sender<CommandReply>,
}
