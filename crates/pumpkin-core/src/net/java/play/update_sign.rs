#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_sign_update(&self, player: &Player, sign_data: &SUpdateSign<'_>) {
        // Vanilla `isImmobile()` (dead or dying) returns before the action time is reset.
        if player.living_entity.is_dead_or_dying() {
            return;
        }
        // `ServerGamePacketListenerImpl#updateSignText` resets the action time before validating.
        player.update_last_action_time();

        let world = player.get_entity().world.load_full();
        // Vanilla checks the chunk before looking up the block entity. The reusable
        // `may_update_text` gate repeats this for the Bedrock path, which has no pre-check.
        if !world
            .level
            .is_chunk_loaded(&sign_data.location.chunk_position())
        {
            // The chunk gate must not strand an editor opened for this player: a
            // refused update must never leave the sign editor open.
            clear_editor_for_player(&world, sign_data.location, player);
            return;
        }

        let Some(block_entity) = world.get_block_entity(&sign_data.location) else {
            return;
        };
        let Some(sign_entity) =
            crate::block::entities::sign::SignEntityRef::from_block_entity(&*block_entity)
        else {
            return;
        };
        if !sign_entity.may_update_text(&world, player) {
            warn!(
                "Player {} just tried to change non-editable sign",
                player.gameprofile.name
            );
            // `may_update_text` clears the editor when the player moved out of
            // range; cover the waxed and unloaded-chunk refusals as well.
            clear_editor_for_player(&world, sign_data.location, player);
            return;
        }

        let lines = vec![
            strip_formatting(sign_data.line_1),
            strip_formatting(sign_data.line_2),
            strip_formatting(sign_data.line_3),
            strip_formatting(sign_data.line_4),
        ];

        if let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id) {
            let mut event = crate::plugin::api::events::block::sign_change::SignChangeEvent::new(
                player_arc,
                sign_data.location,
                lines,
            );
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if event.cancelled {
                return;
            }
        }

        let text = sign_entity.get_text(sign_data.is_front_text);

        let new_messages = [
            Box::<str>::from(strip_formatting(sign_data.line_1)),
            Box::<str>::from(strip_formatting(sign_data.line_2)),
            Box::<str>::from(strip_formatting(sign_data.line_3)),
            Box::<str>::from(strip_formatting(sign_data.line_4)),
        ];

        text.set_client_messages(&new_messages);
        *sign_entity
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        world.update_block_entity(&block_entity);
    }
}

/// Clears the sign editor only when it was opened for `player`, so a different
/// player's editor is never closed by a refused update.
fn clear_editor_for_player(
    world: &crate::world::World,
    pos: pumpkin_util::math::position::BlockPos,
    player: &Player,
) {
    if let Some(block_entity) = world.get_block_entity(&pos)
        && let Some(sign) =
            crate::block::entities::sign::SignEntityRef::from_block_entity(&*block_entity)
    {
        let mut editor = sign
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *editor == Some(player.gameprofile.id) {
            *editor = None;
        }
    }
}

/// Vanilla `ChatFormatting.stripFormatting`: drops `§` plus a legacy format code (`0-9`, `a-f`, `k-o`, `r`).
fn strip_formatting(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '§'
            && chars.peek().is_some_and(
                |code| matches!(code.to_ascii_lowercase(), '0'..='9' | 'a'..='f' | 'k'..='o' | 'r'),
            )
        {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::strip_formatting;

    #[test]
    fn strips_legacy_format_codes() {
        assert_eq!(strip_formatting("§aHi§L§r!"), "Hi!");
    }

    #[test]
    fn keeps_non_format_sequences() {
        assert_eq!(strip_formatting("§xy§"), "§xy§");
    }
}
