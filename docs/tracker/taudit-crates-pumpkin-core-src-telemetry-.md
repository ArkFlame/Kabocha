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
