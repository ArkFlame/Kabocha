use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

/// The reason a player could not enter a bed, mirroring Paper's `BedSleepingProblem`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BedFailReason {
    /// The player is too far from the bed (`TOO_FAR_AWAY`).
    TooFarAway,
    /// The bed is obstructed (`OBSTRUCTED`).
    Obstructed,
    /// The time or weather does not allow sleep (`rule.asProblem()`).
    NoSleep,
    /// A hostile creature is nearby (`NOT_SAFE`).
    NotSafe,
}

/// An event that occurs when a player fails to enter a bed.
///
/// Mirrors Paper's `PlayerBedFailEnterEvent`: cancelling suppresses the
/// failure message and a set `message` overrides it.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBedFailEnterEvent {
    /// The player failing to enter the bed.
    pub player: Arc<Player>,

    /// The position of the bed.
    pub bed_pos: BlockPos,

    /// Why the player could not enter the bed.
    pub reason: BedFailReason,

    /// An optional message that replaces the default failure message.
    pub message: Option<TextComponent>,
}

impl PlayerBedFailEnterEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, bed_pos: BlockPos, reason: BedFailReason) -> Self {
        Self {
            player,
            bed_pos,
            reason,
            message: None,
            cancelled: false,
        }
    }
}

/// Resolves the failure message after the fail event has been fired.
///
/// Returns `None` when the event was cancelled, matching Paper's suppression
/// of the failure message.
#[must_use]
pub fn resolve_fail_message(
    cancelled: bool,
    override_message: Option<TextComponent>,
    default_message: TextComponent,
) -> Option<TextComponent> {
    if cancelled {
        None
    } else {
        Some(override_message.unwrap_or(default_message))
    }
}

/// An event that occurs when a player enters a bed.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBedEnterEvent {
    /// The player entering the bed.
    pub player: Arc<Player>,

    /// The position of the bed.
    pub bed_pos: BlockPos,
}

impl PlayerBedEnterEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, bed_pos: BlockPos) -> Self {
        Self {
            player,
            bed_pos,
            cancelled: false,
        }
    }
}

/// An event that occurs when a player leaves a bed.
#[derive(Event, Clone)]
pub struct PlayerBedLeaveEvent {
    /// The player leaving the bed.
    pub player: Arc<Player>,

    /// The position of the bed.
    pub bed_pos: BlockPos,
}

impl PlayerBedLeaveEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, bed_pos: BlockPos) -> Self {
        Self { player, bed_pos }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_fail_message;
    use pumpkin_util::text::TextComponent;

    #[test]
    fn cancelled_fail_suppresses_message() {
        let default = TextComponent::text("default");
        let override_message = TextComponent::text("override");
        assert_eq!(
            resolve_fail_message(true, Some(override_message), default),
            None
        );
    }

    #[test]
    fn fail_message_prefers_override() {
        let default = TextComponent::text("default");
        let override_message = TextComponent::text("override");
        assert_eq!(
            resolve_fail_message(false, Some(override_message.clone()), default),
            Some(override_message)
        );
    }

    #[test]
    fn fail_message_uses_default_without_override() {
        let default = TextComponent::text("default");
        assert_eq!(
            resolve_fail_message(false, None, default.clone()),
            Some(default)
        );
    }
}
