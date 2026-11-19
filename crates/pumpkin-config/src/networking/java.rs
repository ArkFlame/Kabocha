use crate::{AuthenticationConfig, BookSizeConfig, CompressionConfig, PacketLimiterConfig};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::num::NonZero;

/// Configuration for Java Edition client connections.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct JavaConfig {
    /// Whether Java Edition Clients are Accepted.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// The address and port to which the Java Edition server will bind.
    #[toml(default = Self::default().address, with = toml_spanner::helper::parse_string)]
    pub address: SocketAddr,
    /// Whether packet encryption is enabled. Required when online mode is enabled.
    #[toml(default = Self::default().encryption)]
    pub encryption: bool,
    /// Whether online mode is enabled. Requires valid Minecraft accounts.
    #[toml(default = Self::default().online_mode)]
    pub online_mode: bool,
    /// The maximum number of players allowed on the server. Specifying `0` disables the limit.
    #[toml(default = Self::default().max_players)]
    pub max_players: u32,
    /// The maximum view distance for players.
    #[toml(default = Self::default().view_distance, with = crate::toml_adapters::serde_value)]
    pub view_distance: NonZero<u8>,
    /// The maximum simulated view distance.
    #[toml(default = Self::default().simulation_distance, with = crate::toml_adapters::serde_value)]
    pub simulation_distance: NonZero<u8>,
    /// Time interval in seconds between keep-alive packets sent to Java clients.
    #[serde(
        alias = "keep-alive-time",
        alias = "keep_alive_interval",
        alias = "keep-alive-interval"
    )]
    #[toml(default = Self::default().keep_alive_time, alias = "keep-alive-time", alias = "keep_alive_interval", alias = "keep-alive-interval")]
    pub keep_alive_time: u64,
    /// Java Edition packet compression settings.
    #[toml(default = Self::default().compression)]
    pub compression: CompressionConfig,
    /// Message of the Day; the server's description displayed on the status screen.
    #[toml(default = Self::default().motd)]
    pub motd: String,
    /// Authentication settings for client connections.
    #[toml(default = Self::default().authentication)]
    pub authentication: AuthenticationConfig,
    /// Packet rate limiting settings.
    #[toml(default = Self::default().packet_limiter)]
    pub packet_limiter: PacketLimiterConfig,
    /// Per-IP connection, status and login hit-rate limiting, with an in-process ban list.
    #[toml(default = Self::default().rate_limit)]
    pub rate_limit: RateLimitConfig,
    /// Byte limits for books edited by clients.
    #[toml(default = Self::default().book_size)]
    pub book_size: BookSizeConfig,
}

/// Per-IP hit-rate limiting for Java Edition connections (iptables `hashlimit` style).
///
/// Each rule is `"hits/seconds"`. Every rule in a list must pass for a hit to be allowed.
/// Rejected hits are dropped silently and do not extend the window.
#[derive(Deserialize, Serialize, Clone, Debug, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct RateLimitConfig {
    /// Whether the per-IP rate limiter is enabled. When `false`, every hit is allowed.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Rules applied to every new TCP connection, whatever it is for.
    #[toml(default = Self::default().connections)]
    pub connections: Vec<String>,
    /// Rules applied to server-list status pings.
    #[toml(default = Self::default().status)]
    pub status: Vec<String>,
    /// Rules applied to join attempts (login and transfer intents).
    #[toml(default = Self::default().logins)]
    pub logins: Vec<String>,
    /// IP addresses or `ip/prefix` CIDR ranges that are never limited or banned.
    #[toml(default = Self::default().exempt)]
    pub exempt: Vec<String>,
    /// Ban list applied after repeated rate-limit violations.
    #[toml(default = Self::default().ipset)]
    pub ipset: IpSetConfig,
}

/// In-process ban list (ipset-style) fed by rate-limit violations.
///
/// This does not touch the system `iptables` or `ipset`. Banned IPs are dropped at accept.
#[derive(Deserialize, Serialize, Clone, Debug, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct IpSetConfig {
    /// Whether repeated violations ban the IP.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Number of rejected hits within `ban_window_seconds` that triggers a ban.
    #[toml(default = Self::default().ban_after)]
    pub ban_after: u32,
    /// Sliding window in seconds over which rejected hits are counted.
    #[toml(default = Self::default().ban_window_seconds)]
    pub ban_window_seconds: u64,
    /// How long a banned IP is dropped at accept, in seconds.
    #[toml(default = Self::default().ban_seconds)]
    pub ban_seconds: u64,
    /// Upper bound on tracked IPs and on active bans. Oldest entries are evicted beyond it.
    #[toml(default = Self::default().max_entries)]
    pub max_entries: u64,
}

impl Default for JavaConfig {
    fn default() -> Self {
        let address = "0.0.0.0:25565"
            .parse()
            .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 25565)));
        let view_distance = NonZero::new(16).unwrap_or(NonZero::<u8>::MIN);
        let simulation_distance = NonZero::new(10).unwrap_or(NonZero::<u8>::MIN);
        Self {
            enabled: true,
            address,
            encryption: true,
            online_mode: true,
            max_players: 1000,
            view_distance,
            simulation_distance,
            keep_alive_time: 15,
            compression: CompressionConfig::default(),
            motd: "A blazingly fast Pumpkin server!".to_string(),
            authentication: AuthenticationConfig::default(),
            packet_limiter: PacketLimiterConfig::default(),
            rate_limit: RateLimitConfig::default(),
            book_size: BookSizeConfig::default(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            connections: vec!["20/1".to_string(), "60/10".to_string()],
            status: vec!["10/1".to_string()],
            logins: vec!["2/1".to_string(), "6/10".to_string()],
            exempt: vec!["127.0.0.1".to_string(), "::1".to_string()],
            ipset: IpSetConfig::default(),
        }
    }
}

impl Default for IpSetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ban_after: 10,
            ban_window_seconds: 10,
            ban_seconds: 60,
            max_entries: 65_536,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keep_alive_time() {
        let config = JavaConfig::default();
        assert_eq!(config.keep_alive_time, 15);
    }

    #[test]
    fn keep_alive_time_deserialization() {
        let toml_snake = r"
            keep_alive_time = 20
        ";
        let config: JavaConfig = toml::from_str(toml_snake).unwrap();
        assert_eq!(config.keep_alive_time, 20);

        let toml_kebab = r"
            keep-alive-time = 25
        ";
        let config: JavaConfig = toml::from_str(toml_kebab).unwrap();
        assert_eq!(config.keep_alive_time, 25);

        let toml_interval = r"
            keep_alive_interval = 30
        ";
        let config: JavaConfig = toml::from_str(toml_interval).unwrap();
        assert_eq!(config.keep_alive_time, 30);

        let toml_interval_kebab = r"
            keep-alive-interval = 35
        ";
        let config: JavaConfig = toml::from_str(toml_interval_kebab).unwrap();
        assert_eq!(config.keep_alive_time, 35);
    }

    #[test]
    fn rate_limit_defaults_match_spec() {
        let rl = RateLimitConfig::default();
        assert!(rl.enabled);
        assert_eq!(rl.connections, ["20/1", "60/10"]);
        assert_eq!(rl.status, ["10/1"]);
        assert_eq!(rl.logins, ["2/1", "6/10"]);
        assert_eq!(rl.exempt, ["127.0.0.1", "::1"]);
        assert!(rl.ipset.enabled);
        assert_eq!(rl.ipset.ban_after, 10);
        assert_eq!(rl.ipset.ban_window_seconds, 10);
        assert_eq!(rl.ipset.ban_seconds, 60);
        assert_eq!(rl.ipset.max_entries, 65_536);
    }

    #[test]
    fn rate_limit_toml_override_parses() {
        let toml_str = r#"
            [rate_limit]
            enabled = false
            logins = ["5/1"]
            exempt = ["10.0.0.0/8"]

            [rate_limit.ipset]
            ban_after = 3
            ban_seconds = 120
        "#;
        let config: JavaConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.rate_limit.enabled);
        assert_eq!(config.rate_limit.logins, ["5/1"]);
        assert_eq!(config.rate_limit.exempt, ["10.0.0.0/8"]);
        assert_eq!(config.rate_limit.connections, ["20/1", "60/10"]);
        assert_eq!(config.rate_limit.ipset.ban_after, 3);
        assert_eq!(config.rate_limit.ipset.ban_seconds, 120);
        assert_eq!(config.rate_limit.ipset.max_entries, 65_536);
    }
}
