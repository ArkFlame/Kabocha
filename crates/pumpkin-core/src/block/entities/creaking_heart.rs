use std::sync::Arc;
use std::sync::atomic::Ordering;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::{CreakingHeartLikeProperties, CreakingHeartState};
use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::EnvironmentAttribute;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::block::blocks::creaking_heart::CreakingHeartBlock;
use crate::entity::Entity;
use crate::entity::mob::creaking::CreakingEntity;
use crate::world::World;

use super::BlockEntity;

/// Vanilla `getNearestPlayerThatAffectsSpawning(..., 32.0, ...)` spawn range.
const CREAKING_SPAWN_DISTANCE: f64 = 32.0;

/// Vanilla `CreakingHeartBlockEntity#DISTANCE_CREAKING_TOO_FAR`.
const CREAKING_TOO_FAR_DISTANCE: f64 = 34.0;

pub struct CreakingHeartBlockEntity {
    pub position: BlockPos,
    pub creaking_uuid: AtomicCell<Option<Uuid>>,
}

impl BlockEntity for CreakingHeartBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let creaking_uuid = nbt
            .get_uuid("creaking")
            .or_else(|| {
                nbt.get_string("creaking")
                    .and_then(|uuid_str| Uuid::parse_str(uuid_str).ok())
            })
            .or_else(|| {
                nbt.get_string("creaking_uuid")
                    .and_then(|uuid_str| Uuid::parse_str(uuid_str).ok())
            });
        Self {
            position,
            creaking_uuid: AtomicCell::new(creaking_uuid),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(uuid) = self.creaking_uuid.load() {
            nbt.put_uuid("creaking", uuid);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Some(uuid) = self.creaking_uuid.load() {
            nbt.put_uuid("creaking", uuid);
        }
        Some(nbt)
    }

    fn tick(&self, world: &Arc<World>) {
        // Vanilla `CreakingHeartBlockEntity#serverTick` runs its state/spawn logic on a
        // 20-24 tick timer, spawning only when unbound and despawning only when bound.
        if world.get_world_age() % 20 != 0 {
            return;
        }
        CreakingHeartBlock::update_state(world, &self.position);
        if self.creaking_uuid.load().is_some() {
            self.try_despawn_creaking(world);
        } else {
            self.try_spawn_creaking(world);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CreakingHeartBlockEntity {
    pub const ID: &'static str = "minecraft:creaking_heart";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            creaking_uuid: AtomicCell::new(None),
        }
    }

    pub fn is_protector(&self, creaking_uuid: Uuid) -> bool {
        self.creaking_uuid
            .load()
            .is_some_and(|uuid| uuid == creaking_uuid)
    }

    pub fn set_creaking_uuid(&self, uuid: Option<Uuid>) {
        self.creaking_uuid.store(uuid);
    }

    /// Vanilla `CreakingHeartBlockEntity` spawn branch: an unbound awake heart in a hostile,
    /// mob-spawning world with a player within 32 blocks spawns a creaking and binds its UUID.
    fn try_spawn_creaking(&self, world: &Arc<World>) {
        let (peaceful, spawn_mobs) = {
            let level_info = world.level_info.load();
            (
                level_info.difficulty == Difficulty::Peaceful,
                level_info.game_rules.spawn_mobs,
            )
        };
        if peaceful || !spawn_mobs {
            return;
        }
        let state_id = world.get_block_state_id(&self.position);
        if CreakingHeartLikeProperties::from_state_id(state_id).creaking_heart_state
            != CreakingHeartState::Awake
        {
            return;
        }
        let center = Vector3::new(
            f64::from(self.position.0.x) + 0.5,
            f64::from(self.position.0.y) + 0.5,
            f64::from(self.position.0.z) + 0.5,
        );
        if world
            .get_closest_player(center, CREAKING_SPAWN_DISTANCE)
            .is_none()
        {
            return;
        }
        let Some(spawn_pos) = Self::find_free_spawn_pos(world, &self.position) else {
            return;
        };
        let uuid = Uuid::new_v4();
        let entity = Entity::from_uuid(
            uuid,
            world.clone(),
            Vector3::new(
                f64::from(spawn_pos.0.x) + 0.5,
                f64::from(spawn_pos.0.y),
                f64::from(spawn_pos.0.z) + 0.5,
            ),
            &EntityType::CREAKING,
        );
        let creaking = CreakingEntity::new(entity);
        creaking.set_transient(self.position);
        world.spawn_entity(creaking);
        self.set_creaking_uuid(Some(uuid));
    }

    /// Vanilla `CreakingHeartBlockEntity#serverTick` despawn branch: a bound creaking is torn
    /// down when day breaks, drifts more than 34 blocks from the heart, or traps a player inside
    /// itself. A missing creaking clears the binding so the heart can spawn again.
    fn try_despawn_creaking(&self, world: &Arc<World>) {
        let Some(uuid) = self.creaking_uuid.load() else {
            return;
        };
        let Some(entity) = world.get_entity_by_uuid(uuid) else {
            self.set_creaking_uuid(None);
            return;
        };
        let Some(creaking) = entity.cast_any().downcast_ref::<CreakingEntity>() else {
            self.set_creaking_uuid(None);
            return;
        };
        let creaking_pos = creaking.mob_entity.living_entity.entity.pos.load();
        let too_far = Self::is_creaking_too_far(&self.position, creaking_pos);
        let active = world
            .environment_attributes()
            .get_value_bool(EnvironmentAttribute::GameplayCreakingActive, &self.position);
        let persistent = creaking
            .mob_entity
            .persistence_required
            .load(Ordering::Relaxed);
        let stuck = creaking.player_is_stuck_in_you(&world.get_nearby_players(creaking_pos, 4.0));

        if (!active && !persistent) || too_far || stuck {
            creaking.tear_down();
            self.set_creaking_uuid(None);
        }
    }

    fn is_creaking_too_far(heart: &BlockPos, creaking_pos: Vector3<f64>) -> bool {
        let dx = creaking_pos.x - (f64::from(heart.0.x) + 0.5);
        let dy = creaking_pos.y - f64::from(heart.0.y);
        let dz = creaking_pos.z - (f64::from(heart.0.z) + 0.5);
        (dx * dx + dy * dy + dz * dz).sqrt() > CREAKING_TOO_FAR_DISTANCE
    }

    fn find_free_spawn_pos(world: &Arc<World>, heart: &BlockPos) -> Option<BlockPos> {
        [heart.east(), heart.west(), heart.north(), heart.south()]
            .into_iter()
            .find(|pos| {
                world.get_block_state_id(pos).to_state().is_air()
                    && world.get_block_state_id(&pos.up()).to_state().is_air()
            })
    }

    pub fn creaking_hurt(&self, world: &Arc<World>) {
        world.play_sound(
            Sound::BlockCreakingHeartHurt,
            SoundCategory::Blocks,
            &self.position.to_f64(),
        );
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector3::Vector3;
    use uuid::Uuid;

    use super::CreakingHeartBlockEntity;

    #[test]
    fn unbound_heart_protects_no_creaking() {
        let be = CreakingHeartBlockEntity::new(BlockPos::new(0, 64, 0));
        assert!(!be.is_protector(Uuid::new_v4()));
        let bound = Uuid::new_v4();
        be.set_creaking_uuid(Some(bound));
        assert!(be.is_protector(bound));
        assert!(!be.is_protector(Uuid::new_v4()));
    }

    #[test]
    fn creaking_too_far_boundary() {
        let heart = BlockPos::new(0, 64, 0);
        let near = Vector3::new(33.5, 64.0, 0.5);
        assert!(!CreakingHeartBlockEntity::is_creaking_too_far(&heart, near));
        let far = Vector3::new(35.5, 64.0, 0.5);
        assert!(CreakingHeartBlockEntity::is_creaking_too_far(&heart, far));
    }
}
