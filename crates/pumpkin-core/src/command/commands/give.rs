use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::item::ItemStackArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::item::ItemEntity;

const DESCRIPTION: &str = "Give items to player(s).";
const PERMISSION: &str = "minecraft:command.give";

const ERROR_TOO_MANY_ITEMS: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_GIVE_FAILED_TOOMANYITEMS,
    translation::java::COMMANDS_GIVE_FAILED_TOOMANYITEMS,
);

/// `GiveCommand.MAX_ALLOWED_ITEMSTACKS`
const MAX_ALLOWED_ITEMSTACKS: i32 = 100;

/// Item component with a hover tooltip, as vanilla `ItemStack.getDisplayName`.
fn item_display(stack: &ItemStack) -> TextComponent {
    let id = stack.item.registry_key.to_string();
    TextComponent::text(id.clone()).hover_event(HoverEvent::ShowItem {
        id: id.into(),
        count: Some(i32::from(stack.item_count)),
        components: None,
    })
}

struct GiveExecutor {
    has_count: bool,
}

impl CommandExecutor for GiveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let parsed_stack = ItemStackArgumentType::get(context, "item")?;

        let item_count = if self.has_count {
            IntegerArgumentType::get(context, "count")?
        } else {
            1
        };

        let max_stack = i32::from(parsed_stack.get_max_stack_size());
        // Without the cap a count like i32::MAX would drop billions of item entities.
        let max_count = max_stack * MAX_ALLOWED_ITEMSTACKS;
        if item_count > max_count {
            return Err(ERROR_TOO_MANY_ITEMS.create_without_context_args_slice(&[
                TextComponent::text(max_count.to_string()),
                item_display(&parsed_stack),
            ]));
        }

        for target in &targets {
            let mut remaining = item_count;

            while remaining > 0 {
                let take = remaining.min(max_stack);
                let mut stack = parsed_stack.clone();
                stack.item_count = take as u8;
                target.inventory().insert_stack_anywhere(&mut stack);
                if stack.is_empty() {
                    // Vanilla GiveCommand: the all-fits branch spawns a fake, never-pickable
                    // item so the client animates the pickup, then plays the sound and
                    // broadcasts the inventory change.
                    let pos = target.get_entity().pos.load();
                    let item_pos =
                        pos + Vector3::new(0.0, target.get_entity().get_eye_height() - 0.3, 0.0);
                    let fake = ItemEntity::new(
                        Entity::new(target.world(), item_pos, &EntityType::ITEM),
                        parsed_stack.clone(),
                    );
                    fake.make_fake_item();
                    target.world().spawn_entity(Arc::new(fake));
                    target.world().play_sound_fine(
                        Sound::EntityItemPickup,
                        SoundCategory::Players,
                        &pos,
                        0.2,
                        (rand::random::<f32>() - rand::random::<f32>()).mul_add(0.7, 1.0) * 2.0,
                    );
                    target.sync_inventory_to_client();
                } else {
                    // Vanilla GiveCommand: the stack that does not fit drops with no pickup
                    // delay and is restricted to the player it was given to.
                    let dropped = target.drop_item(stack);
                    dropped.set_pickup_delay(0);
                    dropped.set_target(target.gameprofile.id);
                }
                remaining -= take;
            }
        }

        let msg = if targets.len() == 1 {
            TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_GIVE_SUCCESS_SINGLE,
                pumpkin_data::translation::bedrock::COMMANDS_GIVE_SUCCESS,
                [
                    TextComponent::text(item_count.to_string()),
                    item_display(&parsed_stack),
                    targets[0].as_ref().get_display_name(),
                ],
            )
        } else {
            TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_GIVE_SUCCESS_MULTIPLE,
                pumpkin_data::translation::bedrock::COMMANDS_GIVE_SUCCESS,
                [
                    TextComponent::text(item_count.to_string()),
                    item_display(&parsed_stack),
                    TextComponent::text(targets.len().to_string()),
                ],
            )
        };
        context.source.send_feedback(msg, true);

        Ok(targets.len() as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("give", DESCRIPTION).requires(PERMISSION).then(
            argument("targets", EntityArgumentType::Players).then(
                argument("item", ItemStackArgumentType)
                    .executes(GiveExecutor { has_count: false })
                    .then(
                        argument("count", IntegerArgumentType::with_min(1))
                            .executes(GiveExecutor { has_count: true }),
                    ),
            ),
        ),
    );
}
