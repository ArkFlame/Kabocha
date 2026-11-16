#[cfg(feature = "codegen")]
use proc_macro2::TokenStream;
#[cfg(feature = "codegen")]
use quote::{ToTokens, quote};
use serde::Deserialize;

use super::int_provider::IntProvider;

#[derive(Deserialize, Clone, Debug)]
pub struct Experience {
    /// The experience points, represented as an `IntProvider`.
    pub experience: IntProvider,
}

#[cfg(feature = "codegen")]
impl ToTokens for Experience {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let experience = self.experience.to_token_stream();

        tokens.extend(quote! {
            Experience { experience: #experience }
        });
    }
}

/// Returns the number of points required to progress within a specific level.
///
/// # Arguments
/// * `level` – The level to calculate points for.
#[must_use]
pub const fn points_in_level(level: i32) -> i32 {
    match level {
        0..=15 => 2 * level + 7,
        16..=30 => 5 * level - 38,
        // Java `int` arithmetic wraps here as well at absurd levels.
        _ => 9i32.wrapping_mul(level).wrapping_sub(158),
    }
}

/// Adds experience points the way vanilla `Player#giveExperiencePoints` does: it moves
/// whole levels one at a time instead of deriving the level from a total that saturates
/// at `i32::MAX`, so experience keeps accumulating past that limit.
///
/// Returns the new level and the progress (`0.0` to `1.0`) within that level.
#[must_use]
pub fn add_points(level: i32, progress: f32, added_points: i32) -> (i32, f32) {
    let mut level = level;
    let mut progress = progress + added_points as f32 / points_in_level(level) as f32;

    while progress < 0.0 {
        let overflow = progress * points_in_level(level) as f32;
        if level > 0 {
            level -= 1;
            progress = 1.0 + overflow / points_in_level(level) as f32;
        } else {
            progress = 0.0;
        }
    }

    while progress >= 1.0 {
        progress = (progress - 1.0) * points_in_level(level) as f32;
        level = level.wrapping_add(1);
        progress /= points_in_level(level) as f32;
    }

    (level, progress)
}

/// Calculates the total points required to reach a given level.
///
/// # Arguments
/// * `level` – The target level.
#[must_use]
pub fn points_to_level(level: i32) -> i32 {
    match level {
        0..=16 => level * level + 6 * level,
        17..=31 => {
            (2.5f64.mul_add(f64::from(level * level), -(40.5 * f64::from(level))) + 360.0) as i32
        }
        _ => {
            (4.5f64.mul_add(f64::from(level * level), -(162.5 * f64::from(level))) + 2220.0) as i32
        }
    }
}

/// Converts total experience points into a level and points within that level.
///
/// # Arguments
/// * `total_points` – The total accumulated experience points.
///
/// # Returns
/// A tuple `(level, points_into_level)` representing the current level and
/// remaining points within that level.
#[must_use]
pub fn total_to_level_and_points(total_points: i32) -> (i32, i32) {
    let level = match total_points {
        0..=352 => ((f64::from(total_points) + 9.0).sqrt() - 3.0) as i32,
        353..=1507 => (8.1 + (0.4 * (f64::from(total_points) - (7839.0 / 40.0))).sqrt()) as i32,
        _ => {
            ((325.0 / 18.0) + (2.0 / 9.0 * (f64::from(total_points) - (54215.0 / 72.0))).sqrt())
                as i32
        }
    };
    let level_start = points_to_level(level);
    let points_into_level = total_points - level_start;

    (level, points_into_level)
}

/// Calculates the progress within a level as a value between 0.0 and 1.0.
///
/// # Arguments
/// * `points` – The points accumulated in the current level.
/// * `level` – The current level.
#[must_use]
pub fn progress_in_level(points: i32, level: i32) -> f32 {
    let max_points = points_in_level(level);
    let progress = (points as f32) / (max_points as f32);

    progress.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{add_points, points_in_level};

    #[test]
    fn add_points_keeps_accumulating_past_the_i32_total_limit() {
        // A level whose total experience already exceeds `i32::MAX`; the old
        // total based recalculation clamped it back to ~21863.
        let level = 22_000;
        let (new_level, progress) = add_points(level, 0.0, 1_000);

        assert_eq!(new_level, level);
        let expected = 1_000.0 / points_in_level(level) as f32;
        assert!((progress - expected).abs() < 1e-6, "progress {progress}");
    }

    #[test]
    fn add_points_carries_and_borrows_levels() {
        let (level, progress) = add_points(0, 0.9, 3);
        assert_eq!(level, 1);
        assert!((progress - 0.255_555_6).abs() < 1e-5, "progress {progress}");

        let (level, progress) = add_points(1, 0.05, -1);
        assert_eq!(level, 0);
        assert!((progress - 0.921_428_6).abs() < 1e-5, "progress {progress}");
    }
}
