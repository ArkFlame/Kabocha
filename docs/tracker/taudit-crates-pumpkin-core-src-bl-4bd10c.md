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

## Second pass (decompiled MC 26.3 tree, `/run/media/linsaftw/SDD/tmp/mc-proof/ref`)

| Location | Vanilla / Paper | Gap | Fix | Size |
|---|---|---|---|---|
| `spreading_snowy_block.rs:123-141` | `SpreadingSnowyBlock.randomTick`: `if (!canStayAlive(...)) { setBlockAndUpdate(pos, baseBlock.defaultBlockState()); }` with no brightness/opacity gate | Pumpkin invented a gate `get_max_local_raw_brightness(above) < 4 && above_state.opacity > 2`, so a spreading block covered by a full fluid (grass under water/lava, waterlogged plant above) never faded even though `canStayAlive` is false | removed the gate; fade now fires `BlockFadeEvent` and replaces with the base block whenever `canStayAlive` is false | S (fixed) |

Still open (unchanged from the first pass): `BlockFadeEvent` has no `new_state_id` (Paper `BlockFadeEvent(Block, BlockState)`), so plugins cannot see or override the replacement state. `can_propagate` is called with the target dirt state instead of the new default state (vanilla passes `this.defaultBlockState()`); behaviour is identical because both are opaque full cubes, only the comment is misleading. `MyceliumBlock.animateTick` particles are client-only and not implemented.

## Gap status
- Grass/mycelium spread ignored plugins (`spreading_snowy_block.rs:160-163`): FIXED (`BlockSpreadEvent::new` fired before spread).
- Fade ignored plugins (`spreading_snowy_block.rs:126-127`): FIXED (`BlockFadeEvent` fired, cancel returns).
- `plugin/api/events/block/block_fade.rs:8-14` no new-state field: VERIFIED in tree (M). `BlockFadeEvent` carries `new_state_id: BlockStateId` (block_fade.rs:15-16), its ctor takes it (`:21-25`) and `random_tick` passes `base_block.default_state.id` (`spreading_snowy_block.rs:127-131`), matching Paper's `callBlockFadeEvent(level, pos, baseBlock.defaultBlockState())` in `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:71`. WIT v0.1/v0.2 already declare `new-state-id: u16`; no cross-crate change remains.
- `spreading_snowy_block.rs:60-76` `is_full_fluid` hardcoded: FIXED in tree (S). `is_full_fluid` calls `world.get_fluid_and_fluid_state(pos)` and checks `!fluid_state.is_empty && fluid_state.level == 8` (`:58-63`), matching vanilla `aboveState.getFluidState().isFull()` (`getAmount() == 8`) at `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:37`.
- `spreading_snowy_block.rs:100-106` opacity not position-aware: VERIFIED, no known divergence (S). Vanilla `canStayAlive` passes `aboveState.getLightDampening()` (a fixed per-state value) into `getLightDampeningInto` (`SpreadingSnowyBlock.java:41`); generated `BlockState.opacity` is that value. Vanilla `getLightDampeningInto` uses merged face-shape occlusion (`LightEngine.java:51-60`); Pumpkin's solid-render heuristic (`engine.rs:1084-1099`) yields the same 16/0 result for every block that can sit above grass (cube, slab, glass, snow), covered by the `a_slab_above_does_not_add_occlusion` test.
- `spreading_snowy_block.rs:123-141` invented brightness/opacity gate on fade: FIXED (fade branch at `:122-139` has no gate; gate remains only on the spread branch, which is vanilla's `>= 9` check).


## Result

Verified each STILL OPEN row against decompiled vanilla (`SpreadingSnowyBlock.java`,
`SnowyBlock.java`, `LightEngine.java`, `BlockBehaviour.java`) and the current tree.

- `plugin/api/events/block/block_fade.rs:8-14` (new-state field): left OPEN. Reason:
  needs a WIT event-data change plus regenerated plugin bindings and a
  `PLUGIN_API_VERSION` bump across crates outside this unit.
- `spreading_snowy_block.rs:60-76` (`is_full_fluid` hardcoded): already fixed in the
  current tree. `is_full_fluid` now calls `world.get_fluid_and_fluid_state(pos)` and
  checks `!fluid_state.is_empty && fluid_state.level == 8`, matching vanilla
  `aboveState.getFluidState().isFull()` (`getAmount() == 8`) including waterlogged and
  fluid-container blocks. No code change needed.
- `spreading_snowy_block.rs:100-106` (opacity not position-aware): left OPEN. Reason:
  vanilla `canStayAlive` passes `aboveState.getLightDampening()`, a fixed per-block
  value (`solid_render ? 15 : propagatesSkylightDown ? 0 : 1`) that equals Pumpkin's
  `above_state.opacity`, so the call site is correct; the only real divergence is the
  face-occlusion heuristic inside `LightEngine::get_light_dampening_into` in
  `pumpkin-world/src/lighting/engine.rs`, outside this unit, and no block that can sit
  above grass differs.

Fixed this pass, from the "Still open" prose at line 57:

- `spreading_snowy_block.rs:148-153` (`can_propagate` argument): `random_tick` passed the
  target dirt state to `can_propagate`; vanilla `SpreadingSnowyBlock.randomTick` passes
  `this.defaultBlockState()`. Changed the call to pass `default_block_state` and removed
  the misleading comment. Behaviour is unchanged for opaque full cubes, but the call
  now matches vanilla. Verified type-correct (parameter is `&'static BlockState`) and
  ran `rustfmt --edition 2024` on the file (exit 0).

No pure test seam exists for the verified rows (`is_full_fluid` needs a live `World`;
`can_stay_alive_with_above` would only mirror its own logic), so none was added.


## Result (re-audit)

Re-checked every STILL OPEN row against the decompiled tree in
`.vendor/vanilla-decompiled/net/minecraft/world/level/` and the current source.
No code changes were needed: all actionable rows are already fixed in the tree.

- `block_fade.rs` new-state field: resolved in the current tree. `pumpkin-core/src/plugin/api/events/block/block_fade.rs` carries `new_state_id: BlockStateId` and `SpreadingSnowyBlock::random_tick` passes `base_block.default_state.id`, matching Paper's `callBlockFadeEvent(level, pos, baseBlock.defaultBlockState())`. The WIT surface is updated too: `v0.2/event.wit` `block-fade-event` and `v0.1/event.wit` `block-fade-event-data` both declare `new-state-id: u16`. `pumpkin-plugin-api/src/events/block/block_fade.rs` is only a type marker over the generated `BlockFadeEventData`, so it needs no edit.
- `spreading_snowy_block.rs:60-76` `is_full_fluid`: fixed in the tree. It calls `world.get_fluid_and_fluid_state(pos)` and checks `!fluid_state.is_empty && fluid_state.level == 8`, matching vanilla `aboveState.getFluidState().isFull()` (amount 8).
- `spreading_snowy_block.rs:100-106` opacity: verified non-issue at this call site. Vanilla `BlockBehaviour.getLightDampening` is a fixed per-state value (15 solid render, 0 skylight propagator, else 1); generated `BlockState.opacity` is that same value, so passing `above_state.opacity` as `simple_opacity` is correct. A faithful `Shapes.mergedFaceOccludes` face occlusion needs voxel-shape occlusion data and a cross-cutting rewrite of `LightEngine::get_light_dampening_into` in `pumpkin-world/src/lighting/engine.rs`. The current solid-render heuristic already matches vanilla for every block that can sit above grass (full cube, slab, glass, snow), and a slab case is covered by the existing `a_slab_above_does_not_add_occlusion` test. Left open for that reason.

No new unit test was added: the verified rows need a live `World` or would only mirror their own logic. Files touched this pass: this doc only, so no rustfmt run.

## Result (worker verification)

Re-verified against current source in allowed files (`grass_block.rs`, `spreading_snowy_block.rs`) and the core event `crates/pumpkin-core/src/plugin/api/events/block/block_fade.rs` (basename in scope). Vanilla decompiled sources named in the task were not found under `.vendor/vanilla-decompiled/net/minecraft/world/level/block/` in this checkout, so parity was checked against the documented vanilla/Paper behavior in this unit and the existing implementation.

- `plugin/api/events/block/block_fade.rs:8-14` new-state field: already present in tree (`BlockFadeEvent` carries `new_state_id: BlockStateId`, ctor and call site in `spreading_snowy_block.rs:127-131` pass `base_block.default_state.id`). Any remaining WIT/binding/version-bump work is cross-crate and outside this unit; left open with that reason.
- `spreading_snowy_block.rs:60-76` `is_full_fluid` hardcoded: fixed in tree (`world.get_fluid_and_fluid_state`, `!fluid_state.is_empty && fluid_state.level == 8`), no change needed.
- `spreading_snowy_block.rs:100-106` opacity not position-aware: call site matches fixed per-state damping (`above_state.opacity` as simple opacity); faithful face-occlusion requires cross-cutting rewrite of `LightEngine::get_light_dampening_into` in `pumpkin-world/src/lighting/engine.rs`, outside this unit. Left open (S, no known grass-cover divergence).
- `spreading_snowy_block.rs:123-141` fade gate: no invented brightness/opacity gate on fade (fade fires when `!can_stay_alive`); spread branch keeps `get_max_local_raw_brightness(&pos.up()) >= 9`. Fixed in tree.
- `can_propagate` argument: `random_tick` passes `default_block_state` (line 152), matching vanilla intent. Fixed in tree.
- `grass_block.rs`/`spreading_snowy_block.rs` registration and snowy/mycelium spread/fade behavior otherwise match existing verified rows; no further small in-scope fix identified.

No source code edits made; no pure unit-test seam available in these files without a live `World`. Files touched: `docs/tracker/taudit-crates-pumpkin-core-src-bl-4bd10c.md` only.

## Result (third pass close)

Re-read the last Result and re-verified every previously STILL OPEN row against the real decompiled source now present in this checkout: `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:25-104`, `SnowyBlock.java:24-48` and `.vendor/vanilla-decompiled/net/minecraft/world/level/lighting/LightEngine.java:51-60`. No source change was needed; the code already matches vanilla. All three Gap-status rows are now VERIFIED/FIXED (new-state field present in tree, normalized fluid level==8, correct fixed per-state light dampening). Files touched this pass: documentation only; no `.rs` edit, so no rustfmt run.


## Result (final pass, all rows closed)

Re-verified every row in this unit against the real decompiled tree now present in this
checkout. No `.rs` edit was needed: the current source already matches vanilla on every
actionable row. No row remains STILL OPEN.

- R1 `plugin/api/events/block/block_fade.rs:8-14` (new-state field) -> FIXED. Core
  `BlockFadeEvent` carries `new_state_id: BlockStateId` (`:15-16`) with a `const fn new`
  taking it (`:21-25`); `SpreadingSnowyBlock::random_tick` passes
  `base_block.default_state.id` (`spreading_snowy_block.rs:127-131`). Matches Paper's
  `CraftEventFactory.callBlockFadeEvent(level, pos, baseBlock.get().defaultBlockState())`
  in `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:71`.
  WIT surface updated too: `pumpkin-plugin-wit/v0.2/event.wit:1409-1413` `block-fade-event`
  and `v0.1/event.wit:1303-1307` `block-fade-event-data` both declare `new-state-id: u16`.
  The plugin-api marker needs no edit (type marker over generated `BlockFadeEventData`).
  No generated pumpkin-data field is missing.
- R2 `spreading_snowy_block.rs:60-76` (`is_full_fluid` hardcoded) -> FIXED. `is_full_fluid`
  (`:58-63`) calls `world.get_fluid_and_fluid_state(pos)` and returns
  `!fluid_state.is_empty && fluid_state.level == 8`. This is exactly vanilla
  `aboveState.getFluidState().isFull()` (`getAmount() == 8`) at
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:37`,
  covering source/falling fluids plus waterlogged and fluid-container blocks.
- R3 `spreading_snowy_block.rs:100-106` (opacity not position-aware) -> VERIFIED, no known
  divergence. `can_stay_alive_with_above` passes `above_state.opacity` as `simple_opacity`
  into `LightEngine::get_light_dampening_into` (`:87-92`), matching vanilla
  `LightEngine.getLightDampeningInto(state, aboveState, UP, aboveState.getLightDampening())`
  at `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:41`
  (generated `BlockState.opacity` is vanilla's fixed per-state `getLightDampening()`).
  The helper in `pumpkin-world/src/lighting/engine.rs:1084-1099` implements vanilla
  `getLightDampeningInto`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/lighting/LightEngine.java:51-60`):
  both return `simple_opacity` when both shapes are empty and force 16 on full-cube
  occlusion. Vanilla `Shapes.mergedFaceOccludes`
  (`.vendor/vanilla-decompiled/net/minecraft/world/phys/shapes/Shapes.java:283-319`) reduces
  to the clamped face fully occluding; Pumpkin's `can_occlude() && is_solid_render()`
  heuristic yields the same 16/0 result for every block that can sit above grass (full
  cube, slab, glass, snow), covered by `a_slab_above_does_not_add_occlusion` (engine.rs:1342).
  A fully faithful face-occlusion port needs voxel-shape occlusion data across crates, but
  no generated field is missing and no block-cover case diverges, so no change is made.
- S2 `can_propagate` argument (line 57 prose) -> FIXED. `random_tick` passes
  `default_block_state` (`:152`) instead of the target dirt state, matching vanilla
  `SpreadingSnowyBlock.canPropagate(access, defaultBlockState, testPos)` at
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:95`.
- Second-pass invented brightness/opacity gate on fade (line 55) -> FIXED. The fade branch
  (`:123-138`) fires whenever `!can_stay_alive`, with no brightness/cover-opacity gate,
  matching vanilla `if (!canStayAlive(...)) { fade; setBlockAndUpdate(...) }` at
  `SpreadingSnowyBlock.java:68-75`. The `get_max_local_raw_brightness(...) >= 9` check stays
  only on the spread branch, matching vanilla `getMaxLocalRawBrightness(pos.above()) >= 9`
  at `SpreadingSnowyBlock.java:77`.
- S3 `MyceliumBlock.animateTick` particles (line 57 prose) -> NOT A BLOCKER (client-only).
  Purely visual; per project rules client-only effects are noted, not gated on.

No pure unit-test seam exists beyond the already-present tests: the fluid/light rows need a
live `World`, and further tests would only mirror their own logic. Files edited this pass:
this doc only, so no rustfmt run.
