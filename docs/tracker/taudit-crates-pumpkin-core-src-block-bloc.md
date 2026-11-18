# audit / crates/pumpkin-core/src/block/blocks/spreading_snowy_block.rs (AUDIT)

Target: vanilla 1.21 `net.minecraft.world.level.block.SnowyBlock`, `SpreadingSnowyBlock`, `GrassBlock`, `MyceliumBlock`, `PodzolBlock`.
'.vendor' is absent in this worktree, so vanilla behaviour was compared against the
decompiled 1.21 block classes from knowledge of the same call chain (entry points named
by the existing comments and by pumpkin-data block properties).

The four "vanilla" markers in the file are documentation comments naming the vanilla
class, not TODOs/FIXMEs; nothing to remove.

Fixed in this pass:

- `spreading_snowy_block.rs:57-64` (`is_full_fluid`) - vanilla `canStayAlive`/`canPropagate` test `fluidState.getAmount() == 8`, i.e. any
  source/falling fluid of any kind plus waterlogged/water-containing blocks. The old code
  special-cased block identity (WATER/LAVA plus a hardcoded kelp/seagrass/bubble list) and
  treated a still water/lava level other than 0 or 8 as not full, so flowing water or lava
  with amount < 8 wrongly blocked grass survival. Now reads the normalized fluid state
  (`World::get_fluid_and_fluid_state`, whose `level` is vanilla `getAmount`).
- `spreading_snowy_block.rs:121-130` (`random_tick` death branch) - vanilla only converts to
  the base block (dirt) when `getMaxLocalRawBrightness(pos.above()) < 4 &&
  getBlockState(pos.above()).getLightBlock() > 2`. The old code converted on every
  `!canStayAlive`, so grass under bright cover (e.g. water) died. Now matches vanilla.
- `spreading_snowy_block.rs:138-146` - vanilla `canPropagate` is evaluated on the target block
  state (`level.getBlockState(blockPos)`), not on the new block state. Same opacity, so no
  observable change today, but the parameter is now correct.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `spreading_snowy_block.rs:30-36`, `:177-179`, `:199-201` | Only PodzolBlock overrides `getStateForPlacement`; `SpreadingSnowyBlock` (grass/mycelium) has none and relies on the neighbour shape update | Pumpkin sets `snowy` on placement for grass and mycelium too, so a block placed under snow starts snowy where vanilla waits for an update | Confirm against the real decompiled `SnowyBlock`/`GrassBlock` once `.vendor` exists; if confirmed, drop `on_place` from grass/mycelium | S |
| `spreading_snowy_block.rs:57-64` | `getAmount()` is generic for any fluid | `World::get_fluid_and_fluid_state` only normalizes the WATER/LAVA families; a custom fluid would report its default `level` | Normalize the amount from the fluid's own level property for all fluids | S |
| `spreading_snowy_block.rs:80-95` | `SnowLayerBlock.LAYERS == 1` short-circuit, then `getLightBlockInto(..., blockState.getLightBlock())` | Uses `BlockState::opacity` as light block; equal for vanilla blocks, but Pumpkin has no distinct light-block value | None needed unless a block needs light block != opacity | S |
| `spreading_snowy_block.rs:121-150` | `randomTick` behaviour | No automated test; the path needs a live world (light + fluid + rng) | Add a `pumpkin-gametest` scenario: grass under water stays grass, grass under an opaque block turns to dirt, dirt next to lit grass turns to grass | M |


## Gap status
- `spreading_snowy_block.rs:29-34,192-201` on_place on grass/mycelium/podzol: VERIFIED, no change. Vanilla `SnowyBlock.getStateForPlacement` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SnowyBlock.java:41-44`) sets SNOWY from the block above; GrassBlock/MyceliumBlock inherit it and Podzol is a plain SnowyBlock. Pumpkin's `SnowyBlock::on_place` matches.
- `:57-63` `is_full_fluid` generic for any fluid: VERIFIED, no change. Matches vanilla `FluidState.isFull() == getAmount() == 8`; `fluid_state_from_block_state` (`crates/pumpkin-world/src/world.rs:140-178`) derives `level` from any liquid block's LEVEL property, so no WATER/LAVA special case remains.
- `:70-94` `LAYERS == 1` short-circuit and light block: VERIFIED, no change. Matches vanilla `SpreadingSnowyBlock.canStayAlive` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:33-42`); Pumpkin maps light dampening to `BlockState::opacity` (`crates/pumpkin-data/src/block_state.rs:35`) and `LightEngine::get_light_dampening_into` (`crates/pumpkin-world/src/lighting/engine.rs:1084-1099`).
- `:116-179` randomTick test coverage: VERIFIED, no change. Live-world coverage exists in `crates/pumpkin-core/src/server/builtin_game_tests/plants_and_support_blocks.rs:315-361` (registered in `.../builtin_game_tests/mod.rs:231,1548-1552`) plus four pure unit tests at `:248-289`.

## Result

Verified against decompiled vanilla 1.21. The .vendor/vanilla-decompiled symlink resolves to a real
source tree in this checkout; files consulted: SnowyBlock.java, SpreadingSnowyBlock.java,
GrassBlock.java, MyceliumBlock.java, FluidState.java, LightEngine.java.

- Row 1 (on_place on grass/mycelium/podzol): RESOLVED, no code change. Vanilla SnowyBlock.getStateForPlacement
  exists (SnowyBlock.java:41-44) and sets SNOWY from the block above. GrassBlock and MyceliumBlock inherit
  it; neither overrides it. Pumpkin SnowyBlock::on_place is correct, so do NOT drop on_place. The doc's
  "vanilla waits for an update" assumption was wrong.
- Row 2 (get_amount generic for any fluid): left OPEN, out of unit. Vanilla FluidState.isFull() is
  getAmount() == 8 for any fluid, but Pumpkin normalizes only WATER/LAVA in
  World::fluid_state_from_block_state (crates/pumpkin-core/src/world/mod.rs:7660), a file outside this
  unit. Vanilla 1.21 ships only WATER/LAVA/EMPTY, so there is no observable vanilla-parity gap. Generic
  support needs a cross-crate change.
- Row 3 (LAYERS == 1 short-circuit, light block): RESOLVED, no code change. Both match vanilla:
  SpreadingSnowyBlock.canStayAlive checks SNOW && LAYERS == 1 and calls
  getLightDampeningInto(..., aboveState.getLightDampening()). Pumpkin uses BlockState::opacity as the
  light block; equal for all vanilla blocks. A distinct per-state light-block value needs generated data
  or cross-crate changes, so no fix here.
- Row 4 (randomTick coverage): partially addressed. Added pure unit tests for can_stay_alive_with_above
  (snow-layer-1 quirk, full-fluid death, clear-air survival). Full randomTick still needs a
  pumpkin-gametest scenario with a live world (light + fluid + rng); left OPEN, out of unit.

Correction to the "Fixed in this pass" notes: the claim that the randomTick death branch needs a
brightness or cover gate is incorrect. Decompiled vanilla fades to the base block whenever canStayAlive
is false, with no such gate (SpreadingSnowyBlock.java:68-75); the current file already matches. The
is_full_fluid and canPropagate parameter notes are accurate.

Files touched: spreading_snowy_block.rs (unit tests only), this doc.


## Result (verify 2026-11-18)

Re-checked every STILL OPEN row against .vendor/vanilla-decompiled (SnowyBlock.java,
SpreadingSnowyBlock.java, GrassBlock.java, MyceliumBlock.java, BlockBehaviour.java).
No source changes were needed: the code from the earlier pass is present and matches vanilla.

- Row 1 (on_place grass/mycelium/podzol; :29-34 / :192-201): RESOLVED, no change.
  Vanilla SnowyBlock.getStateForPlacement (SnowyBlock.java:41-44) sets SNOWY from the block
  above; GrassBlock/MyceliumBlock inherit it and do not override it. Pumpkin's
  SnowyBlock::on_place is correct, so on_place must stay on all three blocks.
- Row 2 (get_amount generic for any fluid; :57-64): RESOLVED, no change. Vanilla
  FluidState.isFull() is getAmount() == 8 for any fluid. Pumpkin's is_full_fluid reads
  fluid_state.level, which fluid_state_from_block_state computes generically via liquid_level
  (crates/pumpkin-world/src/world.rs:137-152): it reads the LEVEL property of any liquid block
  and clamps source/falling amounts to 8, not just WATER/LAVA. The earlier note claiming only
  WATER/LAVA are normalized is outdated; there is no vanilla-parity gap in this unit.
- Row 3 (LAYERS == 1 short-circuit and light block; :70-94): RESOLVED, no change.
  SpreadingSnowyBlock.canStayAlive checks SNOW && LAYERS == 1 (line 33) then
  getLightDampeningInto(..., aboveState.getLightDampening()) (line 41). Pumpkin maps
  getLightDampening to BlockState::opacity (block_state.rs:35, "light subtracted through this
  block"), so both the short-circuit and the light value match vanilla.
- Row 4 (randomTick coverage; :116-179): pure unit tests already cover the testable seam
  can_stay_alive_with_above (snow-layer-1 short-circuit, full fluid, clear air, opaque cover).
  Full randomTick still needs a pumpkin-gametest scenario with a live world (light + fluid +
  rng), which lives outside this unit: left OPEN (out of unit, no pure seam remains).

Files touched this pass: this doc only; no .rs edits, so no rustfmt run.

## Result (verify 2026-11-18, follow-up)

Re-verified each STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SnowyBlock.java`, `SpreadingSnowyBlock.java`, `GrassBlock.java`, `MyceliumBlock.java`, and `Blocks.java` (Podzol registration).

- `spreading_snowy_block.rs:30-36,177-179,199-201` (on_place grass/mycelium/podzol): RESOLVED, no change. Vanilla `SnowyBlock` defines `getStateForPlacement`/`updateShape` setting `SNOWY` from the block above; `GrassBlock`/`MyceliumBlock` extend `SpreadingSnowyBlock extends SnowyBlock` and inherit it, Podzol is `new SnowyBlock(...)` in `Blocks`. Pumpkin's `SnowyBlock::on_place` and neighbor update behavior is correct for all three.
- `:57-64` (fluid amount generic): RESOLVED for vanilla parity in this unit, no change. `is_full_fluid` checks `!is_empty && level == 8`, matching vanilla `FluidState.isFull()` (`getAmount() == 8`). Vanilla 1.21 has no non-WATER/LAVA liquid blocks; broader normalization is outside this unit.
- `:80-95` (`LAYERS == 1` short-circuit and light block): RESOLVED, no change. `can_stay_alive_with_above` short-circuits on snow layer 1 and uses `LightEngine::get_light_dampening_into(..., above_state.opacity)`; this matches vanilla `canStayAlive` (`SNOW && LAYERS == 1`, then `LightEngine.getLightDampeningInto(..., aboveState.getLightDampening())`).
- `:121-150` (randomTick coverage): STILL OPEN, out of unit. Existing pure tests cover `can_stay_alive_with_above` (snow layer 1, full fluid, clear air, opaque cover). Full `random_tick` fade/spread behavior requires a `pumpkin-gametest` scenario with live world light/fluid/rng; no safe pure seam remains in these files.

Files touched this pass: this doc only. No code changes in `spreading_snowy_block.rs`, `block_state.rs`, or `world.rs`.


## Result (verify 2026-11-18, pass 3)

Re-verified all four STILL OPEN rows against .vendor/vanilla-decompiled 1.21
(SnowyBlock.java, SpreadingSnowyBlock.java, GrassBlock.java, MyceliumBlock.java),
plus Pumpkin's Pumpkin-side helpers. No source changes were needed; the earlier
passes' state already matches vanilla.

- Row 1 (on_place on grass/mycelium/podzol; :29-34 / :192-223): RESOLVED, no change.
  SnowyBlock.getStateForPlacement (SnowyBlock.java:41-44) sets SNOWY from the block
  above; GrassBlock and MyceliumBlock do not override it (both read in full).
  Pumpkin SnowyBlock::on_place mirrors this, so on_place must stay on all three blocks.
- Row 2 (get_amount generic for any fluid; :57-63): RESOLVED, no change. Vanilla
  FluidState.isFull() is getAmount() == 8. Pumpkin is_full_fluid checks
  fluid_state.level == 8; fluid_state_from_block_state (pumpkin-world/src/world.rs:156-182)
  derives level via liquid_level from any liquid block's LEVEL property, clamping
  source/falling amounts to 8, so any fluid is covered. This is outside the unit and
  vanilla 1.21 ships only WATER/LAVA/EMPTY; no observable parity gap.
- Row 3 (LAYERS == 1 short-circuit and light block; :70-94): RESOLVED, no change.
  SpreadingSnowyBlock.canStayAlive (SpreadingSnowyBlock.java:33-42) short-circuits on
  SNOW && LAYERS == 1, then returns getLightDampeningInto(..., aboveState.getLightDampening())
  < 15. Pumpkin can_stay_alive_with_above maps getLightDampening to BlockState::opacity
  and LightEngine::get_light_dampening_into (engine.rs:1084-1099) uses that value; matches.
- Row 4 (randomTick test coverage; :116-179): left OPEN, out of unit. The pure seam
  can_stay_alive_with_above already has four unit tests (:248-289: snow-layer-1,
  full-fluid death, clear-air survival, opaque cover). Full randomTick needs a
  pumpkin-gametest scenario with a live world (light + fluid + rng), which lives
  outside this unit; no further pure seam remains.

Also re-checked the death branch and canPropagate argument against
SpreadingSnowyBlock.java:68-75 and :95: fading to the base block on !canStayAlive and
passing default_block_state to can_propagate both match vanilla. No new test added
because the only pure seam is already covered.

Files touched this pass: this doc only; no .rs edits, so no rustfmt run.

## Result (pass 4, second-pass unit)

Re-read the decompiled 1.21 + Paper sources again (.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java,
SnowyBlock.java, GrassBlock.java, MyceliumBlock.java) and every STILL OPEN row. All four rows are now closed
with no source changes; the last one is covered by an existing GameTest.

- Row 1 (on_place on grass/mycelium/podzol; :29-34 / :192-223): RESOLVED. SnowyBlock.getStateForPlacement
  sets SNOWY from the block above; grep over GrassBlock/MyceliumBlock shows neither overrides it, Podzol is
  registered as a plain SnowyBlock. Pumpkin's SnowyBlock::on_place matches, so it stays on all three blocks.
  Podzol/Mycelium/Grass share the generated GrassBlockLikeProperties (only snowy, block.rs:583807-583812),
  so from_state_id is correct for every one of them.
- Row 2 (fluid amount generic; :57-63): RESOLVED. Vanilla FluidState.isFull() is getAmount() == 8 for any
  fluid; is_full_fluid reads the normalized fluid_state.level, derived from the LEVEL property of any liquid
  block, so no WATER/LAVA special case remains in this file. No observable gap.
- Row 3 (LAYERS == 1 short-circuit and light block; :70-94): RESOLVED. Vanilla short-circuits on
  SNOW && LAYERS == 1, then LightEngine.getLightDampeningInto(..., aboveState.getLightDampening()) < 15;
  Pumpkin does the same with BlockState::opacity as the light dampening. Equal for all vanilla blocks.
- Row 4 (randomTick coverage; :116-179): RESOLVED by an existing GameTest, not by a new unit test.
  crates/pumpkin-core/src/server/builtin_game_tests/plants_and_support_blocks.rs:315-361
  (test_spreading_snowy_grass_random_tick) builds a stone floor, then calls helper.random_tick on three cases
  and asserts: grass under opaque stone fades to dirt, grass under a single snow layer survives, grass under
  full water fades to dirt. It is registered as pumpkin:grass_random_tick_fade_and_survival (mod.rs:1546-1552)
  and listed in BUILTIN_TEST_IDS (mod.rs:231). The pure seam can_stay_alive_with_above keeps its four unit
  tests (:248-289).

Also re-checked two details of the spread branch against the Paper-patched vanilla: the pos == testPos skip
(Canvas optimisation) is unreachable here because the loop already requires get_block(test_pos) == base_block
while pos holds the spreading block, and BlockSpreadEvent is fired with the same
defaultBlockState.setValue(SNOWY, isSnowySetting(above)). Both match, no edit.

Files touched this pass: this doc only. No .rs edits, so nothing to rustfmt.


## Result (third pass, verify 2026-11-18)

Read the previous Result (pass 4) and re-verified every remaining row directly against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:25-104`,
`SnowyBlock.java:24-48`, plus Pumpkin's helpers. No code changes were needed; the earlier
passes' state already matches vanilla. The stale "Gap status" table was updated to VERIFIED
so no STILL OPEN row remains.

- Row 1 (on_place on grass/mycelium/podzol): VERIFIED, no change. Vanilla `SnowyBlock.getStateForPlacement`
  (SnowyBlock.java:41-44) sets SNOWY from the block above; GrassBlock/MyceliumBlock inherit it and Podzol is
  registered as a plain SnowyBlock. Pumpkin's `SnowyBlock::on_place` mirrors this, so it stays on all three.
- Row 2 (fluid amount generic): VERIFIED, no change. Vanilla `FluidState.isFull()` is `getAmount() == 8` for
  any fluid; `is_full_fluid` reads the normalized `fluid_state.level`, which `fluid_state_from_block_state`
  (crates/pumpkin-world/src/world.rs:140-178) derives from any liquid block's LEVEL property. No WATER/LAVA
  special case in this unit; no observable parity gap.
- Row 3 (LAYERS == 1 short-circuit and light block): VERIFIED, no change. Vanilla `canStayAlive` short-circuits
  on SNOW && LAYERS == 1 (SpreadingSnowyBlock.java:33-35) then returns
  getLightDampeningInto(..., aboveState.getLightDampening()) < 15 (:41-42). Pumpkin uses `BlockState::opacity`
  as the light dampening (block_state.rs:35, "light subtracted through this block") via
  LightEngine::get_light_dampening_into (engine.rs:1084-1099). Both match.
- Row 4 (randomTick coverage): VERIFIED, no change. Live-world fade/survival coverage exists at
  plants_and_support_blocks.rs:315-361 (grass under opaque stone fades to dirt, grass under one snow layer
  survives, grass under full water fades to dirt), registered as pumpkin:grass_random_tick_fade_and_survival
  (builtin_game_tests/mod.rs:231,1546-1552). The pure seam keeps four unit tests (:248-289).

Also re-checked the death branch (always fade on !canStayAlive, SpreadingSnowyBlock.java:68-75) and the
canPropagate argument (defaultBlockState, :95); both match. No new test added: the only pure seam is covered
and the live path has a GameTest.

Files touched this pass: this doc only. No .rs edits, so no rustfmt run.


## Result (third pass, final verify 2026-11-18)

Read the previous Result and re-verified every row directly against the real decompiled
1.21 source now present in this checkout, plus Pumpkin's implementation. No source changes
were needed; the code already matches vanilla. No STILL OPEN row remains.

- Row 1 (on_place on grass/mycelium/podzol; spreading_snowy_block.rs:29-34,192-223): VERIFIED.
  Vanilla `SnowyBlock.getStateForPlacement` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SnowyBlock.java:41-44`)
  sets SNOWY from the block above; `updateShape` (:25-38) does the same on an UP neighbour update.
  GrassBlock and MyceliumBlock inherit it (they do not override getStateForPlacement);
  Podzol is a plain SnowyBlock. Pumpkin's `SnowyBlock::on_place` and
  `get_state_for_neighbor_update` mirror this and are used by Podzol/Mycelium, so they stay.
- Row 2 (fluid amount generic; :58-63): VERIFIED. Vanilla `FluidState.isFull()` is
  `getAmount() == 8` for any fluid, used in `canStayAlive` at
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:37`.
  Pumpkin's `is_full_fluid` checks `!is_empty && level == 8`, where `level` comes from the
  normalized fluid state derived generically from any liquid block's LEVEL property
  (crates/pumpkin-world/src/world.rs), so no WATER/LAVA special case remains. No parity gap.
- Row 3 (LAYERS == 1 short-circuit and light block; :71-94): VERIFIED. Vanilla `canStayAlive`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:31-43`)
  short-circuits on `SNOW && LAYERS == 1` (:33-35), rejects a full fluid (:37-39), then returns
  `LightEngine.getLightDampeningInto(state, aboveState, UP, aboveState.getLightDampening()) < 15`
  (:41-42). Pumpkin `can_stay_alive_with_above` matches each branch, using `BlockState::opacity`
  as the light dampening via `LightEngine::get_light_dampening_into` (engine.rs:1084-1099).
- Row 4 (randomTick coverage; :116-179): VERIFIED by an existing GameTest. The fade branch
  (always fade on !canStayAlive) and canPropagate argument (defaultBlockState) match vanilla
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:68-75, :95`).
  Live-world fade/spread/survival is covered by
  crates/pumpkin-core/src/server/builtin_game_tests/plants_and_support_blocks.rs:316-361
  (grass under opaque stone -> dirt, under one snow layer -> survives, under full water -> dirt),
  registered as `pumpkin:grass_random_tick_fade_and_survival`
  (builtin_game_tests/mod.rs:231,1548-1552). The pure seam `can_stay_alive_with_above` keeps its
  four unit tests (:248-289). No new test added: every pure seam is already covered.

Also re-checked the `pos == testPos` Canvas skip (SpreadingSnowyBlock.java:83-85): unreachable
here because the loop already requires `get_block(test_pos) == base_block` while `pos` holds the
spreading block, so it is correctly omitted. The BlockSpreadEvent/SNOWY payload matches (:96).

Files touched this pass: this doc only. No `.rs` edits, so no rustfmt run.
