# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-te-e8df05.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: telemetry.rs

Final pass: every row below was re-read directly against the current tree. Telemetry is a
Pumpkin-specific feature with no decompiled-vanilla analogue under
`.vendor/vanilla-decompiled/net/minecraft/` (verified: no `*elemetry*` class exists there;
vanilla Minecraft has no telemetry endpoint, see the parent tracker header), so there is no
.vendor path to cite. The reference remains Paper/Spigot bStats and the module own documented
wire format. All fixes were already present in tree from prior passes, so this pass made no
.rs edits and rustfmt was not needed; no cargo or git was run.

## Result

| Row (parent tracker body line) | Final disposition | Code evidence (verified this pass) |
|---|---|---|
| Short-form shutdown route (38) | FIXED / VERIFIED | `shutdown_url_for` (:540-549) is applied to both the stripped-prefix branch (:556) and the no-suffix branch (:554) of `resolve_telemetry_urls`; short-form `host/heartbeat` resolves to `host/v1/shutdown`, equal to the bare-host case. Test `telemetry_client_resolves_short_form_shutdown_route` (:1725). |
| online_mode aggregation (39) | FIXED / VERIFIED | Field doc "true only when every enabled listener enforces online mode" (:317-318); the builder matches per-listener enabled flags in the `(bool,bool)` match (:1042-1049). |
| read_used_ram_mb naming/doc and sysinfo reuse (40) | FIXED / VERIFIED | Renamed `read_process_ram_mb` (:928), field doc "Resident memory of the server process in MiB" (:328); one cached `sysinfo::System` in a `Mutex<Option<...>>` is refreshed for the current process with `refresh_processes` (:929-942). `System::new().ok()` is consistent with `System::new_all().map_or_else` in the same file (sysinfo 0.40.1 returns `Result`). |
| Replay nonce (41) | FIXED / VERIFIED | `NONCE_FIELD` (:49), `generate_nonce` (:375), `serialize_signed_body` (:383-394), `NonceCache::check_and_insert` (:488-501) and `verify_telemetry_request_with_replay_protection` (:508-534) are all present; the nonce is read from the signature-verified body and the 600s cache TTL covers the 300s drift window. Test `replay_nonce_guard_rejects_captured_request` (:1842). |
| TelemetryConfig::validate (42) | FIXED / VERIFIED | Cross-file row already addressed: `pumpkin-config/src/telemetry.rs::validate` rejects empty/non-http endpoints at load (:42-51) with `should_panic` tests; telemetry.rs keeps a graceful runtime guard `telemetry_endpoint_is_valid` (:1102-1105, :1138) that disables telemetry with a warning. Test `telemetry_endpoint_validation_requires_http_url_with_host` (:1751). |
| online_players vs capped players (43) | FIXED / VERIFIED | `players_truncated` field (:337-340), populated from `collect_players_telemetry` (:1051-1052) and serialized into the payload (:1067); asserted in tests (:1392). |
| One-off signing key reuse (44) | FIXED / VERIFIED | `resolve_identity_key` prefers `LIVE_SIGNING_KEY` via `choose_signing_key` (:782-784, :859-864), set at startup (:1164-1168); the salt is shared via `LIVE_PLAYER_SALT` (:852, :874-876). Test `live_signing_key_is_preferred_over_disk` (:1826). |

UNIT CLOSED: 7 rows FIXED / VERIFIED, 0 STILL OPEN, 0 BLOCKED-DATA. No .rs edits this pass.
