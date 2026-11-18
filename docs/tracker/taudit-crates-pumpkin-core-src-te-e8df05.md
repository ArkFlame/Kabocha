# Tracker: audit / crates/pumpkin-core/src/telemetry.rs (DEEP AUDIT)

File reviewed: `crates/pumpkin-core/src/telemetry.rs` (1495 lines, 6 tests).
Vanilla reference: none. Minecraft vanilla servers have no telemetry endpoint, so
"vanilla parity" is not the yardstick here. The nearest analogue is Paper/Spigot
bStats, which is opt-in, reports aggregate counts, rotates its interval from the
server response and never sends customer-controlled identity material. The audit
therefore compares against that contract and against the module own documented
wire format (`HeartbeatPayload`, `HeartbeatResponse`).

Markers: the file contains **no TODO/FIXME/XXX/HACK**. The only "vanilla"
occurrences are the default client brand string (`telemetry.rs:788`) and a test
fixture (`telemetry.rs:1402`), both correct.

## Fixed in this pass

| Gap | Rust | Reference | Fix | Size |
|---|---|---|---|---|
| `HeartbeatResponse.next_ping_seconds` was parsed and then ignored, so backend-driven throttling/backoff never took effect | `telemetry.rs:335`, `:931-941` | field doc "Recommended seconds to wait before the next heartbeat"; bStats honours the server interval | `resolve_next_interval_secs` picks the recommendation (0 = keep, floor 60s) and `interval.reset_after` reschedules the loop; unit test added | S |
| `send_shutdown` discarded transport and HTTP status errors and always returned `Ok`, making the caller `Err` branch dead and losing shutdown failures | `telemetry.rs:545-579` | bStats logs submission failures; a silent `Ok` hides a broken endpoint | added `?` plus `error_for_status()?`; test asserts an unreachable endpoint yields `Err` | S |
| Identity key was written with default umask and only then `chmod 0600`, leaving a short window where the Ed25519 seed was world-readable; existing permissive key files were never tightened | `telemetry.rs:495-506`, `:576-615` | secret material must never be group/world readable | `write_private_key` opens with `mode(0o600)` and `restrict_key_permissions` tightens on every load; extended the existing 0600 test | S |
| `record_leave` dropped sessions past the 1000-entry buffer with no log or counter, so a backend outage during a busy period lost sessions invisibly | `telemetry.rs:168-186` | `MAX_COMPLETED_SESSIONS_PER_HEARTBEAT` doc says buffered, not dropped | `tracing::debug!` on the drop path names the cap | S |
| `MAX_TELEMETRY_PLAYERS` claimed to cap all player breakdown entries but only caps online ones; completed sessions add up to 1000 more | `telemetry.rs:15-17`, `:809-835` | constant docs | constant doc now states the online cap and that completed sessions are bounded separately | S |

## Remaining gaps (not fixed this pass)

| Gap | Rust | Reference | Suggested fix | Size |
|---|---|---|---|---|
| `resolve_telemetry_urls` is asymmetric: a bare `<host>/heartbeat` endpoint resolves shutdown to `<host>/shutdown`, while the no-suffix branch resolves to `<host>/v1/shutdown`. A custom endpoint using the short form will 404 its shutdown notice | `telemetry.rs:419-444` | module wire format; `TelemetryConfig.endpoint` is user configurable | apply the same `/v1` suffix detection to the stripped prefix, or always derive shutdown from the resolved base URL | S (needs backend route confirmation) |
| `online_mode` is reported as `java.online_mode && bedrock.online_mode` while `max_players` uses `max`. With Java online-mode and Bedrock offline-mode the advertised value is `false` even though the Java listener enforces auth | `telemetry.rs:850-857` | field doc "whether online mode authentication is enforced on enabled listeners"; Paper applies online-mode per listener | report per-listener values, or redefine the field as "all enabled listeners enforce online mode" and document it | S (contract decision) |
| FIXED fix(telemetry): reuse sysinfo handle, share live salt, warn on unreadable key (salt part only). `build_heartbeat_payload` (used by `/pumpkin`) re-derives the salt via `resolve_identity_key()`, which re-reads the key from disk. If the running client used an ephemeral key (unwritable data dir) or the file changed, the command shows different anonymous ids than the heartbeats send | `telemetry.rs:697-703` | per-server stable anonymisation is the documented property (`derive_player_telemetry_salt`) | pass the live `SigningKey`/salt from `Server`/`TelemetryClient` instead of reloading | M |
| FIXED fix(telemetry): reuse sysinfo handle, share live salt, warn on unreadable key (reuse part only). `read_used_ram_mb` constructs and fully refreshes a new `sysinfo::System` on every heartbeat; the field is `ram_allocated_mb` but reports process used RAM, and the doc says "allocated/used by the system" | `telemetry.rs:752-756`, field doc `:307` | bStats reports JVM heap with the same ambiguity, but reuses one `System` | keep one shared `System` (or reuse the cached host info) and `refresh_memory()`; rename or fix the doc | S |
| Verification has no replay protection beyond the +/-300s timestamp window: any captured signed body is accepted repeatedly for 5 minutes | `telemetry.rs:386-417` | the module already sends a timestamp plus signature, a nonce would be additive | add a nonce header/body field and a backend-side seen-nonce cache; breaking wire change, needs backend co-design | M |
| FIXED fix(telemetry): reuse sysinfo handle, share live salt, warn on unreadable key. `resolve_identity_key` silently falls back to an ephemeral key when an existing key path exists but cannot be read (permissions, I/O), changing the server identity and every player salt for that run without an operator-visible error | `telemetry.rs:616-664` | identity stability requirement in `load_or_generate_key` docs | log at `warn` (not silently continue) and keep the failed path in the message; do not regenerate over an unreadable existing key | S |
| `TelemetryConfig::validate` is an empty `const fn`; an empty string endpoint or a nonsense URL is accepted and every heartbeat fails at runtime | `crates/pumpkin-config/src/telemetry.rs:39` | `interval_secs` doc already promises a 60s minimum, enforced only inside `start_telemetry_with_config` | validate a non-empty `http(s)` endpoint and clamp/reject `interval_secs < 60` | S |
| `online_players` uses `Server::get_player_count()` while the `players` array is capped at 500; a large server reports `online_players: 900` with 500 entries and no indication of truncation | `telemetry.rs:873` | wire format consumers join the two fields | add a `players_truncated` flag or cap `online_players` consistently | S |

STILL OPEN: `resolve_telemetry_urls` shutdown route for short `<host>/heartbeat` endpoints needs backend route confirmation.
STILL OPEN: `online_mode` aggregation is a contract decision (per-listener vs all-listeners semantics) with no vanilla reference.
STILL OPEN: `read_used_ram_mb` naming/doc (system-wide used RAM vs "allocated"); the reuse part is fixed, the contract is not.
STILL OPEN: replay nonce is a breaking wire change needing backend co-design (M).
STILL OPEN: `TelemetryConfig::validate` lives in pumpkin-config; making it fallible changes config-load API across crates and empty-endpoint semantics are unconfirmed.
STILL OPEN: `online_players` vs capped `players` needs a wire contract choice (truncation flag vs capping the count).
STILL OPEN: `/pumpkin` one-off send and server-link still call `resolve_identity_key()` for the signing key; only the anonymisation salt is shared with the running client.

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
  caps, ASCII-only locale) before they leave the server (`:773-812`).
- Active player lists take `joined_at` from the tracker rather than
  `SystemTime::now`, so a heartbeat does not rewrite session starts (`:849`).
- The heartbeat `Interval` keeps the default `MissedTickBehavior::Burst`, but a
  missed tick is unreachable: the HTTP client times out after 10s while the
  interval floor is 60s, so a send can never overrun its own period.
- `TelemetryConfig::validate` being empty matches the crate-wide pattern
  (`BasicConfig::validate` and `AdvancedConfig::validate` are empty too), so it is
  not counted as a gap; runtime clamping covers `interval_secs`.

## Result

Re-audit of all seven STILL OPEN rows against the current tree. Telemetry has no
decompiled-vanilla analogue, so Paper/Spigot bStats and the module own documented wire
format remain the reference. The current telemetry.rs is ~1885 lines and every STILL OPEN
row has already been resolved in-tree by earlier passes, so this pass makes no new code
change. No cargo or git was run (the orchestrator compiles the batch); no .rs file was
edited, so rustfmt was not needed.

| Row (doc line) | Result | Evidence |
|---|---|---|
| Short-form shutdown route (38) | FIXED in tree | shutdown_url_for is applied to both the stripped prefix and the no-suffix branch; test telemetry_client_resolves_short_form_shutdown_route asserts host/heartbeat -> host/v1/shutdown, equal to the bare-host case. |
| online_mode aggregation (39) | FIXED in tree | Field doc (telemetry.rs:317-318) states true only when every enabled listener enforces online mode; the builder matches per-listener enabled flags (:1042-1049). |
| read_used_ram_mb naming/doc (40) | FIXED in tree | Function renamed read_process_ram_mb (:928), field doc "Resident memory of the server process in MiB" (:328); a cached sysinfo::System is refreshed with refresh_processes. |
| Replay nonce (41) | FIXED in tree | NONCE_FIELD/HEADER_NONCE, generate_nonce, serialize_signed_body, NonceCache, verify_telemetry_request_with_replay_protection and replay_nonce_guard_rejects_captured_request are present (:42-53, :374-394, :471-534, :1813); the nonce is inside the signed body and the 600s cache TTL covers the 300s drift window. |
| TelemetryConfig::validate (42) | FIXED in tree (in-crate mitigation) | telemetry_endpoint_is_valid plus a runtime guard in start_telemetry_with_config (:1095-1121) disable telemetry with a warning on bad endpoints; test telemetry_endpoint_validation_requires_http_url_with_host. The cross-crate config-load API change was avoided (outside this unit). |
| online_players vs capped players (43) | FIXED in tree | players_truncated field added (:337-340), populated from collect_players_telemetry (:1051-1067), serialized and asserted in tests (:1313, :1371). |
| One-off signing key reuse (44) | FIXED in tree | resolve_identity_key prefers LIVE_SIGNING_KEY via choose_signing_key (:782-784, :858-864), set at startup (:1142), so /pumpkin one-off sends and server-link reuse the live key; test live_signing_key_is_preferred_over_disk present (:1797). |

No rows remain open. Files touched: this doc only (no .rs edits were needed).
