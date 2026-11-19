//! Admits a login whose UUID or name is already online. Like vanilla
//! `PlayerList#placeNewPlayer`, the old player is disconnected and the new login proceeds.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use tracing::debug;

use crate::{
    net::{DisconnectReason, GameProfile},
    server::Server,
};

/// How long a new login waits for the kicked player to leave the player list.
const EVICTION_WAIT_LIMIT: Duration = Duration::from_secs(2);
/// Interval between checks while waiting for the kicked player to leave.
const EVICTION_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// What a login does next while it waits for the duplicate to leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionStep {
    /// The duplicate has left, so the new login proceeds.
    Proceed,
    /// The duplicate is still registered and the budget is spent, so the new login is rejected.
    Reject,
    /// The duplicate is still registered and the budget remains.
    Wait,
}

/// Decides the next step from whether a duplicate is still online and the time spent waiting.
#[must_use]
pub fn next_eviction_step(still_online: bool, elapsed: Duration) -> EvictionStep {
    if !still_online {
        EvictionStep::Proceed
    } else if elapsed >= EVICTION_WAIT_LIMIT {
        EvictionStep::Reject
    } else {
        EvictionStep::Wait
    }
}

/// The message shown to a kicked duplicate, and to a new login that is rejected.
#[must_use]
pub fn duplicate_login_message() -> TextComponent {
    TextComponent::translate_cross(
        translation::java::MULTIPLAYER_DISCONNECT_DUPLICATE_LOGIN,
        translation::bedrock::DISCONNECTIONSCREEN_LOGGEDINOTHERLOCATION,
        [],
    )
}

/// Kicks each online player sharing the login's UUID or name, then waits a bounded time for them to leave.
///
/// Returns `false` when a duplicate is still registered after the wait, so the new login must be rejected.
/// No lock is held across the wait.
pub async fn evict_duplicate_players(server: &Server, profile: &GameProfile) -> bool {
    let mut duplicates = Vec::new();
    for candidate in [
        server.get_player_by_uuid(profile.id),
        server.get_player_by_name(&profile.name),
    ]
    .into_iter()
    .flatten()
    {
        if !duplicates
            .iter()
            .any(|known| Arc::ptr_eq(known, &candidate))
        {
            duplicates.push(candidate);
        }
    }
    if duplicates.is_empty() {
        return true;
    }

    for old in &duplicates {
        debug!(
            "Kicking online player '{}' ({}) for a new login with the same UUID or name",
            old.gameprofile.name, old.gameprofile.id
        );
        old.kick(DisconnectReason::Kicked, &duplicate_login_message());
    }

    let started = Instant::now();
    loop {
        let still_online = server.get_player_by_uuid(profile.id).is_some()
            || server.get_player_by_name(&profile.name).is_some();
        match next_eviction_step(still_online, started.elapsed()) {
            EvictionStep::Proceed => return true,
            EvictionStep::Reject => return false,
            EvictionStep::Wait => tokio::time::sleep(EVICTION_POLL_INTERVAL).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EvictionStep, next_eviction_step};
    use std::time::Duration;

    #[test]
    fn proceeds_once_the_duplicate_has_left() {
        assert_eq!(
            next_eviction_step(false, Duration::ZERO),
            EvictionStep::Proceed
        );
        assert_eq!(
            next_eviction_step(false, Duration::from_secs(5)),
            EvictionStep::Proceed
        );
    }

    #[test]
    fn waits_while_the_duplicate_is_within_budget() {
        assert_eq!(
            next_eviction_step(true, Duration::from_millis(1999)),
            EvictionStep::Wait
        );
    }

    #[test]
    fn rejects_when_the_duplicate_outlives_the_budget() {
        assert_eq!(
            next_eviction_step(true, Duration::from_secs(2)),
            EvictionStep::Reject
        );
    }
}
