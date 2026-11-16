use serde::{Deserialize, Deserializer, Serialize};

use crate::{chunk::ChunkConfig, lighting::LightingEngineConfig};

/// Configuration for world and level-specific settings.
///
/// Currently, it includes chunk-related options; more settings may be added later.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct LevelConfig {
    /// Configuration for chunk behaviour and management.
    #[toml(default = Self::default().chunk)]
    pub chunk: ChunkConfig,
    /// Configuration for lighting engine propagation mode.
    #[serde(default)]
    #[toml(default = Self::default().lighting)]
    pub lighting: LightingEngineConfig,
    /// Number of ticks between autosave checks. If 0, autosave is disabled.
    #[serde(default = "default_autosave_ticks")]
    #[toml(default = Self::default().autosave_ticks)]
    pub autosave_ticks: u64,
    /// Margins for ticking entities in parallel islands.
    #[serde(default)]
    #[toml(default = Self::default().entity_islands)]
    pub entity_islands: EntityIslandsConfig,
    /// Maximum number of mobs spawned per category, like Bukkit's `spawn-limits`.
    #[serde(default)]
    #[toml(default = Self::default().spawn_limits)]
    pub spawn_limits: SpawnLimits,
    // TODO: More options
}

/// Maximum mobs per spawn category. Mirrors vanilla's per-category caps (and Bukkit's
/// `spawn-limits`): raise them for a livelier world, lower them to save resources.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, toml_spanner::Toml)]
#[serde(default)]
pub struct SpawnLimits {
    /// Hostile mobs (zombies, skeletons, spiders, ...).
    pub monsters: i32,
    /// Passive animals (cows, pigs, sheep, ...).
    pub animals: i32,
    /// Bats.
    pub ambient: i32,
    /// Axolotls.
    pub axolotls: i32,
    /// Glow squid and other underground water creatures.
    pub water_underground_creature: i32,
    /// Squid, dolphins and other water animals.
    pub water_animals: i32,
    /// Tropical fish and other water ambient mobs.
    pub water_ambient: i32,
}

impl SpawnLimits {
    /// Cap for a mob category by its generated `MobCategory::id`. `None` for categories
    /// without a configurable limit (misc), which keep the vanilla value.
    #[must_use]
    pub const fn for_category_id(&self, id: usize) -> Option<i32> {
        Some(match id {
            0 => self.monsters,
            1 => self.animals,
            2 => self.ambient,
            3 => self.axolotls,
            4 => self.water_underground_creature,
            5 => self.water_animals,
            6 => self.water_ambient,
            _ => return None,
        })
    }
}

impl Default for SpawnLimits {
    fn default() -> Self {
        Self {
            monsters: 70,
            animals: 10,
            ambient: 15,
            axolotls: 5,
            water_underground_creature: 5,
            water_animals: 5,
            water_ambient: 20,
        }
    }
}

/// Safety margins for ticking entities in parallel islands. Lower values tick more entities in
/// parallel but can let interactions run out of tick order.
#[derive(Deserialize, Serialize, Clone, Copy, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct EntityIslandsConfig {
    /// Blocks an entity whose tick is not audited can reach beyond its follow range.
    #[serde(deserialize_with = "EntityIslandsConfig::deserialize_unaudited_reach")]
    #[toml(default = Self::default().unaudited_reach, with = unaudited_reach)]
    pub unaudited_reach: f64,
    /// Blocks a hit from such an entity can throw an audited one in a tick. Raise it for TNT
    /// cannons.
    #[serde(deserialize_with = "EntityIslandsConfig::deserialize_knockback_reach")]
    #[toml(default = Self::default().knockback_reach, with = knockback_reach)]
    pub knockback_reach: f64,
}

impl EntityIslandsConfig {
    fn deserialize_unaudited_reach<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<f64, D::Error> {
        deserialize_margin(deserializer, Self::default().unaudited_reach)
    }

    fn deserialize_knockback_reach<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<f64, D::Error> {
        deserialize_margin(deserializer, Self::default().knockback_reach)
    }
}

fn deserialize_margin<'de, D: Deserializer<'de>>(
    deserializer: D,
    fallback: f64,
) -> Result<f64, D::Error> {
    let value = f64::deserialize(deserializer)?;
    Ok(if value.is_finite() {
        value.max(0.0)
    } else {
        fallback
    })
}

impl Default for EntityIslandsConfig {
    fn default() -> Self {
        Self {
            unaudited_reach: 16.0,
            knockback_reach: 4.0,
        }
    }
}

const fn default_autosave_ticks() -> u64 {
    6000 // Default to 5 minutes at 20 TPS
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self {
            chunk: ChunkConfig::default(),
            lighting: LightingEngineConfig::default(),
            autosave_ticks: default_autosave_ticks(),
            entity_islands: EntityIslandsConfig::default(),
            spawn_limits: SpawnLimits::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_level_config_enables_autosave() {
        assert_eq!(LevelConfig::default().autosave_ticks, 6000);
    }

    #[test]
    fn generated_config_round_trip_keeps_autosave_enabled() {
        let generated = toml::to_string(&LevelConfig::default()).unwrap();
        let reloaded: LevelConfig = toml::from_str(&generated).unwrap();
        assert_eq!(reloaded.autosave_ticks, 6000);
    }

    #[test]
    fn an_explicit_zero_still_disables_autosave() {
        let disabled = LevelConfig {
            autosave_ticks: 0,
            ..LevelConfig::default()
        };
        let generated = toml::to_string(&disabled).unwrap();
        let reloaded: LevelConfig = toml::from_str(&generated).unwrap();
        assert_eq!(reloaded.autosave_ticks, 0);
    }

    #[test]
    fn configured_spawn_limits_round_trip() {
        let config = LevelConfig {
            spawn_limits: SpawnLimits {
                monsters: 140,
                ..SpawnLimits::default()
            },
            ..LevelConfig::default()
        };
        let reloaded: LevelConfig = toml::from_str(&toml::to_string(&config).unwrap()).unwrap();
        assert_eq!(reloaded.spawn_limits.monsters, 140);
        assert_eq!(reloaded.spawn_limits.animals, 10);
        // Missing keys fall back to the vanilla caps.
        let partial: SpawnLimits = toml::from_str("monsters = 20\n").unwrap();
        assert_eq!(partial.monsters, 20);
        assert_eq!(partial.water_ambient, 20);
    }
}

mod unaudited_reach {
    pub fn from_toml<'de>(
        ctx: &mut toml_spanner::Context<'de>,
        item: &toml_spanner::Item<'de>,
    ) -> Result<f64, toml_spanner::Failed> {
        super::toml_margin(
            ctx,
            item,
            super::EntityIslandsConfig::default().unaudited_reach,
        )
    }
}

mod knockback_reach {
    pub fn from_toml<'de>(
        ctx: &mut toml_spanner::Context<'de>,
        item: &toml_spanner::Item<'de>,
    ) -> Result<f64, toml_spanner::Failed> {
        super::toml_margin(
            ctx,
            item,
            super::EntityIslandsConfig::default().knockback_reach,
        )
    }
}

fn toml_margin<'de>(
    ctx: &mut toml_spanner::Context<'de>,
    item: &toml_spanner::Item<'de>,
    fallback: f64,
) -> Result<f64, toml_spanner::Failed> {
    let value = <f64 as toml_spanner::FromToml>::from_toml(ctx, item)?;
    Ok(if value.is_finite() {
        value.max(0.0)
    } else {
        fallback
    })
}
