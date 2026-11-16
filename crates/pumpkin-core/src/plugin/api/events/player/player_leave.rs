use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// Why a player left the server.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayerLeaveReason {
    /// The client disconnected by itself.
    #[default]
    Disconnected,
    /// The player was kicked, for example by an operator, the idle timer or the spam filter.
    Kicked,
    /// The server is stopping.
    ServerShutdown,
    /// The player was transferred to another server.
    Transfer,
}

/// An event that occurs when a player leaves the game.
///
/// This event contains information about the player leaving and a message to display upon leaving.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerLeaveEvent {
    /// The player who is leaving the game.
    pub player: Arc<Player>,

    /// The message to display when the player leaves.
    pub leave_message: TextComponent,

    /// Whether this session requested a server transfer; destination arrival is not confirmed.
    pub transfer_requested: bool,

    /// Why the player left the server.
    pub reason: PlayerLeaveReason,
}

impl PlayerLeaveEvent {
    /// Creates a new instance of `PlayerLeaveEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player leaving the game.
    /// - `leave_message`: The message to display upon leaving.
    ///
    /// # Returns
    /// A new instance of `PlayerLeaveEvent`.
    pub fn new(player: Arc<Player>, leave_message: TextComponent) -> Self {
        let transfer_requested = player.transfer_requested();
        let reason = player.leave_reason();
        Self {
            player,
            leave_message,
            transfer_requested,
            reason,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerLeaveEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
