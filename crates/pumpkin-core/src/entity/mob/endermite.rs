use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const ENDERMITE_HUNTS: &[Hunt] = &[Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true)];
const MAX_LIFETIME: i32 = 2400;

/// Vanilla `Entity#getRandomX(0.5)` / `getRandomZ(0.5)` is
/// `getX((2 * r - 1) * 0.5) = position + width * (r - 0.5)`, a full `+-width * 0.5` spread.
fn particle_xz_offset(r: f64, width: f32) -> f64 {
    (r - 0.5) * f64::from(width)
}

pub struct EndermiteEntity {
    pub mob_entity: MobEntity,
    pub lifetime: AtomicI32,
}

impl EndermiteEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let endermite = Self {
            mob_entity,
            lifetime: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(endermite);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(ClimbOnTopOfPowderSnowGoal));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(3, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true).alerting_others()));
            hunting::apply(target_selector, &mob_arc.mob_entity, ENDERMITE_HUNTS);
        };

        mob_arc
    }
}

impl Mob for EndermiteEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityEndermiteStep, 0.15, 1.0);
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(Sound::EntityEndermiteAmbient)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Lifetime", self.lifetime.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(life) = nbt.get_int("Lifetime") {
            self.lifetime.store(life, Ordering::Relaxed);
        }
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        entity.body_yaw.store(entity.yaw.load());

        // Vanilla `Endermite#customServerAiStep`: age the mite and apply 1.0 generic
        // damage per tick once the lifetime expires (a normal death, not a silent removal).
        if !self.mob_entity.persistence_required.load(Ordering::Relaxed) {
            self.lifetime.fetch_add(1, Ordering::Relaxed);
        }
        if self.lifetime.load(Ordering::Relaxed) >= MAX_LIFETIME {
            self.mob_entity
                .living_entity
                .damage(caller, 1.0, DamageType::GENERIC);
        }
    }

    fn post_tick(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let dimensions = entity.entity_dimension.load();
        let position = entity.pos.load();
        let mut rng = self.get_random();
        for _ in 0..2 {
            // Vanilla `getRandomX(0.5)` / `getRandomZ(0.5)`: full +-width * 0.5 spread.
            let width = dimensions.width;
            let particle_position = pumpkin_util::math::vector3::Vector3::new(
                position.x + particle_xz_offset(rng.random::<f64>(), width),
                position.y + rng.random::<f64>() * f64::from(dimensions.height),
                position.z + particle_xz_offset(rng.random::<f64>(), width),
            );
            let velocity = pumpkin_util::math::vector3::Vector3::new(
                (rng.random::<f64>() - 0.5) * 2.0,
                -rng.random::<f64>(),
                (rng.random::<f64>() - 0.5) * 2.0,
            );
            entity.world.load().spawn_particle(
                particle_position,
                velocity.to_f32_lossy(),
                1.0,
                0,
                pumpkin_data::particle::Particle::Portal,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::particle_xz_offset;

    #[test]
    fn particle_xz_offset_matches_vanilla_get_random_x() {
        // getRandomX(0.5) = getX((2*r - 1) * 0.5) = position + width*(r - 0.5):
        // a full +-width*0.5 spread (width 0.4 => +-0.2), not +-width*0.25.
        let width = 0.4_f32;
        for r in [0.0_f64, 0.25, 0.5, 0.75, 1.0] {
            let vanilla = f64::from(width) * (2.0 * r - 1.0) * 0.5;
            assert!((particle_xz_offset(r, width) - vanilla).abs() < 1e-12);
        }
        assert!((particle_xz_offset(0.0, width).abs() - 0.2).abs() < 1e-6);
    }
}
