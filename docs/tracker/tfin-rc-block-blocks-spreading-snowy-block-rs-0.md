# Final rows from docs/tracker/tpar-rates-pumpkin-core-src-block-blocks-spreading-snowy-block-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: plants_and_support_blocks.rs

Re-read the last Result of each referenced audit doc and re-verified every STILL OPEN row directly against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:25-104`, `SnowyBlock.java:24-48` and `.vendor/vanilla-decompiled/net/minecraft/world/level/lighting/LightEngine.java:51-60`. No source change was needed: the implementation already matches vanilla; every row is now closed.
- **own/taudit-crates-pumpkin-core-src-bl-4bd10c** (3 rows were STILL OPEN): all VERIFIED/FIXED in that doc. `BlockFadeEvent` carries `new_state_id` and `random_tick` passes `base_block.default_state.id` (`.vendor/.../SpreadingSnowyBlock.java:71`); `is_full_fluid` uses normalized `fluid_state.level == 8` (`:37`); opacity is the correct fixed per-state `getLightDampening` value (`:41`, `LightEngine.java:51-60`).
- **own/taudit-crates-pumpkin-core-src-block-bloc** (0 rows STILL OPEN): all four rows VERIFIED in that doc (on_place inherited from `SnowyBlock.getStateForPlacement` `SnowyBlock.java:41-44`, generic fluid amount, `LAYERS == 1` light-block short-circuit, and randomTick coverage by the `plants_and_support_blocks.rs` GameTest).
was needed; the implementation already matches vanilla on every row. No row remains STILL OPEN,

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
