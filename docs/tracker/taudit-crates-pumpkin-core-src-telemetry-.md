# Audit: crates/pumpkin-core/src/telemetry.rs

Category: audit | Coverage: AUDIT | File: 1369 lines | Markers found: 0 (`TODO`/`FIXME`/`XXX` absent; the two tracker-noted "vanilla" hits at `telemetry.rs:743` and `telemetry.rs:1299` are the client-brand fallback string and a test fixture, not markers).

## Scope note

This file is Pumpkin-owned telemetry (heartbeats to `telemetry.pumpkinmc.org`), not Mojang telemetry: there is no vanilla `TelemetryManager`/`ServerMetricsSamplersProvider` analogue to port, so "vanilla parity" here means internal contract parity: every field the backend sends is honoured, every error the client returns is acted on, and the payload semantics documented on each struct are true at runtime.

## Fixed in this audit

| # | file:line (before) | Gap | Fix | Size |
|---|---|---|---|---|
| 1 | `telemetry.rs:565-576` | `TelemetryClient::send_shutdown` discarded the HTTP result (`let _ = ... .send().await`) and always returned `Ok(())`, so its `Result` and the caller log at `telemetry.rs:935` could never report a failure. | Propagate transport and non-2xx errors (`await? .error_for_status()?`). Test `shutdown_request_failures_are_reported`. | S |
| 2 | `telemetry.rs:915-921` | `HeartbeatResponse::next_ping_seconds` was parsed and logged but never applied; the interval stayed at `config.interval_secs` forever, so backend throttling/back-off recommendations were ignored. | `recommended_heartbeat_interval_secs()` (floor 60s) + `interval.reset_after()` on success. Test `backend_recommended_heartbeat_interval_respects_minimum`. | S |

## Remaining gaps (not fixed)

| # | file:line | Gap | Vanilla/Paper reference | Suggested fix | Size |
|---|---|---|---|---|---|
| G1 | `telemetry.rs:900` | `tokio::time::interval` keeps the default `MissedTickBehavior::Burst`: after a long stall (laptop sleep, GC pause, suspended VM) the task fires back-to-back heartbeats with the same drained session set. | n/a (tokio guidance: periodic timers use `Delay`) | `interval.set_missed_tick_behavior(MissedTickBehavior::Delay)`. | S |
| G2 | `telemetry.rs:914-931` | A 200 response whose `status` is not "ok" (e.g. "throttled", "rejected") is treated as delivered: drained completed sessions are dropped even though the backend did not accept them. | n/a (backend contract) | On `resp.status != "ok"`, call `restore_completed(raw_completed)` and only trust `next_ping_seconds` when the backend accepted the heartbeat. | S |
| G3 | `telemetry.rs:811`, `telemetry.rs:718-722` | `ram_allocated_mb` is filled from `System::used_memory()` (whole-host used RAM), not the process allocation its name implies; the value moves with unrelated host load and is easily confused with `total_ram_mb`. | n/a | Report process RSS (sysinfo process memory or /proc/self/status VmRSS), or rename the field in the backend contract. | S |
| G4 | `telemetry.rs:816-823` | `online_mode` is collapsed to `java.online_mode && bedrock.online_mode` and `max_players` to the max of both listeners, so a half-online-mode dual-listener server reports `false` and a Java-only 20-slot / Bedrock 100-slot setup reports 100 for both. | n/a (Pumpkin-specific listeners) | Emit per-listener fields (`java_online_mode`, `java_max_players`, `bedrock_*`) or document the current semantic on the struct. | M |
| G5 | `telemetry.rs:663-668` | Public `build_heartbeat_payload` re-runs `resolve_identity_key()` on every call: blocking file I/O plus a possible key write on the command (tick) thread, and a second copy of the key resolution logic. | n/a | Take the precomputed key/salt (`build_heartbeat_payload_with_salt`) at the call site in `command/commands/pumpkin.rs:718`, resolving the key once at startup. | S |
| G6 | `telemetry.rs:164`, `telemetry.rs:236`, `telemetry.rs:772` | Session/player caps (`MAX_COMPLETED_SESSIONS_PER_HEARTBEAT`, `MAX_TELEMETRY_PLAYERS`) silently truncate; `online_players` can then exceed `players.len()` with no truncated flag for the backend. | n/a | Add a boolean/count field (`players_truncated`) or log at debug when the cap is hit. | S |
| G7 | `telemetry.rs:239-247` | On shutdown, an online player with no `active_joins` entry is reported with `joined_at == left_at == now`, i.e. a fake zero-length session instead of being skipped or marked unknown. | n/a | Skip untracked players (`continue`) or only report sessions recorded by `record_join`. | S |
| G8 | `telemetry.rs:477-509` | `load_or_generate_key` returns `Err` on a read failure (e.g. permissions) instead of falling back to regenerating, and `resolve_identity_key` has four duplicated `if exists && load` branches. | n/a | Collapse into a path-list loop and treat unreadable keys as missing (warn + regenerate). | S |

## Gap status (CODER h4)
- G1 (MissedTickBehavior::Burst after a stall): FIXED (38f31a2a0), set to `Delay`. UNVERIFIED: no unit test; timer behaviour needs a runtime check.
- G2 (non-"ok" heartbeat status dropped sessions): FIXED (38f31a2a0), sessions restored when the backend status is not "ok". VERIFIED-BY-TEST (restore order and cap: `restored_sessions_go_before_newer_ones_and_respect_the_cap`; the status branch still has no mock backend test).
- G7 (shutdown reported untracked players as zero-length sessions): FIXED (38f31a2a0), such players are skipped. UNVERIFIED: no unit test.
- G3, G4, G5, G6, G8: STILL OPEN (not in this slice's budget).

## Result

Verification is internal-contract parity only: telemetry is Pumpkin-owned and has no vanilla/Spigot analogue (see Scope note). No cargo/nextest run (forbidden by brief); findings are by inspection of the current checkout.

- G3 (whole-host RAM in `ram_allocated_mb`): ALREADY FIXED in checkout. `read_process_ram_mb()` refreshes this PID via sysinfo and reports `process.memory() / (1024 * 1024)` (process RSS); the field doc says "Resident memory of the server process". No change needed.
- G4 (collapsed `online_mode`/`max_players`): FIXED via the documentation alternative from the audit. `max_players` now documents that it is the larger value of both enabled listeners (or the single enabled listener's value), matching the `online_mode` doc. Emitting per-listener fields is the M-size alternative and is left open: it changes the backend payload contract, which is outside this unit's files.
- G5 (per-call key resolution in `build_heartbeat_payload`): ALREADY FIXED in checkout. The fallback salt is cached in a `FALLBACK_PLAYER_SALT` OnceLock, so `resolve_identity_key()` runs at most once per process; with telemetry enabled the live salt/key is reused. No change needed.
- G6 (silent cap truncation): ALREADY FIXED in checkout via the audit's "log at debug" alternative. All three drop sites emit `tracing::debug!`: `record_leave` (completed buffer), `drain_for_shutdown` (shutdown cap), `collect_players_telemetry` (player list). Adding a `players_truncated` field would change the backend contract, so it was not added.
- G8 (unreadable key returned Err; duplicated resolution branches): FIXED. The candidate loop was already collapsed in the checkout; `load_existing_identity_key` now reads the seed directly instead of calling `load_or_generate_key`, so a corrupt or unreadable candidate is treated as missing, left untouched, and the loop falls through to the next location (preserving an existing fallback identity). New pure unit test: `corrupt_identity_key_is_treated_as_missing_without_overwrite`.

Files touched: `crates/pumpkin-core/src/telemetry.rs` (rustfmt --edition 2024 clean). `pumpkin.rs` untouched (G5 needs no call-site change).
