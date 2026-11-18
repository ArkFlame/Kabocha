#[allow(clippy::wildcard_imports)]
use super::*;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32};

use crate::block::entities::command_block::CommandBlockEntity;
use pumpkin_protocol::bedrock::server::block_actor_data::SBlockActorData;
use tracing::warn;

fn parse_bedrock_sign_lines(text: &str) -> [Box<str>; 4] {
    let mut lines = [Box::from(""), Box::from(""), Box::from(""), Box::from("")];
    for (i, line) in text.split('\n').take(4).enumerate() {
        lines[i] = Box::from(line);
    }
    lines
}

fn apply_sign_text(
    player: &Arc<Player>,
    world: &Arc<crate::world::World>,
    sign_entity: &crate::block::entities::sign::SignEntityRef,
    block_pos: pumpkin_util::math::position::BlockPos,
    is_front: bool,
    text_comp: &pumpkin_nbt::compound::NbtCompound,
) {
    let Some(raw_text) = text_comp.get_string("Text") else {
        return;
    };
    let lines = parse_bedrock_sign_lines(raw_text);
    let event_lines = lines.iter().map(ToString::to_string).collect();
    let mut event = crate::plugin::api::events::block::sign_change::SignChangeEvent::new(
        player.clone(),
        block_pos,
        event_lines,
    );
    if let Some(server) = world.server.upgrade() {
        server.plugin_manager.fire_blocking(&server, &mut event);
    }
    if event.cancelled {
        return;
    }

    let sign_text = sign_entity.get_text(is_front);
    sign_text.set_client_messages(&lines);

    if let Some(color_val) = text_comp.get_int("SignTextColor") {
        let dye = crate::block::entities::sign::bedrock_sign_text_color_to_dye_color(color_val);
        sign_text.set_color(dye);
    }
    if let Some(glowing) = text_comp
        .get_bool("IgnoreLighting")
        .or_else(|| text_comp.get_byte("IgnoreLighting").map(|b| b != 0))
    {
        sign_text.set_has_glowing_text(glowing);
    }
}

/// Parsed fields from the actor-data NBT a Bedrock client sends back after
/// editing a command block, mirroring vanilla `ServerGamePacketListenerImpl#handleSetCommandBlock`.
struct CommandBlockEdit {
    command: String,
    auto: Option<bool>,
    track_output: Option<bool>,
    conditional: Option<bool>,
    custom_name: Option<String>,
}

/// `None` fields mean the client omitted them, so the caller keeps the stored value.
fn parse_command_block_edit(tags: &pumpkin_nbt::compound::NbtCompound) -> CommandBlockEdit {
    let command = tags.get_string("Command").unwrap_or("");
    CommandBlockEdit {
        // Vanilla strips a single leading slash before storing the command.
        command: command.strip_prefix('/').unwrap_or(command).to_string(),
        auto: tags.get_bool("auto"),
        track_output: tags.get_bool("TrackOutput"),
        conditional: tags.get_bool("conditionalMode"),
        custom_name: tags.get_string("CustomName").map(ToString::to_string),
    }
}

impl BedrockClient {
    pub fn handle_block_actor_data(&self, player: &Arc<Player>, packet: &SBlockActorData) {
        let id = packet.actor_data_tags.get_string("id").unwrap_or("");
        if id == "Sign" || id == "HangingSign" {
            Self::handle_sign_block_actor_data(player, packet);
        } else if id == "CommandBlock" {
            Self::handle_command_block_actor_data(player, packet);
        }
    }

    fn handle_command_block_actor_data(player: &Arc<Player>, packet: &SBlockActorData) {
        player.update_last_action_time();

        if !player.can_use_game_master_blocks() {
            return;
        }

        let world = player.get_entity().world.load_full();
        let Some(block_entity) = world.get_block_entity(&packet.block_position) else {
            return;
        };
        if block_entity.resource_location() != CommandBlockEntity::ID {
            return;
        }
        let Some(old) = block_entity.as_any().downcast_ref::<CommandBlockEntity>() else {
            return;
        };

        let edit = parse_command_block_edit(&packet.actor_data_tags);
        let auto = edit.auto.unwrap_or(old.auto.load(Ordering::SeqCst));
        let track_output = edit
            .track_output
            .unwrap_or(old.track_output.load(Ordering::SeqCst));

        // Conditional lives on the block state, not the block entity.
        let state_id = world.get_block_state_id(&packet.block_position);
        let block = world.get_block(&packet.block_position);
        let mut props =
            pumpkin_data::block_properties::CommandBlockLikeProperties::from_state_id(state_id);
        props.r#conditional = edit.conditional.unwrap_or(props.r#conditional);
        let new_state_id = props.to_state_id(block);
        world.set_block_state(
            &packet.block_position,
            new_state_id,
            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
        );

        let custom_name = edit.custom_name.map_or_else(
            || {
                old.custom_name
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone()
            },
            |name| Some(TextComponent::text(name)),
        );
        let command_block = CommandBlockEntity {
            position: packet.block_position,
            powered: AtomicBool::new(old.powered.load(Ordering::SeqCst)),
            condition_met: AtomicBool::new(old.condition_met.load(Ordering::SeqCst)),
            auto: AtomicBool::new(auto),
            dirty: AtomicBool::new(old.dirty.load(Ordering::SeqCst)),
            command: std::sync::Mutex::new(edit.command.clone()),
            custom_name: std::sync::Mutex::new(custom_name),
            last_output: std::sync::Mutex::new(
                old.last_output
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            ),
            track_output: AtomicBool::new(track_output),
            success_count: AtomicU32::new(0),
            last_execution: AtomicI64::new(old.last_execution.load(Ordering::SeqCst)),
            update_last_execution: AtomicBool::new(
                old.update_last_execution.load(Ordering::SeqCst),
            ),
        };
        world.add_block_entity(Arc::new(command_block));

        player.send_system_message(&TextComponent::text(format!(
            "Command set: {}",
            edit.command
        )));

        // Always-active impulse and repeating blocks run their first tick next.
        if auto && *block != pumpkin_data::Block::CHAIN_COMMAND_BLOCK {
            world.schedule_block_tick(
                block,
                packet.block_position,
                1,
                pumpkin_world::tick::TickPriority::Normal,
            );
        }
    }

    fn handle_sign_block_actor_data(player: &Arc<Player>, packet: &SBlockActorData) {
        player.update_last_action_time();

        let world = player.get_entity().world.load_full();
        let Some(block_entity) = world.get_block_entity(&packet.block_position) else {
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
            return;
        }

        if let Some(front_comp) = packet.actor_data_tags.get_compound("FrontText") {
            apply_sign_text(
                player,
                &world,
                &sign_entity,
                packet.block_position,
                true,
                front_comp,
            );
        } else if let Some(raw_text) = packet.actor_data_tags.get_string("Text") {
            let mut temp_comp = pumpkin_nbt::compound::NbtCompound::new();
            temp_comp.put_string("Text", raw_text.to_string());
            if let Some(color_val) = packet.actor_data_tags.get_int("SignTextColor") {
                temp_comp.put_int("SignTextColor", color_val);
            }
            apply_sign_text(
                player,
                &world,
                &sign_entity,
                packet.block_position,
                true,
                &temp_comp,
            );
        }

        if let Some(back_comp) = packet.actor_data_tags.get_compound("BackText") {
            apply_sign_text(
                player,
                &world,
                &sign_entity,
                packet.block_position,
                false,
                back_comp,
            );
        }

        *sign_entity
            .currently_editing_player()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        world.update_block_entity(&block_entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_command_block_edit_strips_slash_and_reads_flags() {
        let mut tags = pumpkin_nbt::compound::NbtCompound::new();
        tags.put_string("id", "CommandBlock");
        tags.put_string("Command", "/say hi");
        tags.put_bool("auto", true);
        tags.put_bool("TrackOutput", false);
        tags.put_bool("conditionalMode", true);
        tags.put_string("CustomName", "Cmd");
        let edit = parse_command_block_edit(&tags);
        assert_eq!(edit.command, "say hi");
        assert_eq!(edit.auto, Some(true));
        assert_eq!(edit.track_output, Some(false));
        assert_eq!(edit.conditional, Some(true));
        assert_eq!(edit.custom_name.as_deref(), Some("Cmd"));
    }

    #[test]
    fn parse_command_block_edit_defaults_when_fields_absent() {
        let mut tags = pumpkin_nbt::compound::NbtCompound::new();
        tags.put_string("id", "CommandBlock");
        tags.put_string("Command", "say hi");
        let edit = parse_command_block_edit(&tags);
        assert_eq!(edit.command, "say hi");
        assert!(edit.auto.is_none());
        assert!(edit.track_output.is_none());
        assert!(edit.conditional.is_none());
        assert!(edit.custom_name.is_none());
    }
}
