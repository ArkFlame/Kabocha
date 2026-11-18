use std::sync::RwLock;

use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::projectile::owner::ProjectileOwner;
use crate::entity::projectile::{ProjectileHit, ThrownItemEntity};
use crate::entity::{Entity, EntityBase};
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

const GRAVITY: f64 = 0.07;
const WATER_COLOR: i32 = -13_083_194; // 0xFF385DC6

/// Vanilla ThrownExperienceBottle.onHit: orbs offset along the block face
/// normal, or against the flight direction when an entity was hit.
fn orb_award_direction(hit: &ProjectileHit, velocity: Vector3<f64>) -> Vector3<f64> {
    match hit {
        ProjectileHit::Block { face, .. } => {
            let offset = face.to_offset();
            Vector3::new(
                f64::from(offset.x),
                f64::from(offset.y),
                f64::from(offset.z),
            )
        }
        ProjectileHit::Entity { .. } => velocity.multiply(-1.0, -1.0, -1.0),
    }
}

pub struct ExperienceBottleEntity {
    pub thrown: ThrownItemEntity,
    pub item_stack: RwLock<ItemStack>,
}

impl ExperienceBottleEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            thrown: ThrownItemEntity {
                entity,
                owner: ProjectileOwner::none(),
                gravity: GRAVITY,
            },
            item_stack: RwLock::new(ItemStack::new(1, &Item::EXPERIENCE_BOTTLE)),
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        Self {
            thrown: ThrownItemEntity::new(entity, shooter, GRAVITY),
            item_stack: RwLock::new(ItemStack::new(1, &Item::EXPERIENCE_BOTTLE)),
        }
    }

    pub fn set_item_stack(&self, item_stack: ItemStack) {
        let mut write = self
            .item_stack
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *write = item_stack;
    }
}

impl EntityBase for ExperienceBottleEntity {
    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown.owner)
    }

    fn init_data_tracker(&self) {
        let entity = self.get_entity();
        let stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        entity.set_synced_data(
            pumpkin_data::tracked_data::experience_bottle::ITEM_STACK,
            ItemStackSerializer::from(stack.clone()),
        );
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        self.thrown.process_tick(caller, server);
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        let stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut item = NbtCompound::new();
        stack.write_item_stack(&mut item);
        nbt.put_compound("Item", item);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        if let Some(stack) = nbt
            .get_compound("Item")
            .and_then(ItemStack::read_item_stack)
        {
            self.set_item_stack(stack.copy_with_count(1));
        }
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let hit_pos = hit.hit_pos();
        let block_pos = BlockPos::floored(hit_pos.x, hit_pos.y, hit_pos.z);

        let direction = orb_award_direction(&hit, entity.velocity.load());
        let mut rng = rand::rng();
        let xp_count: i32 = 3 + rng.random_range(0..5) + rng.random_range(0..5);
        let mut event = crate::plugin::api::events::entity::exp_bottle::ExpBottleEvent::new(
            entity.entity_id,
            xp_count,
            block_pos,
            true,
        );
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.show_effect {
            world.sync_world_event(
                WorldEvent::ParticlesSpellPotionSplash,
                block_pos,
                WATER_COLOR,
            );
            world.sync_world_event(WorldEvent::SoundSpellPotionSplash, block_pos, 0);
        }
        if event.cancelled {
            return;
        }
        let amount = u32::try_from(event.experience).unwrap_or(0);
        if amount > 0 {
            ExperienceOrbEntity::spawn_with_direction(&world, hit_pos, direction, amount);
        }
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::BlockDirection;

    #[test]
    fn orb_direction_uses_block_face_normal() {
        let velocity = Vector3::new(1.0, 2.0, 3.0);
        for (face, expected) in [
            (BlockDirection::Up, Vector3::new(0.0, 1.0, 0.0)),
            (BlockDirection::Down, Vector3::new(0.0, -1.0, 0.0)),
            (BlockDirection::East, Vector3::new(1.0, 0.0, 0.0)),
            (BlockDirection::North, Vector3::new(0.0, 0.0, -1.0)),
        ] {
            let hit = ProjectileHit::Block {
                pos: BlockPos::floored(0.0, 0.0, 0.0),
                face,
                hit_pos: Vector3::new(0.0, 0.0, 0.0),
                normal: velocity,
                world_border: false,
            };
            assert_eq!(orb_award_direction(&hit, velocity), expected);
        }
    }
}
