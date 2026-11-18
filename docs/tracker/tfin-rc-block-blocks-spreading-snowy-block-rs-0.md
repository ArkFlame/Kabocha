# Final rows from docs/tracker/tpar-rates-pumpkin-core-src-block-blocks-spreading-snowy-block-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: plants_and_support_blocks.rs

Re-read the last Result of each referenced audit doc and re-verified every STILL OPEN row directly against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:25-104`, `SnowyBlock.java:24-48` and `.vendor/vanilla-decompiled/net/minecraft/world/level/lighting/LightEngine.java:51-60`. No source change was needed: the implementation already matches vanilla; every row is now closed.
- **own/taudit-crates-pumpkin-core-src-bl-4bd10c** (3 rows were STILL OPEN): all VERIFIED/FIXED in that doc. `BlockFadeEvent` carries `new_state_id` and `random_tick` passes `base_block.default_state.id` (`.vendor/.../SpreadingSnowyBlock.java:71`); `is_full_fluid` uses normalized `fluid_state.level == 8` (`:37`); opacity is the correct fixed per-state `getLightDampening` value (`:41`, `LightEngine.java:51-60`).
- **own/taudit-crates-pumpkin-core-src-block-bloc** (0 rows STILL OPEN): all four rows VERIFIED in that doc (on_place inherited from `SnowyBlock.getStateForPlacement` `SnowyBlock.java:41-44`, generic fluid amount, `LAYERS == 1` light-block short-circuit, and randomTick coverage by the `plants_and_support_blocks.rs` GameTest).
was needed; the implementation already matches vanilla on every row. No row remains STILL OPEN,

## Result (FINAL PASS, independently verified 2026-11-18)

Re-verified every row in this unit directly against the decompiled 1.21 source in
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/` and
`.vendor/vanilla-decompiled/net/minecraft/world/level/lighting/LightEngine.java`. No source
change was needed: the implementation already matches vanilla on every row. No row remains
STILL OPEN and no row is BLOCKED-DATA (all required values come from existing generated data).

Unit row 1 - `own/taudit-crates-pumpkin-core-src-bl-4bd10c`:
- R1 `BlockFadeEvent` new-state field: FIXED/VERIFIED. `BlockFadeEvent` carries `new_state_id`
  and a `const fn new(pos, block, new_state_id)`; `random_tick` passes
  `base_block.default_state.id` (`spreading_snowy_block.rs:127-131`), matching the fade path
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:71`
  (`CraftEventFactory.callBlockFadeEvent(level, pos, baseBlock.get().defaultBlockState())`).
- R2 `is_full_fluid` hardcoded: FIXED/VERIFIED. `is_full_fluid` returns
  `!fluid_state.is_empty && fluid_state.level == 8` (`spreading_snowy_block.rs:58-63`),
  matching `aboveState.getFluidState().isFull()` (`SpreadingSnowyBlock.java:37`).
- R3 opacity not position-aware: VERIFIED. `can_stay_alive_with_above` passes
  `above_state.opacity` into `LightEngine::get_light_dampening_into`
  (`spreading_snowy_block.rs:87-92`), matching `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:41`
  and `.vendor/vanilla-decompiled/net/minecraft/world/level/lighting/LightEngine.java:51-60`.

Unit row 2 - `own/taudit-crates-pumpkin-core-src-block-bloc`:
- Row 1 on_place on grass/mycelium/podzol: VERIFIED. `SnowyBlock::on_place` and
  `get_state_for_neighbor_update` (`spreading_snowy_block.rs:29-49`, used by Podzol and
  Mycelium at `:191-223`) mirror `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SnowyBlock.java:41-44`
  (`getStateForPlacement`) and `:25-38` (`updateShape`).
- Row 2 fluid amount generic: VERIFIED. `!is_empty && level == 8` matches
  `FluidState.isFull()` for any fluid (`SpreadingSnowyBlock.java:37`).
- Row 3 `LAYERS == 1` short-circuit and light block: VERIFIED. `can_stay_alive_with_above`
  (`spreading_snowy_block.rs:71-94`) matches `canStayAlive` at
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:31-43`.
- Row 4 randomTick coverage: VERIFIED. The fade-on-`!canStayAlive` branch, the `>= 9`
  brightness gate, the 4-offset loop, the `default_block_state` argument and the SNOWY payload
  all match `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:68-96`.
  Live-world fade/survival coverage is `test_spreading_snowy_grass_random_tick`
  (`crates/pumpkin-core/src/server/builtin_game_tests/plants_and_support_blocks.rs:316-361`),
  registered as `pumpkin:grass_random_tick_fade_and_survival` at
  `crates/pumpkin-core/src/server/builtin_game_tests/mod.rs:231,1548-1552` and dispatching
  through the real `GameTestWorld::random_tick` at
  `crates/pumpkin-core/src/server/server_test_manager.rs:928-944`; the pure seam
  `can_stay_alive_with_above` keeps its four unit tests at `spreading_snowy_block.rs:248-289`.

Files touched this pass: this tracker doc only (documentation). No `.rs` edit was needed,
so there is nothing to rustfmt. The "Gap status" table is fully VERIFIED and no STILL OPEN row
remains.
