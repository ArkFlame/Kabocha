# audit / crates/pumpkin-core/src/block/blocks/spreading_snowy_block.rs

Source: `crates/pumpkin-core/src/block/blocks/spreading_snowy_block.rs` (220 lines),
callers `grass_block.rs`, registry. Vanilla compared against the decompiled tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/level/block`
(`SnowyBlock.java`, `SpreadingSnowyBlock.java`, `GrassBlock.java`, `MyceliumBlock.java`)
plus Paper/Canvas patches in the same sources.

## Markers

The tracker counted 4 "TODO/FIXME/vanilla" hits: all four are doc comments naming the
vanilla class (`:18`, `:55`, `:167`, `:189`). There is no TODO or FIXME in the file.

## Verified against vanilla (no change needed)

- `SnowyBlock.isSnowySetting`: vanilla `aboveState.is(BlockTags.SNOW)`; Pumpkin uses the
  `minecraft:snow` block tag, which contains `snow`, `snow_block`, `powder_snow`
  (`tag.rs:17797`).
- `SnowyBlock.getStateForPlacement` / `updateShape`(UP only) match `on_place` and
  `get_state_for_neighbor_update` (`:31-51`); non-UP directions return the state, which is
  `BlockBehaviour#updateShape`'s default.
- `canStayAlive`: snow layer 1 first, then full fluid, then light dampening `< 15`
  (`:84-107`); vanilla uses exactly this order and `LightEngine.getLightDampeningInto(
  state, aboveState, UP, aboveState.getLightDampening())`.
- `canPropagate`: `canStayAlive && !above fluid is water` (`:120-127`); the fluid tag
  `minecraft:water` includes `water` and `flowing_water` (`tag.rs:20790`).
- `randomTick` bounds (`nextInt(3)-1`, `nextInt(5)-3`, `0..3`, four attempts), brightness
  `>= 9`, target `Blocks.DIRT`, replacement `this.defaultBlockState()` with `snowy` from
  the target's above block (`:129-164`); `MyceliumBlock`/`GrassBlock` both pass `DIRT`
  (previously fixed in `0a9ddfd22`).

## Fixed in this pass

| Gap | Rust | Vanilla / Paper | Fix |
|---|---|---|---|
| Grass/mycelium spreading ignored plugins | `spreading_snowy_block.rs:129-176` | Paper `CraftEventFactory.handleBlockSpreadEvent(level, pos, testPos, newState, UPDATE_ALL)` | fires `BlockSpreadEvent` before the spread and skips the target when cancelled |
| Fade back to dirt ignored plugins | `spreading_snowy_block.rs:136-149` | Paper `CraftEventFactory.callBlockFadeEvent(level, pos, baseBlock.defaultBlockState())` | fires `BlockFadeEvent` and returns when cancelled |

## Remaining gaps

| Location | Vanilla / Paper | Gap | Fix | Size |
|---|---|---|---|---|
| `plugin/api/events/block/block_fade.rs:8-14` | `BlockFadeEvent(Block, BlockState)`; `getNewState()` | Pumpkin's fade event carries only the fading block, so plugins cannot see that grass is about to become dirt (the new state our `random_tick` passes) | add `new_state_id: BlockStateId`, mirror in `pumpkin-plugin-api` + WIT, bump `PLUGIN_API_VERSION` | M |
| `spreading_snowy_block.rs:60-76` | `aboveState.getFluidState().isFull()` | `is_full_fluid` hardcodes water/lava levels plus the liquid-container plant list, duplicating `pumpkin_world::world::fluid_from_block_state`; correct for 1.21 and drifts if a new liquid container is added | replace with `world.get_fluid_and_fluid_state(pos).1.level == 8` | S |
| `spreading_snowy_block.rs:100-106` | `LightEngine.getLightDampeningInto(state, aboveState, UP, aboveState.getLightDampening())` | passes `above_state.opacity` (state-independent) into the const helper; position-aware `getLightDampening` is not modelled | none known to differ for blocks that can sit above grass | S |

Client-only, not implemented: vanilla `MyceliumBlock.animateTick` particles. Canvas'
`pos == testPos` skip in the spread loop is a pure optimization; Pumpkin's dirt check
already rejects `pos` itself.
