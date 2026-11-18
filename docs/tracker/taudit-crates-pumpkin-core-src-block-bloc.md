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
- `spreading_snowy_block.rs:30-36,177-179,199-201` no `getStateForPlacement` on grass/mycelium: STILL OPEN (S; vanilla relies on base class, verify before change).
- `:57-64` `get_amount` generic for any fluid: STILL OPEN (M; world fluid normalisation is WATER/LAVA only).
- `:80-95` `LAYERS == 1` short-circuit and light block: STILL OPEN (S).
- `:121-150` randomTick test coverage: STILL OPEN (needs `pumpkin-gametest` scenario; no unit test possible).

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
