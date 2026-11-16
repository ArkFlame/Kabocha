use crate::block::registry::BlockActionResult;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::entity::projectile::firework_rocket::FireworkRocketEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct FireworkRocketItem;

impl ItemMetadata for FireworkRocketItem {
    fn ids() -> Box<[u16]> {
        [Item::FIREWORK_ROCKET.id].into()
    }
}

impl ItemBehaviour for FireworkRocketItem {
    // Vanilla `FireworkRocketItem` has no `useOn`: clicking a block passes through to `use`,
    // which only launches while the player is fall flying. Launching unconditionally here
    // left a rocket attached to the player (and a client-side boost) while walking on ground.
    fn use_on_block(
        &self,
        _item: &mut ItemStack,
        _player: &Player,
        _location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        BlockActionResult::Pass
    }

    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    // Vanilla `FireworkRocketItem.use`: only while fall flying (gliding) does it spawn the
    // owner-attached rocket and consume the item; on the ground it passes and does nothing.
    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        let mut held = match hand {
            pumpkin_util::Hand::Right => player.inventory().held_item(),
            pumpkin_util::Hand::Left => player.inventory().off_hand_item(),
        };
        if held.is_empty() || held.item.id != Item::FIREWORK_ROCKET.id {
            return;
        }
        if !player.get_entity().is_fall_flying() {
            return;
        }

        let world = player.world();
        // Vanilla only breaks leashes while the launch is a glide boost.
        let search_box = player
            .get_entity()
            .bounding_box
            .load()
            .expand(32.0, 32.0, 32.0);
        let player_id = player.entity_id();
        let mut dropped_leash = false;
        for entity_base in world.get_entities_at_box(&search_box) {
            let ent = entity_base.get_entity();
            let is_leashed_to_player = ent.leashed_to.try_lock().is_ok_and(|guard| {
                guard
                    .as_ref()
                    .and_then(std::sync::Weak::upgrade)
                    .is_some_and(|holder| holder.get_entity().entity_id == player_id)
            });
            if is_leashed_to_player && ent.unleash() {
                world.drop_stack(&ent.block_pos.load(), ItemStack::new(1, &Item::LEAD));
                dropped_leash = true;
            }
        }
        if dropped_leash {
            world.play_sound_fine(
                Sound::ItemLeadBreak,
                SoundCategory::Neutral,
                &player.position(),
                1.0,
                1.0,
            );
        }

        let entity = Entity::new(
            world.clone(),
            player.get_entity().pos.load(),
            &EntityType::FIREWORK_ROCKET,
        );
        let entity = FireworkRocketEntity::new_shot_with_stack(
            entity,
            player.get_entity(),
            held.copy_with_count(1),
        );
        world.spawn_entity(Arc::new(entity));

        let item_id = held.item.id;
        held.decrement_unless_creative(player.gamemode.load(), 1);
        match hand {
            pumpkin_util::Hand::Right => player.inventory().set_held_item(held),
            pumpkin_util::Hand::Left => player
                .inventory()
                .set_stack_in_hand(pumpkin_util::Hand::Left, held),
        }
        player.increment_stat(StatisticCategory::Used, i32::from(item_id), 1);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
