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

