# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-block-bloc.md

Scope: close every row by comparing Pumpkin against decompiled 1.21 vanilla under
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/`.
Source files in this unit: grass_block.rs, spreading_snowy_block.rs,
plants_and_support_blocks.rs (GameTest).

## Result

FINAL PASS (2026-11-18). Re-read the four rows and re-verified each one directly
against the decompiled 1.21 source and against Pumpkin's implementation and tests.
No source changes were needed: the code already matches vanilla. Every value comes
from existing generated data, so there is no BLOCKED-DATA row. No `.rs` file was
edited this pass, so there was nothing to rustfmt.

- Row 1 (on_place on grass/mycelium/podzol; spreading_snowy_block.rs:29-49,192-223):
  FIXED / VERIFIED. Vanilla `SnowyBlock.getStateForPlacement`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SnowyBlock.java:41-44`)
  and `updateShape` (`SnowyBlock.java:25-38`) set SNOWY from the block above via
  `isSnowySetting` (`BlockTags.SNOW`, `SnowyBlock.java:46-48`). `GrassBlock`
  (`GrassBlock.java:20`) and `MyceliumBlock` (`MyceliumBlock.java:11`) extend
  `SpreadingSnowyBlock extends SnowyBlock` and override neither method; Podzol is a
  plain `SnowyBlock`. Pumpkin's `SnowyBlock::on_place` /
  `get_state_for_neighbor_update` are shared by Podzol, Mycelium and Grass
  (`grass_block.rs:27-36`), so on_place correctly stays on all three.

- Row 2 (fluid amount generic; spreading_snowy_block.rs:57-63): FIXED / VERIFIED.
  Vanilla `FluidState.isFull()` is `getAmount() == 8`, used in `canStayAlive`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:37`).
  Pumpkin `is_full_fluid` checks `!fluid_state.is_empty && fluid_state.level == 8`;
  `World::get_fluid_and_fluid_state` (`crates/pumpkin-core/src/world/mod.rs:7973-7976`)
  delegates to `fluid_state_from_block_state`
  (`crates/pumpkin-world/src/world.rs:156-182`), which derives `level` generically
  from any liquid block's LEVEL property via `liquid_level` (:140-152), clamping
  source/falling amounts to 8, with unit tests at :189-244. No WATER/LAVA special
  case remains.

- Row 3 (LAYERS == 1 short-circuit and light block; spreading_snowy_block.rs:70-94):
  FIXED / VERIFIED. Vanilla `canStayAlive`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:31-43`)
  short-circuits on `SNOW && LAYERS == 1` (:33-35), rejects a full fluid (:37-39),
  then returns `LightEngine.getLightDampeningInto(state, aboveState, UP, aboveState.getLightDampening()) < 15`
  (:41-42). Pumpkin `can_stay_alive_with_above` mirrors each branch, mapping light
  dampening through `LightEngine::get_light_dampening_into`
  (`crates/pumpkin-world/src/lighting/engine.rs:1084-1099`, from=grass, to=above,
  simple opacity = above_state.opacity). Equal to vanilla for all vanilla states.

- Row 4 (randomTick coverage; spreading_snowy_block.rs:116-179): FIXED / VERIFIED.
  The fade-on-!canStayAlive branch (always fades, no brightness/cover gate,
  :123-138), the `getMaxLocalRawBrightness(above) >= 9` gate (:139), the 4-offset
  spread loop with `nextInt(3)-1, nextInt(5)-3, nextInt(3)-1` (:141-144), the
  `default_block_state` argument to `canPropagate` (:152), and the SNOWY payload
  (:155-160) all match
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:57-104`.
  Paper/CraftBukkit BlockFadeEvent and BlockSpreadEvent cancellation is honoured
  (:127-137, :161-174). Live-world fade/survival/flood is covered by
  `test_spreading_snowy_grass_random_tick`
  (`crates/pumpkin-core/src/server/builtin_game_tests/plants_and_support_blocks.rs:315-361`:
  grass under opaque stone -> dirt, grass under one snow layer -> survives, grass
  under full water -> dirt). The pure seam `can_stay_alive_with_above` keeps four
  unit tests (`spreading_snowy_block.rs:248-289`). No new test added: every pure
  seam is already covered and the live path has a GameTest.

No BLOCKED-DATA rows: every value comes from existing generated data or existing
vanilla-matched logic. No `.rs` file was edited this pass, so there was nothing to
rustfmt.
