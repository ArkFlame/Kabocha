# Final rows from docs/tracker/tpar-crates-pumpkin-config-src-telemetry-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

## Result (final pass, unit closed)

Every row below was re-read directly against the current source this pass. Telemetry is a Pumpkin-specific bStats-style feature: there is no decompiled-vanilla analogue under `.vendor/vanilla-decompiled/net/minecraft/` and therefore no .vendor path to cite (see the headers of `tpar-crates-pumpkin-config-src-telemetry-rs.md` and `taudit-crates-pumpkin-core-src-te-e8df05.md`). The reference is the module's documented wire format plus Paper/Spigot bStats behaviour. No row needs a generated `pumpkin-data` field or a missing subsystem, so there are no BLOCKED-DATA rows. No .rs file needed edits this pass (all fixes are already in tree), so no rustfmt was required; no cargo or git was run.

| Row | Disposition | Evidence (re-verified against current source) |
|---|---|---|
| own/deferred-pass2 - `TelemetryConfig::validate` (deferred-pass2.md:41) | FIXED / VERIFIED | `crates/pumpkin-config/src/telemetry.rs::validate` (:42-51) rejects empty and non-http(s) endpoints with `assert!`; `#[should_panic]` tests at :94-110. Called at config load from `crates/pumpkin-config/src/lib.rs:139` (`self.telemetry.validate()`) and `:489`. `crates/pumpkin-core/src/telemetry.rs` keeps a graceful runtime guard `telemetry_endpoint_is_valid` (:1102-1105, called :1138) with tests (:1752-1760). |
| own/taudit - Short-form shutdown route (:38) | FIXED / VERIFIED | `shutdown_url_for` (:540-548) is applied to both the stripped-prefix branch (:556) and the no-suffix branch (:554) of `resolve_telemetry_urls` (:551-565); short-form `host/heartbeat` resolves to `host/v1/shutdown`, equal to the bare-host case. |
| own/taudit - online_mode aggregation (:39) | FIXED / VERIFIED | Field doc "true only when every enabled listener enforces online mode" (:317-319); the builder matches per-listener enabled flags in the `(bool,bool)` match (:1042-1049), returning `java_net.online_mode && bedrock_net.online_mode` when both listeners are enabled. |
| own/taudit - read_used_ram_mb naming/doc (:40) | FIXED / VERIFIED | Function renamed `read_process_ram_mb` (:928-943); field doc "Resident memory of the server process in MiB" (:327-328); a cached `sysinfo::System` is refreshed with `refresh_processes` for the current pid (:929-942). |
| own/taudit - Replay nonce (:41) | FIXED / VERIFIED | `NONCE_FIELD` (:49), `serialize_signed_body` (:383-394), `NonceCache::check_and_insert` (:488-501) and `verify_telemetry_request_with_replay_protection` (:508-534) are present; the nonce is read from the signature-verified body (:523-528) so it cannot be swapped, and the cache TTL covers the 300s drift window. Tests at :1843-1902. |
| own/taudit - TelemetryConfig::validate (:42) | FIXED / VERIFIED | Same fix as the deferred-pass2 row above: validation at config load (`pumpkin-config/src/telemetry.rs:42-51`, `lib.rs:139,489`) plus the graceful runtime guard `telemetry_endpoint_is_valid` (:1102, :1138). |
| own/taudit - online_players vs capped players / `players_truncated` (:43) | FIXED / VERIFIED | `players_truncated` field (:340), populated by `collect_players_telemetry` (:1051-1052) and serialized into the payload (:1067), with tests (:1334, :1392). |
| own/taudit - One-off signing key reuse (:44) | FIXED / VERIFIED | `resolve_identity_key` prefers `LIVE_SIGNING_KEY` via `choose_signing_key` (:783, :859-864), set at startup (:1165); the salt is shared via `LIVE_PLAYER_SALT` (:852, :874-876, :1168); test `live_signing_key_is_preferred_over_disk` (:1829). |

UNIT CLOSED: 8 row entries FIXED / VERIFIED (7 distinct issues; the `TelemetryConfig::validate` issue appears in two source trackers), 0 open, 0 BLOCKED-DATA. No .rs edits needed this pass.
