use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::translation;
use pumpkin_inventory::Inventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetContainerSlot;
use pumpkin_protocol::ser::json_to_nbt_tag;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::argument_builder::{
    ArgumentBuilder, RequiredArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::argument_type::JavaClientArgumentType;
use crate::command::argument_types::client_parsed::ClientParsed;
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::item::ItemStackArgumentType;
use crate::command::argument_types::item_modifier::{ItemModifier, ItemModifierArgumentType};
use crate::command::argument_types::slot::SlotArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;

const DESCRIPTION: &str = "Modifies items in block or entity inventories.";
const PERMISSION: &str = "minecraft:command.item";

const ERROR_NOT_CONTAINER: CommandErrorType<3> = CommandErrorType::new(
    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
    translation::java::COMMANDS_ITEM_TARGET_NOT_A_CONTAINER,
);

const ERROR_NO_SUCH_SLOT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
    translation::java::COMMANDS_ITEM_TARGET_NO_SUCH_SLOT,
);

const ERROR_UNKNOWN_ITEM_MODIFIER: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ID_UNKNOWN,
    translation::java::ARGUMENT_ID_UNKNOWN,
);

struct BlockReplaceExecutor {
    has_count: bool,
}

impl CommandExecutor for BlockReplaceExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let world = context.source.world();
        let pos = BlockPosArgumentType::get_loaded_block_pos(context, "target")?;
        let slot = SlotArgumentType::get(context, "slots")?;
        let parsed_stack = ItemStackArgumentType::get(context, "item")?;
        let item = parsed_stack.item;
        let count = if self.has_count {
            IntegerArgumentType::get(context, "count")?
        } else {
            1
        };

        let block_entity = world.get_block_entity(&pos).ok_or_else(|| {
            ERROR_NOT_CONTAINER.create_without_context(
                TextComponent::text(pos.0.x.to_string()),
                TextComponent::text(pos.0.y.to_string()),
                TextComponent::text(pos.0.z.to_string()),
            )
        })?;

        let inventory = block_entity.get_inventory().ok_or_else(|| {
            ERROR_NOT_CONTAINER.create_without_context(
                TextComponent::text(pos.0.x.to_string()),
                TextComponent::text(pos.0.y.to_string()),
                TextComponent::text(pos.0.z.to_string()),
            )
        })?;

        if slot >= inventory.size() {
            return Err(
                ERROR_NO_SUCH_SLOT.create_without_context(TextComponent::text(slot.to_string()))
            );
        }

        let mut item_stack = parsed_stack.clone();
        item_stack.item_count = count as u8;
        inventory.set_stack(slot, item_stack);

        let item_name = item.registry_key;
        let msg = TextComponent::translate_cross(
            translation::java::COMMANDS_ITEM_BLOCK_REPLACE_SUCCESS_KNOWN_ITEM,
            translation::java::COMMANDS_ITEM_BLOCK_REPLACE_SUCCESS_KNOWN_ITEM,
            [
                TextComponent::text("1"),
                TextComponent::text(pos.0.x.to_string()),
                TextComponent::text(pos.0.y.to_string()),
                TextComponent::text(pos.0.z.to_string()),
                TextComponent::text("[")
                    .add_child(item.translated_name())
                    .add_child(TextComponent::text("]"))
                    .hover_event(HoverEvent::ShowItem {
                        id: item_name.to_string().into(),
                        count: Some(count),
                        components: None,
                    }),
            ],
        );
        context.source.send_feedback(msg, true);

        Ok(1)
    }
}

struct EntityReplaceExecutor {
    has_count: bool,
}

impl CommandExecutor for EntityReplaceExecutor {
    #[expect(clippy::too_many_lines)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "target")?;
        let mojang_slot = SlotArgumentType::get(context, "slots")?;
        let parsed_stack = ItemStackArgumentType::get(context, "item")?;
        let item = parsed_stack.item;
        let count = if self.has_count {
            IntegerArgumentType::get(context, "count")?
        } else {
            1
        };

        let mut modified_count = 0;
        let mut item_stack = parsed_stack.clone();
        item_stack.item_count = count as u8;

        for target in &targets {
            if let Some(player) = target.get_player() {
                if let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id) {
                    if (200..=226).contains(&mojang_slot) {
                        let ender_slot = mojang_slot - 200;
                        if ender_slot < player.ender_chest_inventory.size() {
                            player_arc
                                .ender_chest_inventory
                                .set_stack(ender_slot, item_stack.clone());
                            modified_count += 1;
                        }
                    } else {
                        let inventory = player.inventory();
                        let mapped_slot = if mojang_slot == 98 {
                            Some(inventory.get_selected_slot() as usize)
                        } else if mojang_slot == 99 {
                            Some(40)
                        } else if mojang_slot == 100 {
                            Some(36)
                        } else if mojang_slot == 101 {
                            Some(37)
                        } else if mojang_slot == 102 {
                            Some(38)
                        } else if mojang_slot == 103 {
                            Some(39)
                        } else if mojang_slot <= 35 {
                            Some(mojang_slot)
                        } else {
                            None
                        };

                        if let Some(slot) = mapped_slot
                            && slot < inventory.size()
                        {
                            player_arc.inventory().set_stack(slot, item_stack.clone());

                            let stack_serializer = ItemStackSerializer::from(item_stack.clone());
                            let packet =
                                CSetContainerSlot::new(0, 0, slot as i16, &stack_serializer);
                            player_arc.enqueue_slot_packet(&packet, None, 0);

                            let eq_slot = if slot == 36 {
                                Some(EquipmentSlot::FEET)
                            } else if slot == 37 {
                                Some(EquipmentSlot::LEGS)
                            } else if slot == 38 {
                                Some(EquipmentSlot::CHEST)
                            } else if slot == 39 {
                                Some(EquipmentSlot::HEAD)
                            } else if slot == 40 {
                                Some(EquipmentSlot::OFF_HAND)
                            } else if slot == inventory.get_selected_slot() as usize {
                                Some(EquipmentSlot::MAIN_HAND)
                            } else {
                                None
                            };

                            if let Some(eq) = eq_slot {
                                player
                                    .living_entity
                                    .send_equipment_changes(&[(eq, item_stack.clone())]);
                            }

                            modified_count += 1;
                        }
                    }
                }
            } else if let Some(living) = target.get_living_entity() {
                let mapped_eq = if mojang_slot == 98 {
                    Some(EquipmentSlot::MAIN_HAND)
                } else if mojang_slot == 99 {
                    Some(EquipmentSlot::OFF_HAND)
                } else if mojang_slot == 100 {
                    Some(EquipmentSlot::FEET)
                } else if mojang_slot == 101 {
                    Some(EquipmentSlot::LEGS)
                } else if mojang_slot == 102 {
                    Some(EquipmentSlot::CHEST)
                } else if mojang_slot == 103 {
                    Some(EquipmentSlot::HEAD)
                } else if mojang_slot == 105 {
                    Some(EquipmentSlot::BODY)
                } else if mojang_slot == 106 {
                    Some(EquipmentSlot::SADDLE)
                } else {
                    None
                };

                if let Some(eq) = mapped_eq {
                    living
                        .entity_equipment
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .put(&eq, item_stack.clone());
                    living.send_equipment_changes(&[(eq, item_stack.clone())]);
                    modified_count += 1;
                }
            }
        }

        if modified_count == 0 {
            return Err(ERROR_NO_SUCH_SLOT
                .create_without_context(TextComponent::text(mojang_slot.to_string())));
        }

        let item_name = item.registry_key;
        let item_component = TextComponent::text("[")
            .add_child(item.translated_name())
            .add_child(TextComponent::text("]"))
            .hover_event(HoverEvent::ShowItem {
                id: item_name.to_string().into(),
                count: Some(count),
                components: None,
            });
        let msg = if targets.len() == 1 {
            TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_ENTITY_REPLACE_SUCCESS_SINGLE_KNOWN_ITEM,
                translation::java::COMMANDS_ITEM_ENTITY_REPLACE_SUCCESS_SINGLE_KNOWN_ITEM,
                [
                    TextComponent::text(modified_count.to_string()),
                    targets[0].as_ref().get_display_name(),
                    item_component,
                ],
            )
        } else {
            TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_ENTITY_REPLACE_SUCCESS_MULTIPLE_KNOWN_ITEM,
                translation::java::COMMANDS_ITEM_ENTITY_REPLACE_SUCCESS_MULTIPLE_KNOWN_ITEM,
                [
                    TextComponent::text(targets.len().to_string()),
                    item_component,
                ],
            )
        };
        context.source.send_feedback(msg, true);

        Ok(modified_count)
    }
}

fn block_item_node() -> RequiredArgumentBuilder {
    argument("item", ItemStackArgumentType)
        .executes(BlockReplaceExecutor { has_count: false })
        .then(
            argument("count", IntegerArgumentType::new(1, 99))
                .executes(BlockReplaceExecutor { has_count: true }),
        )
}

fn entity_item_node() -> RequiredArgumentBuilder {
    argument("item", ItemStackArgumentType)
        .executes(EntityReplaceExecutor { has_count: false })
        .then(
            argument("count", IntegerArgumentType::new(1, 99))
                .executes(EntityReplaceExecutor { has_count: true }),
        )
}

/// Maps a vanilla armor/hand slot index to the equipment slot it names.
const fn equipment_slot(mojang_slot: usize) -> Option<EquipmentSlot> {
    match mojang_slot {
        98 => Some(EquipmentSlot::MAIN_HAND),
        99 => Some(EquipmentSlot::OFF_HAND),
        100 => Some(EquipmentSlot::FEET),
        101 => Some(EquipmentSlot::LEGS),
        102 => Some(EquipmentSlot::CHEST),
        103 => Some(EquipmentSlot::HEAD),
        105 => Some(EquipmentSlot::BODY),
        106 => Some(EquipmentSlot::SADDLE),
        _ => None,
    }
}

/// Reads the stack in a vanilla slot index of an entity, or `None` if it has no such slot.
fn read_entity_slot(target: &Arc<dyn EntityBase>, mojang_slot: usize) -> Option<ItemStack> {
    if let Some(player) = target.get_player() {
        if (200..=226).contains(&mojang_slot) {
            let ender_slot = mojang_slot - 200;
            return (ender_slot < player.ender_chest_inventory.size())
                .then(|| player.ender_chest_inventory.get_stack(ender_slot));
        }
        let inventory = player.inventory();
        let slot = match mojang_slot {
            98 => inventory.get_selected_slot() as usize,
            99 => 40,
            100 => 36,
            101 => 37,
            102 => 38,
            103 => 39,
            0..=35 => mojang_slot,
            _ => return None,
        };
        return (slot < inventory.size()).then(|| inventory.get_stack(slot));
    }
    let living = target.get_living_entity()?;
    let eq = equipment_slot(mojang_slot)?;
    Some(
        living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&eq),
    )
}

/// Writes `item_stack` into the vanilla slot index of an entity, returning whether the
/// entity has that slot.
fn write_entity_slot(
    target: &Arc<dyn EntityBase>,
    mojang_slot: usize,
    item_stack: &ItemStack,
) -> bool {
    if let Some(player) = target.get_player() {
        if let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id) {
            if (200..=226).contains(&mojang_slot) {
                let ender_slot = mojang_slot - 200;
                if ender_slot < player.ender_chest_inventory.size() {
                    player_arc
                        .ender_chest_inventory
                        .set_stack(ender_slot, item_stack.clone());
                    return true;
                }
            } else {
                let inventory = player.inventory();
                let mapped_slot = if mojang_slot == 98 {
                    Some(inventory.get_selected_slot() as usize)
                } else if mojang_slot == 99 {
                    Some(40)
                } else if mojang_slot == 100 {
                    Some(36)
                } else if mojang_slot == 101 {
                    Some(37)
                } else if mojang_slot == 102 {
                    Some(38)
                } else if mojang_slot == 103 {
                    Some(39)
                } else if mojang_slot <= 35 {
                    Some(mojang_slot)
                } else {
                    None
                };

                if let Some(slot) = mapped_slot
                    && slot < inventory.size()
                {
                    player_arc.inventory().set_stack(slot, item_stack.clone());

                    let stack_serializer = ItemStackSerializer::from(item_stack.clone());
                    let packet = CSetContainerSlot::new(0, 0, slot as i16, &stack_serializer);
                    player_arc.enqueue_slot_packet(&packet, None, 0);

                    let eq_slot = if slot == 36 {
                        Some(EquipmentSlot::FEET)
                    } else if slot == 37 {
                        Some(EquipmentSlot::LEGS)
                    } else if slot == 38 {
                        Some(EquipmentSlot::CHEST)
                    } else if slot == 39 {
                        Some(EquipmentSlot::HEAD)
                    } else if slot == 40 {
                        Some(EquipmentSlot::OFF_HAND)
                    } else if slot == inventory.get_selected_slot() as usize {
                        Some(EquipmentSlot::MAIN_HAND)
                    } else {
                        None
                    };

                    if let Some(eq) = eq_slot {
                        player
                            .living_entity
                            .send_equipment_changes(&[(eq, item_stack.clone())]);
                    }

                    return true;
                }
            }
        }
    } else if let Some(living) = target.get_living_entity() {
        let mapped_eq = if mojang_slot == 98 {
            Some(EquipmentSlot::MAIN_HAND)
        } else if mojang_slot == 99 {
            Some(EquipmentSlot::OFF_HAND)
        } else if mojang_slot == 100 {
            Some(EquipmentSlot::FEET)
        } else if mojang_slot == 101 {
            Some(EquipmentSlot::LEGS)
        } else if mojang_slot == 102 {
            Some(EquipmentSlot::CHEST)
        } else if mojang_slot == 103 {
            Some(EquipmentSlot::HEAD)
        } else if mojang_slot == 105 {
            Some(EquipmentSlot::BODY)
        } else if mojang_slot == 106 {
            Some(EquipmentSlot::SADDLE)
        } else {
            None
        };

        if let Some(eq) = mapped_eq {
            living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .put(&eq, item_stack.clone());
            living.send_equipment_changes(&[(eq, item_stack.clone())]);
            return true;
        }
    }
    false
}

/// Replaces a container block's slot with `update(current)`.
fn set_block_slot(
    context: &CommandContext,
    pos: &BlockPos,
    slot: usize,
    update: impl FnOnce(ItemStack) -> ItemStack,
) -> CommandExecutorResult {
    let inventory = context
        .source
        .world()
        .get_block_entity(pos)
        .and_then(<dyn crate::block::entities::BlockEntity>::get_inventory)
        .ok_or_else(|| {
            ERROR_NOT_CONTAINER.create_without_context(
                TextComponent::text(pos.0.x.to_string()),
                TextComponent::text(pos.0.y.to_string()),
                TextComponent::text(pos.0.z.to_string()),
            )
        })?;
    if slot >= inventory.size() {
        return Err(
            ERROR_NO_SUCH_SLOT.create_without_context(TextComponent::text(slot.to_string()))
        );
    }
    inventory.set_stack(slot, update(inventory.get_stack(slot)));
    Ok(1)
}

/// Applies an inline item modifier (a single function or a list of them). Only the
/// functions datapacks commonly use are implemented; others leave the stack as is.
fn apply_item_modifier(stack: &mut ItemStack, modifier: &NbtTag) {
    match modifier {
        NbtTag::List(list) => {
            for function in list {
                apply_item_modifier(stack, function);
            }
        }
        NbtTag::Compound(function) => {
            let name = function.get_string("function").unwrap_or_default();
            match name.strip_prefix("minecraft:").unwrap_or(name) {
                "set_damage" => {
                    let Some(max) = stack.get_max_damage() else {
                        return;
                    };
                    let amount = number_provider(function.get("damage")).unwrap_or(1.0);
                    let add = function.get_bool("add").unwrap_or(false);
                    let base = if add {
                        (max - stack.get_damage()) as f32 / max as f32
                    } else {
                        0.0
                    };
                    let durability = (base + amount).clamp(0.0, 1.0);
                    let damage = max - (durability * max as f32).floor() as i32;
                    stack.set_damage(damage.clamp(0, max));
                }
                "set_count" => {
                    let amount = number_provider(function.get("count")).unwrap_or(1.0) as i32;
                    let add = function.get_bool("add").unwrap_or(false);
                    let count = if add {
                        i32::from(stack.item_count) + amount
                    } else {
                        amount
                    };
                    // Vanilla setCount does not clamp to the item's max stack size.
                    stack.item_count = count.clamp(0, i32::from(u8::MAX)) as u8;
                }
                other => tracing::debug!("Unsupported item modifier function {other}"),
            }
        }
        _ => {}
    }
}

/// Reads a numeric provider: a plain number, or a typed provider such as
/// constant, uniform, binomial or sum (vanilla NumberProvider).
fn number_provider(tag: Option<&NbtTag>) -> Option<f32> {
    match tag? {
        NbtTag::Compound(c) => {
            let kind = c
                .get("type")
                .and_then(NbtTag::extract_string)
                .unwrap_or("constant");
            let kind = kind.strip_prefix("minecraft:").unwrap_or(kind);
            match kind {
                // vanilla UniformGenerator: a uniform sample in [min, max)
                "uniform" => {
                    let min = number_provider(c.get("min"))?;
                    let max = number_provider(c.get("max"))?;
                    Some(min + rand::random::<f32>() * (max - min))
                }
                "binomial" => {
                    let n = number_provider(c.get("n"))? as i32;
                    let p = number_provider(c.get("p"))?;
                    let mut result: i32 = 0;
                    for _ in 0..n {
                        if rand::random::<f32>() < p {
                            result += 1;
                        }
                    }
                    Some(result as f32)
                }
                "sum" => {
                    let inputs = c.get("inputs")?.extract_list()?;
                    inputs.iter().try_fold(0.0_f32, |total, input| {
                        Some(total + number_provider(Some(input))?)
                    })
                }
                _ => c.get("value").and_then(number_as_f32),
            }
        }
        other => number_as_f32(other),
    }
}

/// SNBT numbers parse to several integer and float tag kinds; all are valid providers.
fn number_as_f32(tag: &NbtTag) -> Option<f32> {
    match tag {
        NbtTag::Byte(v) => Some(f32::from(*v)),
        NbtTag::Short(v) => Some(f32::from(*v)),
        NbtTag::Int(v) => Some(*v as f32),
        NbtTag::Long(v) => Some(*v as f32),
        NbtTag::Float(v) => Some(*v),
        NbtTag::Double(v) => Some(*v as f32),
        _ => None,
    }
}

fn resolve_item_modifier(context: &CommandContext) -> Result<NbtTag, CommandSyntaxError> {
    match ItemModifierArgumentType::get(context, "modifier")? {
        ItemModifier::Inline(tag) => Ok(tag.clone()),
        ItemModifier::Reference(id) => {
            let id = id.to_string();
            let value = context
                .source
                .server()
                .datapack_manager
                .get_item_modifier(&id)
                .ok_or_else(|| {
                    ERROR_UNKNOWN_ITEM_MODIFIER.create_without_context(TextComponent::text(id))
                })?;
            Ok(json_to_nbt_tag(&value))
        }
    }
}

/// `item modify block <pos> <slot> <modifier>`.
struct BlockModifyExecutor;

impl CommandExecutor for BlockModifyExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let pos = BlockPosArgumentType::get_loaded_block_pos(context, "pos")?;
        let slot = SlotArgumentType::get(context, "slot")?;
        let modifier = resolve_item_modifier(context)?;
        let modified = set_block_slot(context, &pos, slot, |mut stack| {
            apply_item_modifier(&mut stack, &modifier);
            stack
        })?;
        let msg = TextComponent::translate_cross(
            translation::java::COMMANDS_ITEM_BLOCK_MODIFY_SUCCESS,
            translation::java::COMMANDS_ITEM_BLOCK_MODIFY_SUCCESS,
            [
                TextComponent::text(modified.to_string()),
                TextComponent::text(pos.0.x.to_string()),
                TextComponent::text(pos.0.y.to_string()),
                TextComponent::text(pos.0.z.to_string()),
            ],
        );
        context.source.send_feedback(msg, true);
        Ok(modified)
    }
}

/// `item modify entity <targets> <slot> <modifier>`.
struct EntityModifyExecutor;

impl CommandExecutor for EntityModifyExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let slot = SlotArgumentType::get(context, "slot")?;
        let modifier = resolve_item_modifier(context)?;
        let mut count = 0;
        for target in &targets {
            if let Some(mut stack) = read_entity_slot(target, slot) {
                apply_item_modifier(&mut stack, &modifier);
                if write_entity_slot(target, slot, &stack) {
                    count += 1;
                }
            }
        }
        if count == 0 {
            return Err(
                ERROR_NO_SUCH_SLOT.create_without_context(TextComponent::text(slot.to_string()))
            );
        }
        let msg = if targets.len() == 1 {
            TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_ENTITY_MODIFY_SUCCESS_SINGLE,
                translation::java::COMMANDS_ITEM_ENTITY_MODIFY_SUCCESS_SINGLE,
                [
                    TextComponent::text(count.to_string()),
                    targets[0].as_ref().get_display_name(),
                ],
            )
        } else {
            TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_ENTITY_MODIFY_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_ITEM_ENTITY_MODIFY_SUCCESS_MULTIPLE,
                [TextComponent::text(targets.len().to_string())],
            )
        };
        context.source.send_feedback(msg, true);
        Ok(count)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let block_node = literal("block").then(
        argument("target", BlockPosArgumentType).then(
            argument(
                "slots",
                ClientParsed::new(SlotArgumentType, JavaClientArgumentType::SlotSource),
            )
            .then(literal("with").then(block_item_node())),
        ),
    );

    let entity_node = literal("entity").then(
        argument("target", EntityArgumentType::Entities).then(
            argument(
                "slots",
                ClientParsed::new(SlotArgumentType, JavaClientArgumentType::SlotSource),
            )
            .then(literal("with").then(entity_item_node())),
        ),
    );

    let modify_node = literal("modify")
        .then(literal("block").then(
            argument("pos", BlockPosArgumentType).then(argument("slot", SlotArgumentType).then(
                argument("modifier", ItemModifierArgumentType).executes(BlockModifyExecutor),
            )),
        ))
        .then(
            literal("entity").then(argument("targets", EntityArgumentType::Entities).then(
                argument("slot", SlotArgumentType).then(
                    argument("modifier", ItemModifierArgumentType).executes(EntityModifyExecutor),
                ),
            )),
        );

    dispatcher.register(
        command("item", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("replace").then(block_node).then(entity_node))
            .then(modify_node),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_nbt::compound::NbtCompound;

    #[test]
    fn uniform_number_provider_samples_within_min_and_max() {
        let mut compound = NbtCompound::new();
        compound.put_string("type", "uniform".to_string());
        compound.put_float("min", 2.0);
        compound.put_float("max", 5.0);
        let tag = NbtTag::Compound(compound);
        for _ in 0..256 {
            let sample = number_provider(Some(&tag)).expect("uniform provider yields a sample");
            assert!(
                (2.0..5.0).contains(&sample),
                "sample {sample} out of [2, 5)"
            );
        }
    }

    #[test]
    fn namespaced_uniform_provider_uses_nested_providers() {
        let mut compound = NbtCompound::new();
        compound.put_string("type", "minecraft:uniform".to_string());
        compound.put("min", NbtTag::Int(2));
        compound.put("max", NbtTag::Int(4));
        let tag = NbtTag::Compound(compound);
        for _ in 0..256 {
            let sample = number_provider(Some(&tag));
            assert!(sample.is_some_and(|value| (2.0..4.0).contains(&value)));
        }
    }

    #[test]
    fn sum_provider_adds_all_inputs() {
        let mut compound = NbtCompound::new();
        compound.put_string("type", "minecraft:sum".to_string());
        compound.put(
            "inputs",
            NbtTag::List(vec![NbtTag::Int(2), NbtTag::Float(3.5)]),
        );
        let tag = NbtTag::Compound(compound);
        assert_eq!(number_provider(Some(&tag)), Some(5.5));
    }

    #[test]
    fn binomial_provider_stays_within_zero_and_n() {
        let mut compound = NbtCompound::new();
        compound.put_string("type", "minecraft:binomial".to_string());
        compound.put("n", NbtTag::Int(8));
        compound.put("p", NbtTag::Float(0.5));
        let tag = NbtTag::Compound(compound);
        for _ in 0..256 {
            let sample = number_provider(Some(&tag));
            assert!(sample.is_some_and(|value| (0.0..=8.0).contains(&value)));
        }
    }
}
