use super::{Controls, Goal};

use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::mob::Mob;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, Property, block_properties::DoubleBlockHalf};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

/// Vanilla `BeePollinateGoal.findNearbyFlower` uses
/// `findBlocksInBoxByManhattanDistance(blockPosition(), 5)` in all three axes.
const FLOWER_SEARCH_RADIUS: i32 = 5;
/// Hovering time at a flower before nectar is gathered.
const POLLINATE_TICKS: i32 = 400;

/// Vanilla `BeePollinateGoal.hasPollinatedLongEnough`: strictly greater than 400.
#[must_use]
pub const fn pollination_complete(ticks: i32) -> bool {
    ticks > POLLINATE_TICKS
}

/// Vanilla `BeePollinateGoal.canBeeContinueToUse`: after 400 ticks the bee keeps hovering
/// with a 20% chance each tick (`random.nextFloat() < 0.2F`).
#[must_use]
pub const fn keeps_hovering_after_pollination(roll: f32) -> bool {
    roll < 0.2
}

/// Vanilla `Bee.attractsBees`: an attractive, non-waterlogged block counts, except a
/// sunflower only counts on its upper half.
#[must_use]
pub const fn attracts_bees(attractive_and_dry: bool, is_sunflower: bool, upper_half: bool) -> bool {
    attractive_and_dry && (!is_sunflower || upper_half)
}

const SCAN_INTERVAL_TICKS: i64 = 20;
const REPATH_INTERVAL_TICKS: i32 = 10;
const BEE_SPEED: f64 = 1.0;
/// Squared distance to the flower centre at which the bee hovers.
const ARRIVE_DISTANCE_SQ: f64 = 1.0;

/// Closest block to `origin` that `is_flower` accepts, searched within the flower radius.
#[must_use]
pub fn nearest_flower(origin: BlockPos, is_flower: impl Fn(&BlockPos) -> bool) -> Option<BlockPos> {
    let mut best: Option<(i32, BlockPos)> = None;
    for dx in -FLOWER_SEARCH_RADIUS..=FLOWER_SEARCH_RADIUS {
        for dy in -FLOWER_SEARCH_RADIUS..=FLOWER_SEARCH_RADIUS {
            for dz in -FLOWER_SEARCH_RADIUS..=FLOWER_SEARCH_RADIUS {
                // Vanilla orders the search box by Manhattan distance from the bee.
                let distance = dx.abs() + dy.abs() + dz.abs();
                if best.is_some_and(|(best_distance, _)| best_distance <= distance) {
                    continue;
                }
                let pos = origin.offset(Vector3::new(dx, dy, dz));
                if is_flower(&pos) {
                    best = Some((distance, pos));
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
    on_start: fn(&dyn Mob),
    flower: Option<BlockPos>,
    pollinate_ticks: i32,
    repath_ticks: i32,
    next_scan_tick: i64,
}

impl PollinateFlowerGoal {
    /// `gate` says whether the bee wants nectar and may search. `on_pollinated` records the
    /// nectar; `on_start` and `on_stop` track the vanilla `pollinating` flag, and
    /// `on_no_flower` drives the flower-search cooldown.
    #[must_use]
    pub const fn new(
        gate: MobFilter,
        on_pollinated: fn(&dyn Mob),
        on_no_flower: fn(&dyn Mob),
        on_stop: fn(&dyn Mob),
        on_start: fn(&dyn Mob),
    ) -> Self {
        Self {
            gate,
            on_pollinated,
            on_no_flower,
            on_stop,
            on_start,
            flower: None,
            pollinate_ticks: 0,
            repath_ticks: 0,
            next_scan_tick: 0,
        }
    }

    fn is_flower_at(world: &crate::world::World, pos: &BlockPos) -> bool {
        let (block, state) = world.get_block_and_state(pos);
        let upper_half = block.properties(state.id).is_some_and(|props| {
            props
                .to_props()
                .contains(&Property::DoubleBlockHalf(DoubleBlockHalf::Upper))
        });
        attracts_bees(
            block.has_tag(&tag::Block::MINECRAFT_BEE_ATTRACTIVE) && !state.id.is_waterlogged(),
            block == &Block::SUNFLOWER,
            upper_half,
        )
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
        if !(self.gate)(mob) {
            return false;
        }
        let world = mob.get_entity().world.load();
        // Vanilla `BeePollinateGoal.canBeeContinueToUse` stops when it starts raining.
        if world.is_raining_at(&mob.get_entity().block_pos.load()) {
            return false;
        }
        // Vanilla `BeePollinateGoal.canBeeContinueToUse`: after 400 ticks the bee keeps
        // hovering with a 20% chance each tick instead of stopping immediately.
        if pollination_complete(self.pollinate_ticks)
            && !keeps_hovering_after_pollination(rand::random::<f32>())
        {
            return false;
        }
        let Some(flower) = self.flower else {
            return false;
        };
        Self::is_flower_at(&world, &flower)
    }

    fn start(&mut self, mob: &dyn Mob) {
        // Vanilla `BeePollinateGoal.start` sets `pollinating = true`.
        (self.on_start)(mob);
        self.pollinate_ticks = 0;
        self.repath_ticks = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        // Vanilla `BeePollinateGoal.stop` sets nectar once when pollination lasted long
        // enough, clears the pollinating flag and waits 200 ticks before the next flower.
        if pollination_complete(self.pollinate_ticks) {
            (self.on_pollinated)(mob);
        }
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
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_flower_prefers_the_closest_manhattan_match() {
        let origin = BlockPos::new(0, 64, 0);
        // Manhattan 4 vs 3: the diagonal flower wins despite differing in y.
        let flowers = [BlockPos::new(4, 64, 0), BlockPos::new(-1, 65, 1)];
        let found = nearest_flower(origin, |pos| flowers.contains(pos));
        assert_eq!(found, Some(BlockPos::new(-1, 65, 1)));
    }

    #[test]
    fn nearest_flower_ignores_blocks_outside_the_search_box() {
        let origin = BlockPos::new(0, 64, 0);
        let far = BlockPos::new(FLOWER_SEARCH_RADIUS + 1, 64, 0);
        assert_eq!(nearest_flower(origin, |pos| *pos == far), None);
        let high = BlockPos::new(0, 64 + FLOWER_SEARCH_RADIUS + 1, 0);
        assert_eq!(nearest_flower(origin, |pos| *pos == high), None);
    }

    #[test]
    fn nearest_flower_searches_the_full_vertical_box() {
        let origin = BlockPos::new(0, 64, 0);
        let high = BlockPos::new(0, 64 + FLOWER_SEARCH_RADIUS, 0);
        assert_eq!(nearest_flower(origin, |pos| *pos == high), Some(high));
    }

    #[test]
    fn pollination_needs_more_than_four_hundred_ticks() {
        assert!(!pollination_complete(400));
        assert!(pollination_complete(401));
    }

    #[test]
    fn hover_roll_keeps_twenty_percent_of_pollinating_bees() {
        assert!(keeps_hovering_after_pollination(0.0));
        assert!(keeps_hovering_after_pollination(0.199_999));
        assert!(!keeps_hovering_after_pollination(0.2));
        assert!(!keeps_hovering_after_pollination(1.0));
    }

    #[test]
    fn attracts_bees_filters_waterlogged_and_lower_sunflower() {
        // Arguments: attractive_and_dry, is_sunflower, upper_half.
        assert!(attracts_bees(true, false, false));
        assert!(!attracts_bees(false, false, false));
        assert!(!attracts_bees(true, true, false));
        assert!(attracts_bees(true, true, true));
    }
}
