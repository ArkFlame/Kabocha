# Parity rows for crates/pumpkin-core/src/telemetry.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-telemetry-** [partial/OWN-OPEN] own audit: 3 open, 9 fixed, 3 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-telemetry-.md) docs/tracker/taudit-crates-pumpkin-core-src-telemetry-.md

## Result
Final result (third pass). Telemetry is Pumpkin-owned (heartbeats to `telemetry.pumpkinmc.org`) with no vanilla/Spigot analogue, so there is no `.vendor/vanilla-decompiled/...` class to cite; every row is verified against the internal contract documented on the structs/functions and the pure unit tests in `mod tests` of `crates/pumpkin-core/src/telemetry.rs`. No row remains STILL OPEN.

- G1 (MissedTickBehavior::Burst after a stall): FIXED. `start_telemetry_with_config` sets `tokio::time::MissedTickBehavior::Delay` on the interval (telemetry.rs:1185). No pure unit-test seam exists (timer behaviour needs tokio paused time).
- G2 (non-"ok" status dropped drained sessions): FIXED. `backend_accepted_heartbeat` (telemetry.rs:1085) gates both session restore and interval adoption in the heartbeat loop (telemetry.rs:1201-1222): a non-"ok" status calls `restore_completed` and leaves the interval unchanged. Unit tests: `only_ok_heartbeat_status_accepts_drained_sessions`, `restored_sessions_go_before_newer_ones_and_respect_the_cap`.
- G3 (`ram_allocated_mb` was whole-host used RAM): FIXED. `read_process_ram_mb()` (telemetry.rs:928) refreshes this PID via sysinfo and reports `process.memory() / (1024 * 1024)` (process RSS); the field doc reads "Resident memory of the server process in MiB" (telemetry.rs:328).
- G4 (collapsed `online_mode`/`max_players`): FIXED via the documented-contract option from the audit. `HeartbeatPayload` docs (telemetry.rs:314-319) now state the all-listeners summary semantics (`max_players` = larger enabled listener value, `online_mode` = true only when every enabled listener uses online mode). Emitting per-listener fields is a backend wire-contract change (M), not a generated-data/subsystem blocker, so it is not required for parity.
- G5 (`build_heartbeat_payload` re-ran `resolve_identity_key()` per call): FIXED. The fallback salt is cached in the `FALLBACK_PLAYER_SALT` `OnceLock` and `LIVE_PLAYER_SALT` is reused (telemetry.rs:873-876), so `resolve_identity_key()` runs at most once per process; no call-site change in `command/commands/pumpkin.rs` is needed.
- G6 (silent truncation at the caps): FIXED. All three cap sites log `tracing::debug!` (telemetry.rs:195, 256, 996) and `players_truncated: bool` is serialized (telemetry.rs:340, 1067), asserted by `heartbeat_payload_serialization`.
- G7 (shutdown reported untracked players as zero-length sessions): FIXED. `drain_for_shutdown` skips players with no `active_joins` entry (telemetry.rs:259-263).
- G8 (unreadable key returned `Err`; duplicated resolution branches): FIXED. Candidate locations are a single loop (telemetry.rs:789-799); `load_existing_identity_key` (telemetry.rs:749) treats an unreadable/corrupt candidate as missing and leaves the file untouched. Unit test: `corrupt_identity_key_is_treated_as_missing_without_overwrite`.

No source change was needed this pass: all eight rows were already fixed in the checkout and were re-verified by reading the current file. No cargo/nextest run (forbidden by the brief). Files touched: none.
