use serde::{Deserialize, Serialize};

/// Telemetry configuration options.
#[derive(Deserialize, Serialize, Clone, Debug, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct TelemetryConfig {
    /// Whether anonymous telemetry is enabled. Default is true.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Custom telemetry backend ingestion endpoint.
    #[toml(default = Self::default().endpoint)]
    pub endpoint: String,
    /// Heartbeat interval in seconds (default: 300 seconds / 5 minutes). Minimum 60s.
    #[toml(default = Self::default().interval_secs)]
    pub interval_secs: u64,
    /// Whether to opt-in to displaying this server in the public community directory.
    #[toml(default = Self::default().public)]
    pub public: bool,
    /// Public server name displayed on the analytics dashboard if public is true.
    #[toml(default = Self::default().server_name)]
    pub server_name: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "https://telemetry.pumpkinmc.org/v1/heartbeat".to_string(),
            interval_secs: 300,
            public: false,
            server_name: None,
        }
    }
}

impl TelemetryConfig {
    /// Validates telemetry configuration options.
    ///
    /// Panics if the endpoint is empty or lacks an HTTP scheme: such an endpoint
    /// makes every heartbeat fail at runtime with no startup indication.
    pub fn validate(&self) {
        assert!(
            !self.endpoint.is_empty(),
            "Telemetry endpoint must not be empty"
        );
        assert!(
            self.endpoint.starts_with("http://") || self.endpoint.starts_with("https://"),
            "Telemetry endpoint must be an http(s) URL"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_default() {
        let default_config = TelemetryConfig::default();
        assert!(default_config.enabled);
        assert_eq!(
            default_config.endpoint,
            "https://telemetry.pumpkinmc.org/v1/heartbeat"
        );
        assert_eq!(default_config.interval_secs, 300);
        assert!(!default_config.public);
        assert_eq!(default_config.server_name, None);
    }

    #[test]
    fn telemetry_toml_deserialization() {
        let toml_str = r#"
            enabled = false
            endpoint = "http://localhost:5000/v1/heartbeat"
            interval_secs = 60
            public = true
            server_name = "Test SMP"
        "#;

        let config: TelemetryConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.endpoint, "http://localhost:5000/v1/heartbeat");
        assert_eq!(config.interval_secs, 60);
        assert!(config.public);
        assert_eq!(config.server_name.as_deref(), Some("Test SMP"));
    }

    #[test]
    fn telemetry_default_validates() {
        TelemetryConfig::default().validate();
    }

    #[test]
    #[should_panic(expected = "Telemetry endpoint must not be empty")]
    fn telemetry_rejects_empty_endpoint() {
        let mut config = TelemetryConfig::default();
        config.endpoint.clear();
        config.validate();
    }

    #[test]
    #[should_panic(expected = "Telemetry endpoint must be an http(s) URL")]
    fn telemetry_rejects_non_http_endpoint() {
        let mut config = TelemetryConfig::default();
        config.endpoint = "ftp://example.com/heartbeat".to_string();
        config.validate();
    }
}
