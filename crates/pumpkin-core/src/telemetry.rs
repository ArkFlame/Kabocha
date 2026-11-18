use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use pumpkin_config::TelemetryConfig;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::entity::player::Player;
use crate::net::ClientPlatform;
use crate::server::Server;

/// Maximum number of online player entries included in a single heartbeat payload.
///
/// Completed sessions buffered since the previous heartbeat are reported on top of this,
/// bounded separately by [`MAX_COMPLETED_SESSIONS_PER_HEARTBEAT`].
pub const MAX_TELEMETRY_PLAYERS: usize = 500;

/// Maximum number of completed player sessions buffered between heartbeats.
pub const MAX_COMPLETED_SESSIONS_PER_HEARTBEAT: usize = 1000;

/// Filename used to persist the telemetry identity key across restarts.
pub const IDENTITY_KEY_PATH: &str = ".pumpkin/identity.key";

/// Fallback filename used to persist the telemetry identity key across restarts.
pub const FALLBACK_IDENTITY_KEY_PATH: &str = "telemetry_key.bin";

/// HTTP header name for the Ed25519 public key in lowercase hex format.
pub const HEADER_PUBLIC_KEY: &str = "X-Telemetry-Public-Key";

/// HTTP header name for the Ed25519 signature in lowercase hex format.
pub const HEADER_SIGNATURE: &str = "X-Telemetry-Signature";

/// HTTP header name for the request timestamp in Unix seconds.
pub const HEADER_TIMESTAMP: &str = "X-Telemetry-Timestamp";

/// Maximum allowable clock drift between telemetry client and server (300 seconds / 5 minutes).
pub const MAX_CLOCK_DRIFT_SECS: u64 = 300;

/// HTTP header name for the replay-protection nonce (lowercase hex).
///
/// The authoritative copy is embedded in the signed JSON body; the header is provided for
/// verifier convenience only.
pub const HEADER_NONCE: &str = "X-Telemetry-Nonce";

/// JSON field carrying the replay-protection nonce inside the signed request body.
pub const NONCE_FIELD: &str = "nonce";

/// How long a seen nonce is retained by a verifier, twice the drift window to cover
/// the worst-case skew between a request's timestamp and the time it was first seen.
pub const NONCE_CACHE_TTL_SECS: u64 = MAX_CLOCK_DRIFT_SECS * 2;

/// Minimum heartbeat interval in seconds, applied to the configured value and to backend
/// recommendations.
pub const MIN_HEARTBEAT_INTERVAL_SECS: u64 = 60;

/// Information about an active plugin on the server.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PluginInfo {
    /// The name of the plugin.
    pub name: String,
    /// The version of the plugin.
    pub version: String,
    /// The guest binding language or runtime used by the plugin (e.g. "rust", "go", "javascript", "python", "native").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Type alias for plugin telemetry information.
pub type PluginTelemetryInfo = PluginInfo;

/// Client edition of a connected or disconnected player.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PlayerEdition {
    /// Minecraft: Java Edition client.
    Java,
    /// Minecraft: Bedrock Edition client.
    Bedrock,
}

/// Anonymous player session entry reported in server heartbeats and shutdown payloads.
///
/// Active online players have `left_at: None`; completed sessions since the previous
/// heartbeat report `left_at: Some(unix_secs)`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PlayerTelemetryInfo {
    /// Server-salted anonymous player identifier (32 lowercase hex characters, no PII).
    pub id: String,
    /// Client edition (`"java"` or `"bedrock"`).
    pub edition: PlayerEdition,
    /// Client brand reported via `minecraft:brand` (omitted for Bedrock clients).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_brand: Option<String>,
    /// Client language/locale code (e.g. `"en_us"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Client render/view distance in chunks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_distance: Option<u8>,
    /// Unix epoch timestamp (seconds) when the session started.
    pub joined_at: u64,
    /// Unix epoch timestamp (seconds) when the session ended, or `None` while still online.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_at: Option<u64>,
}

/// Payload transmitted on graceful server shutdown (`/v1/shutdown`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ShutdownPayload {
    /// Human-readable shutdown reason.
    pub reason: String,
    /// Any unsent completed or terminated player sessions flushed at shutdown.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub players: Vec<PlayerTelemetryInfo>,
}

#[derive(Clone, Debug)]
pub struct RawCompletedSession {
    player_uuid: uuid::Uuid,
    edition: PlayerEdition,
    client_brand: Option<String>,
    locale: Option<String>,
    view_distance: Option<u8>,
    joined_at: u64,
    left_at: u64,
}

impl RawCompletedSession {
    fn to_wire(&self, player_salt: &[u8; 32]) -> PlayerTelemetryInfo {
        PlayerTelemetryInfo {
            id: anonymize_player_uuid(player_salt, self.player_uuid),
            edition: self.edition,
            client_brand: self.client_brand.clone(),
            locale: self.locale.clone(),
            view_distance: self.view_distance,
            joined_at: self.joined_at,
            left_at: Some(self.left_at),
        }
    }
}

/// Thread-safe in-memory tracker for active and completed player sessions between heartbeats.
#[derive(Default)]
pub struct TelemetrySessionTracker {
    active_joins: Mutex<FxHashMap<uuid::Uuid, (Instant, u64)>>,
    completed: Mutex<Vec<RawCompletedSession>>,
}

impl TelemetrySessionTracker {
    /// Records the start of a player's session when they join the server.
    pub fn record_join(&self, player_uuid: uuid::Uuid) {
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.active_joins
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(player_uuid, (Instant::now(), now_unix));
    }

    /// Records the exact leave timestamp of a player's session when they disconnect.
    pub fn record_leave(&self, player: &Player) {
        let joined = self
            .active_joins
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&player.gameprofile.id);

        let Some((join_instant, joined_at)) = joined else {
            return;
        };

        let left_at = joined_at.saturating_add(join_instant.elapsed().as_secs());
        let (edition, client_brand, locale, view_distance) = extract_player_client_meta(player);

        let mut completed = self
            .completed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if completed.len() < MAX_COMPLETED_SESSIONS_PER_HEARTBEAT {
            completed.push(RawCompletedSession {
                player_uuid: player.gameprofile.id,
                edition,
                client_brand,
                locale,
                view_distance,
                joined_at,
                left_at,
            });
        } else {
            tracing::debug!(
                "Telemetry session buffer is full ({MAX_COMPLETED_SESSIONS_PER_HEARTBEAT}), dropping completed session"
            );
        }
    }

    /// Returns the Unix start timestamp (`joined_at`) for an active online player.
    pub fn joined_at(&self, player_uuid: uuid::Uuid) -> u64 {
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut active = self
            .active_joins
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let &mut (_, joined_at) = active
            .entry(player_uuid)
            .or_insert_with(|| (Instant::now(), now_unix));
        joined_at
    }

    /// Drains buffered completed sessions and converts them to anonymized wire entries.
    pub fn take_completed(
        &self,
        player_salt: &[u8; 32],
    ) -> (Vec<RawCompletedSession>, Vec<PlayerTelemetryInfo>) {
        let raw = std::mem::take(
            &mut *self
                .completed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let wire = raw.iter().map(|s| s.to_wire(player_salt)).collect();
        (raw, wire)
    }

    /// Non-destructively clones buffered completed sessions (used by `/pumpkin link`).
    pub fn peek_completed(&self, player_salt: &[u8; 32]) -> Vec<PlayerTelemetryInfo> {
        let guard = self
            .completed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.iter().map(|s| s.to_wire(player_salt)).collect()
    }

    /// Drains all buffered completed sessions and finalizes any still-connected players on shutdown.
    pub fn drain_for_shutdown(
        &self,
        server: &Server,
        player_salt: &[u8; 32],
    ) -> Vec<PlayerTelemetryInfo> {
        let (_, mut sessions) = self.take_completed(player_salt);
        let mut active = std::mem::take(
            &mut *self
                .active_joins
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        server.for_each_player(|player| {
            if sessions.len() >= MAX_COMPLETED_SESSIONS_PER_HEARTBEAT {
                tracing::debug!("Telemetry session cap reached, omitting the remaining sessions");
                return;
            }
            // A player without a recorded join has no known session; skip it instead of
            // reporting a fake zero-length one.
            let Some((join_instant, joined_at)) = active.remove(&player.gameprofile.id) else {
                return;
            };
            let (joined_at, left_at) = (
                joined_at,
                joined_at.saturating_add(join_instant.elapsed().as_secs()),
            );
            let (edition, client_brand, locale, view_distance) = extract_player_client_meta(player);
            sessions.push(PlayerTelemetryInfo {
                id: anonymize_player_uuid(player_salt, player.gameprofile.id),
                edition,
                client_brand,
                locale,
                view_distance,
                joined_at,
                left_at: Some(left_at),
            });
        });

        sessions
    }

    /// Re-queues completed sessions if a heartbeat HTTP transmission failed.
    pub fn restore_completed(&self, mut previous: Vec<RawCompletedSession>) {
        if previous.is_empty() {
            return;
        }
        let mut guard = self
            .completed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remaining_cap = MAX_COMPLETED_SESSIONS_PER_HEARTBEAT.saturating_sub(guard.len());
        previous.truncate(remaining_cap);
        if !previous.is_empty() {
            previous.append(&mut *guard);
            *guard = previous;
        }
    }
}

/// The telemetry heartbeat payload transmitted to the backend.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HeartbeatPayload {
    /// The server software implementation type (e.g. "pumpkin").
    pub server_type: String,
    /// The Pumpkin server software version.
    pub server_version: String,
    /// The Minecraft game version string.
    pub minecraft_version: String,
    /// The network protocol version number.
    pub protocol_version: i32,
    /// Current number of connected players.
    pub online_players: usize,
    /// Capacity reported to the backend. With both listeners enabled the larger of their
    /// `max_players` values is used; with a single listener that listener's value is used.
    pub max_players: u32,
    /// true only when every enabled listener enforces online mode authentication.
    /// Per-listener modes are not represented; a single all-listeners summary is reported.
    pub online_mode: bool,
    /// Host operating system name.
    pub os: String,
    /// Host CPU architecture.
    pub arch: String,
    /// Number of available CPU cores.
    pub cpu_cores: usize,
    /// CPU model name or brand.
    pub cpu_model: String,
    /// Resident memory of the server process in MiB.
    pub ram_allocated_mb: u64,
    /// Total physical RAM available to the system in MiB.
    pub total_ram_mb: u64,
    /// List of active plugins.
    pub plugins: Vec<PluginInfo>,
    /// Anonymous player sessions (`left_at: None` for currently online, `left_at: Some(...)` for disconnected).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub players: Vec<PlayerTelemetryInfo>,
    /// True when more than [`MAX_TELEMETRY_PLAYERS`] were online, so some online entries
    /// were omitted from `players`; `online_players` still reports the full count.
    #[serde(default)]
    pub players_truncated: bool,
    /// Whether the server has opted into public directory listing.
    pub is_public: bool,
    /// Public name of the server if opted into public directory listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_name: Option<String>,
}

/// The response payload received from the telemetry backend.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HeartbeatResponse {
    /// Response status ("ok", "throttled", etc.).
    pub status: String,
    /// Status or error message from the backend.
    #[serde(default)]
    pub message: String,
    /// Confirmed server public key acknowledged by the backend.
    #[serde(default)]
    pub server_public_key: String,
    /// Recommended seconds to wait before the next heartbeat.
    #[serde(default)]
    pub next_ping_seconds: u32,
}

/// Computes the signed data byte layout: `[timestamp_ascii_bytes] + [b'.'] + [raw_json_body_bytes]`.
#[must_use]
pub fn compute_signed_data(timestamp_str: &str, body_bytes: &[u8]) -> Vec<u8> {
    let mut signed_data = Vec::with_capacity(timestamp_str.len() + 1 + body_bytes.len());
    signed_data.extend_from_slice(timestamp_str.as_bytes());
    signed_data.push(b'.');
    signed_data.extend_from_slice(body_bytes);
    signed_data
}

/// Generates a fresh random replay-protection nonce as 32 lowercase hex characters.
fn generate_nonce() -> String {
    hex::encode(rand::random::<[u8; 16]>())
}

/// Serializes a telemetry payload and embeds a fresh nonce in the resulting signed body.
///
/// The nonce goes into the JSON body so it is covered by the Ed25519 signature: a captured
/// request cannot be replayed with a swapped nonce without invalidating the signature.
fn serialize_signed_body<T: Serialize>(payload: &T, nonce: &str) -> Vec<u8> {
    #[allow(clippy::expect_used)]
    let mut value = serde_json::to_value(payload).expect("Serialization failed");
    if let serde_json::Value::Object(map) = &mut value {
        map.insert(
            NONCE_FIELD.to_string(),
            serde_json::Value::String(nonce.to_string()),
        );
    }
    #[allow(clippy::expect_used)]
    serde_json::to_vec(&value).expect("Serialization failed")
}

/// Signs telemetry message data using an Ed25519 signing key and returns `(public_key_hex, signature_hex)`.
#[must_use]
pub fn sign_telemetry_payload(
    signing_key: &SigningKey,
    timestamp_str: &str,
    body_bytes: &[u8],
) -> (String, String) {
    let signed_data = compute_signed_data(timestamp_str, body_bytes);
    let signature = signing_key.sign(&signed_data);
    let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let sig_hex = hex::encode(signature.to_bytes());
    (pubkey_hex, sig_hex)
}

/// Errors that can occur during telemetry request signature verification.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TelemetryVerificationError {
    #[error("Invalid timestamp string format")]
    InvalidTimestamp,
    #[error(
        "Timestamp drift exceeded: drift was {drift}s (max allowed is {MAX_CLOCK_DRIFT_SECS}s)"
    )]
    ClockDriftExceeded { drift: u64 },
    #[error("Invalid public key hex encoding")]
    InvalidPublicKeyHex,
    #[error("Invalid public key bytes: {0}")]
    InvalidPublicKey(String),
    #[error("Invalid signature hex encoding")]
    InvalidSignatureHex,
    #[error("Invalid signature bytes: {0}")]
    InvalidSignature(String),
    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),
    #[error("Replay protection: signed request body is missing a valid nonce")]
    InvalidNonce,
    #[error("Replay detected: this request nonce has already been seen")]
    ReplayDetected,
}

/// Verifies a signed telemetry request against an Ed25519 public key and timestamp.
///
/// Ensures clock drift between `current_time_secs` and `timestamp_str` does not exceed `±300` seconds.
pub fn verify_telemetry_request(
    pubkey_hex: &str,
    sig_hex: &str,
    timestamp_str: &str,
    body_bytes: &[u8],
    current_time_secs: u64,
) -> Result<(), TelemetryVerificationError> {
    let ts: u64 = timestamp_str
        .parse()
        .map_err(|_| TelemetryVerificationError::InvalidTimestamp)?;

    let drift = current_time_secs.abs_diff(ts);
    if drift > MAX_CLOCK_DRIFT_SECS {
        return Err(TelemetryVerificationError::ClockDriftExceeded { drift });
    }

    let pubkey_bytes =
        hex::decode(pubkey_hex).map_err(|_| TelemetryVerificationError::InvalidPublicKeyHex)?;
    let vk = VerifyingKey::try_from(pubkey_bytes.as_slice())
        .map_err(|e| TelemetryVerificationError::InvalidPublicKey(e.to_string()))?;

    let sig_bytes =
        hex::decode(sig_hex).map_err(|_| TelemetryVerificationError::InvalidSignatureHex)?;
    let sig = Signature::from_slice(&sig_bytes)
        .map_err(|e| TelemetryVerificationError::InvalidSignature(e.to_string()))?;

    let signed_data = compute_signed_data(timestamp_str, body_bytes);
    vk.verify(&signed_data, &sig)
        .map_err(|e| TelemetryVerificationError::VerificationFailed(e.to_string()))?;

    Ok(())
}

/// Time-bounded cache of nonces already accepted by a telemetry verifier.
///
/// Entries older than [`NONCE_CACHE_TTL_SECS`] are pruned on each check, so the cache stays
/// bounded: a nonce whose request timestamp is that old can no longer pass the drift window.
#[derive(Debug, Default)]
pub struct NonceCache {
    seen: Mutex<FxHashMap<String, u64>>,
}

impl NonceCache {
    /// Creates an empty replay-protection nonce cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a nonce and returns `true` when it was unseen, or `false` on a replay.
    pub fn check_and_insert(&self, nonce: &str, current_time_secs: u64) -> bool {
        let mut seen = self
            .seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        seen.retain(|_, seen_at| {
            current_time_secs.saturating_sub(*seen_at) <= NONCE_CACHE_TTL_SECS
        });
        if seen.contains_key(nonce) {
            return false;
        }
        seen.insert(nonce.to_string(), current_time_secs);
        true
    }
}

/// Verifies a signed telemetry request and rejects replays via its signed-body nonce.
///
/// The nonce is read from the signature-verified body, so an attacker cannot swap in a fresh
/// nonce to bypass the seen-nonce cache. Signature and drift checks run before the cache.
pub fn verify_telemetry_request_with_replay_protection(
    cache: &NonceCache,
    pubkey_hex: &str,
    sig_hex: &str,
    timestamp_str: &str,
    body_bytes: &[u8],
    current_time_secs: u64,
) -> Result<(), TelemetryVerificationError> {
    verify_telemetry_request(
        pubkey_hex,
        sig_hex,
        timestamp_str,
        body_bytes,
        current_time_secs,
    )?;
    let body: serde_json::Value =
        serde_json::from_slice(body_bytes).map_err(|_| TelemetryVerificationError::InvalidNonce)?;
    let nonce = body
        .get(NONCE_FIELD)
        .and_then(serde_json::Value::as_str)
        .ok_or(TelemetryVerificationError::InvalidNonce)?;
    if cache.check_and_insert(nonce, current_time_secs) {
        Ok(())
    } else {
        Err(TelemetryVerificationError::ReplayDetected)
    }
}

/// Derives the shutdown endpoint from an endpoint or its heartbeat-stripped prefix.
///
/// Endpoints that already carry an API-version path keep that path on shutdown;
/// bare hosts get the canonical `/v1/shutdown` suffix.
fn shutdown_url_for(prefix: &str) -> String {
    if prefix.ends_with("/v1")
        || prefix.ends_with("/api/v1/telemetry")
        || prefix.ends_with("/api/v1/rest/telemetry")
    {
        format!("{prefix}/shutdown")
    } else {
        format!("{prefix}/v1/shutdown")
    }
}

fn resolve_telemetry_urls(endpoint: &str) -> (String, String) {
    let trimmed = endpoint.trim_end_matches('/');
    trimmed.strip_suffix("/heartbeat").map_or_else(
        || (trimmed.to_string(), shutdown_url_for(trimmed)),
        |prefix| {
            let shutdown_url = shutdown_url_for(prefix);
            let base_url = prefix
                .strip_suffix("/api/v1/rest/telemetry")
                .or_else(|| prefix.strip_suffix("/api/v1/telemetry"))
                .or_else(|| prefix.strip_suffix("/v1"))
                .unwrap_or(prefix)
                .to_string();
            (base_url, shutdown_url)
        },
    )
}

/// Client for communicating with the Pumpkin telemetry backend.
pub struct TelemetryClient {
    pub signing_key: SigningKey,
    pub http_client: reqwest::Client,
    pub endpoint: String,
    pub api_base_url: String,
    pub shutdown_endpoint: String,
}

impl TelemetryClient {
    /// Creates a new `TelemetryClient` instance.
    #[must_use]
    pub fn new(signing_key: SigningKey, http_client: reqwest::Client, endpoint: String) -> Self {
        let (api_base_url, shutdown_endpoint) = resolve_telemetry_urls(&endpoint);

        Self {
            signing_key,
            http_client,
            endpoint,
            api_base_url,
            shutdown_endpoint,
        }
    }

    /// Returns the 64-character lowercase hexadecimal representation of the client's Ed25519 public key.
    #[must_use]
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing_key.verifying_key().to_bytes())
    }

    /// Loads an existing Ed25519 private key/seed from the given path or generates and persists a new one.
    pub fn load_or_generate_key(key_path: &Path) -> Result<SigningKey, std::io::Error> {
        if key_path.exists() {
            match std::fs::read(key_path) {
                Ok(bytes) => {
                    // The identity key is a secret; tighten permissions inherited from older versions.
                    restrict_key_permissions(key_path);
                    if bytes.len() == 32 {
                        let mut seed = [0u8; 32];
                        seed.copy_from_slice(&bytes);
                        return Ok(SigningKey::from_bytes(&seed));
                    }
                    tracing::warn!(
                        "Failed to read valid 32-byte identity key from {}, generating a new one",
                        key_path.display()
                    );
                }
                Err(err) => tracing::warn!(
                    "Failed to read identity key from {} ({err}), generating a new one",
                    key_path.display()
                ),
            }
        }

        // Generate new key
        let seed: [u8; 32] = rand::random();
        let key = SigningKey::from_bytes(&seed);

        if let Some(parent) = key_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        write_private_key(key_path, &seed)?;

        Ok(key)
    }

    /// Sends a signed heartbeat request to the telemetry backend.
    pub async fn send_heartbeat(
        &self,
        payload: &HeartbeatPayload,
    ) -> Result<HeartbeatResponse, reqwest::Error> {
        let nonce = generate_nonce();
        let body_bytes = serialize_signed_body(payload, &nonce);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let timestamp_str = timestamp.to_string();

        let (pubkey_hex, sig_hex) =
            sign_telemetry_payload(&self.signing_key, &timestamp_str, &body_bytes);

        let response = self
            .http_client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header(HEADER_PUBLIC_KEY, pubkey_hex)
            .header(HEADER_SIGNATURE, sig_hex)
            .header(HEADER_TIMESTAMP, timestamp_str)
            .header(HEADER_NONCE, nonce)
            .body(body_bytes)
            .send()
            .await?
            .error_for_status()?
            .json::<HeartbeatResponse>()
            .await?;

        Ok(response)
    }

    /// Sends a signed server shutdown notice (with any flushed sessions) to the telemetry backend.
    pub async fn send_shutdown(
        &self,
        reason: Option<&str>,
        players: Vec<PlayerTelemetryInfo>,
    ) -> Result<(), reqwest::Error> {
        let payload = ShutdownPayload {
            reason: reason.unwrap_or("server shutdown").to_string(),
            players,
        };
        let nonce = generate_nonce();
        let body_bytes = serialize_signed_body(&payload, &nonce);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let timestamp_str = timestamp.to_string();

        let (pubkey_hex, sig_hex) =
            sign_telemetry_payload(&self.signing_key, &timestamp_str, &body_bytes);

        self.http_client
            .post(&self.shutdown_endpoint)
            .header("Content-Type", "application/json")
            .header(HEADER_PUBLIC_KEY, pubkey_hex)
            .header(HEADER_SIGNATURE, sig_hex)
            .header(HEADER_TIMESTAMP, timestamp_str)
            .header(HEADER_NONCE, nonce)
            .body(body_bytes)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

/// Restricts the identity key file to owner read/write on Unix.
fn restrict_key_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(err) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
            tracing::warn!(
                "Failed to restrict telemetry identity key permissions at {}: {err}",
                path.display()
            );
        }
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// Persists the identity key seed, creating the file with owner-only permissions.
fn write_private_key(path: &Path, seed: &[u8; 32]) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(seed)?;
        restrict_key_permissions(path);
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, seed)
    }
}

/// Loads an existing identity key without generating or writing one.
///
/// An unreadable or corrupt file is treated as a missing candidate so the caller can try the
/// next configured location; the file itself is left untouched.
fn load_existing_identity_key(path: &Path) -> Option<SigningKey> {
    if !path.exists() {
        return None;
    }
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::warn!(
                "Existing telemetry identity key at {} is unreadable: {err}",
                path.display()
            );
            return None;
        }
    };
    if bytes.len() != 32 {
        tracing::warn!(
            "Existing telemetry identity key at {} is not a 32-byte seed, ignoring it",
            path.display()
        );
        return None;
    }
    // The identity key is a secret; tighten permissions inherited from older versions.
    restrict_key_permissions(path);
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes);
    Some(SigningKey::from_bytes(&seed))
}

/// Resolves the server identity key by checking default locations or generating a new one.
///
/// While the heartbeat client is running, its live in-memory key is reused so one-off
/// telemetry sends sign with the identity the backend registered.
#[must_use]
pub fn resolve_identity_key() -> SigningKey {
    choose_signing_key(LIVE_SIGNING_KEY.get(), resolve_identity_key_from_disk)
}

fn resolve_identity_key_from_disk() -> SigningKey {
    let primary = Path::new(IDENTITY_KEY_PATH);
    let data_dir = Path::new("data");
    let existing_candidates = [
        primary.to_path_buf(),
        Path::new(FALLBACK_IDENTITY_KEY_PATH).to_path_buf(),
        data_dir.join(IDENTITY_KEY_PATH),
        data_dir.join(FALLBACK_IDENTITY_KEY_PATH),
    ];
    for candidate in &existing_candidates {
        if let Some(key) = load_existing_identity_key(candidate) {
            return key;
        }
    }

    let data_dir_key = Path::new("data").join(IDENTITY_KEY_PATH);
    if Path::new("data").is_dir()
        && let Ok(key) = TelemetryClient::load_or_generate_key(&data_dir_key)
    {
        return key;
    }

    TelemetryClient::load_or_generate_key(primary).unwrap_or_else(|err| {
        tracing::warn!(
            "Failed to persist telemetry identity key at {}: {err}, using ephemeral key",
            primary.display()
        );
        let seed: [u8; 32] = rand::random();
        SigningKey::from_bytes(&seed)
    })
}

/// Helper function to load or create an identity key at a specified path.
#[must_use]
pub fn get_or_create_identity_key(path: &Path) -> SigningKey {
    TelemetryClient::load_or_generate_key(path).unwrap_or_else(|err| {
        tracing::warn!(
            "Failed to save identity key to {}: {err}, using ephemeral key",
            path.display()
        );
        let seed: [u8; 32] = rand::random();
        SigningKey::from_bytes(&seed)
    })
}

/// Derives a domain-separated 32-byte secret salt from the server's private identity key
/// for irreversibly anonymizing player UUIDs per server.
#[must_use]
pub fn derive_player_telemetry_salt(signing_key: &SigningKey) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"pumpkin-telemetry-player-salt-v1:");
    hasher.update(signing_key.to_bytes());
    hasher.finalize().into()
}

/// Computes a 32-character lowercase hex anonymous identifier for a player on this server.
#[must_use]
pub fn anonymize_player_uuid(player_salt: &[u8; 32], player_uuid: uuid::Uuid) -> String {
    let mut hasher = Sha256::new();
    hasher.update(player_salt);
    hasher.update(player_uuid.as_bytes());
    let digest = hasher.finalize();
    hex::encode(&digest[..16])
}

/// Per-server player salt of the running heartbeat client, set once by `start_telemetry_with_config`.
static LIVE_PLAYER_SALT: OnceLock<[u8; 32]> = OnceLock::new();

/// Live signing key of the running heartbeat client, set once at startup.
/// One-off sends reuse it so they sign with the identity the backend registered.
static LIVE_SIGNING_KEY: OnceLock<SigningKey> = OnceLock::new();

/// Prefers the live in-memory signing key over re-reading the identity key from disk.
fn choose_signing_key(
    live: Option<&SigningKey>,
    from_disk: impl FnOnce() -> SigningKey,
) -> SigningKey {
    live.cloned().unwrap_or_else(from_disk)
}

/// Builds the telemetry heartbeat payload from current server and runtime state.
#[must_use]
pub fn build_heartbeat_payload(
    server: &Server,
    telemetry_config: &TelemetryConfig,
) -> HeartbeatPayload {
    // Resolved once per process: this runs on the tick thread, so the key file is not re-read per call.
    static FALLBACK_PLAYER_SALT: OnceLock<[u8; 32]> = OnceLock::new();
    let salt = LIVE_PLAYER_SALT.get().copied().unwrap_or_else(|| {
        *FALLBACK_PLAYER_SALT.get_or_init(|| derive_player_telemetry_salt(&resolve_identity_key()))
    });
    build_heartbeat_payload_with_salt(server, telemetry_config, &salt)
}

struct StaticHostInfo {
    os: String,
    arch: String,
    cpu_cores: usize,
    cpu_model: String,
    total_ram_mb: u64,
}

fn static_host_info() -> &'static StaticHostInfo {
    static HOST_INFO: OnceLock<StaticHostInfo> = OnceLock::new();
    HOST_INFO.get_or_init(|| {
        let os = sysinfo::System::long_os_version().unwrap_or_else(|_| {
            let name = sysinfo::System::name().unwrap_or_else(|_| std::env::consts::OS.to_string());
            sysinfo::System::os_version().map_or(name.clone(), |ver| format!("{name} {ver}"))
        });
        let arch = std::env::consts::ARCH.to_string();
        let cpu_cores = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);

        let (total_ram_mb, cpu_model) = sysinfo::System::new_all().map_or_else(
            |_| (0, "Unknown".to_string()),
            |sys| {
                let total_ram = sys.total_memory() / (1024 * 1024);
                let cpu_model = sys.cpus().first().map_or_else(
                    || "Unknown".to_string(),
                    |c| {
                        let brand = c.brand().trim();
                        if brand.is_empty() {
                            "Unknown".to_string()
                        } else {
                            brand.to_string()
                        }
                    },
                );
                (total_ram, cpu_model)
            },
        );

        StaticHostInfo {
            os,
            arch,
            cpu_cores,
            cpu_model,
            total_ram_mb,
        }
    })
}

/// Resident memory of this server process in MiB (reported as `ram_allocated_mb`).
fn read_process_ram_mb() -> u64 {
    static PROCESS_SYSTEM: Mutex<Option<sysinfo::System>> = Mutex::new(None);
    let Ok(mut guard) = PROCESS_SYSTEM.lock() else {
        return 0;
    };
    if guard.is_none() {
        *guard = sysinfo::System::new().ok();
    }
    let Some(sys) = guard.as_mut() else {
        return 0;
    };
    let pid = sysinfo::Pid::from_u32(std::process::id());
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
    sys.process(pid)
        .map_or(0, |process| process.memory() / (1024 * 1024))
}

fn extract_player_client_meta(
    player: &Player,
) -> (PlayerEdition, Option<String>, Option<String>, Option<u8>) {
    let (edition, client_brand) = match player.client.as_ref() {
        ClientPlatform::Java(java_client) => {
            let raw_brand = java_client
                .brand
                .load()
                .as_ref()
                .clone()
                .map(|b| {
                    b.chars()
                        .filter(|c| !c.is_control())
                        .take(64)
                        .collect::<String>()
                        .trim()
                        .to_string()
                })
                .filter(|b| !b.is_empty())
                .unwrap_or_else(|| "vanilla".to_string());
            (PlayerEdition::Java, Some(raw_brand))
        }
        ClientPlatform::Bedrock(_) => (PlayerEdition::Bedrock, None),
    };

    let player_cfg = player.config.load();
    let locale = {
        let norm: String = player_cfg
            .locale
            .trim()
            .to_ascii_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .take(16)
            .collect();
        if norm.is_empty() { None } else { Some(norm) }
    };
    let view_distance = Some(player_cfg.view_distance.get());
    (edition, client_brand, locale, view_distance)
}

fn collect_players_telemetry(
    server: &Server,
    player_salt: &[u8; 32],
    mut players: Vec<PlayerTelemetryInfo>,
) -> (Vec<PlayerTelemetryInfo>, bool) {
    let mut online_count = 0usize;
    let mut truncated = false;
    server.for_each_player(|player| {
        if online_count >= MAX_TELEMETRY_PLAYERS {
            truncated = true;
            tracing::debug!(
                "Telemetry player list capped at {MAX_TELEMETRY_PLAYERS}, omitting the rest"
            );
            return;
        }
        online_count += 1;
        let id = anonymize_player_uuid(player_salt, player.gameprofile.id);
        let (edition, client_brand, locale, view_distance) = extract_player_client_meta(player);
        let joined_at = server.telemetry_sessions.joined_at(player.gameprofile.id);

        players.push(PlayerTelemetryInfo {
            id,
            edition,
            client_brand,
            locale,
            view_distance,
            joined_at,
            left_at: None,
        });
    });
    (players, truncated)
}

/// Builds the telemetry heartbeat payload using a precomputed per-server player salt.
#[must_use]
pub fn build_heartbeat_payload_with_salt(
    server: &Server,
    telemetry_config: &TelemetryConfig,
    player_salt: &[u8; 32],
) -> HeartbeatPayload {
    let completed_sessions = server.telemetry_sessions.peek_completed(player_salt);
    build_heartbeat_payload_inner(server, telemetry_config, player_salt, completed_sessions)
}

fn build_heartbeat_payload_inner(
    server: &Server,
    telemetry_config: &TelemetryConfig,
    player_salt: &[u8; 32],
    completed_sessions: Vec<PlayerTelemetryInfo>,
) -> HeartbeatPayload {
    let host = static_host_info();
    let ram_allocated_mb = read_process_ram_mb();
    let plugins = server.plugin_manager.active_plugins_telemetry();

    let java_net = &server.advanced_config.networking.java;
    let bedrock_net = &server.advanced_config.networking.bedrock;
    let (max_players, online_mode) = match (java_net.enabled, bedrock_net.enabled) {
        (true, true) => (
            java_net.max_players.max(bedrock_net.max_players),
            java_net.online_mode && bedrock_net.online_mode,
        ),
        (_, false) => (java_net.max_players, java_net.online_mode),
        (false, true) => (bedrock_net.max_players, bedrock_net.online_mode),
    };

    let (players, players_truncated) =
        collect_players_telemetry(server, player_salt, completed_sessions);

    let is_public = telemetry_config.public;
    let public_name = if is_public {
        telemetry_config.server_name.clone()
    } else {
        None
    };

    HeartbeatPayload {
        server_type: "pumpkin".to_string(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        minecraft_version: pumpkin_data::packet::CURRENT_MC_VERSION.to_string(),
        protocol_version: pumpkin_data::packet::CURRENT_MC_VERSION.protocol_version(),
        online_players: server.get_player_count(),
        players_truncated,
        max_players,
        online_mode,
        os: host.os.clone(),
        arch: host.arch.clone(),
        cpu_cores: host.cpu_cores,
        cpu_model: host.cpu_model.clone(),
        ram_allocated_mb,
        total_ram_mb: host.total_ram_mb,
        plugins,
        players,
        is_public,
        public_name,
    }
}

/// Returns true only when the backend accepted the heartbeat, so the drained sessions can be
/// dropped and the recommended interval is authoritative.
fn backend_accepted_heartbeat(status: &str) -> bool {
    status == "ok"
}

/// Picks the heartbeat interval, honouring the delay recommended by the telemetry backend.
///
/// A zero recommendation means the backend did not ask for a change, and values below the
/// 60 second floor are clamped like the configured interval.
fn resolve_next_interval_secs(current_secs: u64, recommended_secs: u32) -> u64 {
    if recommended_secs == 0 {
        current_secs
    } else {
        u64::from(recommended_secs).max(MIN_HEARTBEAT_INTERVAL_SECS)
    }
}

/// Returns true when the configured endpoint is an http(s) URL with a host.
fn telemetry_endpoint_is_valid(endpoint: &str) -> bool {
    reqwest::Url::parse(endpoint)
        .is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
}

/// Starts the background telemetry reporting task using the server's telemetry configuration.
pub fn start_telemetry(server: Arc<Server>) {
    let config = server.telemetry_config.clone();
    start_telemetry_with_config(server, &config);
}

/// Starts the background telemetry reporting task with a specific configuration.
#[allow(clippy::needless_pass_by_value)]
/// Applies the backend's recommended heartbeat interval, resetting the timer when it changes.
fn apply_heartbeat_interval(
    current_interval_secs: &mut u64,
    interval: &mut tokio::time::Interval,
    recommended_secs: u32,
) {
    let next_interval_secs = resolve_next_interval_secs(*current_interval_secs, recommended_secs);
    if next_interval_secs != *current_interval_secs {
        *current_interval_secs = next_interval_secs;
        interval.reset_after(Duration::from_secs(next_interval_secs));
        tracing::debug!("Telemetry backend requested a {next_interval_secs}s heartbeat interval");
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "public API takes the shared server handle by value"
)]
pub fn start_telemetry_with_config(server: Arc<Server>, config: &TelemetryConfig) {
    if !config.enabled {
        tracing::trace!("Telemetry is disabled in configuration.");
        return;
    }

    if !telemetry_endpoint_is_valid(&config.endpoint) {
        tracing::warn!(
            "Telemetry disabled: configured endpoint is not a valid http(s) URL: {}",
            config.endpoint
        );
        return;
    }

    tracing::info!(
        "Anonymous server telemetry is enabled. Sending periodic heartbeats to {}",
        config.endpoint
    );

    let user_agent = format!("Pumpkin-Server/{}", env!("CARGO_PKG_VERSION"));
    let http_client = match pumpkin_auth::client_builder()
        .timeout(Duration::from_secs(10))
        .user_agent(&user_agent)
        .build()
    {
        Ok(c) => c,
        Err(err) => {
            tracing::debug!("Failed to create telemetry HTTP client: {err}");
            return;
        }
    };

    let signing_key = resolve_identity_key();
    let _ = LIVE_SIGNING_KEY.set(signing_key.clone());
    let player_salt = derive_player_telemetry_salt(&signing_key);
    // Share the running client's salt so `/pumpkin` reports the same anonymous ids.
    let _ = LIVE_PLAYER_SALT.set(player_salt);
    let telemetry_client = Arc::new(TelemetryClient::new(
        signing_key,
        http_client,
        config.endpoint.clone(),
    ));

    let interval_secs = config.interval_secs.max(MIN_HEARTBEAT_INTERVAL_SECS);
    let config = config.clone();
    let server_task = server.clone();

    server.spawn_task(async move {
        let mut current_interval_secs = interval_secs;
        let mut interval = tokio::time::interval(Duration::from_secs(current_interval_secs));
        // After a stall (suspend, GC pause) send one heartbeat, not a burst of
        // back-to-back ones carrying the same drained sessions.
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = interval.tick() => {}
                () = server_task.stop_token.cancelled() => {
                    break;
                }
            }

            let (raw_completed, wire_completed) =
                server_task.telemetry_sessions.take_completed(&player_salt);
            let payload =
                build_heartbeat_payload_inner(&server_task, &config, &player_salt, wire_completed);

            match telemetry_client.send_heartbeat(&payload).await {
                Ok(resp) => {
                    if backend_accepted_heartbeat(&resp.status) {
                        tracing::trace!(
                            "Telemetry heartbeat successfully transmitted (status: {}, public_key: {})",
                            resp.status,
                            resp.server_public_key
                        );
                        apply_heartbeat_interval(
                            &mut current_interval_secs,
                            &mut interval,
                            resp.next_ping_seconds,
                        );
                    } else {
                        // The backend did not accept the sessions, so keep them for the next heartbeat.
                        // A rejected heartbeat's suggested interval is not authoritative.
                        server_task
                            .telemetry_sessions
                            .restore_completed(raw_completed);
                        tracing::debug!(
                            "Telemetry backend did not accept heartbeat (status: {}), keeping sessions for the next one",
                            resp.status
                        );
                    }
                }
                Err(err) => {
                    server_task
                        .telemetry_sessions
                        .restore_completed(raw_completed);
                    tracing::debug!("Telemetry heartbeat failed to send: {err}");
                }
            }
        }

        // Flush any remaining player sessions and trigger shutdown telemetry on termination
        let flushed_players = server_task
            .telemetry_sessions
            .drain_for_shutdown(&server_task, &player_salt);
        if let Err(err) = telemetry_client
            .send_shutdown(Some("server shutdown"), flushed_players)
            .await
        {
            tracing::debug!("Telemetry shutdown message failed to send: {err}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::extract::Request;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use axum::routing::post;

    #[cfg(unix)]
    fn assert_loose_key_is_tightened(key_path: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(key_path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let _ = TelemetryClient::load_or_generate_key(key_path).unwrap();
        assert_eq!(
            std::fs::metadata(key_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn identity_key_generation_and_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join(".pumpkin/identity.key");

        assert!(!key_path.exists());
        let key1 = TelemetryClient::load_or_generate_key(&key_path).unwrap();
        assert!(key_path.exists());

        // Check key file has 32 bytes
        let file_bytes = std::fs::read(&key_path).unwrap();
        assert_eq!(file_bytes.len(), 32);

        #[cfg(unix)]
        assert_eq!(
            std::os::unix::fs::PermissionsExt::mode(
                &std::fs::metadata(&key_path).unwrap().permissions()
            ) & 0o777,
            0o600
        );

        // Reload existing key
        let key2 = TelemetryClient::load_or_generate_key(&key_path).unwrap();
        assert_eq!(
            key1.verifying_key().to_bytes(),
            key2.verifying_key().to_bytes()
        );

        // A pre-existing world-readable key file is tightened when loaded.
        std::fs::write(&key_path, key1.to_bytes()).unwrap();
        #[cfg(unix)]
        assert_loose_key_is_tightened(&key_path);

        // Corrupted file test (less than 32 bytes)
        std::fs::write(&key_path, b"corrupted-key-data").unwrap();
        let key3 = TelemetryClient::load_or_generate_key(&key_path).unwrap();
        assert_ne!(
            key1.verifying_key().to_bytes(),
            key3.verifying_key().to_bytes()
        );
        let new_file_bytes = std::fs::read(&key_path).unwrap();
        assert_eq!(new_file_bytes.len(), 32);
    }

    #[test]
    fn corrupt_identity_key_is_treated_as_missing_without_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let corrupt = dir.path().join("identity.key");
        let corrupt_bytes = b"not-a-32-byte-seed";
        std::fs::write(&corrupt, corrupt_bytes).unwrap();

        assert!(load_existing_identity_key(&corrupt).is_none());
        assert_eq!(std::fs::read(&corrupt).unwrap(), corrupt_bytes.to_vec());

        let valid = dir.path().join("valid.key");
        let seed = [11u8; 32];
        std::fs::write(&valid, seed).unwrap();
        let loaded = load_existing_identity_key(&valid).unwrap();
        assert_eq!(loaded.to_bytes(), seed);
    }

    #[test]
    fn heartbeat_payload_serialization() {
        let payload = HeartbeatPayload {
            server_type: "pumpkin".to_string(),
            server_version: "0.3.2".to_string(),
            minecraft_version: "1.21.4".to_string(),
            protocol_version: 769,
            online_players: 42,
            players_truncated: false,
            max_players: 100,
            online_mode: true,
            os: "Ubuntu 24.04 LTS".to_string(),
            arch: "x86_64".to_string(),
            cpu_cores: 8,
            cpu_model: "AMD Ryzen 7 5800X 8-Core Processor".to_string(),
            ram_allocated_mb: 8192,
            total_ram_mb: 16384,
            plugins: vec![
                PluginInfo {
                    name: "Essential Admin Tools".to_string(),
                    version: "1.0.0".to_string(),
                    language: Some("rust".to_string()),
                },
                PluginInfo {
                    name: "PumpkinAuth".to_string(),
                    version: "1.2.0".to_string(),
                    language: Some("go".to_string()),
                },
            ],
            players: vec![
                PlayerTelemetryInfo {
                    id: "0123456789abcdef0123456789abcdef".to_string(),
                    edition: PlayerEdition::Java,
                    client_brand: Some("fabric".to_string()),
                    locale: Some("de_de".to_string()),
                    view_distance: Some(12),
                    joined_at: 1_763_150_000,
                    left_at: None,
                },
                PlayerTelemetryInfo {
                    id: "fedcba9876543210fedcba9876543210".to_string(),
                    edition: PlayerEdition::Bedrock,
                    client_brand: None,
                    locale: Some("en_us".to_string()),
                    view_distance: Some(10),
                    joined_at: 1_763_149_500,
                    left_at: Some(1_763_149_545),
                },
            ],
            is_public: true,
            public_name: Some("My High-Performance Pumpkin SMP".to_string()),
        };

        let json_str = serde_json::to_string_pretty(&payload).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Ensure server_id and legacy duration fields are completely absent
        assert!(parsed.get("server_id").is_none());
        assert!(parsed.get("players_data").is_none());
        assert!(parsed.get("completed_sessions").is_none());

        assert_eq!(parsed["server_type"], "pumpkin");
        assert_eq!(parsed["server_version"], "0.3.2");
        assert_eq!(parsed["minecraft_version"], "1.21.4");
        assert_eq!(parsed["protocol_version"], 769);
        assert_eq!(parsed["online_players"], 42);
        assert_eq!(parsed["players_truncated"], false);
        assert_eq!(parsed["max_players"], 100);
        assert_eq!(parsed["online_mode"], true);
        assert_eq!(parsed["os"], "Ubuntu 24.04 LTS");
        assert_eq!(parsed["arch"], "x86_64");
        assert_eq!(parsed["cpu_cores"], 8);
        assert_eq!(parsed["cpu_model"], "AMD Ryzen 7 5800X 8-Core Processor");
        assert_eq!(parsed["ram_allocated_mb"], 8192);
        assert_eq!(parsed["total_ram_mb"], 16384);
        assert_eq!(parsed["is_public"], true);
        assert_eq!(parsed["public_name"], "My High-Performance Pumpkin SMP");
        assert_eq!(parsed["plugins"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["plugins"][0]["language"], "rust");
        assert_eq!(parsed["plugins"][1]["language"], "go");
        assert_eq!(parsed["players"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["players"][0]["edition"], "java");
        assert_eq!(parsed["players"][0]["client_brand"], "fabric");
        assert_eq!(parsed["players"][0]["locale"], "de_de");
        assert_eq!(parsed["players"][0]["view_distance"], 12);
        assert_eq!(parsed["players"][0]["joined_at"], 1_763_150_000);
        assert!(parsed["players"][0].get("left_at").is_none());
        assert_eq!(parsed["players"][1]["edition"], "bedrock");
        assert!(parsed["players"][1].get("client_brand").is_none());
        assert_eq!(parsed["players"][1]["joined_at"], 1_763_149_500);
        assert_eq!(parsed["players"][1]["left_at"], 1_763_149_545);
    }

    #[tokio::test]
    async fn shutdown_request_failures_are_reported() {
        let _ = rustls::crypto::ring::default_provider().install_default();

        let app = Router::new().route(
            "/v1/shutdown",
            post(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "nope").into_response() }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let endpoint = format!("http://127.0.0.1:{port}/v1/heartbeat");
        let http_client = pumpkin_auth::client_builder().build().unwrap();
        let client =
            TelemetryClient::new(SigningKey::from_bytes(&[3u8; 32]), http_client, endpoint);

        assert!(client.send_shutdown(None, Vec::new()).await.is_err());
    }

    #[test]
    fn heartbeat_response_deserialization() {
        let json_data = r#"{
            "status": "ok",
            "message": "Heartbeat accepted",
            "server_public_key": "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
            "next_ping_seconds": 300
        }"#;

        let resp: HeartbeatResponse = serde_json::from_str(json_data).unwrap();
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.message, "Heartbeat accepted");
        assert_eq!(
            resp.server_public_key,
            "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60"
        );
        assert_eq!(resp.next_ping_seconds, 300);
    }

    #[test]
    fn message_signing_and_verification() {
        let seed = [42u8; 32];
        let signing_key = SigningKey::from_bytes(&seed);
        let timestamp_str = "1757185000";
        let body = br#"{"server_type":"pumpkin","online_players":5}"#;

        // Verify signed data format: "{timestamp}.{body}"
        let signed_data = compute_signed_data(timestamp_str, body);
        let expected_prefix = format!("{timestamp_str}.");
        assert!(signed_data.starts_with(expected_prefix.as_bytes()));
        assert_eq!(&signed_data[expected_prefix.len()..], body);

        // Sign payload
        let (pubkey_hex, sig_hex) = sign_telemetry_payload(&signing_key, timestamp_str, body);
        assert_eq!(pubkey_hex.len(), 64);
        assert_eq!(sig_hex.len(), 128);

        // All hex chars must be lowercase
        assert_eq!(pubkey_hex, pubkey_hex.to_lowercase());
        assert_eq!(sig_hex, sig_hex.to_lowercase());

        // Verify with matching timestamp
        let current_time: u64 = 1757185100; // 100s drift (within 300s)
        let verify_result =
            verify_telemetry_request(&pubkey_hex, &sig_hex, timestamp_str, body, current_time);
        assert!(verify_result.is_ok());

        // Drift exceeded (> 300s)
        let expired_time: u64 = 1757185000 + 301;
        let expired_result =
            verify_telemetry_request(&pubkey_hex, &sig_hex, timestamp_str, body, expired_time);
        assert!(matches!(
            expired_result,
            Err(TelemetryVerificationError::ClockDriftExceeded { .. })
        ));

        // Corrupted signature
        let mut corrupted_sig = sig_hex.clone();
        let flipped_char = if corrupted_sig.starts_with('0') {
            '1'
        } else {
            '0'
        };
        corrupted_sig.replace_range(0..1, &flipped_char.to_string());
        let corrupted_result = verify_telemetry_request(
            &pubkey_hex,
            &corrupted_sig,
            timestamp_str,
            body,
            current_time,
        );
        assert!(matches!(
            corrupted_result,
            Err(TelemetryVerificationError::VerificationFailed(_))
        ));

        // Corrupted body
        let tampered_body = br#"{"server_type":"pumpkin","online_players":6}"#;
        let tampered_result = verify_telemetry_request(
            &pubkey_hex,
            &sig_hex,
            timestamp_str,
            tampered_body,
            current_time,
        );
        assert!(matches!(
            tampered_result,
            Err(TelemetryVerificationError::VerificationFailed(_))
        ));
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn telemetry_client_send_heartbeat_and_shutdown() {
        let _ = rustls::crypto::ring::default_provider().install_default();

        let seed = [7u8; 32];
        let signing_key = SigningKey::from_bytes(&seed);
        let expected_pubkey = hex::encode(signing_key.verifying_key().to_bytes());

        // Set up mock axum server
        let app = Router::new()
            .route(
                "/v1/heartbeat",
                post(move |req: Request| async move {
                    let headers = req.headers().clone();
                    let pubkey = headers.get(HEADER_PUBLIC_KEY).and_then(|v| v.to_str().ok());
                    let signature = headers.get(HEADER_SIGNATURE).and_then(|v| v.to_str().ok());
                    let timestamp = headers.get(HEADER_TIMESTAMP).and_then(|v| v.to_str().ok());

                    let (Some(pubkey), Some(sig), Some(ts)) = (pubkey, signature, timestamp) else {
                        return (StatusCode::UNAUTHORIZED, "Missing required headers")
                            .into_response();
                    };

                    let body = axum::body::to_bytes(req.into_body(), 1024 * 1024)
                        .await
                        .unwrap();

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    if verify_telemetry_request(pubkey, sig, ts, &body, now).is_err() {
                        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
                    }

                    let response = HeartbeatResponse {
                        status: "ok".to_string(),
                        message: "Heartbeat accepted".to_string(),
                        server_public_key: pubkey.to_string(),
                        next_ping_seconds: 300,
                    };
                    let json_bytes = serde_json::to_vec(&response).unwrap();
                    (
                        StatusCode::OK,
                        [(axum::http::header::CONTENT_TYPE, "application/json")],
                        json_bytes,
                    )
                        .into_response()
                }),
            )
            .route(
                "/v1/shutdown",
                post(move |req: Request| async move {
                    let headers = req.headers().clone();
                    let pubkey = headers.get(HEADER_PUBLIC_KEY).and_then(|v| v.to_str().ok());
                    let signature = headers.get(HEADER_SIGNATURE).and_then(|v| v.to_str().ok());
                    let timestamp = headers.get(HEADER_TIMESTAMP).and_then(|v| v.to_str().ok());

                    let (Some(pubkey), Some(sig), Some(ts)) = (pubkey, signature, timestamp) else {
                        return (StatusCode::UNAUTHORIZED, "Missing headers").into_response();
                    };

                    let body = axum::body::to_bytes(req.into_body(), 1024 * 1024)
                        .await
                        .unwrap();

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    if verify_telemetry_request(pubkey, sig, ts, &body, now).is_err() {
                        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
                    }

                    (StatusCode::OK, "Shutdown received").into_response()
                }),
            );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let endpoint = format!("http://127.0.0.1:{port}/v1/heartbeat");
        let http_client = pumpkin_auth::client_builder().build().unwrap();
        let client = TelemetryClient::new(signing_key, http_client, endpoint);

        let payload = HeartbeatPayload {
            server_type: "pumpkin".to_string(),
            server_version: "0.1.0".to_string(),
            minecraft_version: "1.21.4".to_string(),
            protocol_version: 769,
            online_players: 1,
            players_truncated: false,
            max_players: 20,
            online_mode: false,
            os: "Linux".to_string(),
            arch: "x86_64".to_string(),
            cpu_cores: 4,
            cpu_model: "Test CPU".to_string(),
            ram_allocated_mb: 1024,
            total_ram_mb: 2048,
            plugins: vec![],
            players: vec![],
            is_public: false,
            public_name: None,
        };

        // Send valid heartbeat -> 200 OK
        let resp = client.send_heartbeat(&payload).await.unwrap();
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.message, "Heartbeat accepted");
        assert_eq!(resp.server_public_key, expected_pubkey);
        assert_eq!(resp.next_ping_seconds, 300);

        // Send valid shutdown with flushed session -> 200 OK
        let shutdown_res = client
            .send_shutdown(
                Some("maintenance"),
                vec![PlayerTelemetryInfo {
                    id: "0123456789abcdef0123456789abcdef".to_string(),
                    edition: PlayerEdition::Java,
                    client_brand: Some("vanilla".to_string()),
                    locale: Some("en_us".to_string()),
                    view_distance: Some(10),
                    joined_at: 1_763_150_000,
                    left_at: Some(1_763_150_042),
                }],
            )
            .await;
        assert!(shutdown_res.is_ok());

        // Transport failures must surface as errors instead of a silent Ok.
        let unreachable_client = TelemetryClient::new(
            SigningKey::from_bytes(&seed),
            pumpkin_auth::client_builder()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap(),
            "http://127.0.0.1:1/v1/heartbeat".to_string(),
        );
        assert!(
            unreachable_client
                .send_shutdown(None, Vec::new())
                .await
                .is_err()
        );

        // Test invalid signature / unauthorized
        let body_bytes = serde_json::to_vec(&payload).unwrap();
        let timestamp_str = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        let response = pumpkin_auth::client_builder()
            .build()
            .unwrap()
            .post(format!("http://127.0.0.1:{port}/v1/heartbeat"))
            .header("Content-Type", "application/json")
            .header(HEADER_PUBLIC_KEY, &expected_pubkey)
            .header(HEADER_SIGNATURE, "00".repeat(64)) // invalid signature
            .header(HEADER_TIMESTAMP, timestamp_str)
            .body(body_bytes)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn telemetry_client_resolves_canonical_and_legacy_urls() {
        let (base, shutdown) =
            resolve_telemetry_urls("https://telemetry.pumpkinmc.org/v1/heartbeat");
        assert_eq!(base, "https://telemetry.pumpkinmc.org");
        assert_eq!(shutdown, "https://telemetry.pumpkinmc.org/v1/shutdown");

        let (legacy_base, legacy_shutdown) =
            resolve_telemetry_urls("https://market.pumpkinmc.org/api/v1/rest/telemetry/heartbeat");
        assert_eq!(legacy_base, "https://market.pumpkinmc.org");
        assert_eq!(
            legacy_shutdown,
            "https://market.pumpkinmc.org/api/v1/rest/telemetry/shutdown"
        );
    }

    #[test]
    fn telemetry_client_resolves_short_form_shutdown_route() {
        let (base, shutdown) = resolve_telemetry_urls("https://telemetry.example.org/heartbeat");
        assert_eq!(base, "https://telemetry.example.org");
        assert_eq!(shutdown, "https://telemetry.example.org/v1/shutdown");

        let (base, shutdown) = resolve_telemetry_urls("https://telemetry.example.org");
        assert_eq!(base, "https://telemetry.example.org");
        assert_eq!(shutdown, "https://telemetry.example.org/v1/shutdown");
    }

    #[test]
    fn backend_recommended_interval_is_honoured_and_clamped() {
        assert_eq!(resolve_next_interval_secs(300, 0), 300);
        assert_eq!(resolve_next_interval_secs(300, 120), 120);
        assert_eq!(resolve_next_interval_secs(300, 10), 60);
    }

    #[test]
    fn only_ok_heartbeat_status_accepts_drained_sessions() {
        assert!(backend_accepted_heartbeat("ok"));
        assert!(!backend_accepted_heartbeat("throttled"));
        assert!(!backend_accepted_heartbeat("rejected"));
        assert!(!backend_accepted_heartbeat(""));
    }

    #[test]
    fn telemetry_endpoint_validation_requires_http_url_with_host() {
        assert!(telemetry_endpoint_is_valid(
            "https://telemetry.pumpkinmc.org/v1/heartbeat"
        ));
        assert!(telemetry_endpoint_is_valid(
            "http://localhost:5000/v1/heartbeat"
        ));
        assert!(!telemetry_endpoint_is_valid(""));
        assert!(!telemetry_endpoint_is_valid("not a url"));
        assert!(!telemetry_endpoint_is_valid("ftp://example.com/heartbeat"));
    }

    fn raw_session(joined_at: u64) -> RawCompletedSession {
        RawCompletedSession {
            player_uuid: uuid::Uuid::nil(),
            edition: PlayerEdition::Java,
            client_brand: None,
            locale: None,
            view_distance: None,
            joined_at,
            left_at: joined_at + 1,
        }
    }

    #[test]
    fn restored_sessions_go_before_newer_ones_and_respect_the_cap() {
        let tracker = TelemetrySessionTracker::default();
        tracker.restore_completed(vec![raw_session(1), raw_session(2)]);
        tracker.restore_completed(vec![raw_session(3)]);
        let order: Vec<u64> = tracker
            .completed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|s| s.joined_at)
            .collect();
        assert_eq!(order, vec![3, 1, 2]);

        let full: Vec<RawCompletedSession> = (0..=MAX_COMPLETED_SESSIONS_PER_HEARTBEAT as u64)
            .map(raw_session)
            .collect();
        let tracker = TelemetrySessionTracker::default();
        tracker.restore_completed(full);
        tracker.restore_completed(vec![raw_session(9_999)]);
        let len = tracker
            .completed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        assert_eq!(len, MAX_COMPLETED_SESSIONS_PER_HEARTBEAT);
    }

    #[test]
    fn player_uuid_anonymization_is_deterministic_per_server_and_isolated_across_servers() {
        let key_a = SigningKey::from_bytes(&[11u8; 32]);
        let key_b = SigningKey::from_bytes(&[22u8; 32]);
        let salt_a = derive_player_telemetry_salt(&key_a);
        let salt_b = derive_player_telemetry_salt(&key_b);
        let player_uuid = uuid::Uuid::from_u128(0x069a79f4_44e9_4726_a5be_fca90e38aaf5);

        let anon_a1 = anonymize_player_uuid(&salt_a, player_uuid);
        let anon_a2 = anonymize_player_uuid(&salt_a, player_uuid);
        let anon_b = anonymize_player_uuid(&salt_b, player_uuid);

        assert_eq!(anon_a1.len(), 32);
        assert!(
            anon_a1
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
        assert_eq!(anon_a1, anon_a2);
        assert_ne!(anon_a1, anon_b);
    }

    #[test]
    fn live_signing_key_is_preferred_over_disk() {
        let live = SigningKey::from_bytes(&[9u8; 32]);
        let disk = SigningKey::from_bytes(&[8u8; 32]);
        let resolved = choose_signing_key(Some(&live), || disk.clone());
        assert_eq!(
            resolved.verifying_key().to_bytes(),
            live.verifying_key().to_bytes()
        );
        let from_disk = choose_signing_key(None, || disk.clone());
        assert_eq!(
            from_disk.verifying_key().to_bytes(),
            disk.verifying_key().to_bytes()
        );
    }

    #[test]
    fn replay_nonce_guard_rejects_captured_request() {
        let cache = NonceCache::new();
        let signing_key = SigningKey::from_bytes(&[13u8; 32]);
        let timestamp_str = "1757185000";
        let now = 1_757_185_000u64;

        let mut body = serde_json::json!({ "server_type": "pumpkin" });
        body[NONCE_FIELD] = serde_json::Value::String(generate_nonce());
        let body_bytes = serde_json::to_vec(&body).unwrap();
        let (pubkey_hex, sig_hex) =
            sign_telemetry_payload(&signing_key, timestamp_str, &body_bytes);

        // First delivery of the signed request is accepted.
        assert!(
            verify_telemetry_request_with_replay_protection(
                &cache,
                &pubkey_hex,
                &sig_hex,
                timestamp_str,
                &body_bytes,
                now
            )
            .is_ok()
        );

        // Replaying the exact captured request is rejected by the nonce cache.
        assert_eq!(
            verify_telemetry_request_with_replay_protection(
                &cache,
                &pubkey_hex,
                &sig_hex,
                timestamp_str,
                &body_bytes,
                now
            ),
            Err(TelemetryVerificationError::ReplayDetected)
        );

        // A request carrying a fresh signed nonce is accepted.
        body[NONCE_FIELD] = serde_json::Value::String(generate_nonce());
        let fresh_body = serde_json::to_vec(&body).unwrap();
        let (fresh_pubkey, fresh_sig) =
            sign_telemetry_payload(&signing_key, timestamp_str, &fresh_body);
        assert!(
            verify_telemetry_request_with_replay_protection(
                &cache,
                &fresh_pubkey,
                &fresh_sig,
                timestamp_str,
                &fresh_body,
                now
            )
            .is_ok()
        );

        // A signed body without a nonce is rejected.
        let no_nonce =
            serde_json::to_vec(&serde_json::json!({ "server_type": "pumpkin" })).unwrap();
        let (pk, sig) = sign_telemetry_payload(&signing_key, timestamp_str, &no_nonce);
        assert_eq!(
            verify_telemetry_request_with_replay_protection(
                &cache,
                &pk,
                &sig,
                timestamp_str,
                &no_nonce,
                now
            ),
            Err(TelemetryVerificationError::InvalidNonce)
        );
    }
}
