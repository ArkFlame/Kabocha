use std::borrow::Cow;
use std::sync::Arc;

use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::data_component_impl::{
    CustomModelDataImpl, EnchantmentsImpl, PotionContentsImpl, StoredEnchantmentsImpl, read_data,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Potion;
use pumpkin_data::translation;
use pumpkin_inventory::Inventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetContainerSlot;
use pumpkin_protocol::ser::json_to_nbt_tag;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;
use rand::RngExt;

use crate::command::argument_builder::{
    ArgumentBuilder, RequiredArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::client_parsed::ClientParsed;
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::item::ItemStackArgumentType;
use crate::command::argument_types::item_modifier::{ItemModifier, ItemModifierArgumentType};
use crate::command::argument_types::nbt_path::NbtPathArgumentType;
use crate::command::argument_types::slot::{SlotArgumentType, SlotsArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::source::DummySource;
use crate::command::string_reader::StringReader;
use crate::entity::EntityBase;
use crate::entity::mob::piglin::PiglinEntity;
use crate::entity::mob::pillager::PillagerEntity;
use crate::entity::passive::villager::VillagerEntity;
use crate::entity::passive::wandering_trader::WanderingTraderEntity;
use crate::server::Server;

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

#[derive(Clone, Copy)]
enum SuccessFeedback {
    Replace,
    Set,
}

struct BlockReplaceExecutor {
    has_count: bool,
    feedback: SuccessFeedback,
}

impl CommandExecutor for BlockReplaceExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let world = context.source.world();
        let pos = BlockPosArgumentType::get_loaded_block_pos(context, "target")?;
        let single_slot;
        let slots: &[usize] = match self.feedback {
            SuccessFeedback::Replace => SlotsArgumentType::get(context, "slots")?,
            SuccessFeedback::Set => {
                single_slot = SlotArgumentType::get(context, "slots")?;
                std::slice::from_ref(&single_slot)
            }
        };
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

        let mut item_stack = parsed_stack.clone();
        item_stack.item_count = count as u8;

        let mut replaced = 0usize;
        for &slot in slots {
            if slot < inventory.size() {
                inventory.set_stack(slot, item_stack.clone());
                replaced += 1;
            }
        }
        if replaced == 0 {
            if matches!(self.feedback, SuccessFeedback::Set) {
                return Err(
                    ERROR_NO_SUCH_SLOT.create_without_context(TextComponent::text(
                        slots.first().copied().unwrap_or(0).to_string(),
                    )),
                );
            }
            // No slot accepted the item: vanilla reports commands.item.target.failed*.
            let item_name = item.registry_key;
            let item_component = TextComponent::text("[")
                .add_child(item.translated_name())
                .add_child(TextComponent::text("]"))
                .hover_event(HoverEvent::ShowItem {
                    id: item_name.to_string().into(),
                    count: Some(count),
                    components: None,
                });
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_TARGET_FAILED_KNOWN_ITEM,
                translation::java::COMMANDS_ITEM_TARGET_FAILED_KNOWN_ITEM,
                [item_component],
            );
            context.source.send_feedback(msg, true);
            return Ok(0);
        }
        // Mirrors vanilla Container.setChanged() so the container's NBT is saved.
        inventory.mark_dirty();

        let item_name = item.registry_key;
        let item_component = TextComponent::text("[")
            .add_child(item.translated_name())
            .add_child(TextComponent::text("]"))
            .hover_event(HoverEvent::ShowItem {
                id: item_name.to_string().into(),
                count: Some(count),
                components: None,
            });
        let msg = match self.feedback {
            SuccessFeedback::Replace => TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_BLOCK_REPLACE_SUCCESS_KNOWN_ITEM,
                translation::java::COMMANDS_ITEM_BLOCK_REPLACE_SUCCESS_KNOWN_ITEM,
                [
                    TextComponent::text(replaced.to_string()),
                    TextComponent::text(pos.0.x.to_string()),
                    TextComponent::text(pos.0.y.to_string()),
                    TextComponent::text(pos.0.z.to_string()),
                    item_component,
                ],
            ),
            SuccessFeedback::Set => TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_BLOCK_SET_SUCCESS,
                translation::java::COMMANDS_ITEM_BLOCK_SET_SUCCESS,
                [
                    TextComponent::text(pos.0.x.to_string()),
                    TextComponent::text(pos.0.y.to_string()),
                    TextComponent::text(pos.0.z.to_string()),
                    item_component,
                ],
            ),
        };
        context.source.send_feedback(msg, true);

        Ok(replaced as i32)
    }
}

/// Vanilla `CommandResponseTracker` inputs: (total changed slots, non-zero target count, sole non-zero index).
const fn response_counts(per_target: &[usize]) -> (usize, usize, Option<usize>) {
    let mut total = 0usize;
    let mut succeeded = 0usize;
    let mut sole = None;
    let mut i = 0;
    while i < per_target.len() {
        let changed = per_target[i];
        total += changed;
        if changed > 0 {
            succeeded += 1;
            sole = Some(i);
        }
        i += 1;
    }
    if succeeded != 1 {
        sole = None;
    }
    (total, succeeded, sole)
}

struct EntityReplaceExecutor {
    has_count: bool,
    feedback: SuccessFeedback,
}

impl CommandExecutor for EntityReplaceExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "target")?;
        let single_slot;
        let slots: &[usize] = match self.feedback {
            SuccessFeedback::Replace => SlotsArgumentType::get(context, "slots")?,
            SuccessFeedback::Set => {
                single_slot = SlotArgumentType::get(context, "slots")?;
                std::slice::from_ref(&single_slot)
            }
        };
        let parsed_stack = ItemStackArgumentType::get(context, "item")?;
        let item = parsed_stack.item;
        let count = if self.has_count {
            IntegerArgumentType::get(context, "count")?
        } else {
            1
        };

        // Vanilla's CommandResponseTracker tracks a count per target and dispatches over
        // ElementType.NON_ZERO: only targets that changed a slot count towards success.
        let mut per_target = Vec::with_capacity(targets.len());
        let mut item_stack = parsed_stack.clone();
        item_stack.item_count = count as u8;

        for target in &targets {
            let mut target_count = 0usize;
            for &slot in slots {
                if write_entity_slot(target, slot, &item_stack) {
                    target_count += 1;
                }
            }
            per_target.push(target_count);
        }
        let (modified_count, succeeded_targets, sole_target) = response_counts(&per_target);

        if succeeded_targets == 0 {
            if matches!(self.feedback, SuccessFeedback::Set) {
                return Err(
                    ERROR_NO_SUCH_SLOT.create_without_context(TextComponent::text(
                        slots.first().copied().unwrap_or(0).to_string(),
                    )),
                );
            }
            // No target accepted the item: vanilla reports commands.item.target.failed*.
            let item_name = item.registry_key;
            let item_component = TextComponent::text("[")
                .add_child(item.translated_name())
                .add_child(TextComponent::text("]"))
                .hover_event(HoverEvent::ShowItem {
                    id: item_name.to_string().into(),
                    count: Some(count),
                    components: None,
                });
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_TARGET_FAILED_KNOWN_ITEM,
                translation::java::COMMANDS_ITEM_TARGET_FAILED_KNOWN_ITEM,
                [item_component],
            );
            context.source.send_feedback(msg, true);
            return Ok(0);
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
        let msg = match (self.feedback, sole_target) {
            (SuccessFeedback::Replace, Some(i)) => TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_ENTITY_REPLACE_SUCCESS_SINGLE_KNOWN_ITEM,
                translation::java::COMMANDS_ITEM_ENTITY_REPLACE_SUCCESS_SINGLE_KNOWN_ITEM,
                [
                    TextComponent::text(modified_count.to_string()),
                    targets[i].as_ref().get_display_name(),
                    item_component,
                ],
            ),
            (SuccessFeedback::Replace, None) => TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_ENTITY_REPLACE_SUCCESS_MULTIPLE_KNOWN_ITEM,
                translation::java::COMMANDS_ITEM_ENTITY_REPLACE_SUCCESS_MULTIPLE_KNOWN_ITEM,
                [
                    TextComponent::text(succeeded_targets.to_string()),
                    item_component,
                ],
            ),
            (SuccessFeedback::Set, Some(i)) => TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_SINGLE,
                translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_SINGLE,
                [targets[i].as_ref().get_display_name(), item_component],
            ),
            (SuccessFeedback::Set, None) => TextComponent::translate_cross(
                translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_ITEM_ENTITY_SET_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(succeeded_targets.to_string()),
                    item_component,
                ],
            ),
        };
        context.source.send_feedback(msg, true);

        Ok(modified_count as i32)
    }
}

fn block_item_node(feedback: SuccessFeedback) -> RequiredArgumentBuilder {
    argument("item", ItemStackArgumentType)
        .executes(BlockReplaceExecutor {
            has_count: false,
            feedback,
        })
        .then(
            argument("count", IntegerArgumentType::new(1, 99)).executes(BlockReplaceExecutor {
                has_count: true,
                feedback,
            }),
        )
}

fn entity_item_node(feedback: SuccessFeedback) -> RequiredArgumentBuilder {
    argument("item", ItemStackArgumentType)
        .executes(EntityReplaceExecutor {
            has_count: false,
            feedback,
        })
        .then(
            argument("count", IntegerArgumentType::new(1, 99)).executes(EntityReplaceExecutor {
                has_count: true,
                feedback,
            }),
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

/// `mob.inventory.0..=7` (vanilla slot 300..=307) to a zero-based index.
const fn mob_inventory_index(mojang_slot: usize) -> Option<usize> {
    match mojang_slot {
        300..=307 => Some(mojang_slot - 300),
        _ => None,
    }
}

/// Vanilla mobs back `mob.inventory.*` with a fixed-size `SimpleContainer`:
/// villagers and piglins have 8 slots, pillagers 5 (`Pillager.getSlot` uses
/// `inventory.getContainerSize()`). Pumpkin stores these as growable `Vec`s that
/// start empty, so unallocated slots must read as air and writes must grow them.
const MOB_INVENTORY_SIZE: usize = 8;
const PILLAGER_INVENTORY_SIZE: usize = 5;

fn pad_inventory_to(inventory: &mut Vec<ItemStack>, index: usize) {
    if inventory.len() <= index {
        inventory.resize(index + 1, ItemStack::EMPTY.clone());
    }
}

/// Reads the player's cursor (499) and 2x2 crafting grid (500..=503). Vanilla
/// `Player.getSlot` resolves these through the player's own menu.
fn read_player_menu_slot(
    player: &crate::entity::player::Player,
    mojang_slot: usize,
) -> Option<ItemStack> {
    let handler = player
        .player_screen_handler
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match mojang_slot {
        499 => Some(
            handler
                .get_behaviour()
                .cursor_stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        ),
        500..=503 => Some(handler.get_slot(1 + (mojang_slot - 500)).get_stack()),
        _ => None,
    }
}

/// Writes the player's cursor or crafting grid.
fn write_player_menu_slot(
    player: &crate::entity::player::Player,
    mojang_slot: usize,
    item_stack: &ItemStack,
) -> bool {
    let handler = player
        .player_screen_handler
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match mojang_slot {
        499 => {
            *handler
                .get_behaviour()
                .cursor_stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = item_stack.clone();
            true
        }
        500..=503 => {
            handler
                .get_slot(1 + (mojang_slot - 500))
                .set_stack(item_stack.clone());
            true
        }
        _ => false,
    }
}

/// Vanilla `AbstractHorse.getSlot`: `CHEST_SLOT_OFFSET` 499 toggles the chest and
/// `INVENTORY_SLOT_OFFSET` 500 indexes the horse inventory.
fn read_horse_slot(
    horse: &dyn crate::entity::passive::abstract_horse::AbstractHorse,
    mojang_slot: usize,
) -> Option<ItemStack> {
    match mojang_slot {
        499 => Some(if horse.has_chest() {
            ItemStack::new(1, &Item::CHEST)
        } else {
            ItemStack::EMPTY.clone()
        }),
        500..=514 => {
            let index = mojang_slot - 500;
            let inventory = horse.inventory();
            (index < inventory.size()).then(|| inventory.get_stack(index))
        }
        _ => None,
    }
}

fn write_horse_slot(
    horse: &dyn crate::entity::passive::abstract_horse::AbstractHorse,
    mojang_slot: usize,
    item_stack: &ItemStack,
) -> bool {
    match mojang_slot {
        499 => {
            if item_stack.is_empty() {
                if horse.has_chest() {
                    horse.set_chest(false);
                    horse.create_inventory();
                }
                true
            } else if item_stack.item == &Item::CHEST {
                if !horse.has_chest() {
                    horse.set_chest(true);
                    horse.create_inventory();
                }
                true
            } else {
                false
            }
        }
        500..=514 => {
            let index = mojang_slot - 500;
            let inventory = horse.inventory();
            if index < inventory.size() {
                inventory.set_stack(index, item_stack.clone());
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Villagers, piglins and pillagers expose their personal inventory at 300..=307.
fn read_mob_slot(target: &Arc<dyn EntityBase>, mojang_slot: usize) -> Option<ItemStack> {
    let index = mob_inventory_index(mojang_slot)?;
    let any = target.cast_any();
    if let Some(villager) = any.downcast_ref::<VillagerEntity>() {
        return (index < MOB_INVENTORY_SIZE).then(|| {
            villager
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(index)
                .cloned()
                .unwrap_or_else(|| ItemStack::EMPTY.clone())
        });
    }
    if let Some(trader) = any.downcast_ref::<WanderingTraderEntity>() {
        let inventory = &trader.merchant_inventory;
        return (index < inventory.size()).then(|| inventory.get_stack(index));
    }
    if let Some(piglin) = any.downcast_ref::<PiglinEntity>() {
        return (index < MOB_INVENTORY_SIZE).then(|| {
            piglin
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(index)
                .cloned()
                .unwrap_or_else(|| ItemStack::EMPTY.clone())
        });
    }
    if let Some(pillager) = any.downcast_ref::<PillagerEntity>() {
        return (index < PILLAGER_INVENTORY_SIZE).then(|| {
            pillager
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(index)
                .cloned()
                .unwrap_or_else(|| ItemStack::EMPTY.clone())
        });
    }
    None
}

fn write_mob_slot(
    target: &Arc<dyn EntityBase>,
    mojang_slot: usize,
    item_stack: &ItemStack,
) -> bool {
    let Some(index) = mob_inventory_index(mojang_slot) else {
        return false;
    };
    let any = target.cast_any();
    if let Some(villager) = any.downcast_ref::<VillagerEntity>() {
        if index >= MOB_INVENTORY_SIZE {
            return false;
        }
        let mut inventory = villager
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        pad_inventory_to(&mut inventory, index);
        inventory[index] = item_stack.clone();
        return true;
    }
    if let Some(trader) = any.downcast_ref::<WanderingTraderEntity>() {
        let inventory = &trader.merchant_inventory;
        return if index < inventory.size() {
            inventory.set_stack(index, item_stack.clone());
            true
        } else {
            false
        };
    }
    if let Some(piglin) = any.downcast_ref::<PiglinEntity>() {
        if index >= MOB_INVENTORY_SIZE {
            return false;
        }
        let mut inventory = piglin
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        pad_inventory_to(&mut inventory, index);
        inventory[index] = item_stack.clone();
        return true;
    }
    if let Some(pillager) = any.downcast_ref::<PillagerEntity>() {
        if index >= PILLAGER_INVENTORY_SIZE {
            return false;
        }
        let mut inventory = pillager
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        pad_inventory_to(&mut inventory, index);
        inventory[index] = item_stack.clone();
        return true;
    }
    false
}

/// Chest boats and container minecarts expose 0-based slots through vanilla
/// `ContainerEntity.getChestVehicleSlot`.
fn read_container_slot(target: &Arc<dyn EntityBase>, mojang_slot: usize) -> Option<ItemStack> {
    let inventory = target.get_container_inventory()?;
    (mojang_slot < inventory.size()).then(|| inventory.get_stack(mojang_slot))
}

fn write_container_slot(
    target: &Arc<dyn EntityBase>,
    mojang_slot: usize,
    item_stack: &ItemStack,
) -> bool {
    let Some(inventory) = target.get_container_inventory() else {
        return false;
    };
    if mojang_slot < inventory.size() {
        inventory.set_stack(mojang_slot, item_stack.clone());
        true
    } else {
        false
    }
}

fn read_extended_entity_slot(
    target: &Arc<dyn EntityBase>,
    mojang_slot: usize,
) -> Option<ItemStack> {
    if let Some(player) = target.get_player() {
        return read_player_menu_slot(player, mojang_slot);
    }
    if let Some(horse) = target.get_mob().and_then(|mob| mob.as_abstract_horse())
        && let Some(stack) = read_horse_slot(horse, mojang_slot)
    {
        return Some(stack);
    }
    if let Some(stack) = read_mob_slot(target, mojang_slot) {
        return Some(stack);
    }
    read_container_slot(target, mojang_slot)
}

fn write_extended_entity_slot(
    target: &Arc<dyn EntityBase>,
    mojang_slot: usize,
    item_stack: &ItemStack,
) -> bool {
    if let Some(player) = target.get_player() {
        return write_player_menu_slot(player, mojang_slot, item_stack);
    }
    if let Some(horse) = target.get_mob().and_then(|mob| mob.as_abstract_horse())
        && write_horse_slot(horse, mojang_slot, item_stack)
    {
        return true;
    }
    if write_mob_slot(target, mojang_slot, item_stack) {
        return true;
    }
    write_container_slot(target, mojang_slot, item_stack)
}

/// Reads the stack in a vanilla slot index of an entity, or `None` if it has no such slot.
fn read_entity_slot(target: &Arc<dyn EntityBase>, mojang_slot: usize) -> Option<ItemStack> {
    if let Some(stack) = read_extended_entity_slot(target, mojang_slot) {
        return Some(stack);
    }
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
    if write_extended_entity_slot(target, mojang_slot, item_stack) {
        return true;
    }
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
    // Mirrors vanilla Container.setChanged() so the container's NBT is saved.
    inventory.mark_dirty();
    Ok(1)
}

/// Applies an inline item modifier (a single function or a list of them). Only the
/// functions datapacks commonly use are implemented; others leave the stack as is.
fn apply_item_modifier(stack: &mut ItemStack, modifier: &NbtTag, ctx: &ProviderContext) {
    match modifier {
        NbtTag::List(list) => {
            for function in list {
                apply_item_modifier(stack, function, ctx);
            }
        }
        NbtTag::Compound(function) => {
            let name = function.get_string("function").unwrap_or_default();
            match name.strip_prefix("minecraft:").unwrap_or(name) {
                "set_damage" => apply_set_damage(stack, function, ctx),
                "set_count" => {
                    let amount =
                        number_provider_with(function.get("count"), ctx).unwrap_or(1.0) as i32;
                    let add = function.get_bool("add").unwrap_or(false);
                    let count = if add {
                        i32::from(stack.item_count) + amount
                    } else {
                        amount
                    };
                    // Vanilla setCount does not clamp to the item's max stack size.
                    stack.item_count = count.clamp(0, i32::from(u8::MAX)) as u8;
                }
                "set_name" => {
                    let Some(name) = function.get("name") else {
                        return;
                    };
                    let target = match function.get_string("target").unwrap_or("custom_name") {
                        "item_name" | "minecraft:item_name" => DataComponent::ItemName,
                        _ => DataComponent::CustomName,
                    };
                    set_component(stack, target, name);
                }
                "set_lore" => {
                    if let Some(lore) = function.get("lore") {
                        set_component(stack, DataComponent::Lore, lore);
                    }
                }
                "set_enchantments" => {
                    let add = function.get_bool("add").unwrap_or(false);
                    if let Some(NbtTag::Compound(enchantments)) = function.get("enchantments") {
                        for (name, level_tag) in &enchantments.child_tags {
                            let Some(enchantment) = Enchantment::from_name(name) else {
                                continue;
                            };
                            let level =
                                number_provider_with(Some(level_tag), ctx).unwrap_or(1.0) as i32;
                            set_enchantment_level(stack, enchantment, level, add);
                        }
                    }
                }
                "set_components" => {
                    if let Some(NbtTag::Compound(components)) = function.get("components") {
                        for (name, value) in &components.child_tags {
                            if let Some(name) = name.strip_prefix('!') {
                                if let Some(id) = DataComponent::try_from_name(name) {
                                    stack.patch.retain(|(existing, _)| *existing != id);
                                    stack.patch.push((id, None));
                                }
                            } else if let Some(id) = DataComponent::try_from_name(name) {
                                set_component(stack, id, value);
                            }
                        }
                    }
                }
                "enchant_randomly" => enchant_randomly(stack, function),
                "set_custom_model_data" => set_custom_model_data(stack, function),
                "set_potion" => {
                    if let Some(id) = function.get_string("id") {
                        let id = id.strip_prefix("minecraft:").unwrap_or(id);
                        if let Some(potion) = Potion::from_name(id) {
                            stack.set_data_component(PotionContentsImpl {
                                potion_id: Some(potion.id as i32),
                                custom_color: None,
                                custom_effects: Vec::new(),
                                custom_name: None,
                            });
                        }
                    }
                }
                // vanilla SequenceFunction: run each child function in order
                "sequence" => {
                    if let Some(functions) = function.get("functions") {
                        apply_item_modifier(stack, functions, ctx);
                    }
                }
                // vanilla LimitCount/IntLimit: clamp the count to optional min and max providers
                "limit_count" => apply_limit_count(stack, function, ctx),
                other => tracing::debug!("Unsupported item modifier function {other}"),
            }
        }
        _ => {}
    }
    // Vanilla `ItemCommands#applyModifier` calls `newItem.limitSize(newItem.getMaxStackSize())`.
    stack.item_count = stack.item_count.min(stack.get_max_stack_size());
}

/// Applies the vanilla `SetItemDamageFunction` to a stack.
fn apply_set_damage(stack: &mut ItemStack, function: &NbtCompound, ctx: &ProviderContext) {
    let Some(max) = stack.get_max_damage() else {
        return;
    };
    let amount = number_provider_with(function.get("damage"), ctx).unwrap_or(1.0);
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

/// Applies the vanilla `LimitCount` function, clamping the count to optional min and max providers.
fn apply_limit_count(stack: &mut ItemStack, function: &NbtCompound, ctx: &ProviderContext) {
    let Some(NbtTag::Compound(limit)) = function.get("limit") else {
        return;
    };
    let min = number_provider_with(limit.get("min"), ctx).map(|v| v as i32);
    let max = number_provider_with(limit.get("max"), ctx).map(|v| v as i32);
    let mut count = i32::from(stack.item_count);
    if let Some(min) = min
        && count < min
    {
        count = min;
    }
    if let Some(max) = max
        && count > max
    {
        count = max;
    }
    stack.item_count = count.clamp(0, i32::from(u8::MAX)) as u8;
}

/// Applies the vanilla `EnchantRandomlyFunction`, picking one enchantment.
fn enchant_randomly(stack: &mut ItemStack, function: &NbtCompound) {
    let candidates: Vec<&'static Enchantment> = function.get("options").map_or_else(
        || Enchantment::ALL.to_vec(),
        |tag| {
            tag.extract_list()
                .unwrap_or(&[])
                .iter()
                .filter_map(NbtTag::extract_string)
                .filter_map(|name| {
                    Enchantment::from_name(name.strip_prefix("minecraft:").unwrap_or(name))
                })
                .collect()
        },
    );
    if candidates.is_empty() {
        return;
    }
    let only_compatible = function.get_bool("only_compatible").unwrap_or(true);
    let is_book = stack.item.id == Item::BOOK.id;
    let compatible: Vec<&'static Enchantment> = candidates
        .into_iter()
        .filter(|enc| is_book || !only_compatible || enc.can_enchant(stack.item))
        .collect();
    if compatible.is_empty() {
        return;
    }
    let picked = compatible[rand::rng().random_range(0..compatible.len())];
    let level = rand::rng().random_range(1..=picked.max_level.max(1));
    if is_book {
        stack.item = &Item::ENCHANTED_BOOK;
        set_stored_enchantment_level(stack, picked, level, true);
    } else {
        set_enchantment_level(stack, picked, level, true);
    }
}

/// Sets a data component from its SNBT representation, matching the vanilla
/// component codecs used by `set_name`, `set_lore` and `set_components`.
fn set_component(stack: &mut ItemStack, id: DataComponent, data: &NbtTag) {
    if let Some(component) = read_data(id, data) {
        stack.patch.retain(|(existing, _)| *existing != id);
        stack.patch.push((id, Some(component)));
    }
}

/// Sets or adds an enchantment level like vanilla `SetEnchantmentsFunction`.
fn set_enchantment_level(
    stack: &mut ItemStack,
    enchantment: &'static Enchantment,
    level: i32,
    add: bool,
) {
    let level = level.clamp(0, 255);
    let combined = if add {
        (stack.get_enchantment_level(enchantment) + level).clamp(0, 255)
    } else {
        level
    };
    if let Some(data) = stack.get_data_component_mut::<EnchantmentsImpl>() {
        let entries = data.enchantment.to_mut();
        let mut found = false;
        for (enc, old_level) in entries.iter_mut() {
            if *enc == enchantment {
                *old_level = combined;
                found = true;
                break;
            }
        }
        if !found {
            entries.push((enchantment, combined));
        }
    } else {
        stack.set_data_component(EnchantmentsImpl {
            enchantment: Cow::Owned(vec![(enchantment, combined)]),
        });
    }
}

/// Applies vanilla list operations to the custom model data component.
fn set_custom_model_data(stack: &mut ItemStack, function: &NbtCompound) {
    let current = stack
        .get_data_component::<CustomModelDataImpl>()
        .cloned()
        .unwrap_or_else(|| CustomModelDataImpl {
            floats: Vec::new(),
            flags: Vec::new(),
            strings: Vec::new(),
            colors: Vec::new(),
        });
    stack.set_data_component(CustomModelDataImpl {
        floats: apply_list_operation(
            current.floats,
            function.get("floats"),
            NbtTag::as_numeric_float,
        ),
        flags: apply_list_operation(current.flags, function.get("flags"), NbtTag::extract_bool),
        strings: apply_list_operation(current.strings, function.get("strings"), |tag| {
            tag.extract_string().map(str::to_string)
        }),
        colors: apply_list_operation(current.colors, function.get("colors"), read_color),
    });
}

/// Applies a vanilla list operation (values plus a mode) to a component list.
fn apply_list_operation<T: Clone>(
    current: Vec<T>,
    field: Option<&NbtTag>,
    parse: impl Fn(&NbtTag) -> Option<T>,
) -> Vec<T> {
    let Some(field) = field else {
        return current;
    };
    let Some(operation) = field.extract_compound() else {
        return current;
    };
    let Some(values) = operation.get("values").and_then(NbtTag::extract_list) else {
        return current;
    };
    let replacement: Vec<T> = values.iter().filter_map(parse).collect();
    match operation.get_string("mode").unwrap_or("replace_all") {
        "append" => {
            let mut result = current;
            result.extend(replacement);
            result
        }
        "insert" => {
            let offset = operation
                .get_int("offset")
                .unwrap_or(0)
                .clamp(0, current.len() as i32) as usize;
            let mut result = Vec::with_capacity(current.len() + replacement.len());
            result.extend_from_slice(&current[..offset]);
            result.extend(replacement);
            result.extend_from_slice(&current[offset..]);
            result
        }
        "replace_section" => {
            let offset = operation
                .get_int("offset")
                .unwrap_or(0)
                .clamp(0, current.len() as i32) as usize;
            let size = operation
                .get_int("size")
                .unwrap_or(replacement.len() as i32)
                .max(0) as usize;
            let mut result = Vec::with_capacity(current.len() + replacement.len());
            result.extend_from_slice(&current[..offset]);
            result.extend(replacement);
            let resume = offset.saturating_add(size).min(current.len());
            result.extend_from_slice(&current[resume..]);
            result
        }
        _ => replacement,
    }
}

/// Converts an SNBT color into its ARGB integer form.
fn read_color(tag: &NbtTag) -> Option<i32> {
    if let Some(rgb) = tag.as_numeric_int() {
        return Some(rgb);
    }
    let channels = tag.extract_list()?;
    if channels.len() != 3 {
        return None;
    }
    let r = channels[0].as_numeric_float()?;
    let g = channels[1].as_numeric_float()?;
    let b = channels[2].as_numeric_float()?;
    Some(pumpkin_util::text::color::ARGBColor::from_float(1.0, r, g, b).to_argb_int())
}

/// Adds a stored enchantment to an enchanted-book stack.
fn set_stored_enchantment_level(
    stack: &mut ItemStack,
    enchantment: &'static Enchantment,
    level: i32,
    add: bool,
) {
    let level = level.clamp(0, 255);
    let mut entries: Vec<(&'static Enchantment, i32)> = stack
        .get_data_component::<StoredEnchantmentsImpl>()
        .map_or_else(Vec::new, |data| data.enchantment.to_vec());
    let combined = if add {
        entries
            .iter()
            .find(|(enc, _)| *enc == enchantment)
            .map_or(level, |(_, old)| (*old + level).clamp(0, 255))
    } else {
        level
    };
    if let Some((_, old)) = entries.iter_mut().find(|(enc, _)| *enc == enchantment) {
        *old = combined;
    } else {
        entries.push((enchantment, combined));
    }
    stack.set_data_component(StoredEnchantmentsImpl {
        enchantment: Cow::Owned(entries),
    });
}

/// Scoreboard and command-storage access that vanilla exposes through a
/// `LootContext`. Item modifiers supply these; tests use the empty default.
#[derive(Default)]
struct ProviderContext<'a> {
    server: Option<&'a Arc<Server>>,
    entity: Option<&'a Arc<dyn EntityBase>>,
}

impl ProviderContext<'_> {
    /// Resolves a vanilla `score` provider, or its fallback when no score exists.
    fn resolve_score(&self, c: &NbtCompound) -> f32 {
        let fallback = number_provider(c.get("fallback")).unwrap_or(0.0);
        let objective = c.get_string("score").unwrap_or("");
        let value = self
            .server
            .and_then(|server| {
                self.scoreboard_name(c.get("target"))
                    .map(|name| (server, name))
            })
            .and_then(|(server, name)| {
                server
                    .scoreboard
                    .lock()
                    .ok()
                    .and_then(|guard| guard.get_score_value(&name, objective))
            })
            .unwrap_or(fallback as i32);
        value as f32
    }

    /// Resolves a vanilla `storage` provider, or its fallback when no numeric tag exists.
    fn resolve_storage(&self, c: &NbtCompound) -> f32 {
        let fallback = number_provider(c.get("fallback")).unwrap_or(0.0);
        self.server
            .and_then(|server| {
                let storage = c.get_string("storage")?;
                let path = c.get_string("path")?;
                Some((server, storage, path))
            })
            .and_then(|(server, storage, path)| {
                let guard = server.command_storage.lock().ok()?;
                let root = guard.get(storage)?;
                let mut reader = StringReader::new(path.to_string());
                let parsed = <NbtPathArgumentType as ArgumentType<DummySource>>::parse(
                    &NbtPathArgumentType,
                    &mut reader,
                )
                .ok()?;
                let matches = parsed.get(&NbtTag::Compound(root.clone())).ok()?;
                matches.first().and_then(number_as_f32)
            })
            .unwrap_or(fallback)
    }

    /// Resolves the scoreboard holder name a vanilla `target` refers to.
    fn scoreboard_name(&self, target: Option<&NbtTag>) -> Option<String> {
        match target? {
            NbtTag::String(target) => {
                let target: &str = target;
                if target == "this" || target == "minecraft:this" {
                    self.entity.map(|e| e.get_scoreboard_name())
                } else {
                    None
                }
            }
            NbtTag::Compound(c) => {
                let kind = c.get_string("type")?;
                let kind = kind.strip_prefix("minecraft:").unwrap_or(kind);
                match kind {
                    "fixed" => c.get_string("name").map(str::to_string),
                    "context" => {
                        let target = c.get_string("target")?;
                        if target == "this" || target == "minecraft:this" {
                            self.entity.map(|e| e.get_scoreboard_name())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// Reads a numeric provider: a plain number, or a typed provider such as
/// constant, uniform, binomial, sum, score or storage (vanilla `NumberProvider`).
fn number_provider(tag: Option<&NbtTag>) -> Option<f32> {
    number_provider_with(tag, &ProviderContext::default())
}

/// Context-aware numeric provider used by item modifiers.
fn number_provider_with(tag: Option<&NbtTag>, ctx: &ProviderContext) -> Option<f32> {
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
                    let min = number_provider_with(c.get("min"), ctx)?;
                    let max = number_provider_with(c.get("max"), ctx)?;
                    Some(min + rand::random::<f32>() * (max - min))
                }
                "binomial" => {
                    let n = number_provider_with(c.get("n"), ctx)? as i32;
                    let p = number_provider_with(c.get("p"), ctx)?;
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
                    inputs.iter().try_fold(0.0f32, |total, input| {
                        Some(total + number_provider_with(Some(input), ctx)?)
                    })
                }
                "score" => Some(ctx.resolve_score(c)),
                "storage" => Some(ctx.resolve_storage(c)),
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
        let provider_ctx = ProviderContext {
            server: Some(context.source.server()),
            entity: context.source.entity.as_ref(),
        };
        let modified = set_block_slot(context, &pos, slot, |mut stack| {
            apply_item_modifier(&mut stack, &modifier, &provider_ctx);
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
        let provider_ctx = ProviderContext {
            server: Some(context.source.server()),
            entity: context.source.entity.as_ref(),
        };
        let mut count = 0;
        for target in &targets {
            if let Some(mut stack) = read_entity_slot(target, slot) {
                apply_item_modifier(&mut stack, &modifier, &provider_ctx);
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
                ClientParsed::new(SlotsArgumentType, JavaClientArgumentType::SlotSource),
            )
            .then(literal("with").then(block_item_node(SuccessFeedback::Replace))),
        ),
    );

    let entity_node = literal("entity").then(
        argument("target", EntityArgumentType::Entities).then(
            argument(
                "slots",
                ClientParsed::new(SlotsArgumentType, JavaClientArgumentType::SlotSource),
            )
            .then(literal("with").then(entity_item_node(SuccessFeedback::Replace))),
        ),
    );

    let block_set_node = literal("block").then(
        argument("target", BlockPosArgumentType)
            .then(argument("slots", SlotArgumentType).then(block_item_node(SuccessFeedback::Set))),
    );

    let entity_set_node = literal("entity")
        .then(argument("target", EntityArgumentType::Entities).then(
            argument("slots", SlotArgumentType).then(entity_item_node(SuccessFeedback::Set)),
        ));

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
            .then(literal("set").then(block_set_node).then(entity_set_node))
            .then(modify_node),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mob_inventory_slots_map_to_zero_based_index() {
        assert_eq!(mob_inventory_index(300), Some(0));
        assert_eq!(mob_inventory_index(307), Some(7));
        assert_eq!(mob_inventory_index(299), None);
        assert_eq!(mob_inventory_index(308), None);
    }

    #[test]
    fn pad_inventory_to_fills_unallocated_slots_with_air() {
        let mut inventory = Vec::new();
        pad_inventory_to(&mut inventory, 4);
        assert_eq!(inventory.len(), 5);
        assert!(inventory.iter().all(ItemStack::is_empty));

        let diamond = ItemStack::new(1, &Item::DIAMOND);
        inventory[2] = diamond.clone();
        pad_inventory_to(&mut inventory, 1);
        assert_eq!(inventory.len(), 5);
        assert!(inventory[2].are_items_and_components_equal(&diamond));
    }

    #[test]
    fn uniform_number_provider_samples_within_min_and_max() {
        let mut compound = NbtCompound::new();
        compound.put_string("type", "uniform".to_string());
        compound.put_float("min", 2.0);
        compound.put_float("max", 5.0);
        let tag = NbtTag::Compound(compound);
        for _ in 0..256 {
            let sample = number_provider(Some(&tag));
            assert!(
                sample.is_some_and(|value| (2.0..5.0).contains(&value)),
                "sample {sample:?} out of [2, 5)"
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

    #[test]
    fn set_name_sets_custom_and_item_name() {
        let name_tag = NbtTag::String("{\"text\":\"Hello\"}".to_string().into());
        let mut function = NbtCompound::new();
        function.put_string("function", "minecraft:set_name".to_string());
        function.put("name", name_tag.clone());
        let mut stack = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function),
            &ProviderContext::default(),
        );
        assert_eq!(
            stack.get_custom_name().map(|name| name.clone().get_text()),
            Some("Hello".to_string())
        );

        let mut item_name_function = NbtCompound::new();
        item_name_function.put_string("function", "minecraft:set_name".to_string());
        item_name_function.put("name", name_tag);
        item_name_function.put_string("target", "item_name".to_string());
        let mut stack = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(item_name_function),
            &ProviderContext::default(),
        );
        assert!(
            stack
                .patch
                .iter()
                .any(|(id, _)| *id == DataComponent::ItemName)
        );
    }

    #[test]
    fn set_enchantments_sets_and_adds_levels() {
        let mut function = NbtCompound::new();
        function.put_string("function", "minecraft:set_enchantments".to_string());
        let mut enchantments = NbtCompound::new();
        enchantments.put("minecraft:sharpness", NbtTag::Int(2));
        function.put("enchantments", NbtTag::Compound(enchantments));

        let mut stack = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function.clone()),
            &ProviderContext::default(),
        );
        assert_eq!(stack.get_enchantment_level(&Enchantment::SHARPNESS), 2);

        function.put_bool("add", true);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function),
            &ProviderContext::default(),
        );
        assert_eq!(stack.get_enchantment_level(&Enchantment::SHARPNESS), 4);
    }

    #[test]
    fn set_components_sets_and_removes() {
        let mut function = NbtCompound::new();
        function.put_string("function", "minecraft:set_components".to_string());
        let mut components = NbtCompound::new();
        components.put("minecraft:max_stack_size", NbtTag::Int(8));
        components.put("!minecraft:glider", NbtTag::Byte(1));
        function.put("components", NbtTag::Compound(components));

        let mut stack = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function),
            &ProviderContext::default(),
        );
        assert_eq!(stack.get_max_stack_size(), 8);
        assert!(
            stack
                .patch
                .iter()
                .any(|(id, value)| *id == DataComponent::Glider && value.is_none())
        );
    }
    #[test]
    fn response_counts_match_vanilla_non_zero_dispatch() {
        // (total changed slots, non-zero target count, sole changed target index)
        assert_eq!(response_counts(&[0, 0, 0]), (0, 0, None));
        assert_eq!(response_counts(&[3, 0, 2]), (5, 2, None));
        assert_eq!(response_counts(&[0, 4, 0]), (4, 1, Some(1)));
        assert_eq!(response_counts(&[7]), (7, 1, Some(0)));
    }

    #[test]
    fn enchant_randomly_uses_options_and_respects_compatibility() {
        let mut function = NbtCompound::new();
        function.put_string("function", "minecraft:enchant_randomly".to_string());
        function.put(
            "options",
            NbtTag::List(vec![
                NbtTag::String("minecraft:sharpness".to_string().into()),
                NbtTag::String("minecraft:aqua_affinity".to_string().into()),
            ]),
        );
        let mut stack = ItemStack::new(1, &Item::DIAMOND_SWORD);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function),
            &ProviderContext::default(),
        );
        assert!(stack.get_enchantment_level(&Enchantment::SHARPNESS) >= 1);
        assert_eq!(stack.get_enchantment_level(&Enchantment::AQUA_AFFINITY), 0);
    }

    #[test]
    fn enchant_randomly_converts_book_to_stored_enchantments() {
        let mut function = NbtCompound::new();
        function.put_string("function", "minecraft:enchant_randomly".to_string());
        function.put(
            "options",
            NbtTag::List(vec![NbtTag::String(
                "minecraft:sharpness".to_string().into(),
            )]),
        );
        let mut stack = ItemStack::new(1, &Item::BOOK);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function),
            &ProviderContext::default(),
        );
        assert_eq!(stack.item.id, Item::ENCHANTED_BOOK.id);
        let stored = stack.get_data_component::<StoredEnchantmentsImpl>();
        assert!(stored.is_some());
        if let Some(stored) = stored {
            assert!(
                stored
                    .enchantment
                    .iter()
                    .any(|(enc, level)| { **enc == Enchantment::SHARPNESS && *level >= 1 })
            );
        }
    }

    #[test]
    fn set_custom_model_data_applies_list_operations() {
        let mut function = NbtCompound::new();
        function.put_string("function", "minecraft:set_custom_model_data".to_string());
        let mut append = NbtCompound::new();
        append.put_string("mode", "append".to_string());
        append.put("values", NbtTag::List(vec![NbtTag::Float(0.5)]));
        function.put("floats", NbtTag::Compound(append));

        let mut stack = ItemStack::new(1, &Item::STONE);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function.clone()),
            &ProviderContext::default(),
        );
        let data = stack.get_data_component::<CustomModelDataImpl>();
        assert!(data.is_some());
        if let Some(data) = data {
            assert_eq!(data.floats, vec![0.5]);
        }

        let mut replace = NbtCompound::new();
        replace.put_string("mode", "replace_all".to_string());
        replace.put(
            "values",
            NbtTag::List(vec![NbtTag::Float(1.0), NbtTag::Float(2.0)]),
        );
        function.put("floats", NbtTag::Compound(replace));
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function),
            &ProviderContext::default(),
        );
        let data = stack.get_data_component::<CustomModelDataImpl>();
        assert!(data.is_some());
        if let Some(data) = data {
            assert_eq!(data.floats, vec![1.0, 2.0]);
        }
    }

    #[test]
    fn set_potion_sets_potion_contents() {
        let mut function = NbtCompound::new();
        function.put_string("function", "minecraft:set_potion".to_string());
        function.put_string("id", "minecraft:swiftness".to_string());
        let mut stack = ItemStack::new(1, &Item::POTION);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function),
            &ProviderContext::default(),
        );
        let contents = stack.get_data_component::<PotionContentsImpl>();
        assert!(contents.is_some());
        if let Some(contents) = contents {
            assert_eq!(
                contents.potion_id,
                Some(pumpkin_data::potion::Potion::SWIFTNESS.id as i32)
            );
        }
    }

    #[test]
    fn sequence_runs_children_then_limit_count_clamps() {
        let mut count = NbtCompound::new();
        count.put_string("function", "minecraft:set_count".to_string());
        count.put("count", NbtTag::Int(40));

        let mut limit = NbtCompound::new();
        limit.put("max", NbtTag::Int(5));
        let mut limit_fn = NbtCompound::new();
        limit_fn.put_string("function", "minecraft:limit_count".to_string());
        limit_fn.put("limit", NbtTag::Compound(limit));

        let mut sequence = NbtCompound::new();
        sequence.put_string("function", "minecraft:sequence".to_string());
        sequence.put(
            "functions",
            NbtTag::List(vec![NbtTag::Compound(count), NbtTag::Compound(limit_fn)]),
        );

        let mut stack = ItemStack::new(1, &Item::STONE);
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(sequence),
            &ProviderContext::default(),
        );
        assert_eq!(stack.item_count, 5);
    }

    #[test]
    fn set_count_clamps_to_max_stack_size_like_vanilla_limit_size() {
        let mut function = NbtCompound::new();
        function.put_string("function", "minecraft:set_count".to_string());
        function.put("count", NbtTag::Int(100));

        let mut stack = ItemStack::new(1, &Item::STONE);
        let max = stack.get_max_stack_size();
        apply_item_modifier(
            &mut stack,
            &NbtTag::Compound(function),
            &ProviderContext::default(),
        );
        assert_eq!(stack.item_count, max);
    }

    #[test]
    fn score_and_storage_providers_fall_back_without_context() {
        let mut score = NbtCompound::new();
        score.put_string("type", "score".into());
        score.put_string("score", "objective".into());
        score.put("target", NbtTag::String("this".into()));
        score.put("fallback", NbtTag::Int(7));
        assert_eq!(number_provider(Some(&NbtTag::Compound(score))), Some(7.0));

        let mut storage = NbtCompound::new();
        storage.put_string("type", "storage".into());
        storage.put_string("storage", "minecraft:test".into());
        storage.put_string("path", "value".into());
        storage.put("fallback", NbtTag::Int(3));
        assert_eq!(number_provider(Some(&NbtTag::Compound(storage))), Some(3.0));
    }
}
