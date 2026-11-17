use super::{Controls, Goal};

use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::mob::Mob;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

// UNVERIFIED: vanilla 1.21.x `BeePollinateGoal` values, from memory. Not checked against a decompile.
const FLOWER_SEARCH_HORIZONTAL: i32 = 5;
const FLOWER_SEARCH_VERTICAL: i32 = 2;
/// Hovering time at a flower before nectar is gathered.
const POLLINATE_TICKS: i32 = 400;
const SCAN_INTERVAL_TICKS: i64 = 20;
const REPATH_INTERVAL_TICKS: i32 = 10;
const BEE_SPEED: f64 = 1.0;
/// Squared distance to the flower centre at which the bee hovers.
const ARRIVE_DISTANCE_SQ: f64 = 1.0;

/// Closest block to `origin` that `is_flower` accepts, searched within the flower radius.
#[must_use]
pub fn nearest_flower(origin: BlockPos, is_flower: impl Fn(&BlockPos) -> bool) -> Option<BlockPos> {
    let mut best: Option<(i32, BlockPos)> = None;
    for dx in -FLOWER_SEARCH_HORIZONTAL..=FLOWER_SEARCH_HORIZONTAL {
        for dy in -FLOWER_SEARCH_VERTICAL..=FLOWER_SEARCH_VERTICAL {
            for dz in -FLOWER_SEARCH_HORIZONTAL..=FLOWER_SEARCH_HORIZONTAL {
                let dist_sq = dx * dx + dy * dy + dz * dz;
                if best.is_some_and(|(best_sq, _)| best_sq <= dist_sq) {
                    continue;
                }
                let pos = origin.offset(Vector3::new(dx, dy, dz));
                if is_flower(&pos) {
                    best = Some((dist_sq, pos));
                }
            }
        }
    }
    best.map(|(_, pos)| pos)
}

/// Vanilla bee pollination: fly to a nearby flower, hover, then gather nectar.
pub struct PollinateFlowerGoal {
    gate: MobFilter,
    on_pollinated: fn(&dyn Mob),
    on_no_flower: fn(&dyn Mob),
    on_stop: fn(&dyn Mob),
    flower: Option<BlockPos>,
    pollinate_ticks: i32,
    repath_ticks: i32,
    next_scan_tick: i64,
}

impl PollinateFlowerGoal {
    /// `gate` says whether the bee wants nectar and may search. `on_pollinated` records the
    /// nectar; `on_no_flower` and `on_stop` drive the vanilla flower-search cooldown.
    #[must_use]
    pub const fn new(
        gate: MobFilter,
        on_pollinated: fn(&dyn Mob),
        on_no_flower: fn(&dyn Mob),
        on_stop: fn(&dyn Mob),
    ) -> Self {
        Self {
            gate,
            on_pollinated,
            on_no_flower,
            on_stop,
            flower: None,
            pollinate_ticks: 0,
            repath_ticks: 0,
            next_scan_tick: 0,
        }
    }

    fn is_flower_at(world: &crate::world::World, pos: &BlockPos) -> bool {
        world.get_block(pos).has_tag(&tag::Block::MINECRAFT_FLOWERS)
    }
}

impl Goal for PollinateFlowerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !(self.gate)(mob) {
            return false;
        }
        let world = mob.get_entity().world.load();
        let now = world.get_world_age();
        if now < self.next_scan_tick {
            return false;
        }
        self.next_scan_tick = now + SCAN_INTERVAL_TICKS;
        let bee_pos = mob.get_entity().block_pos.load();
        if world.is_raining_at(&bee_pos) {
            return false;
        }
        self.flower = nearest_flower(bee_pos, |pos| Self::is_flower_at(&world, pos));
        if self.flower.is_some() {
            true
        } else {
            // Vanilla `BeePollinateGoal.canBeeUse` sets a 20-60 tick retry when no flower is found.
            (self.on_no_flower)(mob);
            false
        }
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if !(self.gate)(mob) || self.pollinate_ticks >= POLLINATE_TICKS {
            return false;
        }
        let Some(flower) = self.flower else {
            return false;
        };
        Self::is_flower_at(&mob.get_entity().world.load(), &flower)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.pollinate_ticks = 0;
        self.repath_ticks = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        // Vanilla `BeePollinateGoal.stop` waits 200 ticks before locating another flower.
        (self.on_stop)(mob);
        mob.get_mob_entity().stop_navigation();
        self.flower = None;
        self.pollinate_ticks = 0;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(flower) = self.flower else {
            return;
        };
        let target = flower.to_centered_f64();
        let mob_pos = mob.get_entity().pos.load();
        if mob_pos.squared_distance_to_vec(&target) > ARRIVE_DISTANCE_SQ {
            if self.repath_ticks > 0 {
                self.repath_ticks -= 1;
                return;
            }
            self.repath_ticks = REPATH_INTERVAL_TICKS;
            mob.navigate_to(mob_pos, target, BEE_SPEED);
            return;
        }
        mob.get_mob_entity().stop_navigation();
        self.pollinate_ticks += 1;
        if self.pollinate_ticks >= POLLINATE_TICKS {
            (self.on_pollinated)(mob);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_flower_prefers_the_closest_match() {
        let origin = BlockPos::new(0, 64, 0);
        let flowers = [BlockPos::new(3, 64, 0), BlockPos::new(-1, 65, 1)];
        let found = nearest_flower(origin, |pos| flowers.contains(pos));
        assert_eq!(found, Some(BlockPos::new(-1, 65, 1)));
    }

    #[test]
    fn nearest_flower_ignores_blocks_outside_the_search_box() {
        let origin = BlockPos::new(0, 64, 0);
        let far = BlockPos::new(FLOWER_SEARCH_HORIZONTAL + 1, 64, 0);
        assert_eq!(nearest_flower(origin, |pos| *pos == far), None);
        let high = BlockPos::new(0, 64 + FLOWER_SEARCH_VERTICAL + 1, 0);
        assert_eq!(nearest_flower(origin, |pos| *pos == high), None);
    }
}
