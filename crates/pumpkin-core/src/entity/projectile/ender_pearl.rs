use crate::block::blocks::bubble_column;
use crate::entity::mob::spawn::{EntitySpawnReason, finalize_spawn};
use crate::entity::projectile::owner::ProjectileOwner;
use crate::plugin::api::events::player::player_teleport::TeleportCause;
use pumpkin_util::math::position::BlockPos;

use crate::entity::projectile::ProjectileHit;
use crate::{
    entity::{
        Entity, EntityBase, EntityType, mob::endermite::EndermiteEntity,
        projectile::ThrownItemEntity,
    },
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

const GRAVITY: f64 = 0.03;
const PARTICLE_COUNT: i32 = 32;
const ENDERMITE_SPAWN_CHANCE: f32 = 0.05;

/// Standard normal sample matching vanilla RandomSource nextGaussian via Box-Muller.
/// rand 0.10 core has no gaussian distribution, so avoid a rand_distr dependency.
fn next_gaussian() -> f64 {
    // rand::random::<f64>() is in [0, 1), so 1.0 - it is in (0, 1] and ln stays finite.
    let u1 = 1.0 - rand::random::<f64>();
    let u2 = rand::random::<f64>();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

pub struct EnderPearlEntity {
    pub thrown: ThrownItemEntity,
}

impl EnderPearlEntity {
    pub fn new(entity: Entity) -> Self {
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));

        let thrown = ThrownItemEntity {
            entity,
            owner: ProjectileOwner::none(),
            gravity: GRAVITY,
        };

        Self { thrown }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown.entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));
        Self { thrown }
    }
}

impl EntityBase for EnderPearlEntity {
    /// Pearls take the regular entity push, not the projectile one.
    fn on_above_bubble_column(&self, drag_down: bool, pos: &BlockPos) {
        let entity = self.get_entity();
        bubble_column::push_entity(entity, drag_down, true);
        bubble_column::send_bubble_column_particles(&entity.world.load(), pos);
    }

    fn on_inside_bubble_column(&self, drag_down: bool) {
        bubble_column::push_entity(self.get_entity(), drag_down, false);
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown.owner)
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
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let entity = self.get_entity();
        let world = entity.world.load();

        let attacker = self.get_owner_entity();

        // Vanilla spawns the PORTAL burst at the pearl's position with no X/Z
        // position spread, a uniform 0..2 Y offset, and gaussian X/Z velocity.
        let pearl_pos = entity.pos.load();
        for _ in 0..PARTICLE_COUNT {
            let particle_pos = Vector3::new(
                pearl_pos.x,
                pearl_pos.y + rand::random::<f64>() * 2.0,
                pearl_pos.z,
            );
            let speed = Vector3::new(next_gaussian(), 0.0, next_gaussian());

            world.spawn_particle(
                particle_pos,
                Vector3::new(speed.x as f32, speed.y as f32, speed.z as f32),
                1.0,
                1,
                Particle::Portal,
            );
        }

        // Vanilla `EnderPearlEntity#onCollision` teleports the owner to the pearl's
        // collision position, not to where the pearl was one tick earlier.
        let teleport_pos = entity.pos.load();

        // Like vanilla, a zero hit from the pearl, credited to its thrower if any.
        if let ProjectileHit::Entity {
            entity: hit_entity,
            hit_pos,
            ..
        } = &hit
        {
            hit_entity.damage_with_context(
                &**hit_entity,
                0.0,
                DamageType::THROWN,
                Some(*hit_pos),
                Some(self),
                attacker.as_deref(),
            );
        }

        // Vanilla only teleports an owner that is still in the pearl's world.
        if let Some(owner) = attacker
            && std::sync::Arc::ptr_eq(&owner.get_entity().world.load(), &world)
            && owner.get_entity().is_alive()
            // Vanilla rejects a player pearl owner whose connection is no longer open.
            && owner.get_player().is_none_or(|player| !player.client.closed())
            && owner.get_living_entity().is_none_or(|living| {
                living.health.load() > 0.0 && owner.get_entity().pose.load() != EntityPose::Sleeping
            })
        {
            let is_player = owner.get_player().is_some();
            if is_player {
                // Vanilla `stopRiding`s the shooter before teleporting it.
                if let Some(vehicle) = owner.get_entity().get_vehicle() {
                    vehicle
                        .get_entity()
                        .remove_passenger_before_teleport(owner.get_entity().entity_id);
                }
                let should_spawn_endermite = rand::random::<f32>() < ENDERMITE_SPAWN_CHANCE;
                // Vanilla gates the endermite on `DO_MOB_SPAWNING` alone, not on difficulty.
                let do_mob_spawning = world.level_info.load().game_rules.spawn_mobs;
                if do_mob_spawning && should_spawn_endermite {
                    let pearl_pos = entity.pos.load();
                    let pearl_yaw = entity.yaw.load();
                    let entity = Entity::new(world.clone(), pearl_pos, &EntityType::ENDERMITE);
                    entity.set_rotation(pearl_yaw, 0.0);
                    let endermite = EndermiteEntity::new(entity);
                    finalize_spawn(
                        &(endermite.clone() as std::sync::Arc<dyn EntityBase>),
                        &world,
                        EntitySpawnReason::Triggered,
                        None,
                    );
                    world.spawn_entity(endermite);
                }
            }

            // In vanilla, teleport handles everything including sound
            owner.teleport_with_cause(
                teleport_pos,
                None,
                None,
                world.clone(),
                TeleportCause::EnderPearl,
            );

            // Play teleport sound at new position
            world.play_sound(
                Sound::EntityPlayerTeleport,
                SoundCategory::Players,
                &teleport_pos,
            );

            if let Some(living) = owner.get_living_entity() {
                living.fall_distance.store(0.0);
                living.reset_current_impulse_context();
            }
            if is_player {
                owner.damage(
                    owner.as_ref(),
                    5.0,
                    pumpkin_data::damage::DamageType::ENDER_PEARL,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaussian_is_standard_normal() {
        const N: usize = 10_000;
        let mut sum = 0.0f64;
        let mut sum_sq = 0.0f64;
        for _ in 0..N {
            let value = next_gaussian();
            assert!(value.is_finite());
            sum += value;
            sum_sq += value * value;
        }
        let mean = sum / N as f64;
        let variance = sum_sq / N as f64 - mean * mean;
        assert!(mean.abs() < 0.1, "mean {mean}");
        assert!((variance - 1.0).abs() < 0.15, "variance {variance}");
    }
}
