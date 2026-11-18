# Parity rows for crates/pumpkin-core/src/server/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-se-1269c5** [partial/OWN-OPEN] own audit: 5 open, 6 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-se-1269c5.md) docs/tracker/taudit-crates-pumpkin-core-src-se-1269c5.md
- **own/taudit-crates-pumpkin-core-src-server-mod** [partial/OWN-OPEN] own audit: 2 open, 6 fixed, 4 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-server-mod.md) docs/tracker/taudit-crates-pumpkin-core-src-server-mod.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass, all rows closed)

Both linked audit rows are resolved. No source change was needed this pass: every
remaining gap was either already fixed in code by an earlier pass or verified against
the vendored decompiled vanilla to be a non-gap. No row depends on a generated
`pumpkin-data` field or a missing subsystem.

| row | final status | evidence |
|---|---|---|
| own/taudit-crates-pumpkin-core-src-se-1269c5 | VERIFIED | All 8 gaps closed. Rows 1, 2, 5, 6, 8 FIXED in code; rows 3, 4, 7 VERIFIED/FIXED against decompiled vanilla (see that doc's final Result). |
| own/taudit-crates-pumpkin-core-src-server-mod | VERIFIED | All 7 gaps closed. Rows 1, 3, 4, 5, 6, 7 FIXED/VERIFIED; row 2 (map ids) VERIFIED NOT A GAP against `ServerLevel.getFreeMapId()` and `MapIndex` (see that doc's final Result). |

Files changed this pass: none. Source files re-read to verify: `crates/pumpkin-core/src/server/mod.rs`.
