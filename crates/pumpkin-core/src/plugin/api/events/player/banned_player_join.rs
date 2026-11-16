use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::net::SocketAddr;
use uuid::Uuid;

/// An event that occurs when a banned player attempts to join the server.
///
/// Plugins can change [`Self::kick_message`] to display a custom ban screen, or
/// cancel the event to ignore the ban and let the player join (for example to
/// unban them based on custom logic).
#[cancellable]
#[derive(Event, Clone)]
pub struct BannedPlayerJoinEvent {
    /// The player username.
    pub player_name: String,

    /// The player unique ID.
    pub player_uuid: Uuid,

    /// The remote IP address.
    pub ip_address: SocketAddr,

    /// The message shown to the player. Defaults to the ban reason screen.
    pub kick_message: TextComponent,
}
