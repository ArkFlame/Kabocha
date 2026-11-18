use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState, BlockStateId,
    attributes::Attributes,
    damage::DamageType,
    entity::EntityType,
    fluid::Fluid,
    game_event::GameEvent,
    particle::Particle,
    sound::Sound,
    tag::{Tag, Taggable},
};
use pumpkin_util::math::{
    boundingbox::BoundingBox, position::BlockPos, vector2::Vector2, vector3::Vector3,
};
use pumpkin_util::random::RandomImpl;
use pumpkin_world::chunk::ChunkData;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    block::{
        ExplodeArgs,
        blocks::fire::{FireBlockBase, fire::FireBlock},
        drop_loot,
    },
    entity::{Entity, EntityBase, projectile::is_redirectable, tnt::TNTEntity},
    net::ClientPlatform,
    world::loot::LootContextParameters,
};

use super::{BlockFlags, World, game_event::GameEventContext};

/// Defines the type of explosion interaction with the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplosionInteraction {
    None,
    Block,
    Mob,
    Tnt,
    Trigger,
}

/// Defines how an explosion interacts with blocks in the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockInteraction {
    /// Keeps blocks intact (no block damage, no drops).
    Keep,
    /// Destroys blocks and drops 100% of items without decay.
    Destroy,
    /// Destroys blocks and applies loot decay based on explosion radius.
    DestroyWithDecay,
    /// Triggers block effects without destroying them.
    TriggerBlock,
}

/// Defines how damage and block destruction are calculated for an explosion.
pub trait ExplosionDamageCalculator: Send + Sync {
    /// Returns the block's explosion resistance. If None, the block is treated as air/empty.
    fn get_block_explosion_resistance(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        block: &Block,
        fluid: &pumpkin_data::fluid::FluidState,
    ) -> Option<f32> {
        if block.default_state.is_air() && fluid.is_empty {
            None
        } else {
            Some(fluid.blast_resistance.max(block.blast_resistance))
        }
    }

    /// Returns whether this block should be destroyed / affected by the explosion.
    fn should_block_explode(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        _block: &Block,
        _power: f32,
    ) -> bool {
        true
    }

    /// Returns whether the entity should take damage from the explosion.
    fn should_damage_entity(&self, _explosion: &Explosion, _entity: &dyn EntityBase) -> bool {
        true
    }

    /// Returns knockback multiplier for the given entity (default 1.0).
    fn get_knockback_multiplier(&self, _entity: &dyn EntityBase) -> f32 {
        1.0
    }

    /// Calculates the damage amount to deal to the entity given the exposure.
    fn get_entity_damage_amount(
        &self,
        explosion: &Explosion,
        entity: &dyn EntityBase,
        exposure: f32,
    ) -> f32 {
        let radius = explosion.power as f64 * 2.0;
        let distance = (entity
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&explosion.pos))
        .sqrt()
            / radius;
        let damage_multiplier = (1.0 - distance) * exposure as f64;
        (f64::midpoint(damage_multiplier * damage_multiplier, damage_multiplier) * 7.0 * radius
            + 1.0) as f32
    }
}

const fn should_create_fire(roll: u8, state: &BlockState, below: &BlockState) -> bool {
    roll == 0 && state.is_air() && below.is_solid_render()
}

/// Vanilla `getChunk` force-loads chunks the explosion ray enters.
///
/// Loaded chunks use the sync read; unloaded ones go through the async chunk system,
/// blocking the caller. The tick thread and the Wasm blocking pools run outside the
/// runtime, and a runtime worker moves aside first.
fn load_chunk_for_explosion(world: &World, chunk_pos: Vector2<i32>) -> Option<Arc<ChunkData>> {
    if let Some(chunk) = world
        .level
        .read_chunk_sync(&chunk_pos, std::clone::Clone::clone)
    {
        return Some(chunk);
    }
    let handle = world.level.rt_handle.get()?;
    let fetch = || {
        world
            .level
            .get_or_fetch_chunk(chunk_pos, std::clone::Clone::clone)
    };
    let Ok(current) = tokio::runtime::Handle::try_current() else {
        return Some(handle.block_on(fetch()));
    };
    // `block_in_place` and `Handle::block_on` both panic on a current-thread runtime, and
    // the chunk system's IO tasks need that runtime to drive them, so a single-threaded
    // caller keeps the sync read's air fallback instead of deadlocking.
    if current.runtime_flavor() == tokio::runtime::RuntimeFlavor::CurrentThread {
        tracing::debug!("Explosion chunk force-load skipped on a current-thread runtime");
        return None;
    }
    Some(tokio::task::block_in_place(|| handle.block_on(fetch())))
}

/// Default explosion damage calculator implementing vanilla standard explosion rules.
pub struct DefaultExplosionDamageCalculator;

impl ExplosionDamageCalculator for DefaultExplosionDamageCalculator {}

/// A configurable explosion damage calculator (e.g. for wind charges, mace wind bursts).
pub struct SimpleExplosionDamageCalculator {
    pub explodes_blocks: bool,
    pub damages_entities: bool,
    pub knockback_multiplier: Option<f32>,
    pub immune_blocks: Option<&'static Tag>,
}

impl SimpleExplosionDamageCalculator {
    #[must_use]
    pub const fn new(
        explodes_blocks: bool,
        damages_entities: bool,
        knockback_multiplier: Option<f32>,
        immune_blocks: Option<&'static Tag>,
    ) -> Self {
        Self {
            explodes_blocks,
            damages_entities,
            knockback_multiplier,
            immune_blocks,
        }
    }
}

impl ExplosionDamageCalculator for SimpleExplosionDamageCalculator {
    fn get_block_explosion_resistance(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        block: &Block,
        fluid: &pumpkin_data::fluid::FluidState,
    ) -> Option<f32> {
        if let Some(immune_tag) = self.immune_blocks {
            return block.has_tag(immune_tag).then_some(3_600_000.0);
        }
        if block.default_state.is_air() && fluid.is_empty {
            None
        } else {
            Some(fluid.blast_resistance.max(block.blast_resistance))
        }
    }

    fn should_block_explode(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        _block: &Block,
        _power: f32,
    ) -> bool {
        self.explodes_blocks
    }

    fn should_damage_entity(&self, _explosion: &Explosion, _entity: &dyn EntityBase) -> bool {
        self.damages_entities
    }

    fn get_knockback_multiplier(&self, _entity: &dyn EntityBase) -> f32 {
        self.knockback_multiplier.unwrap_or(1.0)
    }
}

pub struct ExplosionResult {
    pub block_count: u32,
    pub player_knockback: FxHashMap<i32, Vector3<f64>>,
}

pub struct Explosion<'a> {
    power: f32,
    pos: Vector3<f64>,
    block_interaction: BlockInteraction,
    damage_calculator: Option<Arc<dyn ExplosionDamageCalculator>>,
    preserve_rails: bool,
    source: Option<&'a dyn EntityBase>,
    /// The entity credited with the explosion when it isn't the source itself.
    indirect_source: Option<Arc<dyn EntityBase>>,
    source_is_indirect: bool,
    damage_type: Option<DamageType>,
    small_particle: Particle,
    large_particle: Particle,
    sound: Sound,
    from_wind_charge: bool,
    create_fire: bool,
}

impl<'a> Explosion<'a> {
    #[must_use]
    pub const fn new(power: f32, pos: Vector3<f64>, block_interaction: BlockInteraction) -> Self {
        Self {
            power,
            pos,
            block_interaction,
            damage_calculator: None,
            preserve_rails: false,
            source: None,
            indirect_source: None,
            source_is_indirect: false,
            damage_type: None,
            small_particle: Particle::Explosion,
            large_particle: Particle::ExplosionEmitter,
            sound: Sound::EntityGenericExplode,
            from_wind_charge: false,
            create_fire: false,
        }
    }

    #[must_use]
    pub fn with_damage_calculator(
        mut self,
        calculator: Arc<dyn ExplosionDamageCalculator>,
    ) -> Self {
        self.damage_calculator = Some(calculator);
        self
    }

    #[must_use]
    pub const fn preserving_rails(mut self) -> Self {
        self.preserve_rails = true;
        self
    }

    /// Sets the entity that exploded and who gets the credit for it:
    /// primed TNT credits whoever lit it, a living entity itself and a projectile its living owner.
    #[must_use]
    pub fn with_source(mut self, source: &'a dyn EntityBase) -> Self {
        self.source = Some(source);
        self.source_is_indirect = source.get_living_entity().is_some();
        self.indirect_source = if self.source_is_indirect {
            None
        } else {
            let world = source.get_entity().world.load();
            let owner = source.cast_any().downcast_ref::<TNTEntity>().map_or_else(
                || {
                    source
                        .get_projectile_owner()
                        .and_then(|owner| owner.get(&world))
                },
                |tnt| tnt.owner.get(&world),
            );
            owner.filter(|owner| owner.get_living_entity().is_some())
        };
        self
    }

    /// Credits the explosion to `cause` instead of the source's owner, for sources that pass their own damage source.
    #[must_use]
    pub fn with_indirect_source(mut self, cause: Option<Arc<dyn EntityBase>>) -> Self {
        self.source_is_indirect = false;
        self.indirect_source = cause;
        self
    }

    #[must_use]
    pub const fn with_damage_type(mut self, damage_type: DamageType) -> Self {
        self.damage_type = Some(damage_type);
        self
    }

    #[must_use]
    pub const fn center(&self) -> Vector3<f64> {
        self.pos
    }

    #[must_use]
    pub const fn power(&self) -> f32 {
        self.power
    }

    #[must_use]
    pub const fn source(&self) -> Option<&'a dyn EntityBase> {
        self.source
    }

    #[must_use]
    pub fn indirect_source(&self) -> Option<&dyn EntityBase> {
        if self.source_is_indirect {
            self.source
        } else {
            self.indirect_source.as_deref()
        }
    }

    /// Vanilla `DamageSources#explosion` selects `player_explosion` whenever both the direct
    /// source and its indirect source exist: the living source itself (creeper, wither) or a
    /// TNT/projectile owner. Only an unattributed explosion uses the plain `explosion` type.
    #[must_use]
    pub fn damage_type(&self) -> DamageType {
        self.damage_type.unwrap_or(
            if self.source.is_some() && self.indirect_source().is_some() {
                DamageType::PLAYER_EXPLOSION
            } else {
                DamageType::EXPLOSION
            },
        )
    }

    #[must_use]
    pub const fn with_particles_and_sound(
        mut self,
        small_particle: Particle,
        large_particle: Particle,
        sound: Sound,
    ) -> Self {
        self.small_particle = small_particle;
        self.large_particle = large_particle;
        self.sound = sound;
        self
    }

    #[must_use]
    pub const fn from_wind_charge(mut self) -> Self {
        self.from_wind_charge = true;
        self
    }

    /// Mirrors `ServerExplosion.canTriggerBlocks`: only a wind charge triggers blocks, and a
    /// breeze wind charge needs `mobGriefing` (a thrown wind charge always may).
    #[must_use]
    pub fn can_trigger_blocks(&self, world: &World) -> bool {
        if self.block_interaction != BlockInteraction::TriggerBlock {
            return false;
        }
        match self.source {
            Some(source)
                if source.get_entity().entity_type.id == EntityType::BREEZE_WIND_CHARGE.id =>
            {
                world.level_info.load().game_rules.mob_griefing
            }
            _ => true,
        }
    }

    /// Mirrors `ServerExplosion.isSmall`.
    #[must_use]
    pub const fn is_small(&self) -> bool {
        self.power < 2.0 || matches!(self.block_interaction, BlockInteraction::Keep)
    }

    #[must_use]
    pub const fn particle(&self) -> Particle {
        if self.is_small() {
            self.small_particle
        } else {
            self.large_particle
        }
    }

    #[must_use]
    pub const fn sound(&self) -> Sound {
        self.sound
    }

    #[must_use]
    pub const fn with_fire(mut self) -> Self {
        self.create_fire = true;
        self
    }

    fn protects_rail(&self, world: &World, pos: &BlockPos, block: &Block) -> bool {
        self.preserve_rails && (Self::is_rail(block) || Self::is_rail(world.get_block(&pos.up())))
    }

    fn is_rail(block: &Block) -> bool {
        block.id == Block::RAIL.id
            || block.id == Block::POWERED_RAIL.id
            || block.id == Block::DETECTOR_RAIL.id
            || block.id == Block::ACTIVATOR_RAIL.id
    }

    fn place_fire(
        world: &Arc<World>,
        affected_positions: &FxHashSet<BlockPos>,
        fire_roll: &mut impl FnMut() -> u8,
    ) {
        for pos in affected_positions {
            if !should_create_fire(
                fire_roll(),
                world.get_block_state(pos),
                world.get_block_state(&pos.down()),
            ) {
                continue;
            }

            let fire_block = FireBlockBase::get_fire_type(world, pos);
            let fire_state_id = FireBlock.get_state_for_position(world, &fire_block, pos);
            // Vanilla re-checks `BlockState.isAir()`, which also covers cave and void air.
            let _ =
                world.set_block_state_if(pos, fire_state_id, BlockFlags::NOTIFY_ALL, |state_id| {
                    BlockState::from_id(state_id).is_air()
                });
        }
    }

    #[allow(clippy::too_many_lines)]
    fn get_blocks_to_destroy(
        &self,
        world: &World,
    ) -> (
        FxHashMap<BlockPos, (&'static Block, &'static BlockState)>,
        FxHashSet<BlockPos>,
        u32,
    ) {
        let mut map = FxHashMap::default();
        let mut affected_positions = FxHashSet::default();
        let mut to_blow_positions = FxHashSet::default();

        let mut chunk_cache: FxHashMap<
            pumpkin_util::math::vector2::Vector2<i32>,
            Option<Arc<ChunkData>>,
        > = FxHashMap::default();

        let default_calc = DefaultExplosionDamageCalculator;
        let calc: &dyn ExplosionDamageCalculator = match &self.damage_calculator {
            Some(c) => c.as_ref(),
            None => &default_calc,
        };

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    if x > 0 && x < 15 && y > 0 && y < 15 && z > 0 && z < 15 {
                        continue;
                    }

                    let mut dir_x = f64::from(x) / 7.5 - 1.0;
                    let mut dir_y = f64::from(y) / 7.5 - 1.0;
                    let mut dir_z = f64::from(z) / 7.5 - 1.0;

                    let length = (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z).sqrt();
                    dir_x /= length;
                    dir_y /= length;
                    dir_z /= length;

                    let mut pos_x = self.pos.x;
                    let mut pos_y = self.pos.y;
                    let mut pos_z = self.pos.z;

                    let random_val = world
                        .explosion_random
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .next_f32();
                    let mut h = self.power * random_val.mul_add(0.6, 0.7);

                    while h > 0.0 {
                        let block_pos = BlockPos::floored(pos_x, pos_y, pos_z);

                        if !world.is_in_build_limit(block_pos) {
                            break;
                        }

                        let (chunk_pos, relative) = block_pos.chunk_and_chunk_relative_position();

                        // Vanilla `getBlockState` force-loads unloaded chunks. Mirror that so
                        // rays entering a neighbouring unloaded chunk still destroy its blocks.
                        let chunk_opt = chunk_cache
                            .entry(chunk_pos)
                            .or_insert_with(|| load_chunk_for_explosion(world, chunk_pos));

                        let state_id = if let Some(chunk) = chunk_opt {
                            chunk
                                .section
                                .get_block_absolute_y(
                                    relative.x as usize,
                                    block_pos.0.y,
                                    relative.z as usize,
                                )
                                .unwrap_or(Block::AIR.default_state.id)
                        } else {
                            Block::AIR.default_state.id
                        };

                        let (block, state) = BlockState::from_id_with_block(state_id);

                        let (_fluid, fluid_state) = Fluid::from_state_id(state_id).map_or_else(
                            || {
                                if block.is_waterlogged(state_id) {
                                    (&Fluid::FLOWING_WATER, &Fluid::FLOWING_WATER.states[0])
                                } else {
                                    (&Fluid::EMPTY, &Fluid::EMPTY.states[0])
                                }
                            },
                            |raw_fluid| {
                                let f = raw_fluid.to_flowing();
                                (f, &f.states[0])
                            },
                        );

                        if h > 0.0 {
                            affected_positions.insert(block_pos);
                        }

                        if !state.is_air() || !fluid_state.is_empty {
                            let protects_rail = self.protects_rail(world, &block_pos, block);
                            let resistance = if protects_rail {
                                Some(0.0)
                            } else {
                                calc.get_block_explosion_resistance(
                                    self,
                                    world,
                                    &block_pos,
                                    block,
                                    fluid_state,
                                )
                            };

                            // The exploding entity adjusts resistance
                            // unless the explosion brought its own calculator.
                            let resistance = match (self.source, &self.damage_calculator) {
                                (Some(source), None) => resistance
                                    .map(|r| source.get_block_explosion_resistance(block, r)),
                                _ => resistance,
                            };
                            if let Some(resistance) = resistance {
                                h -= (resistance + 0.3) * 0.3;
                            }

                            if h > 0.0
                                && !protects_rail
                                && calc.should_block_explode(self, world, &block_pos, block, h)
                            {
                                map.insert(block_pos, (block, state));
                                // Vanilla adds every destroyable position to `toBlow`, and
                                // `explode()` returns `toBlow.size()` for the client smoke count.
                                to_blow_positions.insert(block_pos);
                            }
                        } else if self.create_fire
                            && h > 0.0
                            && calc.should_block_explode(self, world, &block_pos, block, h)
                        {
                            // Vanilla keeps air positions in `toBlow` when fire is enabled.
                            to_blow_positions.insert(block_pos);
                        }

                        pos_x += dir_x * 0.3;
                        pos_y += dir_y * 0.3;
                        pos_z += dir_z * 0.3;
                        h -= 0.225_000_01;
                    }
                }
            }
        }
        (map, affected_positions, to_blow_positions.len() as u32)
    }

    fn damage_entities(&self, world: &Arc<World>) -> FxHashMap<i32, Vector3<f64>> {
        let mut player_knockback = FxHashMap::default();
        // Explosion is too small
        if self.power < 1.0e-5 {
            return player_knockback;
        }

        let radius = self.power as f64 * 2.0;
        let min_x = (self.pos.x - radius - 1.0).floor() as i32;
        let max_x = (self.pos.x + radius + 1.0).floor() as i32;
        let min_y = (self.pos.y - radius - 1.0).floor() as i32;
        let max_y = (self.pos.y + radius + 1.0).floor() as i32;
        let min_z = (self.pos.z - radius - 1.0).floor() as i32;
        let max_z = (self.pos.z + radius + 1.0).floor() as i32;

        let search_box = BoundingBox::new(
            Vector3::new(min_x as f64, min_y as f64, min_z as f64),
            Vector3::new(max_x as f64, max_y as f64, max_z as f64),
        );

        let entities = world.get_all_at_box(&search_box);

        let default_calc = DefaultExplosionDamageCalculator;
        let calc: &dyn ExplosionDamageCalculator = match &self.damage_calculator {
            Some(c) => c.as_ref(),
            None => &default_calc,
        };

        for entity_base in entities {
            if entity_base.is_immune_to_explosion() {
                continue;
            }

            // Skip spectators (no damage, no knockback)
            if entity_base.is_spectator() {
                continue;
            }

            let entity = entity_base.get_entity();
            if self
                .source
                .is_some_and(|source| source.get_entity().entity_id == entity.entity_id)
            {
                continue;
            }

            let distance = (entity.pos.load().squared_distance_to_vec(&self.pos)).sqrt() / radius;
            if distance > 1.0 {
                continue;
            }

            let should_damage = calc.should_damage_entity(self, entity_base.as_ref());
            let knockback_multiplier = calc.get_knockback_multiplier(entity_base.as_ref()) as f64;

            let exposure = if !should_damage && knockback_multiplier == 0.0 {
                0.0
            } else {
                Self::calculate_exposure(&self.pos, entity, world) as f64
            };

            if exposure == 0.0 {
                continue;
            }

            if should_damage
                && self
                    .source
                    .is_none_or(|source| source.do_teams_allow_damage(entity_base.as_ref()))
            {
                let damage =
                    calc.get_entity_damage_amount(self, entity_base.as_ref(), exposure as f32);
                entity_base.damage_with_context(
                    entity_base.as_ref(),
                    damage,
                    self.damage_type(),
                    None,
                    self.source,
                    self.indirect_source(),
                );
            }

            // Calculate and apply knockback
            let dir_pos = if entity.entity_type == &EntityType::TNT {
                entity.pos.load()
            } else {
                entity.get_eye_pos()
            };
            let direction = (dir_pos - self.pos).normalize();

            let knockback_resistance = entity_base.get_living_entity().map_or(0.0, |living| {
                living.get_attribute_value(&Attributes::EXPLOSION_KNOCKBACK_RESISTANCE)
            });
            let knockback_power =
                (1.0 - distance) * exposure * knockback_multiplier * (1.0 - knockback_resistance);
            let knockback = direction * knockback_power;
            // Vanilla `ServerExplosion.hurtEntities`: creative flyers get no knockback.
            if !entity_base
                .get_player()
                .is_some_and(|player| player.is_creative() && player.is_flying())
            {
                // A living entity's goes through `EntityKnockbackEvent`
                let knockback = if entity_base.get_living_entity().is_some() {
                    entity.fire_knockback_events(knockback_power, knockback, None)
                } else {
                    Some(knockback)
                };
                if let Some(knockback) = knockback {
                    if entity_base.get_player().is_some_and(|player| {
                        matches!(player.client.as_ref(), ClientPlatform::Java(_))
                    }) {
                        // Java applies this impulse to its own current motion in handleExplosion.
                        // Sending server velocity would replay previous client-authoritative launches.
                        player_knockback.insert(entity.entity_id, knockback);
                    } else {
                        entity.add_velocity(knockback);
                    }
                }
            }

            self.on_explosion_hit(entity_base.as_ref());
        }
        player_knockback
    }

    /// A pushed redirectable projectile now belongs to whoever caused the explosion, and a player remembers the
    /// impulse for fall damage.
    fn on_explosion_hit(&self, entity_base: &dyn EntityBase) {
        if is_redirectable(entity_base.get_entity())
            && let Some(owner) = entity_base.get_projectile_owner()
        {
            owner.set_uuid(
                self.indirect_source()
                    .map(|cause| cause.get_entity().entity_uuid),
            );
        } else if let Some(player) = entity_base.get_player() {
            player
                .living_entity
                .set_ignore_fall_damage_from_current_impulse(
                    self.from_wind_charge,
                    player.position(),
                );
        }
    }

    fn calculate_exposure(
        explosion_pos: &Vector3<f64>,
        entity: &Entity,
        world: &Arc<World>,
    ) -> f32 {
        let bbox = entity.bounding_box.load();

        let step_x = 1.0 / ((bbox.max.x - bbox.min.x) * 2.0 + 1.0);
        let step_y = 1.0 / ((bbox.max.y - bbox.min.y) * 2.0 + 1.0);
        let step_z = 1.0 / ((bbox.max.z - bbox.min.z) * 2.0 + 1.0);

        if step_x < 0.0 || step_y < 0.0 || step_z < 0.0 {
            return 0.0;
        }

        let offset_x = (1.0 - (1.0 / step_x).floor() * step_x) / 2.0;
        let offset_z = (1.0 - (1.0 / step_z).floor() * step_z) / 2.0;

        let mut visible_points = 0;
        let mut total_points = 0;

        let mut k = 0.0;
        while k <= 1.0 {
            let mut l = 0.0;
            while l <= 1.0 {
                let mut m = 0.0;
                while m <= 1.0 {
                    let n = bbox.min.x + (bbox.max.x - bbox.min.x) * k;
                    let o = bbox.min.y + (bbox.max.y - bbox.min.y) * l;
                    let p = bbox.min.z + (bbox.max.z - bbox.min.z) * m;

                    let vec3d = Vector3::new(n + offset_x, o, p + offset_z);

                    if world
                        .clip(&pumpkin_world::clip::ClipContext::new(
                            vec3d,
                            *explosion_pos,
                            pumpkin_world::clip::ClipBlock::Collider,
                            pumpkin_world::clip::ClipFluid::None,
                        ))
                        .is_miss
                    {
                        visible_points += 1;
                    }

                    total_points += 1;
                    m += step_z;
                }
                l += step_y;
            }
            k += step_x;
        }

        if total_points == 0 {
            return 0.0;
        }

        visible_points as f32 / total_points as f32
    }

    /// Damages entities, then interacts with blocks. Returns the removed block count and the
    /// knockback owed to Java players, who apply it themselves.
    pub fn explode(&self, world: &Arc<World>) -> ExplosionResult {
        // Vanilla `ServerExplosion#explode` announces the explosion before it touches blocks or
        // entities, so sculk sensors and vibration listeners hear it.
        world.emit_game_event_with_context(
            GameEvent::Explode.name(),
            self.pos,
            &GameEventContext {
                source_entity: self.source,
            },
        );
        let player_knockback = self.damage_entities(world);
        let block_count = self.explode_with_fire_roll(world, || {
            u8::try_from(
                world
                    .explosion_random
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .next_bounded_i32(3),
            )
            .unwrap_or_default()
        });
        ExplosionResult {
            block_count,
            player_knockback,
        }
    }

    fn fire_explode_event(
        &self,
        world: &Arc<World>,
        event_blocks: Vec<BlockPos>,
        default_yield: f32,
    ) -> (bool, f32, Vec<BlockPos>) {
        if let Some(source) = self.source {
            let mut event =
                crate::plugin::api::events::entity::entity_explode::EntityExplodeEvent::new(
                    source.get_entity().entity_id,
                    self.pos,
                    event_blocks,
                    default_yield,
                    world.clone(),
                );
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            (event.cancelled, event.yield_rate, event.blocks)
        } else {
            let center_pos = BlockPos::floored(self.pos.x, self.pos.y, self.pos.z);
            let mut event =
                crate::plugin::api::events::block::block_explode::BlockExplodeEvent::new(
                    center_pos,
                    event_blocks,
                    default_yield,
                    world.clone(),
                );
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            (event.cancelled, event.yield_rate, event.blocks)
        }
    }

    fn explode_with_fire_roll(&self, world: &Arc<World>, mut fire_roll: impl FnMut() -> u8) -> u32 {
        match self.block_interaction {
            BlockInteraction::Keep => {
                if self.create_fire {
                    let (_, affected_positions, _) = self.get_blocks_to_destroy(world);
                    Self::place_fire(world, &affected_positions, &mut fire_roll);
                }
                0
            }
            BlockInteraction::TriggerBlock => {
                let (blocks, affected_positions, _) = self.get_blocks_to_destroy(world);
                for (pos, (block, _state)) in &blocks {
                    let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);
                    if let Some(pumpkin_block) = pumpkin_block {
                        pumpkin_block.explode(ExplodeArgs {
                            world,
                            block,
                            position: pos,
                            explosion: self,
                        });
                    }
                }
                if self.create_fire {
                    Self::place_fire(world, &affected_positions, &mut fire_roll);
                }
                0
            }
            BlockInteraction::Destroy | BlockInteraction::DestroyWithDecay => {
                let (mut blocks, affected_positions, to_blow_count) =
                    self.get_blocks_to_destroy(world);
                let decay_drops = self.block_interaction == BlockInteraction::DestroyWithDecay;

                // Paper ServerExplosion.interactWithBlocks fires the event after the destroyed
                // list is known and replaces it with the plugin-filtered event list. The default
                // yield is 1/radius when drops decay, otherwise 1.
                let default_yield = if !decay_drops {
                    1.0
                } else if self.power > 0.0 && self.power.is_finite() {
                    1.0 / self.power
                } else {
                    0.0
                };
                let event_blocks: Vec<BlockPos> = blocks.keys().copied().collect();
                let (cancelled, yield_rate, allowed) =
                    self.fire_explode_event(world, event_blocks, default_yield);

                // Vanilla BlockBehaviour.dropResources feeds 1/yield into the explosion_radius
                // loot parameter, and only when yield is below 1.
                let explosion_radius = explosion_radius_from_yield(yield_rate);
                let allowed: FxHashSet<BlockPos> = allowed.into_iter().collect();
                blocks = filter_destroyed_blocks(blocks, &allowed);

                if cancelled {
                    if self.create_fire {
                        Self::place_fire(world, &affected_positions, &mut fire_roll);
                    }
                    return 0;
                }

                for (pos, (block, state)) in &blocks {
                    let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);

                    // Vanilla onExplosionHit runs before the block is removed, so wind-charge
                    // reactions (button press, lever pull, candle extinguish) still see it placed.
                    if let Some(pumpkin_block) = pumpkin_block {
                        pumpkin_block.explode(ExplodeArgs {
                            world,
                            block,
                            position: pos,
                            explosion: self,
                        });
                    }

                    if pumpkin_block.is_none_or(|s| s.should_drop_items_on_explosion()) {
                        let is_raining = world.is_raining();
                        let is_thundering = world.is_thundering();
                        let params = LootContextParameters {
                            block_state: Some(state),
                            explosion_radius,
                            position: Some(pumpkin_util::math::vector3::Vector3::new(
                                pos.0.x as f64,
                                pos.0.y as f64,
                                pos.0.z as f64,
                            )),
                            world_time: world.level_info.load().day_time as u64,
                            // Vanilla Explosion.finalizeExplosion passes getIndirectSourceEntity()
                            // to Block.dropResources, so loot tables see this_entity.
                            this_entity: self.indirect_source(),
                            is_raining: Some(is_raining),
                            is_thundering: Some(is_thundering),
                            ..Default::default()
                        };
                        drop_loot(world, block, pos, None, false, &params);
                    }

                    world.set_block_state(pos, BlockStateId::AIR, BlockFlags::NOTIFY_ALL);
                    world.close_container_screens_at(pos);
                }
                if self.create_fire {
                    Self::place_fire(world, &affected_positions, &mut fire_roll);
                }

                to_blow_count
            }
        }
    }
}

fn explosion_radius_from_yield(yield_rate: f32) -> Option<f32> {
    (yield_rate < 1.0).then_some(1.0 / yield_rate)
}

fn filter_destroyed_blocks(
    blocks: FxHashMap<BlockPos, (&'static Block, &'static BlockState)>,
    allowed: &FxHashSet<BlockPos>,
) -> FxHashMap<BlockPos, (&'static Block, &'static BlockState)> {
    blocks
        .into_iter()
        .filter(|(pos, _)| allowed.contains(pos))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Weak};

    use arc_swap::ArcSwap;
    use pumpkin_config::world::LevelConfig;
    use pumpkin_data::{
        Block, BlockDirection, damage::DamageType, dimension::Dimension, entity::EntityType,
        fluid::Fluid, particle::Particle, sound::Sound, tag,
    };
    use pumpkin_util::{
        math::{position::BlockPos, vector2::Vector2, vector3::Vector3},
        world_seed::Seed,
    };
    use pumpkin_world::{chunk::ChunkData, level::Level};

    use crate::{
        block::registry::default_registry,
        entity::{Entity, passive::allay::AllayEntity},
        world::{LevelData, World},
    };

    use super::{
        BlockFlags, BlockInteraction, Explosion, ExplosionDamageCalculator,
        SimpleExplosionDamageCalculator, explosion_radius_from_yield, filter_destroyed_blocks,
        should_create_fire,
    };

    type TestWorld = (Arc<World>, Arc<Level>, tempfile::TempDir);

    fn test_world() -> Result<TestWorld, Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            folder.path().to_path_buf(),
            0,
            Dimension::OVERWORLD,
        );
        let level_info = Arc::new(ArcSwap::from_pointee(LevelData::default(Seed(0))));
        let world = Arc::new(World::load(
            level.clone(),
            level_info,
            Dimension::OVERWORLD,
            default_registry(),
            crate::world::weather::WeatherSetup::default(),
            Weak::new(),
        ));
        level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        Ok((world, level, folder))
    }

    #[test]
    fn explosions_are_non_incendiary_by_default() {
        let explosion = Explosion::new(5.0, Vector3::new(0.0, 0.0, 0.0), BlockInteraction::Destroy);
        assert!(!explosion.create_fire);
    }

    #[test]
    fn explosions_can_opt_in_to_fire() {
        let explosion =
            Explosion::new(5.0, Vector3::new(0.0, 0.0, 0.0), BlockInteraction::Destroy).with_fire();
        assert!(explosion.create_fire);
    }

    #[test]
    fn fire_requires_air_a_zero_roll_and_solid_render_support() {
        let air = &Block::AIR.default_state;
        let stone = &Block::STONE.default_state;

        assert!(should_create_fire(0, air, stone));
        assert!(!should_create_fire(1, air, stone));
        assert!(!should_create_fire(2, air, stone));
        assert!(!should_create_fire(u8::MAX, air, stone));
        assert!(!should_create_fire(0, Block::FIRE.default_state, stone));
        assert!(!should_create_fire(0, air, Block::AIR.default_state));
        let glass = &Block::GLASS.default_state;
        assert!(glass.is_side_solid(BlockDirection::Up));
        assert!(!glass.is_solid_render());
        assert!(!should_create_fire(0, air, glass));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn keep_explosion_ignites_affected_air_without_destroying_blocks()
    -> Result<(), Box<dyn std::error::Error>> {
        let (world, level, _folder) = test_world()?;
        for x in 4..=11 {
            for z in 4..=11 {
                world.set_block_state(
                    &BlockPos::new(x, 63, z),
                    Block::STONE.default_state.id,
                    BlockFlags::empty(),
                );
                // Cave air still counts as air for vanilla fire placement.
                world.set_block_state(
                    &BlockPos::new(x, 64, z),
                    Block::CAVE_AIR.default_state.id,
                    BlockFlags::empty(),
                );
            }
        }

        let explosion =
            Explosion::new(4.0, Vector3::new(8.5, 64.0, 8.5), BlockInteraction::Keep).with_fire();
        let destroyed = explosion.explode_with_fire_roll(&world, || 0);
        let fire_was_created = (4..=11)
            .any(|x| (4..=11).any(|z| world.get_block(&BlockPos::new(x, 64, z)) == &Block::FIRE));
        let floor_remains = (4..=11)
            .all(|x| (4..=11).all(|z| world.get_block(&BlockPos::new(x, 63, z)) == &Block::STONE));
        level.shutdown().await;

        assert_eq!(destroyed, 0);
        assert!(fire_was_created);
        assert!(floor_remains);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn incendiary_block_count_includes_air_positions()
    -> Result<(), Box<dyn std::error::Error>> {
        let (world, level, _folder) = test_world()?;
        let center = Vector3::new(8.5, 64.0, 8.5);

        // The loaded chunk is all air, so no blocks are destroyed. Only a fire-enabled
        // explosion counts those air positions, matching vanilla's `toBlow.size()` that
        // the client uses for its smoke-particle count.
        let incendiary = Explosion::new(4.0, center, BlockInteraction::Destroy).with_fire();
        let with_fire = incendiary.explode_with_fire_roll(&world, || 0);
        let non_incendiary = Explosion::new(4.0, center, BlockInteraction::Destroy);
        let without_fire = non_incendiary.explode_with_fire_roll(&world, || 0);

        level.shutdown().await;

        assert_eq!(without_fire, 0);
        assert!(with_fire > 0);
        Ok(())
    }

    #[tokio::test]
    async fn only_wind_charge_explosions_can_trigger_blocks()
    -> Result<(), Box<dyn std::error::Error>> {
        let (world, level, _folder) = test_world()?;
        let destroy = Explosion::new(3.0, Vector3::new(0.0, 64.0, 0.0), BlockInteraction::Destroy);
        assert!(!destroy.can_trigger_blocks(&world));

        let wind_charge = Explosion::new(
            1.2,
            Vector3::new(0.0, 64.0, 0.0),
            BlockInteraction::TriggerBlock,
        );
        assert!(wind_charge.can_trigger_blocks(&world));

        level.shutdown().await;
        Ok(())
    }

    #[test]
    fn tnt_minecart_rail_protection_covers_every_rail_type() {
        for rail in [
            &Block::RAIL,
            &Block::POWERED_RAIL,
            &Block::DETECTOR_RAIL,
            &Block::ACTIVATOR_RAIL,
        ] {
            assert!(Explosion::is_rail(rail));
        }
        assert!(!Explosion::is_rail(&Block::STONE));
    }

    #[tokio::test]
    async fn wind_burst_explosion_uses_large_gust_particle_and_immune_block_resistance()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let explosion = Explosion::new(
            3.5,
            Vector3::new(0.0, 64.0, 0.0),
            BlockInteraction::TriggerBlock,
        )
        .with_particles_and_sound(
            Particle::GustEmitterSmall,
            Particle::GustEmitterLarge,
            Sound::EntityWindChargeWindBurst,
        );
        assert!(!explosion.is_small());
        assert_eq!(explosion.particle(), Particle::GustEmitterLarge);
        assert_eq!(explosion.sound(), Sound::EntityWindChargeWindBurst);

        let calc = SimpleExplosionDamageCalculator::new(
            true,
            false,
            Some(1.2),
            Some(&tag::Block::MINECRAFT_BLOCKS_WIND_CHARGE_EXPLOSIONS),
        );
        let pos = BlockPos::new(0, 64, 0);
        let empty_fluid = &Fluid::EMPTY.states[0];
        assert_eq!(
            calc.get_block_explosion_resistance(
                &explosion,
                &world,
                &pos,
                &Block::BEDROCK,
                empty_fluid
            ),
            Some(3_600_000.0)
        );
        assert_eq!(
            calc.get_block_explosion_resistance(
                &explosion,
                &world,
                &pos,
                &Block::LEVER,
                empty_fluid
            ),
            None
        );
        assert!(calc.should_block_explode(&explosion, &world, &pos, &Block::LEVER, 1.0));

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn entity_explosion_uses_player_explosion_damage_type()
    -> Result<(), Box<dyn std::error::Error>> {
        let (world, level, _folder) = test_world()?;
        let source = AllayEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::ALLAY,
        ));

        let explosion =
            Explosion::new(3.0, Vector3::new(0.0, 64.0, 0.0), BlockInteraction::Destroy)
                .with_source(source.as_ref());
        assert_eq!(explosion.damage_type(), DamageType::PLAYER_EXPLOSION);

        let no_source =
            Explosion::new(3.0, Vector3::new(0.0, 64.0, 0.0), BlockInteraction::Destroy);
        assert_eq!(no_source.damage_type(), DamageType::EXPLOSION);

        level.shutdown().await;
        Ok(())
    }

    use rustc_hash::{FxHashMap, FxHashSet};

    #[test]
    fn explode_event_filters_destroyed_blocks() {
        let keep = BlockPos::new(1, 2, 3);
        let drop = BlockPos::new(4, 5, 6);
        let mut blocks = FxHashMap::default();
        blocks.insert(keep, (&Block::STONE, Block::STONE.default_state));
        blocks.insert(drop, (&Block::DIRT, Block::DIRT.default_state));
        let allowed: FxHashSet<_> = [keep].into_iter().collect();

        let filtered = filter_destroyed_blocks(blocks, &allowed);

        assert!(filtered.contains_key(&keep));
        assert!(!filtered.contains_key(&drop));
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn explosion_radius_uses_inverse_yield_only_for_decay() {
        assert!(explosion_radius_from_yield(0.25).is_some_and(|r| (r - 4.0).abs() < 1e-6));
        assert!(explosion_radius_from_yield(1.0).is_none());
        assert!(explosion_radius_from_yield(2.0).is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn explosion_force_loads_unloaded_chunks() -> Result<(), Box<dyn std::error::Error>> {
        let (world, level, _folder) = test_world()?;
        // Only chunk (0,0) is loaded by test_world; an explosion ray reaching (1,0)
        // must force-load it instead of reading the old code's `None` as air.
        let target = Vector2::new(1, 0);
        let injected = ChunkData::empty_sync(target.x, target.y);
        let producer_level = level.clone();
        tokio::spawn(async move {
            for _ in 0..200 {
                producer_level
                    .chunk_listener
                    .process_new_chunk(target, &injected);
                if producer_level.loaded_chunks.contains_key(&target) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        });

        let loaded = super::load_chunk_for_explosion(&world, target);
        level.shutdown().await;
        assert!(loaded.is_some());
        Ok(())
    }
}
