# Parity rows for crates/pumpkin-config/src/telemetry.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/deferred-pass2** [partial/OWN-OPEN] own audit: 30 open, 0 fixed, 0 verified, 2 blocked-data (docs/tracker/deferred-pass2.md) docs/tracker/deferred-pass2.md
- **own/taudit-crates-pumpkin-core-src-te-e8df05** [partial/OWN-OPEN] own audit: 10 open, 17 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-te-e8df05.md) docs/tracker/taudit-crates-pumpkin-core-src-te-e8df05.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass, final disposition)

Re-verified both STILL OPEN rows against the current source. Telemetry is a
Pumpkin-specific bStats-style feature: there is no decompiled-vanilla analogue in
`.vendor/vanilla-decompiled/net/minecraft/` and therefore no .vendor path to cite
(see taudit-crates-pumpkin-core-src-te-e8df05.md header). The reference is the
module's documented wire format and Paper/Spigot bStats behaviour. No generated
pumpkin-data field or missing subsystem is involved, so there are no
BLOCKED-DATA rows.

| Row | Disposition | Evidence (verified this pass) |
|---|---|---|
| own/deferred-pass2 (TelemetryConfig::validate, row 42) | FIXED / VERIFIED | `crates/pumpkin-config/src/telemetry.rs::validate` rejects empty and non-http(s) endpoints with `assert!` (:42-51) and `#[should_panic]` tests (:94-110); it is called at config load from `crates/pumpkin-config/src/lib.rs:139` and `:489`. `crates/pumpkin-core/src/telemetry.rs` keeps a graceful runtime guard `telemetry_endpoint_is_valid` (:1102, :1139). |
| own/taudit-crates-pumpkin-core-src-te-e8df05 | FIXED / VERIFIED | All seven audited rows are FIXED/VERIFIED in tree (see that doc's final pass, lines 136-156): short-form shutdown route (`shutdown_url_for` :540-566), online_mode aggregation (:317-318, :1042-1049), read_process_ram_mb naming (:328, :928), replay nonce (`NONCE_FIELD` :49, `serialize_signed_body` :383, `NonceCache` :476, `verify_telemetry_request_with_replay_protection` :508), validate (:42-51), `players_truncated` (:340, :1051-1052, :1067), and one-off signing key reuse (`LIVE_SIGNING_KEY` / `LIVE_PLAYER_SALT` :782-784, :852). |

No STILL OPEN rows remain. No .rs file was edited this pass (all fixes already
present in tree), so no rustfmt was needed; no cargo or git was run.
