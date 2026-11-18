# Parity rows for crates/pumpkin-core/src/block/blocks/spreading_snowy_block.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-4bd10c** [partial/OWN-OPEN] own audit: 5 open, 3 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-4bd10c.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-4bd10c.md
- **own/taudit-crates-pumpkin-core-src-block-bloc** [partial/OWN-OPEN] own audit: 9 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-block-bloc.md) docs/tracker/taudit-crates-pumpkin-core-src-block-bloc.md

## Result
All rows closed (see Result sections below). Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field.

## Result (third pass close)

Re-read the last Result of each referenced audit doc and re-verified every STILL OPEN row directly against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:25-104`, `SnowyBlock.java:24-48` and `.vendor/vanilla-decompiled/net/minecraft/world/level/lighting/LightEngine.java:51-60`. No source change was needed: the implementation already matches vanilla; every row is now closed.

- **own/taudit-crates-pumpkin-core-src-bl-4bd10c** (3 rows were STILL OPEN): all VERIFIED/FIXED in that doc. `BlockFadeEvent` carries `new_state_id` and `random_tick` passes `base_block.default_state.id` (`.vendor/.../SpreadingSnowyBlock.java:71`); `is_full_fluid` uses normalized `fluid_state.level == 8` (`:37`); opacity is the correct fixed per-state `getLightDampening` value (`:41`, `LightEngine.java:51-60`).
- **own/taudit-crates-pumpkin-core-src-block-bloc** (0 rows STILL OPEN): all four rows VERIFIED in that doc (on_place inherited from `SnowyBlock.getStateForPlacement` `SnowyBlock.java:41-44`, generic fluid amount, `LAYERS == 1` light-block short-circuit, and randomTick coverage by the `plants_and_support_blocks.rs` GameTest).

Files touched: this doc and the 4bd10c audit doc (documentation only); no `.rs` edit, so no rustfmt run.


append-test-marker
