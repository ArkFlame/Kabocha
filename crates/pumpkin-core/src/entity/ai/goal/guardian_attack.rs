use std::sync::Arc;

use pumpkin_data::{
    damage::DamageType,
    entity::{EntityStatus, tracked_data},
};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_util::Difficulty;

use super::{Controls, Goal};
use crate::entity::{EntityBase, mob::Mob};

/// Vanilla Guardian.GuardianAttackGoal: stop moving, charge while staring at the target,
/// then fire the beam as indirect magic and clear the target so the client stops drawing it.
pub struct GuardianAttackGoal {
    attack_time: i32,
    attack_duration: i32,
    elder: bool,
}

impl GuardianAttackGoal {
    #[must_use]
    pub const fn new(elder: bool) -> Self {
        Self {
            attack_time: -10,
            attack_duration: if elder { 60 } else { 80 },
            elder,
        }
    }

    fn active_target(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let target = mob.get_mob_entity().get_target()?;
        if !target.get_entity().is_alive() {
            return None;
        }
        Some(target)
    }

    fn set_beam_target(mob: &dyn Mob, entity_id: i32) {
        mob.get_entity()
            .set_synced_data(tracked_data::guardian::DATA_ID_ATTACK_TARGET, entity_id);
    }

    /// Vanilla GuardianAttackGoal.tick damage: base 1.0, +2 on hard, +2 for elders.
    const fn magic_damage(elder: bool, hard: bool) -> f32 {
        let damage: f32 = 1.0;
        let damage = if hard { damage + 2.0 } else { damage };
        if elder { damage + 2.0 } else { damage }
    }
}

impl Goal for GuardianAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        Self::active_target(mob).is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(target) = Self::active_target(mob) else {
            return false;
        };
        // Vanilla keeps the elder beam going at melee range; a guardian stops within 3 blocks.
        if self.elder {
            return true;
        }
        let mob_pos = mob.get_entity().pos.load();
        target
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&mob_pos)
            > 9.0
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.attack_time = -10;
        let mut navigator = mob.get_mob_entity().steered().navigator();
        navigator.stop();
        if let Some(target) = Self::active_target(mob) {
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at_entity_with_range(&target, 90.0, 90.0);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        Self::set_beam_target(mob, 0);
        mob.get_mob_entity().set_target(None);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = Self::active_target(mob) else {
            Self::set_beam_target(mob, 0);
            return;
        };
        let target_entity = target.get_entity();

        let mut navigator = mob.get_mob_entity().steered().navigator();
        navigator.stop();
        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 90.0, 90.0);

        if !mob.has_line_of_sight(target_entity) {
            mob.get_mob_entity().set_target(None);
            Self::set_beam_target(mob, 0);
            return;
        }

        self.attack_time += 1;
        if self.attack_time == 0 {
            Self::set_beam_target(mob, target_entity.entity_id);
            let entity = mob.get_entity();
            if !entity.is_silent() {
                entity.world.load().send_entity_status(
                    entity,
                    EntityStatus::GuardianAttackSound,
                    ActorEventID::None,
                );
            }
        } else if self.attack_time >= self.attack_duration {
            let entity = mob.get_entity();
            let hard = entity.world.load().level_info.load().difficulty == Difficulty::Hard;
            let damage = Self::magic_damage(self.elder, hard);
            let attacker = mob as &dyn EntityBase;
            target.damage_with_context(
                target.as_ref(),
                damage,
                DamageType::INDIRECT_MAGIC,
                Some(entity.pos.load()),
                Some(attacker),
                Some(attacker),
            );
            mob.do_hurt_target(target.as_ref());
            mob.get_mob_entity().set_target(None);
            Self::set_beam_target(mob, 0);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_damage_matches_vanilla_difficulty_scaling() {
        assert_eq!(GuardianAttackGoal::magic_damage(false, false), 1.0);
        assert_eq!(GuardianAttackGoal::magic_damage(false, true), 3.0);
        assert_eq!(GuardianAttackGoal::magic_damage(true, false), 3.0);
        assert_eq!(GuardianAttackGoal::magic_damage(true, true), 5.0);
    }
}
