use std::sync::Arc;

use pumpkin_data::data_component_impl::{EnchantmentsImpl, EquipmentSlot};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Enchantment, translation};
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::resource::{ENCHANTMENT_ARGUMENT, ResourceArgument};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::entity::player::Player;

const DESCRIPTION: &str = "Adds an enchantment to a player's selected item, subject to the same restrictions as an anvil. Also works on any mob or entity holding a weapon/tool/armor in its main hand.";
const PERMISSION: &str = "minecraft:command.enchant";

const ERROR_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_ENCHANT_FAILED,
    translation::java::COMMANDS_ENCHANT_FAILED,
);

const ERROR_FAILED_ENTITY: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_ENCHANT_FAILED_ENTITY,
    translation::java::COMMANDS_ENCHANT_FAILED_ENTITY,
);

const ERROR_FAILED_LEVEL: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_ENCHANT_FAILED_LEVEL,
    translation::java::COMMANDS_ENCHANT_FAILED_LEVEL,
);

const ERROR_FAILED_ITEMLESS: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_ENCHANT_FAILED_ITEMLESS,
    translation::java::COMMANDS_ENCHANT_FAILED_ITEMLESS,
);

const ERROR_FAILED_INCOMPATIBLE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_ENCHANT_FAILED_INCOMPATIBLE,
    translation::java::COMMANDS_ENCHANT_FAILED_INCOMPATIBLE,
);

/// Applies the enchantment to the held stack, mirroring vanilla `EnchantCommand`:
/// an empty hand, an enchantment that cannot apply to the item, or one that
/// conflicts with the item's existing enchantments is rejected.
fn enchant_held_stack(
    item: &mut ItemStack,
    enchantment: &'static Enchantment,
    level: i32,
    target_name: &TextComponent,
) -> Result<(), CommandSyntaxError> {
    if item.is_empty() {
        return Err(ERROR_FAILED_ITEMLESS.create_without_context(target_name.clone()));
    }

    if !enchantment.can_enchant(item.item) {
        return Err(ERROR_FAILED_INCOMPATIBLE.create_without_context(item.item.translated_name()));
    }

    if let Some(data) = item.get_data_component::<EnchantmentsImpl>()
        && !enchantment.is_enchantment_compatible(data)
    {
        return Err(ERROR_FAILED_INCOMPATIBLE.create_without_context(item.item.translated_name()));
    }

    item.enchant(enchantment, level);
    Ok(())
}

fn enchant_player(
    player: &Player,
    enchantment: &'static Enchantment,
    level: i32,
) -> Result<(), CommandSyntaxError> {
    let mut item = player.inventory().held_item();
    enchant_held_stack(&mut item, enchantment, level, &player.get_display_name())?;
    let inventory = player.inventory();
    inventory.set_held_item(item.clone());

    player.sync_hand_slot(inventory.get_selected_slot() as usize, item);

    Ok(())
}

fn enchant_target(
    target: &Arc<dyn EntityBase>,
    enchantment: &'static Enchantment,
    level: i32,
) -> Result<(), CommandSyntaxError> {
    if let Some(player) = target.get_player() {
        return enchant_player(player, enchantment, level);
    }

    let Some(living) = target.get_living_entity() else {
        return Err(ERROR_FAILED_ENTITY.create_without_context(target.get_display_name()));
    };

    let mut item = living.held_item(target.as_ref());
    enchant_held_stack(&mut item, enchantment, level, &target.get_display_name())?;
    living
        .entity_equipment
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .put(&EquipmentSlot::MAIN_HAND, item.clone());
    living.send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, item)]);

    Ok(())
}

/// Vanilla rejects only levels strictly above the enchantment's maximum.
fn level_exceeds_max(level: i32, max_level: i32) -> bool {
    level > max_level
}

struct EnchantExecutor {
    has_level: bool,
}

impl CommandExecutor for EnchantExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let enchantment = ResourceArgument::get_enchantment(context, "enchantment")?;
        let level = if self.has_level {
            IntegerArgumentType::get(context, "level")?
        } else {
            1
        };

        if level_exceeds_max(level, enchantment.max_level) {
            return Err(ERROR_FAILED_LEVEL.create_without_context(
                TextComponent::text(level.to_string()),
                TextComponent::text(enchantment.max_level.to_string()),
            ));
        }

        let mut successful_targets = 0;

        if targets.len() == 1 {
            enchant_target(&targets[0], enchantment, level)?;
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_ENCHANT_SUCCESS_SINGLE,
                translation::bedrock::COMMANDS_ENCHANT_SUCCESS,
                [
                    enchantment.get_fullname(level),
                    targets[0].as_ref().get_display_name(),
                ],
            );
            context.source.send_feedback(msg, true);
            return Ok(1);
        }

        for target in &targets {
            if enchant_target(target, enchantment, level).is_ok() {
                successful_targets += 1;
            }
        }

        if successful_targets == 0 {
            return Err(ERROR_FAILED.create_without_context());
        }

        let msg = TextComponent::translate_cross(
            translation::java::COMMANDS_ENCHANT_SUCCESS_MULTIPLE,
            translation::bedrock::COMMANDS_ENCHANT_SUCCESS,
            [
                enchantment.get_fullname(level),
                TextComponent::text(targets.len().to_string()),
            ],
        );
        context.source.send_feedback(msg, true);

        Ok(successful_targets)
    }
}

#[cfg(test)]
mod tests {
    use super::level_exceeds_max;

    #[test]
    fn level_at_max_is_accepted_and_above_is_rejected() {
        assert!(!level_exceeds_max(5, 5));
        assert!(!level_exceeds_max(0, 5));
        assert!(level_exceeds_max(6, 5));
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("enchant", DESCRIPTION).requires(PERMISSION).then(
            argument("targets", EntityArgumentType::Entities).then(
                argument("enchantment", ENCHANTMENT_ARGUMENT.clone())
                    .executes(EnchantExecutor { has_level: false })
                    .then(
                        argument("level", IntegerArgumentType::with_min(0))
                            .executes(EnchantExecutor { has_level: true }),
                    ),
            ),
        ),
    );
}
