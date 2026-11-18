# Parity rows for crates/pumpkin-core/src/block/blocks/lodestone.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tblocks-lodestone** [partial/OWN-OPEN] own audit: 13 open, 5 fixed, 0 verified, 0 blocked-data (docs/tracker/tblocks-lodestone.md) docs/tracker/tblocks-lodestone.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (final verification pass 3, 2026-11-18)

Row `own/tblocks-lodestone`: VERIFIED, matches vanilla, no source change needed. The board snapshot line ("13 open, 5 fixed") is stale; the referenced sub-tracker `docs/tracker/tblocks-lodestone.md` now resolves all six gap rows (1, 2, 3, 6 no-gap/verified; 4 and 5 verified against vanilla in its final verification pass).

- In-unit file `crates/pumpkin-core/src/block/blocks/lodestone.rs` is `#[pumpkin_block] pub struct LodestoneBlock;` plus an empty `impl BlockBehaviour for LodestoneBlock {}` with an explanatory comment, matching the `structure_void.rs` plain-block pattern. The dead `placed`/`on_state_replaced` POI hooks are gone. Vanilla registers `Blocks.LODESTONE` as a plain `Block` with only `BlockBehaviour.Properties` and no overrides (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/Blocks.java:4973-4981`). POI add/remove is engine-derived from block state via `PoiType::from_state_id`; lodestone's sole state 25187 maps to `PoiType::Lodestone` (`crates/pumpkin-data/src/generated/poi_type.rs:28`), so no block-level hook is vanilla.
- Supporting rows re-checked against the decompiled tree: `CompassItem.useOn` gates only on the clicked block being `LODESTONE` and transmutes to `Items.COMPASS` (`.vendor/vanilla-decompiled/net/minecraft/world/item/CompassItem.java:46-70`); `LodestoneTracker.tick` returns unchanged on a dimension mismatch and keeps the tracker for an unloaded/null chunk (`.vendor/vanilla-decompiled/net/minecraft/world/item/component/LodestoneTracker.java:26-42`). Pumpkin already matches both; rows 4 and 5 live in `player.rs`, outside this unit's file list, and were verified no-gap in the sub-tracker.

No `.rs` file edited this pass (lodestone.rs already correct), so no rustfmt run and no compile-pitfall diff to review. No new pure unit-test seam: a plain block with no overrides has no behaviour to unit-test, and POI add/remove requires a live World/PoiStorage already covered by the `poi_scan` state-id test.

Status: PASS fixed=0 open=0 files=docs/tracker/tpar-crates-pumpkin-core-src-block-blocks-lodestone-rs.md
