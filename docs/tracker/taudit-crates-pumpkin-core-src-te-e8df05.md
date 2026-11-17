# Tracker: audit / crates/pumpkin-core/src/telemetry.rs (DEEP AUDIT)

File reviewed: `crates/pumpkin-core/src/telemetry.rs` (1400 lines, 6 tests).
Vanilla reference: none. Minecraft vanilla servers have no telemetry endpoint, so
"vanilla parity" is not the yardstick here. The nearest analogue is Paper/Spigot
bStats, which is opt-in, reports aggregate counts, rotates its interval from the
server response and never sends customer-controlled identity material. The audit
therefore compares against that contract and against the module own documented
wire format (`HeartbeatPayload`, `HeartbeatResponse`).

Markers: the file contains **no TODO/FIXME/XXX/HACK**. The only "vanilla"
occurrences are the default client brand string (`telemetry.rs:777`) and a test
fixture (`telemetry.rs:1333`), both correct.

## Fixed in this pass

| Gap | Rust | Reference | Fix | Size |
|---|---|---|---|---|
| `HeartbeatResponse.next_ping_seconds` was parsed and then ignored, so backend-driven throttling/backoff never took effect | `telemetry.rs:335`, `:931-941` | field doc "Recommended seconds to wait before the next heartbeat"; bStats honours the server interval | `resolve_next_interval_secs` picks the recommendation (0 = keep, floor 60s) and `interval.reset_after` reschedules the loop; unit test added | S |
| `send_shutdown` discarded transport and HTTP status errors and always returned `Ok`, making the caller `Err` branch dead and losing shutdown failures | `telemetry.rs:545-579` | bStats logs submission failures; a silent `Ok` hides a broken endpoint | added `?` plus `error_for_status()?`; test asserts an unreachable endpoint yields `Err` | S |
| Identity key was written with default umask and only then `chmod 0600`, leaving a short window where the Ed25519 seed was world-readable; existing permissive key files were never tightened | `telemetry.rs:495-506`, `:576-615` | secret material must never be group/world readable | `write_private_key` opens with `mode(0o600)` and `restrict_key_permissions` tightens on every load; extended the existing 0600 test | S |

## Remaining gaps (not fixed this pass)

| Gap | Rust | Reference | Suggested fix | Size |
|---|---|---|---|---|
| `resolve_telemetry_urls` is asymmetric: a bare `<host>/heartbeat` endpoint resolves shutdown to `<host>/shutdown`, while the no-suffix branch resolves to `<host>/v1/shutdown`. A custom endpoint using the short form will 404 its shutdown notice | `telemetry.rs:419-444` | module wire format; `TelemetryConfig.endpoint` is user configurable | apply the same `/v1` suffix detection to the stripped prefix, or always derive shutdown from the resolved base URL | S (needs backend route confirmation) |
| `MAX_TELEMETRY_PLAYERS` is documented as "maximum number of player breakdown entries included in a single heartbeat payload" but only caps the *online* entries. Completed sessions add up to `MAX_COMPLETED_SESSIONS_PER_HEARTBEAT` (1000) on top, so one payload can carry 1500 entries | `telemetry.rs:16`, `:19`, `:806` | constant docs; backend body limit unknown | truncate the combined vector in `build_heartbeat_payload_inner`, or reword the constant | S |
| `online_mode` is reported as `java.online_mode && bedrock.online_mode` while `max_players` uses `max`. With Java online-mode and Bedrock offline-mode the advertised value is `false` even though the Java listener enforces auth | `telemetry.rs:850-857` | field doc "whether online mode authentication is enforced on enabled listeners"; Paper applies online-mode per listener | report per-listener values, or redefine the field as "all enabled listeners enforce online mode" and document it | S (contract decision) |
| `build_heartbeat_payload` (used by `/pumpkin`) re-derives the salt via `resolve_identity_key()`, which re-reads the key from disk. If the running client used an ephemeral key (unwritable data dir) or the file changed, the command shows different anonymous ids than the heartbeats send | `telemetry.rs:697-703` | per-server stable anonymisation is the documented property (`derive_player_telemetry_salt`) | pass the live `SigningKey`/salt from `Server`/`TelemetryClient` instead of reloading | M |
| `read_used_ram_mb` constructs and fully refreshes a new `sysinfo::System` on every heartbeat; the field is `ram_allocated_mb` but reports process used RAM, and the doc says "allocated/used by the system" | `telemetry.rs:752-756`, field doc `:307` | bStats reports JVM heap with the same ambiguity, but reuses one `System` | keep one shared `System` (or reuse the cached host info) and `refresh_memory()`; rename or fix the doc | S |
| Verification has no replay protection beyond the +/-300s timestamp window: any captured signed body is accepted repeatedly for 5 minutes | `telemetry.rs:386-417` | the module already sends a timestamp plus signature, a nonce would be additive | add a nonce header/body field and a backend-side seen-nonce cache; breaking wire change, needs backend co-design | M |
| `resolve_identity_key` silently falls back to an ephemeral key when an existing key path exists but cannot be read (permissions, I/O), changing the server identity and every player salt for that run without an operator-visible error | `telemetry.rs:616-664` | identity stability requirement in `load_or_generate_key` docs | log at `warn` (not silently continue) and keep the failed path in the message; do not regenerate over an unreadable existing key | S |
| `record_leave` silently drops completed sessions past the 1000-entry buffer with no counter or log, so a backend outage during a busy period loses sessions invisibly | `telemetry.rs:151-175` | `MAX_COMPLETED_SESSIONS_PER_HEARTBEAT` doc says "buffered", not "dropped" | emit a `tracing::debug!`/counter once per overflow, or evict oldest instead of newest | S |
| `TelemetryConfig::validate` is an empty `const fn`; an empty string endpoint or a nonsense URL is accepted and every heartbeat fails at runtime | `crates/pumpkin-config/src/telemetry.rs:39` | `interval_secs` doc already promises a 60s minimum, enforced only inside `start_telemetry_with_config` | validate a non-empty `http(s)` endpoint and clamp/reject `interval_secs < 60` | S |
| `online_players` uses `Server::get_player_count()` while the `players` array is capped at 500; a large server reports `online_players: 900` with 500 entries and no indication of truncation | `telemetry.rs:873` | wire format consumers join the two fields | add a `players_truncated` flag or cap `online_players` consistently | S |

## Verified correct (no change)

- Signed data layout `"timestamp.body"` is built and verified with the same
  helper on both sides (`compute_signed_data`, `telemetry.rs:340-346`), hex is
  lowercase on both edges, and the drift check uses `abs_diff` so future
  timestamps are rejected symmetrically (`:386-417`).
- Player anonymisation is `SHA-256(salt || uuid)` truncated to 16 bytes and
  domain-separated with `pumpkin-telemetry-player-salt-v1:` (`:678-694`), so ids
  are stable per server and unlinkable across servers; covered by a test.
- Session tracking keeps join `Instant` plus wall-clock start and computes
  `left_at` from the monotonic delta (`:146-175`, `:218-261`), avoiding clock
  jumps mid-session; the shutdown path finalises still-online players.
- Heartbeat payload buffering on send failure restores the raw sessions and
  respects the cap (`take_completed`/`restore_completed`, `:198-278`).
- Client brands and locales are sanitised (control chars stripped, 64/16 char
  caps, ASCII-only locale) before they leave the server (`:758-797`).
- Active player lists take `joined_at` from the tracker rather than
  `SystemTime::now`, so a heartbeat does not rewrite session starts (`:842`).
