use super::{Controls, Goal};

use crate::entity::EntityBase;
use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

const MAX_ATTACK_TIME: i64 = 20;

/// Vanilla `MeleeAttackGoal.tick` recalculation gate: the timer has to have elapsed and the
/// target either moved a full block since the last requested path or the 5% roll fired. A
/// missing last position is vanilla's zeroed `pathedTarget*` sentinel and always recalculates.
fn should_recalculate_path(
    countdown_ticks: i32,
    last_target_position: Option<Vector3<f64>>,
    target_pos: &Vector3<f64>,
    roll: impl FnOnce() -> f32,
) -> bool {
    // Vanilla short-circuits, so the random roll is only drawn when the target has not moved.
    countdown_ticks <= 0
        && (last_target_position
            .is_none_or(|last_pos| target_pos.squared_distance_to_vec(&last_pos) >= 1.0)
            || roll() < 0.05)
}

type AttackCheck = fn(&mut MeleeAttackGoal, &dyn Mob, &dyn EntityBase);

pub struct MeleeAttackGoal {
    goal_control: Controls,
    speed: f64,
    follow_even_if_unseen: bool,
    path: Option<Path>,
    #[expect(dead_code)]
    target_location: Vector3<f64>,
    update_countdown_ticks: i32,
    pub cooldown: i32,
    attack_interval_ticks: i32,
    last_update_time: i64,
    last_target_position: Option<Vector3<f64>>,
    gate: Option<MobFilter>,
    attack: Option<fn(&dyn Mob, &dyn EntityBase)>,
    // A goal with `check` or `attack` can do anything in its hook, so its `tick_reach` must stay `None`.
    check: Option<AttackCheck>,
    on_stop: Option<fn(&dyn Mob)>,
}

impl MeleeAttackGoal {
    #[must_use]
    pub fn new(speed: f64, follow_even_if_unseen: bool) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            follow_even_if_unseen,
            path: None,
            target_location: Vector3::new(0.0, 0.0, 0.0),
            update_countdown_ticks: 0,
            cooldown: 0,
            attack_interval_ticks: 20,
            last_update_time: 0,
            last_target_position: None,
            gate: None,
            attack: None,
            check: None,
            on_stop: None,
        }
    }

    /// Vanilla MeleeAttackGoal attack interval (attackInterval).
    #[must_use]
    pub const fn with_attack_interval(mut self, ticks: i32) -> Self {
        self.attack_interval_ticks = ticks;
        self
    }

    /// Extra condition for starting and for continuing
    #[must_use]
    pub const fn gated_by(mut self, gate: MobFilter) -> Self {
        self.gate = Some(gate);
        self
    }

    /// Replaces the swing and hit, vanilla `checkAndPerformAttack` overrides.
    #[must_use]
    pub const fn performing(mut self, attack: fn(&dyn Mob, &dyn EntityBase)) -> Self {
        self.attack = Some(attack);
        self
    }

    /// Replaces the whole `checkAndPerformAttack`, which runs every tick once the target is set.
    /// When set, `performing` is ignored.
    #[must_use]
    pub const fn checking(mut self, check: AttackCheck) -> Self {
        self.check = Some(check);
        self
    }

    /// Runs at the start of `stop`, for subclasses that override it.
    #[must_use]
    pub const fn on_stop(mut self, step: fn(&dyn Mob)) -> Self {
        self.on_stop = Some(step);
        self
    }

    #[must_use]
    pub fn get_max_cooldown(&self) -> i32 {
        self.get_tick_count(self.attack_interval_ticks)
    }

    pub fn reset_attack_cooldown(&mut self) {
        self.cooldown = self.get_max_cooldown();
    }

    #[must_use]
    pub const fn is_time_to_attack(&self) -> bool {
        self.cooldown <= 0
    }

    /// Vanilla `MeleeAttackGoal.canPerformAttack`: cooldown, melee reach and an unobstructed view.
    #[must_use]
    pub fn can_perform_attack(&self, mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        self.is_time_to_attack()
            && mob.is_in_attack_range(target)
            && mob.has_line_of_sight(target.get_entity())
    }

    fn check_and_perform_attack(&mut self, mob: &dyn Mob, target: &dyn EntityBase) {
        if let Some(check) = self.check {
            check(self, mob, target);
            return;
        }
        if !self.can_perform_attack(mob, target) {
            return;
        }
        self.reset_attack_cooldown();
        if let Some(attack) = self.attack {
            attack(mob, target);
            return;
        }

        mob.get_mob_entity().living_entity.swing_hand();
        let damaged = mob.do_hurt_target(target);
        if damaged {
            mob.on_attack(target);
        }
    }
}

impl Goal for MeleeAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.gate.is_some_and(|gate| !gate(mob)) {
            return false;
        }
        let time = mob.get_entity().world.load().get_world_age();

        if time - self.last_update_time < MAX_ATTACK_TIME {
            return false;
        }
        self.last_update_time = time;

        let target = mob.get_mob_entity().get_target();

        let Some(target) = target.as_ref() else {
            return false;
        };
        if !target.get_entity().is_alive() {
            return false;
        }

        // Vanilla `createPath(target, 0)`: without a route the goal starts only
        // when the target is already inside melee reach.
        let target_pos = target.get_entity().pos.load();
        let path = {
            let steered = mob.get_mob_entity().steered();
            steered
                .navigator()
                .create_path(&steered.mob_entity().living_entity, target_pos, 0)
        };
        self.path = path;
        self.path.is_some() || mob.is_in_attack_range(target.as_ref())
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if self.gate.is_some_and(|gate| !gate(mob)) {
            return false;
        }
        let target = mob.get_mob_entity().get_target().clone();

        let Some(target) = target else {
            return false;
        };
        if !target.get_entity().is_alive() {
            return false;
        }

        if !self.follow_even_if_unseen {
            let is_idle = mob.is_navigator_idle();
            return !is_idle;
        }

        mob.get_mob_entity()
            .is_in_position_target_range_pos(&target.get_entity().block_pos.load())
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().set_attacking(true);

        let steered = mob.get_mob_entity().steered();
        steered.navigator().move_to_path(
            self.path.take(),
            self.speed,
            &steered.mob_entity().living_entity,
        );
        self.update_countdown_ticks = 0;
        self.cooldown = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(on_stop) = self.on_stop {
            on_stop(mob);
        }
        mob.get_mob_entity().set_attacking(false);

        // Vanilla: this.mob.getNavigation().stop()
        mob.get_mob_entity().steered().navigator().stop();
        self.last_target_position = None;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let target = mob.get_mob_entity().get_target().clone();
        let Some(target) = target else {
            return;
        };

        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 30.0, 30.0);

        self.update_countdown_ticks = (self.update_countdown_ticks - 1).max(0);

        let current_target_pos = target.get_entity().pos.load();
        let has_line_of_sight =
            self.follow_even_if_unseen || mob.has_line_of_sight(target.get_entity());
        let should_update_nav = has_line_of_sight
            && should_recalculate_path(
                self.update_countdown_ticks,
                self.last_target_position,
                &current_target_pos,
                || mob.get_random().random::<f32>(),
            );

        if should_update_nav {
            let mob_pos = mob.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&current_target_pos);
            let steered = mob.get_mob_entity().steered();
            let mut navigator = steered.navigator();
            navigator.set_progress(NavigatorGoal {
                current_progress: mob_pos,
                destination: current_target_pos,
                speed: self.speed,
            });
            self.last_target_position = Some(current_target_pos);
            self.update_countdown_ticks = 4 + mob.get_random().random_range(0..7);
            if dist_sq > 1024.0 {
                self.update_countdown_ticks += 10;
            } else if dist_sq > 256.0 {
                self.update_countdown_ticks += 5;
            }
            self.update_countdown_ticks = self.get_tick_count(self.update_countdown_ticks);
        }

        // Vanilla adds 15 ticks when `moveTo` fails to find a route. Pumpkin defers the
        // search to the navigator, so consume its result on the following tick.
        {
            let steered = mob.get_mob_entity().steered();
            let mut navigator = steered.navigator();
            if navigator.take_last_path_search_failed() {
                self.update_countdown_ticks += 15;
            }
        }

        self.cooldown = (self.cooldown - 1).max(0);
        self.check_and_perform_attack(mob, target.as_ref());
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recalculation_waits_for_the_timer_and_the_moved_block() {
        let origin = Vector3::new(0.0, 64.0, 0.0);
        let nearby = Vector3::new(0.5, 64.0, 0.0);
        let moved = Vector3::new(1.5, 64.0, 0.0);

        assert!(should_recalculate_path(0, None, &nearby, || 0.99));
        assert!(should_recalculate_path(0, Some(origin), &moved, || 0.99));
        assert!(should_recalculate_path(0, Some(origin), &nearby, || 0.01));
        assert!(!should_recalculate_path(0, Some(origin), &nearby, || 0.99));
        assert!(!should_recalculate_path(7, None, &moved, || 0.01));
    }

    #[test]
    fn a_moved_target_skips_the_random_roll() {
        let origin = Vector3::new(0.0, 64.0, 0.0);
        let moved = Vector3::new(2.0, 64.0, 0.0);

        let mut drew_roll = false;
        assert!(should_recalculate_path(0, Some(origin), &moved, || {
            drew_roll = true;
            0.99
        }));
        assert!(
            !drew_roll,
            "the roll is only drawn when the target has not moved"
        );
    }

    #[tokio::test]
    async fn a_blocked_view_holds_the_attack() -> Result<(), Box<dyn std::error::Error>> {
        use pumpkin_data::Block;
        use pumpkin_data::entity::EntityType;
        use pumpkin_util::math::position::BlockPos;
        use pumpkin_util::math::vector2::Vector2;
        use pumpkin_world::chunk::ChunkData;
        use pumpkin_world::world::BlockFlags;

        use crate::entity::Entity;
        use crate::entity::ai::test_support::world;
        use crate::entity::passive::polar_bear::PolarBearEntity;

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        for chunk_x in -1..=1 {
            for chunk_z in -1..=1 {
                world.level.loaded_chunks.insert(
                    Vector2::new(chunk_x, chunk_z),
                    ChunkData::empty_sync(chunk_x, chunk_z),
                );
            }
        }
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        let spawn = |z| {
            PolarBearEntity::new(Entity::new(
                world.clone(),
                Vector3::new(0.5, 64.0, z),
                &EntityType::POLAR_BEAR,
            ))
        };
        // 1.5 blocks apart: inside melee reach, with the wall block in between.
        let bear = spawn(0.5);
        let target = spawn(2.0);
        let goal = MeleeAttackGoal::new(1.25, true);
        assert!(goal.can_perform_attack(bear.as_ref(), target.as_ref()));

        for y in 64..=66 {
            world.set_block_state(
                &BlockPos::new(0, y, 1),
                Block::STONE.default_state.id,
                flags,
            );
        }
        assert!(!goal.can_perform_attack(bear.as_ref(), target.as_ref()));

        for y in 64..=66 {
            world.set_block_state(&BlockPos::new(0, y, 1), Block::AIR.default_state.id, flags);
        }
        assert!(goal.can_perform_attack(bear.as_ref(), target.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }
}
