# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-te-e8df05.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No STILL OPEN rows remain. Files touched this pass: this doc only.

## Result
UNIT CLOSED: all seven rows are FIXED and VERIFIED by reading the current source directly.
Telemetry is Pumpkin-specific and has no decompiled-vanilla analogue under
`.vendor/vanilla-decompiled/net/minecraft/` (vanilla Minecraft has no telemetry endpoint; see the
parent doc header), so there is no .vendor path to cite. The reference remains Paper/Spigot bStats
and the module's documented wire format. No row needs a generated pumpkin-data field or a missing
subsystem, so there are no BLOCKED-DATA rows. No source file was edited this pass (every fix was
already present in tree from earlier passes), so no rustfmt was needed; no cargo or git was run.

| Row (parent body line) | Final disposition | Code evidence verified this pass |
|---|---|---|
| Short-form shutdown route (38) | FIXED / VERIFIED | `shutdown_url_for` (telemetry.rs:540-549) is applied to both the stripped prefix (:556) and the no-suffix branch (:554) of `resolve_telemetry_urls`; short-form `host/heartbeat` resolves to `host/v1/shutdown`, equal to the bare-host case. |
| online_mode aggregation (39) | FIXED / VERIFIED | Builder matches per-listener enabled flags in the `(bool,bool)` match (telemetry.rs:1042-1049); field doc states true only when every enabled listener enforces online mode. |
| read_used_ram_mb naming/doc (40) | FIXED / VERIFIED | Renamed `read_process_ram_mb` (:928), called at :1037; a cached `sysinfo::System` is refreshed for the current process; field doc "Resident memory of the server process in MiB". |
| Replay nonce (41) | FIXED / VERIFIED | `NONCE_FIELD` (:49), `generate_nonce` (:375), `serialize_signed_body` (:383-388), `NonceCache` (:476), `verify_telemetry_request_with_replay_protection` (:508) all present; nonce read from the signature-verified body (:526), cache TTL covers the drift window. |
| TelemetryConfig::validate (42) | FIXED / VERIFIED | `pumpkin-config/src/telemetry.rs::validate` rejects empty/non-http endpoints (:42-51) with `should_panic` tests (:95-106); telemetry.rs keeps a graceful runtime guard `telemetry_endpoint_is_valid` (:1102, :1138). |
| online_players vs capped players (43) | FIXED / VERIFIED | `players_truncated` field (:340), populated by `collect_players_telemetry` (:1051-1052), serialized into the payload (:1067), asserted in tests (:1392). |
| One-off signing key reuse (44) | FIXED / VERIFIED | `resolve_identity_key` prefers `LIVE_SIGNING_KEY` via `choose_signing_key` (:783, :859-864), set at startup (:1165); salt shared via `LIVE_PLAYER_SALT` (:852, :874, :1168); test `live_signing_key_is_preferred_over_disk` (:1826). |

Files touched this pass: this doc only. 0 open, 0 BLOCKED-DATA.
