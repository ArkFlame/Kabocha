use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::{
    CreakingHeartLikeProperties, CreakingHeartState, GlowLichenLikeProperties,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::EnvironmentAttribute;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::{RngExt, seq::SliceRandom};
use uuid::Uuid;

use crate::block::blocks::creaking_heart::CreakingHeartBlock;
use crate::entity::mob::creaking::CreakingEntity;
use crate::entity::spawn_util::{SpawnStrategy, try_spawn_mob};
use crate::world::World;

use super::BlockEntity;

/// Vanilla `getNearestPlayerThatAffectsSpawning(..., 32.0, ...)` spawn range.
const CREAKING_SPAWN_DISTANCE: f64 = 32.0;

/// Vanilla `spawnProtector` `trySpawnMob` arguments: 5 attempts within +/-16 XZ and +/-8 Y.
const CREAKING_SPAWN_ATTEMPTS: i32 = 5;
const CREAKING_SPAWN_RANGE_XZ: i32 = 16;
const CREAKING_SPAWN_RANGE_Y: i32 = 8;

/// Vanilla `CreakingHeartBlockEntity#DISTANCE_CREAKING_TOO_FAR`.
const CREAKING_TOO_FAR_DISTANCE: f64 = 34.0;

/// Vanilla `BlockPos.breadthFirstTraversal(origin, 2, 64, ...)` limits for resin spread.
const RESIN_SPREAD_MAX_DEPTH: u32 = 2;
const RESIN_SPREAD_MAX_NODES: u32 = 64;

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
        // Vanilla `CreakingHeartBlockEntity#spawnProtector` calls
        // `SpawnUtil.trySpawnMob(..., 5, 16, 8, ON_TOP_OF_COLLIDER_NO_LEAVES, true)`.
        let Some(creaking) = try_spawn_mob(
            &EntityType::CREAKING,
            CreakingEntity::new,
            world,
            &self.position,
            CREAKING_SPAWN_ATTEMPTS,
            CREAKING_SPAWN_RANGE_XZ,
            CREAKING_SPAWN_RANGE_Y,
            SpawnStrategy::OnTopOfColliderNoLeaves,
            true,
        ) else {
            return;
        };
        creaking.set_transient(self.position);
        self.set_creaking_uuid(Some(creaking.mob_entity.living_entity.entity.entity_uuid));
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

    pub fn creaking_hurt(&self, world: &Arc<World>) {
        world.play_sound(
            Sound::BlockCreakingHeartHurt,
            SoundCategory::Blocks,
            &self.position.to_f64(),
        );

        // Vanilla `creakingHurt` grows resin only while the heart is awake.
        let state_id = world.get_block_state_id(&self.position);
        if CreakingHeartLikeProperties::from_state_id(state_id).creaking_heart_state
            != CreakingHeartState::Awake
        {
            return;
        }

        // Vanilla `nextIntBetweenInclusive(2, 3)` places that many resin clumps.
        let count = rand::rng().random_range(2..4);
        for _ in 0..count {
            let Some(pos) = self.spread_resin(world) else {
                continue;
            };
            world.play_sound(Sound::BlockResinPlace, SoundCategory::Blocks, &pos.to_f64());
            world.emit_game_event(GameEvent::BlockPlace.name(), pos.to_f64());
        }
    }

    /// Vanilla `spreadResin`: a breadth-first walk over nearby pale oak logs (depth 2, at most
    /// 64 nodes) that places one multiface resin clump on the first free adjacent face.
    fn spread_resin(&self, world: &Arc<World>) -> Option<BlockPos> {
        let mut queue = VecDeque::from([(self.position, 0u32)]);
        let mut visited = HashSet::new();
        let mut count = 0u32;

        while let Some((pos, depth)) = queue.pop_front() {
            if !visited.insert(pos) {
                continue;
            }

            let placed = if Self::is_pale_oak_log(world, &pos) {
                Self::try_place_resin_on_log(world, &pos)
            } else {
                None
            };
            if let Some(placed) = placed {
                return Some(placed);
            }

            count += 1;
            if count >= RESIN_SPREAD_MAX_NODES {
                return None;
            }
            if depth < RESIN_SPREAD_MAX_DEPTH {
                Self::enqueue_log_neighbors(world, &pos, depth + 1, &mut queue);
            }
        }
        None
    }

    fn enqueue_log_neighbors(
        world: &Arc<World>,
        pos: &BlockPos,
        depth: u32,
        queue: &mut VecDeque<(BlockPos, u32)>,
    ) {
        let mut dirs = BlockDirection::all();
        dirs.shuffle(&mut rand::rng());
        for dir in dirs {
            let neighbor = pos.offset(dir.to_offset());
            if Self::is_pale_oak_log(world, &neighbor) {
                queue.push_back((neighbor, depth));
            }
        }
    }

    fn is_pale_oak_log(world: &Arc<World>, pos: &BlockPos) -> bool {
        world
            .get_block_state_id(pos)
            .to_block()
            .has_tag(&tag::Block::MINECRAFT_PALE_OAK_LOGS)
    }

    /// Adds one face of a resin clump next to a log, matching vanilla's shuffled multiface fill.
    fn try_place_resin_on_log(world: &Arc<World>, pos: &BlockPos) -> Option<BlockPos> {
        let mut dirs = BlockDirection::all();
        dirs.shuffle(&mut rand::rng());
        for dir in dirs {
            let neighbor_pos = pos.offset(dir.to_offset());
            let state_id = world.get_block_state_id(&neighbor_pos);
            let block = state_id.to_block();
            let opposite = dir.opposite();

            let props = if block == &Block::AIR {
                Some(GlowLichenLikeProperties::default(&Block::RESIN_CLUMP))
            } else if block == &Block::WATER && world.is_water_source(&neighbor_pos) {
                let mut props = GlowLichenLikeProperties::default(&Block::RESIN_CLUMP);
                props.waterlogged = true;
                Some(props)
            } else if block == &Block::RESIN_CLUMP {
                Some(GlowLichenLikeProperties::from_state_id(state_id))
            } else {
                None
            };

            if let Some(mut props) = props
                && !Self::has_face(&props, opposite)
            {
                Self::set_face(&mut props, opposite);
                world.set_block_state(
                    &neighbor_pos,
                    props.to_state_id(&Block::RESIN_CLUMP),
                    BlockFlags::NOTIFY_ALL,
                );
                return Some(neighbor_pos);
            }
        }
        None
    }

    const fn has_face(props: &GlowLichenLikeProperties, direction: BlockDirection) -> bool {
        match direction {
            BlockDirection::Down => props.down,
            BlockDirection::Up => props.up,
            BlockDirection::North => props.north,
            BlockDirection::South => props.south,
            BlockDirection::West => props.west,
            BlockDirection::East => props.east,
        }
    }

    const fn set_face(props: &mut GlowLichenLikeProperties, direction: BlockDirection) {
        match direction {
            BlockDirection::Down => props.down = true,
            BlockDirection::Up => props.up = true,
            BlockDirection::North => props.north = true,
            BlockDirection::South => props.south = true,
            BlockDirection::West => props.west = true,
            BlockDirection::East => props.east = true,
        }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::block_properties::GlowLichenLikeProperties;
    use pumpkin_data::{Block, BlockDirection};
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

    #[test]
    fn resin_clump_face_helpers_roundtrip() {
        let block = &Block::RESIN_CLUMP;
        for direction in BlockDirection::all() {
            let mut props = GlowLichenLikeProperties::default(block);
            CreakingHeartBlockEntity::set_face(&mut props, direction);
            assert!(CreakingHeartBlockEntity::has_face(&props, direction));
        }
        let mut props = GlowLichenLikeProperties::default(block);
        CreakingHeartBlockEntity::set_face(&mut props, BlockDirection::Down);
        CreakingHeartBlockEntity::set_face(&mut props, BlockDirection::Up);
        assert!(CreakingHeartBlockEntity::has_face(
            &props,
            BlockDirection::Down
        ));
        assert!(CreakingHeartBlockEntity::has_face(
            &props,
            BlockDirection::Up
        ));
    }
}
