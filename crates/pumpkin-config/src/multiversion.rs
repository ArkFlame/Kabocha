use serde::{Deserialize, Serialize};

/// Built-in multi-version support for older Java clients.
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct MultiversionConfig {
    /// Whether Java clients older than the server version may join (translated by `pumpkin-multiversion`).
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
}

impl Default for MultiversionConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
