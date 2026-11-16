use serde::{Deserialize, Serialize};

/// Resolves a configured secret, reading it from a file when the value starts
/// with `@` (the path is relative to the working directory).
///
/// This allows secrets to live outside the configuration file, e.g.
/// `secret = "@forwarding.secret"`. If the file cannot be read the raw value
/// is returned so authentication fails instead of silently succeeding.
#[must_use]
pub fn resolve_secret(value: &str) -> String {
    let Some(path) = value.strip_prefix('@') else {
        return value.to_string();
    };
    match std::fs::read_to_string(path) {
        Ok(contents) => contents.trim().to_string(),
        Err(err) => {
            tracing::warn!("Couldn't read secret file at {path}: {err}");
            value.to_string()
        }
    }
}

/// Configuration for proxy support.
///
/// Allows integration with proxy servers like Velocity and `BungeeCord`.
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct ProxyConfig {
    /// Whether proxy support is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Configuration for `HAProxy` PROXY protocol integration.
    pub haproxy: HAProxyConfig,
    /// Configuration for Velocity proxy integration.
    #[toml(default = Self::default().velocity)]
    pub velocity: VelocityConfig,
    /// Configuration for `BungeeCord` proxy integration.
    #[toml(default = Self::default().bungeecord)]
    pub bungeecord: BungeeCordConfig,
    /// Configuration for Vine modern proxy integration with Ed25519 authentication.
    #[toml(default = Self::default().vine)]
    pub vine: VineConfig,
}

/// Configuration for `HAProxy` PROXY protocol integration.
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct HAProxyConfig {
    /// Whether `HAProxy` PROXY protocol (v1 and v2) support is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
}

/// Configuration for `BungeeCord` proxy integration.
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct BungeeCordConfig {
    /// Whether `BungeeCord` support is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Shared secret for authenticating connections from the `BungeeCord`
    /// proxy, as provided by the `BungeeGuard` plugin. When set, the forwarded
    /// profile properties must contain a `bungeeguard-token` property holding
    /// this secret, otherwise the connection is rejected. This also blocks
    /// players connecting directly instead of through the proxy.
    #[toml(default = Self::default().secret)]
    pub secret: String,
}

/// Configuration for Velocity proxy integration.
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct VelocityConfig {
    /// Whether Velocity support is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Shared secret for authenticating connections from the Velocity proxy.
    /// Prefix the value with `@` to read it from a file instead, e.g.
    /// `"@forwarding.secret"`.
    #[toml(default = Self::default().secret)]
    pub secret: String,
}

/// Configuration for Vine proxy integration with Ed25519 authentication and replay protection.
#[derive(Deserialize, Serialize, Default, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct VineConfig {
    /// Whether Vine support is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Ed25519 public key (64 hex characters) of the Vine proxy.
    /// Backend only needs this public key to verify forwarded player identities.
    #[toml(default = Self::default().public_key)]
    pub public_key: String,
    /// Optional shared secret string. If provided and `public_key` is empty,
    /// the public key is automatically derived from this secret.
    #[toml(default = Self::default().secret)]
    pub secret: String,
}

#[cfg(test)]
mod tests {
    use super::resolve_secret;

    #[test]
    fn resolves_secret_from_file() {
        let path = std::env::temp_dir().join(format!("kabocha-secret-{}", std::process::id()));
        std::fs::write(&path, "file-secret\n").unwrap();
        let value = format!("@{}", path.display());
        assert_eq!(resolve_secret(&value), "file-secret");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn keeps_plain_secret() {
        assert_eq!(resolve_secret("plain-secret"), "plain-secret");
    }
}
