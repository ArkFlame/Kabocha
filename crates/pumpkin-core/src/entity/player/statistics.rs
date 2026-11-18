pub use pumpkin_data::statistic::{CustomStatistic, StatisticCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use rustc_hash::FxHashMap;

#[derive(Default)]
pub struct Statistics {
    /// (Category ID, Statistic ID) -> Value
    pub stats: FxHashMap<(i32, i32), i32>,
}

impl Statistics {
    pub fn increment(&mut self, category: StatisticCategory, stat: i32, amount: i32) {
        let entry = self.stats.entry((category as i32, stat)).or_insert(0);
        *entry = entry.saturating_add(amount);
    }

    pub fn increment_custom(&mut self, stat: CustomStatistic, amount: i32) {
        self.increment(StatisticCategory::Custom, stat as i32, amount);
    }

    pub fn set(&mut self, category: StatisticCategory, stat: i32, value: i32) {
        self.stats.insert((category as i32, stat), value);
    }

    #[must_use]
    pub fn get(&self, category: StatisticCategory, stat: i32) -> i32 {
        *self.stats.get(&(category as i32, stat)).unwrap_or(&0)
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        let mut stats_compound = NbtCompound::new();
        for ((category, stat), value) in &self.stats {
            stats_compound.put_int(&format!("{category}:{stat}"), *value);
        }
        nbt.put_compound("Statistics", stats_compound);
    }

    pub fn read_nbt(&mut self, nbt: &NbtCompound) {
        if let Some(stats_compound) = nbt.get_compound("Statistics") {
            for (key, tag) in &stats_compound.child_tags {
                let parts: Vec<&str> = key.split(':').collect();
                if let (NbtTag::Int(value), [cat_str, stat_str]) = (tag, parts.as_slice())
                    && let (Ok(category), Ok(stat)) =
                        (cat_str.parse::<i32>(), stat_str.parse::<i32>())
                {
                    self.stats.insert((category, stat), *value);
                }
            }
        }
    }
}

/// Branch state of vanilla `ServerPlayer.checkMovementStatistics` for one accepted move.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Default)]
pub struct MovementState {
    pub swimming: bool,
    pub eye_in_water: bool,
    pub in_water: bool,
    pub on_climbable: bool,
    pub on_ground: bool,
    pub sprinting: bool,
    pub crouching: bool,
    pub fall_flying: bool,
}

/// Selects the vanilla statistic for a non-passenger move from the branch state.
///
/// Vanilla orders `isFallFlying()` after the swimming, water, climbable and ground branches, so
/// an elytra glide only awards `AVIATE_ONE_CM` once every earlier branch has been ruled out.
#[must_use]
pub const fn movement_statistic(state: MovementState) -> CustomStatistic {
    if state.swimming {
        CustomStatistic::SwimOneCm
    } else if state.eye_in_water {
        CustomStatistic::WalkUnderWaterOneCm
    } else if state.in_water {
        CustomStatistic::WalkOnWaterOneCm
    } else if state.on_climbable {
        CustomStatistic::ClimbOneCm
    } else if state.on_ground {
        if state.sprinting {
            CustomStatistic::SprintOneCm
        } else if state.crouching {
            CustomStatistic::CrouchOneCm
        } else {
            CustomStatistic::WalkOneCm
        }
    } else if state.fall_flying {
        CustomStatistic::AviateOneCm
    } else {
        CustomStatistic::FlyOneCm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_nbt_round_trip() {
        let mut stats = Statistics::default();
        stats.increment_custom(CustomStatistic::PlayTime, 123);
        stats.increment(StatisticCategory::Mined, 42, 7);

        let mut nbt = NbtCompound::new();
        stats.write_nbt(&mut nbt);

        let mut loaded = Statistics::default();
        loaded.read_nbt(&nbt);

        assert_eq!(
            loaded.get(StatisticCategory::Custom, CustomStatistic::PlayTime as i32),
            123
        );
        assert_eq!(loaded.get(StatisticCategory::Mined, 42), 7);
    }

    #[test]
    fn movement_statistic_follows_vanilla_check_order() {
        let glide = MovementState {
            fall_flying: true,
            ..MovementState::default()
        };
        assert_eq!(movement_statistic(glide), CustomStatistic::AviateOneCm);

        // Vanilla checks swimming, water, climbable and ground before fall flying.
        let swimming_glide = MovementState {
            swimming: true,
            fall_flying: true,
            ..MovementState::default()
        };
        assert_eq!(
            movement_statistic(swimming_glide),
            CustomStatistic::SwimOneCm
        );

        let grounded_glide = MovementState {
            on_ground: true,
            sprinting: true,
            fall_flying: true,
            ..MovementState::default()
        };
        assert_eq!(
            movement_statistic(grounded_glide),
            CustomStatistic::SprintOneCm
        );

        let submerged = MovementState {
            in_water: true,
            eye_in_water: true,
            ..MovementState::default()
        };
        assert_eq!(
            movement_statistic(submerged),
            CustomStatistic::WalkUnderWaterOneCm
        );

        let wading = MovementState {
            in_water: true,
            ..MovementState::default()
        };
        assert_eq!(
            movement_statistic(wading),
            CustomStatistic::WalkOnWaterOneCm
        );

        let climbing = MovementState {
            on_climbable: true,
            ..MovementState::default()
        };
        assert_eq!(movement_statistic(climbing), CustomStatistic::ClimbOneCm);

        let crouching = MovementState {
            on_ground: true,
            crouching: true,
            ..MovementState::default()
        };
        assert_eq!(movement_statistic(crouching), CustomStatistic::CrouchOneCm);

        let walking = MovementState {
            on_ground: true,
            ..MovementState::default()
        };
        assert_eq!(movement_statistic(walking), CustomStatistic::WalkOneCm);

        let airborne = MovementState::default();
        assert_eq!(movement_statistic(airborne), CustomStatistic::FlyOneCm);
    }
}
