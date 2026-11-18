use rustc_hash::FxHashSet;
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering},
};

use pumpkin_data::{BlockDirection, BlockId};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::WorldEvent;
use pumpkin_util::Difficulty;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::blocks::fire::FireBlockBase;
use crate::block::blocks::fire::fire::FireBlock;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::server::Server;
use crate::world::World;

pub struct LightningBoltEntity {
    entity: Entity,
    life: AtomicI32,
    seed: AtomicI64,
    flashes: AtomicI32,
    visual_only: AtomicBool,
    cause: Mutex<Option<Weak<Player>>>,
    hit_entities: Mutex<FxHashSet<i32>>,
    blocks_set_on_fire: AtomicI32,
}

impl LightningBoltEntity {
    pub fn new(entity: Entity) -> Self {
        entity.no_physics.store(true, Ordering::Relaxed);
        let seed = rand::rng().random::<i64>();
        let flashes = rand::rng().random_range(1..=3);
        Self {
            entity,
            life: AtomicI32::new(2),
            seed: AtomicI64::new(seed),
            flashes: AtomicI32::new(flashes),
            visual_only: AtomicBool::new(false),
            cause: Mutex::new(None),
            hit_entities: Mutex::new(FxHashSet::default()),
            blocks_set_on_fire: AtomicI32::new(0),
        }
    }

    pub fn set_visual_only(&self, visual_only: bool) {
        self.visual_only.store(visual_only, Ordering::Relaxed);
    }

    pub fn is_visual_only(&self) -> bool {
        self.visual_only.load(Ordering::Relaxed)
    }

    pub fn set_cause(&self, cause: Option<&Arc<Player>>) {
        if let Ok(mut c) = self.cause.lock() {
            *c = cause.map(Arc::downgrade);
        }
    }

    pub fn get_cause(&self) -> Option<Arc<Player>> {
        self.cause
            .lock()
            .ok()
            .and_then(|c| c.as_ref().and_then(Weak::upgrade))
    }

    pub fn get_blocks_set_on_fire(&self) -> i32 {
        self.blocks_set_on_fire.load(Ordering::Relaxed)
    }

    pub fn get_seed(&self) -> i64 {
        self.seed.load(Ordering::Relaxed)
    }

    pub fn set_seed(&self, seed: i64) {
        self.seed.store(seed, Ordering::Relaxed);
    }

    fn get_strike_position(&self) -> BlockPos {
        let pos = self.entity.pos.load();
        Vector3::new(pos.x, pos.y - 1.0e-6, pos.z).to_block_pos()
    }

    fn power_lightning_rod(&self, world: &Arc<World>) {
        let strike_pos = self.get_strike_position();
        let block = world.get_block(&strike_pos);
        if pumpkin_data::tag::Taggable::has_tag(
            block,
            &pumpkin_data::tag::Block::MINECRAFT_LIGHTNING_RODS,
        ) {
            crate::block::blocks::redstone::lightning_rod::LightningRodBlock::trigger(
                world,
                &strike_pos,
            );
        }
    }

    fn spawn_fire(&self, world: &Arc<World>, additional_sources: i32) {
        if self.visual_only.load(Ordering::Relaxed) {
            return;
        }

        // Vanilla LightningBolt.spawnFire gates both the main and the additional
        // placements on the same radius check the fire tick uses.
        let pos = self.entity.block_pos.load();
        if !FireBlock::can_spread_fire_around(world, &pos) {
            return;
        }

        let try_place = |p: BlockPos| {
            if world.get_block_state(&p).is_air() && FireBlockBase::can_place_at(world, &p, BlockDirection::Up) {
                let state = FireBlock.get_placement_state(world, &p, 0);
                world.set_block_state(&p, state, BlockFlags::NOTIFY_ALL);
                self.blocks_set_on_fire.fetch_add(1, Ordering::Relaxed);
            }
        };

        try_place(pos);

        for _ in 0..additional_sources {
            let dx = rand::rng().random_range(-1..=1);
            let dy = rand::rng().random_range(-1..=1);
            let dz = rand::rng().random_range(-1..=1);
            let nearby_pos = pos.offset(Vector3::new(dx, dy, dz));
            try_place(nearby_pos);
        }
    }

    fn clear_copper_on_lightning_strike(&self, world: &Arc<World>) {
        let strike_pos = self.get_strike_position();
        let struck_state = world.get_block_state(&strike_pos);
        let struck_block = struck_state.id.to_block();

        let is_waxed = is_waxed_copper(struck_block.id);
        let is_weathering = is_weathering_copper(struck_block.id);

        if is_weathering || is_waxed {
            if is_weathering && let Some(first_block_id) = get_first_stage_copper(struck_block.id) {
                let first_block = first_block_id.to_block();
                let new_state_id = first_block.with_properties_of(struck_block, struck_state.id);
                world.set_block_state(&strike_pos, new_state_id, BlockFlags::NOTIFY_ALL);
            }

            let strikes_count = rand::rng().random_range(3..=5);

            for _ in 0..strikes_count {
                let step_count = rand::rng().random_range(1..=8);
                Self::random_walk_cleaning_copper(world, &strike_pos, step_count);
            }
        }
    }

    fn random_walk_cleaning_copper(
        world: &Arc<World>,
        original_strike_pos: &BlockPos,
        step_count: i32,
    ) {
        let mut work_pos = *original_strike_pos;

        for _ in 0..step_count {
            if let Some(next_pos) = Self::random_step_cleaning_copper(world, &work_pos) {
                work_pos = next_pos;
            } else {
                break;
            }
        }
    }

    fn random_step_cleaning_copper(world: &Arc<World>, pos: &BlockPos) -> Option<BlockPos> {
        let candidates = random_in_cube(10, pos, 1);
        for candidate in candidates {
            let state = world.get_block_state(&candidate);
            let block = state.id.to_block();

            if is_weathering_copper(block.id)
                && let Some(prev_block_id) = get_previous_stage_copper(block.id)
            {
                let prev_block = prev_block_id.to_block();
                let new_state_id = prev_block.with_properties_of(block, state.id);
                world.set_block_state(&candidate, new_state_id, BlockFlags::NOTIFY_ALL);
                world.sync_world_event(WorldEvent::ParticlesElectricSpark, candidate, -1);
                return Some(candidate);
            }
        }
        None
    }

    fn strike_entities(&self, entity: &Entity) {
        let current_life = self.life.load(Ordering::Relaxed);
        if current_life < 0 || self.visual_only.load(Ordering::Relaxed) {
            return;
        }
        let world = entity.world.load();
        let pos = entity.pos.load();
        let damage_box = BoundingBox::new(
            Vector3::new(pos.x - 3.0, pos.y - 3.0, pos.z - 3.0),
            Vector3::new(pos.x + 3.0, pos.y + 9.0, pos.z + 3.0),
        );

        let bolt_pos = self.entity.pos.load();
        let entities = world.get_all_at_box(&damage_box);
        if let Ok(mut hit_guard) = self.hit_entities.lock() {
            let mut struck_entities = Vec::new();
            for hit_entity in entities {
                if hit_entity.get_entity().entity_id == entity.entity_id {
                    continue;
                }
                let hit_id = hit_entity.get_entity().entity_id;
                if hit_guard.insert(hit_id) {
                    hit_entity.on_lightning_strike(hit_entity.as_ref(), self);
                    if let Some(player) = self.get_cause() {
                        let type_name = format!(
                            "minecraft:{}",
                            hit_entity.get_entity().entity_type.resource_name
                        );
                        player.trigger_advancement(
                            crate::entity::player::advancement::trigger::AdvancementTrigger::ChanneledLightning {
                                victim_type: type_name.clone(),
                            },
                        );
                        let hit_pos = hit_entity.get_entity().pos.load();
                        struck_entities.push(
                            crate::entity::player::advancement::trigger::LightningStruckEntity {
                                entity_type: type_name,
                                distance: Some(
                                    crate::entity::player::advancement::predicate::EntityDistance::from_delta(
                                        hit_pos.x - bolt_pos.x,
                                        hit_pos.y - bolt_pos.y,
                                        hit_pos.z - bolt_pos.z,
                                    ),
                                ),
                                is_baby: None,
                            },
                        );
                    }
                }
            }
            // Vanilla fires this once so bystander and entity_struck predicates
            // are tested against the struck entities independently.
            if let Some(player) = self.get_cause()
                && !struck_entities.is_empty()
            {
                let distance = player.position().squared_distance_to_vec(&bolt_pos).sqrt();
                let bystander_types = struck_entities
                    .iter()
                    .map(|e| e.entity_type.clone())
                    .collect();
                player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::LightningStrike {
                        bystander_types,
                        struck_entities,
                        blocks_set_on_fire: self.blocks_set_on_fire.load(Ordering::Relaxed),
                        distance,
                    },
                );
            }
        }
    }
}

impl EntityBase for LightningBoltEntity {
    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;
        let life = self.life.load(Ordering::Relaxed);

        if life == 2 {
            let world = entity.world.load();
            let pos = entity.pos.load();

            let pitch_thunder = 0.8 + rand::rng().random::<f32>() * 0.2;
            world.play_sound_fine(
                Sound::EntityLightningBoltThunder,
                SoundCategory::Weather,
                &pos,
                10000.0,
                pitch_thunder,
            );
            let pitch_impact = 0.5 + rand::rng().random::<f32>() * 0.2;
            world.play_sound_fine(
                Sound::EntityLightningBoltImpact,
                SoundCategory::Weather,
                &pos,
                2.0,
                pitch_impact,
            );

            let difficulty = world.level_info.load().difficulty;
            if difficulty == Difficulty::Normal || difficulty == Difficulty::Hard {
                self.spawn_fire(&world, 4);
            }

            self.power_lightning_rod(&world);
            self.clear_copper_on_lightning_strike(&world);
        }

        let new_life = life - 1;
        self.life.store(new_life, Ordering::Relaxed);

        if new_life < 0 {
            let flashes = self.flashes.load(Ordering::Relaxed);
            if flashes == 0 {
                entity.remove();
                return;
            } else if new_life < -rand::rng().random_range(0..10) {
                self.flashes.store(flashes - 1, Ordering::Relaxed);
                self.life.store(1, Ordering::Relaxed);
                self.seed.store(rand::random::<i64>(), Ordering::Relaxed);
                let world = entity.world.load();
                self.spawn_fire(&world, 0);
            }
        }

        self.strike_entities(entity);
    }

    fn init_data_tracker(&self) {}

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn random_in_cube(count: usize, center: &BlockPos, radius: i32) -> Vec<BlockPos> {
    let mut list = Vec::with_capacity(count);
    for _ in 0..count {
        let dx = rand::rng().random_range(-radius..=radius);
        let dy = rand::rng().random_range(-radius..=radius);
        let dz = rand::rng().random_range(-radius..=radius);
        list.push(center.offset(Vector3::new(dx, dy, dz)));
    }
    list
}

#[must_use]
pub const fn is_waxed_copper(id: BlockId) -> bool {
    pumpkin_data::block_transformer::get_unwaxed(id).is_some()
}

#[must_use]
pub const fn is_weathering_copper(id: BlockId) -> bool {
    pumpkin_data::block_transformer::get_previous_weathering(id).is_some()
        || pumpkin_data::block_transformer::get_next_weathering(id).is_some()
}

#[must_use]
pub const fn get_first_stage_copper(id: BlockId) -> Option<BlockId> {
    if !is_weathering_copper(id) {
        return None;
    }
    let mut cur = id;
    while let Some(prev) = pumpkin_data::block_transformer::get_previous_weathering(cur) {
        cur = prev;
    }
    Some(cur)
}

#[must_use]
pub const fn get_previous_stage_copper(id: BlockId) -> Option<BlockId> {
    pumpkin_data::block_transformer::get_previous_weathering(id)
}
