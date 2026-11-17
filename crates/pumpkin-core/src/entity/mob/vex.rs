use std::sync::{Arc, Mutex, PoisonError, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::control::flying_move_control::FlyingMoveControl;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::mob::raider;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{Controls, Goal, look_at_entity::LookAtEntityGoal, swim::SwimGoal, to_goal_ticks},
    mob::{Mob, MobEntity},
};

/// Vanilla `VexMoveControl` turn rate.
const VEX_MAX_TURN: i32 = 20;
/// Vanilla `reducedTickDelay(7)` for both the charge and the random move roll.
const VEX_ROLL_TICKS: i32 = 7;
/// Vanilla `VexRandomMoveGoal` tries three blocks per pick.
const VEX_RANDOM_MOVE_ATTEMPTS: usize = 3;
const VEX_RANDOM_MOVE_SPEED: f64 = 0.25;
const VEX_CHARGE_SPEED: f64 = 1.0;
/// Squared distance under which a charging vex keeps steering at the target.
const VEX_CHARGE_STEER_DISTANCE_SQ: f64 = 9.0;
/// Squared distance over which a vex starts a charge.
const VEX_CHARGE_START_DISTANCE_SQ: f64 = 4.0;
/// Vanilla `Vex.tick` starves the vex once per this many ticks at the end of its life.
const VEX_STARVE_INTERVAL_TICKS: i32 = 20;

const VEX_HUNTS: &[Hunt] = &[Hunt::new(3, TargetMatch::Type(&EntityType::PLAYER), true)];

pub struct VexEntity {
    pub mob_entity: MobEntity,
    /// Vanilla `Vex.limitedLifeTicks`, `None` when the vex has no limited life.
    limited_life_ticks: Mutex<Option<i32>>,
}

impl VexEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .lock()
            .unwrap_or_else(PoisonError::into_inner) =
            Box::new(FlyingMoveControl::new(VEX_MAX_TURN, true));
        let vex = Self {
            mob_entity,
            limited_life_ticks: Mutex::new(None),
        };
        let mob_arc = Arc::new(vex);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        // Vanilla `Vex.tick` calls `setNoGravity(true)` on every tick and the vex never walks.
        mob_arc
            .mob_entity
            .living_entity
            .entity
            .set_has_no_gravity(true);

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(VexChargeAttackGoal::default()));
            goal_selector.add_goal(8, Box::new(VexRandomMoveGoal));
            // Vanilla `LookAtPlayerGoal(this, Player.class, 3.0F, 1.0F)`.
            goal_selector.add_goal(
                9,
                Box::new(LookAtEntityGoal::new(
                    mob_weak.clone(),
                    &EntityType::PLAYER,
                    3.0,
                    1.0,
                    false,
                )),
            );
            // Vanilla `LookAtPlayerGoal(this, Mob.class, 8.0F)`.
            goal_selector.add_goal(
                10,
                Box::new(LookAtEntityGoal::matching(
                    mob_weak,
                    TargetMatch::Mob,
                    8.0,
                    0.02,
                    false,
                )),
            );

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(1, raider::revenge_goal());
            // TODO: VexCopyOwnerTargetGoal at 2 needs owner tracking from the evoker summon.
            hunting::apply(target_selector, &mob_arc.mob_entity, VEX_HUNTS);
        };

        mob_arc
    }

    /// Vanilla `Vex.setLimitedLife`: the vex starves once `ticks` run out.
    pub fn set_limited_life(&self, ticks: i32) {
        *self
            .limited_life_ticks
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(ticks);
    }
}

impl Mob for VexEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        let starve = {
            let mut life = self
                .limited_life_ticks
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            life.as_mut().is_some_and(|remaining| {
                *remaining -= 1;
                if *remaining <= 0 {
                    *remaining = VEX_STARVE_INTERVAL_TICKS;
                    true
                } else {
                    false
                }
            })
        };
        if starve {
            self.mob_entity
                .living_entity
                .damage(caller, 1.0, DamageType::STARVE);
        }
    }
}

fn move_to(mob: &dyn Mob, pos: Vector3<f64>, speed: f64) {
    mob.get_mob_entity()
        .steered()
        .move_control()
        .set_wanted_position(pos.x, pos.y, pos.z, speed);
}

/// Vanilla `Vex.VexChargeAttackGoal`: swoop at the target's eyes, hit it on contact.
#[derive(Default)]
struct VexChargeAttackGoal {
    charging: bool,
}

impl Goal for VexChargeAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let Some(target) = mob_entity.get_target() else {
            return false;
        };
        if mob_entity.steered().move_control().has_wanted()
            || mob
                .get_random()
                .random_range(0..to_goal_ticks(VEX_ROLL_TICKS).max(1))
                != 0
        {
            return false;
        }
        let mob_pos = mob_entity.living_entity.entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        mob_pos.squared_distance_to_vec(&target_pos) > VEX_CHARGE_START_DISTANCE_SQ
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        self.charging
            && mob_entity.steered().move_control().has_wanted()
            && mob_entity
                .get_target()
                .is_some_and(|target| target.get_entity().is_alive())
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.charging = true;
        if let Some(target) = mob.get_mob_entity().get_target() {
            let eye = target.get_entity().get_eye_pos();
            move_to(mob, eye, VEX_CHARGE_SPEED);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };
        let target_entity = target.get_entity();
        let bounding_box = mob
            .get_mob_entity()
            .living_entity
            .entity
            .bounding_box
            .load();
        if bounding_box.intersects(&target_entity.bounding_box.load()) {
            mob.do_hurt_target(target.as_ref());
            self.charging = false;
            return;
        }
        let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        if mob_pos.squared_distance_to_vec(&target_entity.pos.load()) < VEX_CHARGE_STEER_DISTANCE_SQ
        {
            move_to(mob, target_entity.get_eye_pos(), VEX_CHARGE_SPEED);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

/// Vanilla `Vex.VexRandomMoveGoal`: picks a free block near the bound origin. Vanilla falls back
/// to the vex's own block while no origin is set, which is the only case modelled here.
struct VexRandomMoveGoal;

impl Goal for VexRandomMoveGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        !mob.get_mob_entity().steered().move_control().has_wanted()
            && mob
                .get_random()
                .random_range(0..to_goal_ticks(VEX_ROLL_TICKS).max(1))
                == 0
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let world = entity.world.load_full();
        let origin = entity.block_pos.load();
        let mut rng = mob.get_random();
        for _ in 0..VEX_RANDOM_MOVE_ATTEMPTS {
            let offset = Vector3::new(
                rng.random_range(-7..8),
                rng.random_range(-5..6),
                rng.random_range(-7..8),
            );
            let pos = origin.offset(offset);
            if !world.get_block_state(&pos).is_air() {
                continue;
            }
            let target = Vector3::new(
                f64::from(pos.0.x) + 0.5,
                f64::from(pos.0.y) + 0.5,
                f64::from(pos.0.z) + 0.5,
            );
            move_to(mob, target, VEX_RANDOM_MOVE_SPEED);
            if mob_entity.get_target().is_none() {
                mob_entity
                    .look_control
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .look_at_with_range(target.x, target.y, target.z, 180.0, 20.0);
            }
            break;
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
