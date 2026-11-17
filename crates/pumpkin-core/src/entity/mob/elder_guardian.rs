use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_protocol::java::client::play::{CGameEvent, GameEvent};
use pumpkin_util::GameMode;

use crate::entity::ai::behavior::hunting;
use crate::entity::custom_sound::CustomSound;
use crate::entity::mob::guardian::{
    GUARDIAN_HUNTS, GUARDIAN_IDLE_SINK, GUARDIAN_WATER_DRAG, GUARDIAN_WATER_SPEED,
    GuardianMoveControl,
};
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        move_towards_restriction::MoveTowardsRestrictionGoal, random_swimming::RandomSwimmingGoal,
    },
    mob::{Mob, MobEntity},
};

const EFFECT_INTERVAL: i32 = 1200;
/// Vanilla `MobEffectUtil` display limit: an effect ending within this many ticks is refreshed.
const EFFECT_DISPLAY_LIMIT: i32 = 1200;
const HOME_RADIUS: i32 = 16;
/// Mining fatigue amplifier applied by the elder guardian's appearance effect.
const FATIGUE_AMPLIFIER: u8 = 2;

/// Vanilla `MobEffectUtil.addEffectToPlayersAround`: skip a player already holding an equal or
/// stronger effect with enough time left. The tuple is `(amplifier, remaining ticks)`.
fn should_apply_fatigue(effect: Option<(u8, i32)>) -> bool {
    match effect {
        Some((amplifier, remaining)) => {
            amplifier < FATIGUE_AMPLIFIER || remaining < EFFECT_DISPLAY_LIMIT
        }
        None => true,
    }
}

pub struct ElderGuardianEntity {
    pub mob_entity: MobEntity,
    moving: Arc<AtomicBool>,
}

impl ElderGuardianEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let moving = Arc::new(AtomicBool::new(false));
        let mut mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(GuardianMoveControl::new(moving.clone()));
        let guardian = Self { mob_entity, moving };
        let mob_arc = Arc::new(guardian);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(4, Box::new(RandomSwimmingGoal::new(1.0, 10)));
            // Vanilla `Guardian.registerGoals`: restriction at 5 keeps the elder near home.
            goal_selector.add_goal(5, Box::new(MoveTowardsRestrictionGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            hunting::apply(target_selector, &mob_arc.mob_entity, GUARDIAN_HUNTS);
        };

        mob_arc
    }
}

impl CustomSound for ElderGuardianEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        let is_water = entity.touching_water.load(Ordering::Relaxed);
        Some(if is_water {
            Sound::EntityElderGuardianHurt
        } else {
            Sound::EntityElderGuardianHurtLand
        })
    }

    fn death_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        let is_water = entity.touching_water.load(Ordering::Relaxed);
        Some(if is_water {
            Sound::EntityElderGuardianDeath
        } else {
            Sound::EntityElderGuardianDeathLand
        })
    }
}

impl Mob for ElderGuardianEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla `ElderGuardian.getAmbientSound`: inherits `Guardian`'s 160 tick interval.
    fn get_ambient_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        Some(if entity.touching_water.load(Ordering::Relaxed) {
            Sound::EntityElderGuardianAmbient
        } else {
            Sound::EntityElderGuardianAmbientLand
        })
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        160
    }

    // Vanilla `ElderGuardian` inherits `Guardian#travelInWater`.
    fn travel_in_water(&self, caller: &dyn EntityBase, _gravity: f64, _falling: bool) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        entity.update_velocity_from_input(living.movement_input.load(), GUARDIAN_WATER_SPEED);
        living.make_move(caller);
        let mut velocity = entity.velocity.load() * GUARDIAN_WATER_DRAG;
        if !self.moving.load(Ordering::Relaxed) && self.mob_entity.get_target().is_none() {
            velocity.y -= GUARDIAN_IDLE_SINK;
        }
        entity.velocity.store(velocity);
    }

    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        if !self.mob_entity.has_position_target() {
            self.mob_entity
                .set_home_to(entity.block_pos.load(), HOME_RADIUS);
        }

        if (entity.tick_count.load(Ordering::Relaxed) + entity.entity_id) % EFFECT_INTERVAL == 0 {
            let pos = entity.pos.load();
            let world = entity.world.load();
            let players = world.get_nearby_players(pos, 50.0);
            let amplitude = if entity.is_silent() { 0.0 } else { 1.0 };
            let packet = CGameEvent::new(GameEvent::PlayElderGuardianMobAppearance, amplitude);

            for player in players {
                // Vanilla only affects survival/adventure players, and refreshes an effect that
                // is weaker or about to expire.
                if matches!(
                    player.gamemode.load(),
                    GameMode::Creative | GameMode::Spectator
                ) {
                    continue;
                }
                let fatigue = player
                    .living_entity
                    .get_effect(&StatusEffect::MINING_FATIGUE)
                    .map(|effect| (effect.amplifier, effect.duration));
                if !should_apply_fatigue(fatigue) {
                    continue;
                }
                player.try_send_client_packet(&packet);
                player.living_entity.add_effect(Effect {
                    effect_type: &StatusEffect::MINING_FATIGUE,
                    duration: 6000,
                    amplifier: FATIGUE_AMPLIFIER,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }
        }
    }

    fn on_damage_with_context(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        crate::entity::mob::guardian::reflect_thorns(
            self.get_entity(),
            damage_type,
            source,
            self.moving.load(Ordering::Relaxed),
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fatigue_application_matches_vanilla_display_limit() {
        assert!(should_apply_fatigue(None));
        assert!(should_apply_fatigue(Some((0, 6000))));
        assert!(should_apply_fatigue(Some((2, 1199))));
        assert!(!should_apply_fatigue(Some((2, 1200))));
        assert!(!should_apply_fatigue(Some((2, 6000))));
        assert!(!should_apply_fatigue(Some((3, 6000))));
    }
}
