use serde::{Deserialize, Serialize};

/// Limits on how many bytes a book edited by a client may hold.
///
/// Vanilla only bounds the page count and characters per page, which still lets
/// a shulker of books full of multi-byte characters exceed the client's packet
/// size limit and kick everyone who loads it.
#[derive(Deserialize, Serialize, Clone, Debug, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct BookSizeConfig {
    /// Whether book edits are checked against the byte budget.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Byte budget of a full page. Shorter pages get a proportionally smaller budget.
    #[serde(alias = "page-max")]
    #[toml(default = Self::default().page_max, alias = "page-max")]
    pub page_max: u32,
    /// Factor applied to each page's budget, clamped to `0.3..=1.0`.
    #[serde(alias = "total-multiplier")]
    #[toml(default = Self::default().total_multiplier, alias = "total-multiplier")]
    pub total_multiplier: f64,
}

impl Default for BookSizeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            page_max: 2560,
            total_multiplier: 0.98,
        }
    }
}
