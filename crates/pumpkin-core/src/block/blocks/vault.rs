use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{VaultLikeProperties, VaultState};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::vault::VaultBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, OnPlaceArgs, PlacedArgs, UseWithItemArgs};
use crate::entity::item::ItemEntity;
use crate::entity::{Entity, EntityBase};
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};

#[pumpkin_block("minecraft:vault")]
pub struct VaultBlock;

impl BlockBehaviour for VaultBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = VaultLikeProperties::default(args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = VaultBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }

    #[expect(
        clippy::too_many_lines,
        reason = "single use flow matches vanilla branch order"
    )]
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if args.item_stack.is_empty() {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = VaultLikeProperties::from_state_id(state_id);
        if props.vault_state != VaultState::Active {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return BlockActionResult::PassToDefaultBlockAction;
        };
        let Some(vault_entity) = block_entity.as_any().downcast_ref::<VaultBlockEntity>() else {
            return BlockActionResult::PassToDefaultBlockAction;
        };

        let item_id = args.item_stack.item.id;
        let is_valid_key = if props.ominous {
            item_id == Item::OMINOUS_TRIAL_KEY.id
        } else {
            item_id == Item::TRIAL_KEY.id
        };

        if !is_valid_key {
            args.world.play_sound(
                Sound::BlockVaultInsertItemFail,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            return BlockActionResult::SuccessServer;
        }

        let player_uuid = args.player.gameprofile.id;

        if vault_entity.has_rewarded(&player_uuid) {
            args.world.play_sound(
                Sound::BlockVaultRejectRewardedPlayer,
                SoundCategory::Blocks,
                &args.position.to_f64(),
            );
            return BlockActionResult::SuccessServer;
        }

        vault_entity.mark_rewarded(player_uuid);

        args.item_stack
            .decrement_unless_creative(args.player.gamemode.load(), 1);

        args.world.play_sound(
            Sound::BlockVaultInsertItem,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );
        args.world.play_sound(
            Sound::BlockVaultOpenShutter,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );

        props.vault_state = VaultState::Ejecting;
        args.world.set_block_state(
            args.position,
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL,
        );

        args.world.play_sound(
            Sound::BlockVaultEjectItem,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );

        let custom_loot_table = vault_entity
            .config
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|cfg| cfg.get_string("loot_table").map(str::to_owned));
        let default_loot_table = if props.ominous {
            "minecraft:chests/trial_chambers/reward_ominous"
        } else {
            "minecraft:chests/trial_chambers/reward"
        };
        let loot_table_key = custom_loot_table.as_deref().unwrap_or(default_loot_table);
        let loot_stacks = args
            .world
            .get_loot_table(loot_table_key)
            .map(|handle| {
                let params = LootContextParameters {
                    world: Some(args.world),
                    position: Some(args.position.to_centered_f64()),
                    this_entity: Some(args.player.as_ref() as &dyn EntityBase),
                    ..Default::default()
                };
                generate_loot_from_handle(&handle, rand::random(), &params)
            })
            .unwrap_or_default();

        let mut dispense_event =
            crate::plugin::block::block_dispense_loot::BlockDispenseLootEvent::new(
                *args.position,
                args.world.clone(),
                loot_stacks,
            );
        if let Some(server) = args.world.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut dispense_event);
        }
        // Vanilla `VaultBlockEntity.Server.unlock` shows the next item to eject, and the
        // VaultDisplayItemEvent lets plugins replace that displayed item.
        let display_item = dispense_event
            .items
            .last()
            .cloned()
            .unwrap_or_else(|| (*ItemStack::EMPTY).clone());
        let mut display_event =
            crate::plugin::block::vault_display_item::VaultDisplayItemEvent::new(
                *args.position,
                args.world.clone(),
                display_item,
            );
        if let Some(server) = args.world.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut display_event);
        }
        if !display_event.cancelled {
            vault_entity.set_display_item(&display_event.item);
        }

        for stack in dispense_event.items {
            if stack.is_empty() {
                continue;
            }
            let spawn_pos = ejection_spawn_position(args.position);
            let entity = Entity::new(args.world.clone(), spawn_pos, &EntityType::ITEM);
            let mut item_event =
                crate::plugin::api::events::entity::item_spawn::ItemSpawnEvent::new(
                    entity.entity_id,
                    spawn_pos,
                    stack.item.registry_key.to_string(),
                );
            if let Some(server) = args.world.server.upgrade() {
                server
                    .plugin_manager
                    .fire_blocking(&server, &mut item_event);
            }
            if item_event.cancelled {
                continue;
            }
            // Vanilla `DefaultDispenseItemBehavior#spawnItem`, speed 2 and Direction.UP.
            let velocity = Vector3::new(
                rand::random::<f64>().mul_add(0.015, -0.0075),
                0.6 + rand::random::<f64>().mul_add(0.015, -0.0075),
                rand::random::<f64>().mul_add(0.015, -0.0075),
            );
            let item_entity = Arc::new(ItemEntity::new_with_velocity(entity, stack, velocity, 10));
            args.world.spawn_entity(item_entity);
        }

        props.vault_state = VaultState::Active;
        args.world.set_block_state(
            args.position,
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL,
        );

        BlockActionResult::SuccessServer
    }
}

/// Vanilla `VaultState#ejectResultItem`: `Vec3.atBottomCenterOf(pos).relative(Direction.UP, 1.2)`.
fn ejection_spawn_position(pos: &BlockPos) -> Vector3<f64> {
    Vector3::new(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y) + 1.2,
        f64::from(pos.0.z) + 0.5,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ejection_spawns_at_bottom_center_raised_to_vanilla_height() {
        let spawn = ejection_spawn_position(&BlockPos::new(3, -4, 5));
        assert_eq!(spawn.x, 3.5);
        assert_eq!(spawn.y, -4.0 + 1.2);
        assert_eq!(spawn.z, 5.5);
        // Distinct from the old `World::drop_stack` origin (block center, y + 0.5).
        assert!(spawn.y > -4.0 + 0.5);
    }
}
