use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, PoisonError, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ageable::AgeableMob;
use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        patrol::{LongDistancePatrolGoal, PatrolData, PatrollingMonster},
        raider::{
            self, HoldGroundAttackGoal, PathfindToRaidGoal, Raider, RaiderCelebrationGoal,
            RaiderData, RaiderMoveThroughVillageGoal,
        },
    },
};

const RAVAGER_HUNTS: &[Hunt] = &[
    Hunt::new(3, TargetMatch::Type(&EntityType::PLAYER), true),
    Hunt::new(4, TargetMatch::VILLAGER_LIKE, true).only_mobs(is_adult),
    Hunt::new(4, TargetMatch::Type(&EntityType::IRON_GOLEM), true),
];

fn is_adult(mob: &dyn Mob) -> bool {
    !mob.as_ageable().is_some_and(AgeableMob::is_baby)
}

/// Vanilla `Ravager.aiStep` walk speeds: 0.35 while it has a target, 0.3 otherwise.
const HUNTING_SPEED: f64 = 0.35;
const WANDERING_SPEED: f64 = 0.3;
/// Vanilla `Mth.lerp(0.1, ...)` step applied to the walk speed each tick.
const SPEED_LERP: f64 = 0.1;

const ATTACK_DURATION: i32 = 10;
const STUN_DURATION: i32 = 40;
const ROAR_TRIGGER_TICK: i32 = 10;
const ROAR_AFTER_STUN_TICKS: i32 = 20;
const ROAR_RADIUS: f64 = 4.0;
const ROAR_DAMAGE: f32 = 6.0;
const STUN_CHANCE: f64 = 0.5;
const STRONG_KNOCKBACK: f64 = 4.0;

/// The three vanilla `Ravager` tick counters (`attackTick`, `stunnedTick`, `roarTick`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct RavagerTimers {
    attack: i32,
    stunned: i32,
    roar: i32,
}

/// Side effects vanilla fires when a timer crosses its trigger value.
struct TimerActions {
    roar: bool,
    stun_finished: bool,
}

/// Advances the counters in the same order as vanilla `Ravager.aiStep`.
const fn advance_timers(timers: &mut RavagerTimers) -> TimerActions {
    let mut actions = TimerActions {
        roar: false,
        stun_finished: false,
    };
    if timers.roar > 0 {
        timers.roar -= 1;
        if timers.roar == ROAR_TRIGGER_TICK {
            actions.roar = true;
        }
    }
    if timers.attack > 0 {
        timers.attack -= 1;
    }
    if timers.stunned > 0 {
        timers.stunned -= 1;
        if timers.stunned == 0 {
            actions.stun_finished = true;
            timers.roar = ROAR_AFTER_STUN_TICKS;
        }
    }
    actions
}

pub struct RavagerEntity {
    pub mob_entity: MobEntity,
    pub raider_data: RaiderData,
    timers: Mutex<RavagerTimers>,
}

impl RavagerEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let ravager = Self {
            mob_entity,
            raider_data: RaiderData::default(),
            timers: Mutex::new(RavagerTimers::default()),
        };
        let mob_arc = Arc::new(ravager);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(HoldGroundAttackGoal::new(10.0)));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(4, Box::new(LongDistancePatrolGoal::new(0.7, 0.595)));
            goal_selector.add_goal(4, Box::new(RaiderMoveThroughVillageGoal::new(1.05)));
            goal_selector.add_goal(4, Box::new(PathfindToRaidGoal::default()));
            goal_selector.add_goal(5, Box::new(RaiderCelebrationGoal));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(2, raider::revenge_goal());
            hunting::apply(target_selector, &mob_arc.mob_entity, RAVAGER_HUNTS);
        };

        mob_arc
    }

    /// Vanilla `Ravager.roar`: damage nearby living entities and knock non-players back.
    fn roar(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }
        let world = entity.world.load();
        let mob_griefing = world.level_info.load().game_rules.mob_griefing;
        let search_box = entity
            .bounding_box
            .load()
            .expand(ROAR_RADIUS, ROAR_RADIUS, ROAR_RADIUS);
        let attacker = self as &dyn EntityBase;
        for target in world.get_all_at_box(&search_box) {
            if target.get_living_entity().is_none() {
                continue;
            }
            let target_entity = target.get_entity();
            if !target_entity.is_alive() || target_entity.entity_type == &EntityType::RAVAGER {
                continue;
            }
            if !mob_griefing && target_entity.entity_type == &EntityType::ARMOR_STAND {
                continue;
            }
            // Vanilla spares AbstractIllager from the roar damage (but still knocks them back).
            if !target_entity
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_ILLAGER)
            {
                target.damage_with_context(
                    target.as_ref(),
                    ROAR_DAMAGE,
                    DamageType::MOB_ATTACK,
                    Some(entity.pos.load()),
                    Some(attacker),
                    Some(attacker),
                );
            }
            if target.get_player().is_none() {
                self.strong_knockback(target.as_ref());
            }
        }
        world.broadcast_entity_event(entity, EntityStatus::RavagerRoared, ActorEventID::None);
    }

    /// Vanilla `Ravager.strongKnockback`.
    fn strong_knockback(&self, target: &dyn EntityBase) {
        let self_pos = self.mob_entity.living_entity.entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let xd = target_pos.x - self_pos.x;
        let zd = target_pos.z - self_pos.z;
        let dd = (xd * xd + zd * zd).max(0.001);
        target.get_entity().add_velocity(Vector3::new(
            xd / dd * STRONG_KNOCKBACK,
            0.2,
            zd / dd * STRONG_KNOCKBACK,
        ));
    }

    /// Vanilla `Ravager.blockedByItem`: a shield block stuns the ravager half the time.
    fn on_attack_blocked(&self, defender: &dyn EntityBase) {
        {
            let timers = self.timers.lock().unwrap_or_else(PoisonError::into_inner);
            if timers.roar != 0 {
                return;
            }
        }
        if rand::random::<f64>() < STUN_CHANCE {
            self.timers
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .stunned = STUN_DURATION;
            self.play_entity_sound(Sound::EntityRavagerStunned, 1.0, 1.0);
            let entity = &self.mob_entity.living_entity.entity;
            entity.world.load().broadcast_entity_event(
                entity,
                EntityStatus::RavagerStunned,
                ActorEventID::None,
            );
            defender.push(self as &dyn EntityBase);
        } else {
            self.strong_knockback(defender);
        }
    }
}

impl Mob for RavagerEntity {
    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let living = &self.mob_entity.living_entity;
        if living.is_dead_or_dying() {
            return;
        }
        let max_speed = if self.mob_entity.get_target().is_some() {
            HUNTING_SPEED
        } else {
            WANDERING_SPEED
        };
        let (immobile, actions) = {
            let mut timers = self.timers.lock().unwrap_or_else(PoisonError::into_inner);
            let immobile = timers.attack > 0 || timers.stunned > 0 || timers.roar > 0;
            (immobile, advance_timers(&mut timers))
        };
        let mut attributes = living
            .attributes
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
            speed.base_value = if immobile {
                0.0
            } else {
                speed.base_value + SPEED_LERP * (max_speed - speed.base_value)
            };
            speed.dirty.store(true, Ordering::Relaxed);
        }
        drop(attributes);

        if actions.stun_finished {
            self.play_entity_sound(Sound::EntityRavagerRoar, 1.0, 1.0);
        }
        if actions.roar {
            self.roar();
        }
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityRavagerStep, 0.15, 1.0);
    }

    fn play_attack_sound(&self) {
        // Vanilla `Ravager.doHurtTarget` plays this sound (net/minecraft/world/entity/monster/Ravager.java).
        self.play_entity_sound(Sound::EntityRavagerAttack, 1.0, 1.0);
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        self.timers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .attack = ATTACK_DURATION;
        let entity = self.get_entity();
        entity.world.load().broadcast_entity_event(
            entity,
            EntityStatus::StartAttacking,
            ActorEventID::None,
        );
        let was_blocking = target
            .get_living_entity()
            .is_some_and(crate::entity::living::LivingEntity::is_blocking);
        let damaged = self.get_mob_entity().try_attack(entity, target);
        if was_blocking {
            self.on_attack_blocked(target);
        }
        // Vanilla plays the attack sound unconditionally in `doHurtTarget`.
        self.play_attack_sound();
        damaged
    }

    fn get_max_head_rotation(&self) -> f32 {
        // Vanilla `Ravager.getMaxHeadYRot`
        45.0
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_attack_bounding_box(
        &self,
        horizontal_expansion: f64,
    ) -> pumpkin_util::math::boundingbox::BoundingBox {
        // Vanilla `Ravager.getAttackBoundingBox`
        super::attack_bounding_box(&self.mob_entity.living_entity.entity, horizontal_expansion)
            .expand(-0.05, 0.0, -0.05)
    }

    fn is_mob_immobile(&self) -> bool {
        let timers = self.timers.lock().unwrap_or_else(PoisonError::into_inner);
        self.mob_entity.living_entity.is_dead_or_dying()
            || timers.attack > 0
            || timers.stunned > 0
            || timers.roar > 0
    }

    fn has_line_of_sight(&self, target: &crate::entity::Entity) -> bool {
        {
            let timers = self.timers.lock().unwrap_or_else(PoisonError::into_inner);
            if timers.stunned > 0 || timers.roar > 0 {
                return false;
            }
        }
        let mob_entity = &self.mob_entity;
        mob_entity
            .sensing
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .has_line_of_sight(&mob_entity.living_entity.entity, target)
    }

    fn as_patrolling_monster(&self) -> Option<&dyn PatrollingMonster> {
        Some(self)
    }

    fn as_raider(&self) -> Option<&dyn Raider> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_raider_nbt(nbt);
        let timers = self.timers.lock().unwrap_or_else(PoisonError::into_inner);
        nbt.put_int("AttackTick", timers.attack);
        nbt.put_int("StunTick", timers.stunned);
        nbt.put_int("RoarTick", timers.roar);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_raider_nbt(nbt);
        let mut timers = self.timers.lock().unwrap_or_else(PoisonError::into_inner);
        timers.attack = nbt.get_int("AttackTick").unwrap_or(0);
        timers.stunned = nbt.get_int("StunTick").unwrap_or(0);
        timers.roar = nbt.get_int("RoarTick").unwrap_or(0);
    }
}

impl PatrollingMonster for RavagerEntity {
    fn get_patrol_data(&self) -> &PatrolData {
        &self.raider_data.patrol_data
    }

    fn can_be_leader(&self) -> bool {
        false
    }
}

impl Raider for RavagerEntity {
    fn get_raider_data(&self) -> &RaiderData {
        &self.raider_data
    }

    fn get_celebrate_sound(&self) -> Sound {
        Sound::EntityRavagerCelebrate
    }
}

#[cfg(test)]
mod tests {
    use super::{RavagerTimers, advance_timers};

    #[test]
    fn timers_schedule_roar_ten_ticks_before_zero() {
        let mut timers = RavagerTimers {
            attack: 2,
            stunned: 0,
            roar: 20,
        };
        let mut saw_roar = false;
        for _ in 0..20 {
            saw_roar |= advance_timers(&mut timers).roar;
        }
        assert!(saw_roar);
        assert_eq!(timers.roar, 0);
        assert_eq!(timers.attack, 0);
    }

    #[test]
    fn stun_finish_schedules_roar() {
        let mut timers = RavagerTimers {
            attack: 0,
            stunned: 1,
            roar: 0,
        };
        let actions = advance_timers(&mut timers);
        assert!(actions.stun_finished);
        assert!(!actions.roar);
        assert_eq!(timers.stunned, 0);
        assert_eq!(timers.roar, 20);
    }
}
