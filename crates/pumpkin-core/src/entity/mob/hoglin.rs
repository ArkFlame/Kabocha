use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::brain::memory::types;
use crate::entity::ai::brain::sensing::SensorType;
use crate::entity::ai::brain::{Brain, BrainProvider, memory::PackedMemories};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::mob::spawn::SpawnGroupData;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    living::LivingEntity,
    mob::{Mob, MobEntity},
};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::world::World;

/// Vanilla `HoglinAi.piglinsOutnumberHoglins`: piglins outnumber when there are strictly more
/// visible adult piglins than visible adult hoglins plus the fleeing hoglin itself.
const fn outnumbered_by_piglins(piglin_count: i32, hoglin_count: i32) -> bool {
    piglin_count > hoglin_count + 1
}

/// Reads the visible-adult counts the brain sensor stores and decides whether to retreat.
fn piglins_outnumber_hoglins(hoglin: &HoglinEntity) -> bool {
    if hoglin.is_baby() {
        return false;
    }
    hoglin
        .mob_entity
        .read_brain(|state| {
            outnumbered_by_piglins(
                state
                    .get(types::VISIBLE_ADULT_PIGLIN_COUNT)
                    .copied()
                    .unwrap_or(0),
                state
                    .get(types::VISIBLE_ADULT_HOGLIN_COUNT)
                    .copied()
                    .unwrap_or(0),
            )
        })
        .unwrap_or(false)
}

/// Goal predicate mirroring the piglin retreat: flee piglins while outnumbered by them.
fn flee_from_piglins_when_outnumbered(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<HoglinEntity>()
        .is_some_and(piglins_outnumber_hoglins)
}

/// Hunting is allowed only while the hoglin is not pacified by a nearby repellent.
///
/// Mirrors vanilla `HoglinAi.isPacified` driven by `BecomePassiveIfMemoryPresent(NEAREST_REPELLENT, 200)`:
/// while a repellent is sensed the hoglin neither starts nor continues a player hunt.
fn can_hunt_while_not_pacified(mob: &dyn Mob) -> bool {
    !mob.cast_any()
        .downcast_ref::<HoglinEntity>()
        .is_some_and(|hoglin| {
            hoglin
                .mob_entity
                .read_brain(|state| state.get(types::NEAREST_REPELLENT).is_some())
                .unwrap_or(false)
        })
}

pub struct HoglinEntity {
    pub mob_entity: MobEntity,
    pub immune_to_zombification: AtomicBool,
    pub time_in_overworld: AtomicI32,
    pub cannot_be_hunted: AtomicBool,
    pub ageable_data: AgeableData,
}

impl HoglinEntity {
    pub const CONVERSION_TIME: i32 = 300;
    pub const XP_REWARD: u32 = 5;
    pub const BABY_XP_REWARD: u32 = 3;

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut attributes = mob_entity
                .living_entity
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(health) = attributes.get_mut(&Attributes::MAX_HEALTH.id) {
                health.base_value = 40.0;
                health.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
                speed.base_value = 0.3;
                speed.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(knockback_res) = attributes.get_mut(&Attributes::KNOCKBACK_RESISTANCE.id) {
                knockback_res.base_value = 0.6;
                knockback_res.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(attack_kb) = attributes.get_mut(&Attributes::ATTACK_KNOCKBACK.id) {
                attack_kb.base_value = 1.0;
                attack_kb.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(damage) = attributes.get_mut(&Attributes::ATTACK_DAMAGE.id) {
                damage.base_value = 6.0;
                damage.dirty.store(true, Ordering::Relaxed);
            }
        }
        mob_entity.living_entity.health.store(40.0);

        let hoglin = Self {
            mob_entity,
            immune_to_zombification: AtomicBool::new(false),
            time_in_overworld: AtomicI32::new(0),
            cannot_be_hunted: AtomicBool::new(false),
            ageable_data: AgeableData::default(),
        };
        let mob_arc = Arc::new(hoglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                2,
                Box::new(
                    AvoidEntityGoal::new(TargetMatch::Type(&EntityType::PIGLIN), 8.0, 1.0, 1.2)
                        .as_long_as(flee_from_piglins_when_outnumbered),
                ),
            );
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true)
                    .gated_by(can_hunt_while_not_pacified),
            );
        };

        mob_arc
    }

    pub fn is_immune_to_zombification(&self) -> bool {
        self.immune_to_zombification.load(Ordering::Relaxed)
    }

    pub fn set_immune_to_zombification(&self, immune: bool) {
        self.immune_to_zombification
            .store(immune, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.set_synced_data(
            pumpkin_data::tracked_data::hoglin::DATA_IMMUNE_TO_ZOMBIFICATION,
            immune,
        );
    }

    pub fn is_converting(&self, world: &World) -> bool {
        !self.is_immune_to_zombification()
            && !self.mob_entity.is_no_ai()
            && world.dimension.piglins_zombify
    }

    fn convert_to_zoglin(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        if world.level_info.load().difficulty != pumpkin_util::Difficulty::Peaceful {
            world.play_sound(
                Sound::EntityHoglinConvertedToZombified,
                SoundCategory::Hostile,
                &pos,
            );
        }

        let zoglin = crate::entity::r#type::from_type(
            &EntityType::ZOGLIN,
            pos,
            &world,
            uuid::Uuid::new_v4(),
        );

        let zoglin_base = zoglin.get_entity();
        zoglin_base.set_rotation(entity.yaw.load(), entity.pitch.load());
        zoglin_base.head_yaw.store(entity.head_yaw.load());
        zoglin_base.velocity.store(entity.velocity.load());

        if let Some(living) = zoglin.get_living_entity() {
            living.set_health(self.mob_entity.living_entity.health.load());
            // Vanilla conversion adds nausea to the resulting zoglin.
            living.add_effect(Effect {
                effect_type: &StatusEffect::NAUSEA,
                duration: 200,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            });
        }

        if let Some(custom_name) = &**entity.custom_name.load() {
            zoglin_base.set_custom_name(custom_name.clone());
        }

        // Vanilla ConversionType.SINGLE keeps the baby state.
        if self.is_baby()
            && let Some(mob) = zoglin.get_mob()
        {
            mob.spawn_as_baby();
        }

        world.spawn_entity(zoglin);
        entity.remove();
    }
}

impl AgeableMob for HoglinEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn age_boundary_reached(&self) {
        let damage = if self.is_baby() { 0.5 } else { 6.0 };
        self.mob_entity
            .living_entity
            .set_attribute_base(&Attributes::ATTACK_DAMAGE, damage);
        // Apply the common growth dismount rule to address MC-188617.
        self.age_boundary_reached_super();
    }
}

// TODO: vanilla hoglins are `Animal`s: crimson fungus breeds adults and grows babies.
impl Mob for HoglinEntity {
    fn can_be_leashed(&self) -> bool {
        true
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityHoglinStep, 0.15, 1.0);
    }

    /// Vanilla `Hoglin.getWalkTargetValue`: avoid tiles near a repellent, prefer crimson nylium.
    fn get_walk_target_value(&self, pos: &BlockPos) -> f32 {
        let entity = &self.get_mob_entity().living_entity.entity;
        let world = entity.world.load();
        let origin = entity.block_pos.load();
        let repellent = find_nearest_repellent(origin, |candidate| {
            world
                .get_block(&candidate)
                .has_tag(&pumpkin_data::tag::Block::MINECRAFT_HOGLIN_REPELLENTS)
        });
        if repellent.is_some_and(|repellent| is_near_repellent(repellent, *pos)) {
            return HOGLIN_REPELLENT_AVOID_VALUE;
        }
        if world.get_block(&pos.down()).id == pumpkin_data::Block::CRIMSON_NYLIUM.id {
            return 10.0;
        }
        0.0
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Builds a brain whose sensor records visible adult piglins and hoglins.
    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        BrainProvider {
            memory_types: &[],
            sensor_types: &[
                SensorType::NearestLivingEntities,
                SensorType::HoglinSpecific,
            ],
            activities: |_: &dyn Mob| Vec::new(),
        }
        .make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: crate::entity::mob::spawn::EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        if self.get_random().random::<f32>() < 0.2 {
            self.set_baby(true);
        }
        self.default_finalize_spawn(world, group_data)
    }

    fn should_drop_experience(&self) -> bool {
        true
    }

    /// Vanilla `Hoglin` implements `Monster` despite being ageable, so Peaceful despawn and
    /// loot/xp rules treat it as a monster.
    fn is_monster_class(&self) -> bool {
        true
    }

    fn get_base_experience_reward(&self) -> u32 {
        if self.is_baby() {
            Self::BABY_XP_REWARD
        } else {
            Self::XP_REWARD
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if self.is_immune_to_zombification() {
            nbt.put_bool("IsImmuneToZombification", true);
        }
        let time = self.time_in_overworld.load(Ordering::Relaxed);
        if time > 0 {
            nbt.put_int("TimeInOverworld", time);
        }
        if self.cannot_be_hunted.load(Ordering::Relaxed) {
            nbt.put_bool("CannotBeHunted", true);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(immune) = nbt.get_bool("IsImmuneToZombification") {
            self.set_immune_to_zombification(immune);
        }
        if let Some(time) = nbt.get_int("TimeInOverworld") {
            self.time_in_overworld.store(time, Ordering::Relaxed);
        }
        if let Some(cannot_hunt) = nbt.get_bool("CannotBeHunted") {
            self.cannot_be_hunted.store(cannot_hunt, Ordering::Relaxed);
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load();
        if self.is_converting(&world) {
            let time = self.time_in_overworld.fetch_add(1, Ordering::Relaxed) + 1;
            if time > Self::CONVERSION_TIME {
                self.convert_to_zoglin();
            }
        } else {
            self.time_in_overworld.store(0, Ordering::Relaxed);
        }
    }

    fn on_attack(&self, target: &dyn EntityBase) {
        // Vanilla HoglinBase.hurtAndThrowTarget: babies don't throw.
        if self.is_baby() {
            return;
        }
        throw_target(&self.mob_entity.living_entity, target);
        // Vanilla `HoglinAi.onHitTarget`: hitting a piglin while piglins outnumber hoglins
        // makes the hoglin flee instead of fighting.
        if target.get_entity().entity_type.id == EntityType::PIGLIN.id
            && piglins_outnumber_hoglins(self)
        {
            <Self as Mob>::set_mob_target(self, None, TargetReason::Fleeing);
            self.mob_entity.steered().navigator().stop();
        }
    }
}

/// Vanilla `HoglinBase.throwTarget`, shared with zoglins.
/// Walk target value for tiles inside the repellent avoidance radius (vanilla returns -1.0).
const HOGLIN_REPELLENT_AVOID_VALUE: f32 = -1.0;
/// Vanilla `HoglinSpecificSensor.findNearestRepellent` search box: `findBlocksInBoxByManhattanDistance(pos, 8, 4)`.
const REPELLENT_SEARCH_XZ: i32 = 8;
const REPELLENT_SEARCH_Y: i32 = 4;
/// Vanilla `HoglinAi.isPosNearNearestRepellent` uses a strict `closerThan(pos, 8.0)`.
const REPELLENT_AVOID_RADIUS: i32 = 8;

/// Returns the repellent with the smallest Manhattan distance from `origin` inside the search box.
/// Ties keep the first block found in scan order.
fn find_nearest_repellent(
    origin: BlockPos,
    is_repellent: impl Fn(BlockPos) -> bool,
) -> Option<BlockPos> {
    let mut best: Option<(i32, BlockPos)> = None;
    for dy in -REPELLENT_SEARCH_Y..=REPELLENT_SEARCH_Y {
        for dx in -REPELLENT_SEARCH_XZ..=REPELLENT_SEARCH_XZ {
            for dz in -REPELLENT_SEARCH_XZ..=REPELLENT_SEARCH_XZ {
                let distance = dx.abs() + dy.abs() + dz.abs();
                if best.is_some_and(|(best_distance, _)| best_distance <= distance) {
                    continue;
                }
                let candidate = BlockPos(Vector3::new(
                    origin.0.x + dx,
                    origin.0.y + dy,
                    origin.0.z + dz,
                ));
                if is_repellent(candidate) {
                    best = Some((distance, candidate));
                }
            }
        }
    }
    best.map(|(_, pos)| pos)
}

/// Strict Euclidean check of `pos` against the avoidance radius around `repellent`.
fn is_near_repellent(repellent: BlockPos, pos: BlockPos) -> bool {
    let dx = i64::from(repellent.0.x - pos.0.x);
    let dy = i64::from(repellent.0.y - pos.0.y);
    let dz = i64::from(repellent.0.z - pos.0.z);
    let radius = i64::from(REPELLENT_AVOID_RADIUS);
    dx * dx + dy * dy + dz * dz < radius * radius
}

pub fn throw_target(attacker: &LivingEntity, target: &dyn EntityBase) {
    // Vanilla: strength = attacker ATTACK_KNOCKBACK - target KNOCKBACK_RESISTANCE.
    let resistance = target.get_living_entity().map_or(0.0, |living| {
        living.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
    });
    let strength = attacker.get_attribute_value(&Attributes::ATTACK_KNOCKBACK) - resistance;
    if strength <= 0.0 {
        return;
    }
    let my_pos = attacker.entity.pos.load();
    let target_entity = target.get_entity();
    let target_pos = target_entity.pos.load();
    let dx = target_pos.x - my_pos.x;
    let dz = target_pos.z - my_pos.z;
    let dist = dx.hypot(dz).max(0.001);
    // Impulse adds to existing motion (keeps upward velocity).
    let vel = target_entity.velocity.load();
    target_entity.velocity.store(Vector3::new(
        vel.x + (dx / dist) * strength * 0.5,
        vel.y + 0.5,
        vel.z + (dz / dist) * strength * 0.5,
    ));
    target_entity
        .velocity_dirty
        .store(true, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin() -> BlockPos {
        BlockPos(Vector3::new(0, 64, 0))
    }

    fn at(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos(Vector3::new(x, y, z))
    }

    #[test]
    fn nearest_repellent_uses_manhattan_distance() {
        let found = find_nearest_repellent(origin(), |p| p == at(3, 64, 0) || p == at(1, 64, 1));
        assert_eq!(found, Some(at(1, 64, 1)));
    }

    #[test]
    fn repellent_search_respects_box_bounds() {
        assert_eq!(
            find_nearest_repellent(origin(), |p| p == at(9, 64, 0)),
            None
        );
        assert_eq!(
            find_nearest_repellent(origin(), |p| p == at(0, 69, 0)),
            None
        );
        assert_eq!(
            find_nearest_repellent(origin(), |p| p == at(0, 68, 0)),
            Some(at(0, 68, 0))
        );
    }

    #[test]
    fn avoidance_radius_is_strict() {
        let repellent = at(0, 64, 0);
        assert!(is_near_repellent(repellent, at(7, 64, 0)));
        assert!(!is_near_repellent(repellent, at(8, 64, 0)));
        assert!(!is_near_repellent(repellent, at(6, 64, 6)));
    }

    #[test]
    fn conversion_time_matches_vanilla() {
        assert_eq!(HoglinEntity::CONVERSION_TIME, 300);
    }

    #[test]
    fn outnumbered_by_piglins_matches_vanilla_boundary() {
        // Vanilla: piglinCount > hoglinCount + 1, where hoglinCount excludes the fleeing hoglin.
        assert!(!outnumbered_by_piglins(1, 0));
        assert!(!outnumbered_by_piglins(2, 1));
        assert!(outnumbered_by_piglins(2, 0));
        assert!(outnumbered_by_piglins(3, 1));
    }
}
