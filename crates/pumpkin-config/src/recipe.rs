use serde::{Deserialize, Serialize};

/// Recipe-related configuration.
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct RecipeConfig {
    /// Whether the recipe book packets are sent when a player joins. The stonecutter recipes are
    /// always sent, because the stonecutter does not work without them.
    #[toml(default = Self::default().send_recipes)]
    pub send_recipes: bool,
}

impl Default for RecipeConfig {
    fn default() -> Self {
        Self { send_recipes: true }
    }
}
