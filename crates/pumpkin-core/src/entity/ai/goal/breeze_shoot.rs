use std::sync::{Arc, Weak};

use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal};
use crate::entity::projectile::{
    ThrownItemEntity, apply_on_projectile_spawned,
    wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity},
};
use crate::entity::{Entity, EntityBase, mob::Mob, mob::breeze::BreezeEntity};
use crate::world::World;

/// Vanilla breeze shooting range, as a squared distance.
const ATTACK_RANGE_SQ: f64 = 256.0;
/// Vanilla breeze wind-charge launch tuning.
const UNCERTAINTY_BASE: i32 = 5;
const UNCERTAINTY_MULTIPLIER: i32 = 4;
const PROJECTILE_MOVEMENT_SCALE: f64 = 0.7;
/// Vanilla breeze inhale, burst and recovery timing.
const SHOOT_INITIAL_DELAY_TICKS: i32 = 15;
const SHOOT_RECOVER_DELAY_TICKS: i32 = 4;
const SHOOT_COOLDOWN_TICKS: i32 = 10;
const SHOOT_VOLUME: f32 = 1.5;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Charging,
    Recovering,
    Done,
}

/// Vanilla breeze ranged attack: inhale into the shooting pose, launch a breeze wind
/// charge, then recover and cool down.
///
/// Goals are the current Kabocha equivalent of the upstream brain behavior; aim, timing,
/// pose and sound values are kept from vanilla.
pub struct BreezeShootGoal {
    breeze: Weak<BreezeEntity>,
    phase: Phase,
    timer: i32,
    cooldown: i32,
}

impl BreezeShootGoal {
    #[must_use]
    pub const fn new(breeze: Weak<BreezeEntity>) -> Self {
        Self {
            breeze,
            phase: Phase::Idle,
            timer: 0,
            cooldown: 0,
        }
    }

    /// Turns the head and body toward the target, matching vanilla breeze aiming.
    fn look_from_eyes(entity: &Entity, target: Vector3<f64>) {
        entity.look_at(target.add_raw(0.0, -entity.get_eye_height(), 0.0));
        let yaw = entity.yaw.load();
        entity.head_yaw.store(yaw);
        entity.body_yaw.store(yaw);
    }

    /// Fires one breeze wind charge owned by the shooter, then runs projectile-spawned
    /// enchantment hooks against it.
    fn fire(breeze: &BreezeEntity, target: &dyn EntityBase, world: &Arc<World>) {
        let entity = &breeze.mob_entity.living_entity.entity;
        let position = entity.pos.load();
        let firing_position = position.add_raw(0.0, f64::from(entity.height() / 2.0) + 0.3, 0.0);

        let target_entity = target.get_entity();
        let aim_fraction = if target.is_passenger() { 0.8 } else { 0.3 };
        let aim_at = target_entity.pos.load().add_raw(
            0.0,
            f64::from(target_entity.height()) * aim_fraction,
            0.0,
        );
        let direction = aim_at - firing_position;

        let difficulty = i32::from(u8::from(world.level_info.load().difficulty));
        let projectile = Entity::new(
            world.clone(),
            firing_position,
            &EntityType::BREEZE_WIND_CHARGE,
        );
        let thrown = ThrownItemEntity::owned(projectile, entity, WIND_CHARGE_GRAVITY);
        thrown.set_velocity(
            direction.x,
            direction.y,
            direction.z,
            PROJECTILE_MOVEMENT_SCALE,
            f64::from(UNCERTAINTY_BASE - difficulty * UNCERTAINTY_MULTIPLIER),
        );

        let charge: Arc<dyn EntityBase> = Arc::new(WindChargeEntity::new_breeze(thrown));
        if world.spawn_entity(charge.clone()) {
            apply_on_projectile_spawned(charge.get_entity(), ItemStack::EMPTY, None, None);
        }
        breeze.play_entity_sound(Sound::EntityBreezeShoot, SHOOT_VOLUME, 1.0);
    }
}

impl Goal for BreezeShootGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        // Inactive goals are polled once per selector tick, so the cooldown runs down here.
        self.cooldown = (self.cooldown - 1).max(0);
        let Some(breeze) = self.breeze.upgrade() else {
            return false;
        };
        if self.cooldown > 0
            || !breeze.mob_entity.living_entity.is_alive()
            || breeze.mob_entity.living_entity.entity.pose.load() != EntityPose::Standing
        {
            return false;
        }
        let Some(target) = breeze.mob_entity.get_target() else {
            return false;
        };
        target.get_entity().is_alive()
            && mob.can_attack(target.as_ref())
            && breeze
                .mob_entity
                .living_entity
                .entity
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load())
                < ATTACK_RANGE_SQ
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.phase != Phase::Done
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.phase = Phase::Charging;
        self.timer = 0;
        if let Some(breeze) = self.breeze.upgrade() {
            breeze
                .mob_entity
                .living_entity
                .entity
                .set_pose(EntityPose::Shooting);
            breeze.play_entity_sound(Sound::EntityBreezeInhale, 1.0, 1.0);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(breeze) = self.breeze.upgrade()
            && breeze.mob_entity.living_entity.entity.pose.load() == EntityPose::Shooting
        {
            breeze
                .mob_entity
                .living_entity
                .entity
                .set_pose(EntityPose::Standing);
        }
        self.phase = Phase::Idle;
        self.cooldown = SHOOT_COOLDOWN_TICKS;
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(breeze) = self.breeze.upgrade() else {
            self.phase = Phase::Done;
            return;
        };
        let entity = &breeze.mob_entity.living_entity.entity;
        let Some(target) = breeze.mob_entity.get_target() else {
            self.phase = Phase::Done;
            return;
        };
        if !breeze.mob_entity.living_entity.is_alive() || !target.get_entity().is_alive() {
            self.phase = Phase::Done;
            return;
        }

        let target_pos = target.get_entity().pos.load();
        Self::look_from_eyes(entity, target_pos);

        match self.phase {
            Phase::Charging => {
                self.timer += 1;
                if self.timer >= SHOOT_INITIAL_DELAY_TICKS {
                    let world = entity.world.load_full();
                    Self::fire(&breeze, target.as_ref(), &world);
                    self.phase = Phase::Recovering;
                    self.timer = 0;
                }
            }
            Phase::Recovering => {
                self.timer += 1;
                if self.timer >= SHOOT_RECOVER_DELAY_TICKS {
                    entity.set_pose(EntityPose::Standing);
                    self.phase = Phase::Done;
                }
            }
            Phase::Idle | Phase::Done => {}
        }
    }

    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::test_support;
    use crate::entity::passive::iron_golem::IronGolemEntity;

    #[tokio::test]
    async fn breeze_charges_then_fires_an_owned_wind_charge() -> Result<(), Box<dyn Error>> {
        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::BREEZE,
        ));
        let target: Arc<dyn EntityBase> = IronGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(5.0, 64.0, 0.0),
            &EntityType::IRON_GOLEM,
        ));
        breeze.mob_entity.set_target(Some(target.clone()));

        let mut goal = BreezeShootGoal::new(Arc::downgrade(&breeze));
        goal.start(breeze.as_ref());
        assert_eq!(breeze.get_entity().pose.load(), EntityPose::Shooting);
        assert!(
            world
                .entities
                .load()
                .iter()
                .all(|entity| entity.get_entity().entity_type != &EntityType::BREEZE_WIND_CHARGE)
        );

        for _ in 0..SHOOT_INITIAL_DELAY_TICKS {
            goal.tick(breeze.as_ref());
        }

        let charges: Vec<_> = world
            .entities
            .load()
            .iter()
            .filter(|entity| entity.get_entity().entity_type == &EntityType::BREEZE_WIND_CHARGE)
            .cloned()
            .collect();
        assert_eq!(charges.len(), 1);
        assert_eq!(
            charges[0]
                .get_projectile_owner()
                .and_then(|owner| owner.uuid()),
            Some(breeze.get_entity().entity_uuid)
        );

        world.level.shutdown().await;
        Ok(())
    }
}
