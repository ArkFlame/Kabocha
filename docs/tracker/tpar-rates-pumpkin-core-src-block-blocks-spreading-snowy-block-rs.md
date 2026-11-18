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


## Result (FINAL PASS, verify 2026-11-18)

Independently re-read `spreading_snowy_block.rs` and every row in the two referenced audit docs
against the decompiled 1.21 source in `.vendor/vanilla-decompiled/net/minecraft/`. No source change
was needed; the implementation already matches vanilla on every row. No row remains STILL OPEN,
and no row is BLOCKED-DATA (every value comes from existing generated data).

Unit row 1 - `own/taudit-crates-pumpkin-core-src-bl-4bd10c`:
- R1 `BlockFadeEvent` new-state field: FIXED. `BlockFadeEvent` carries `new_state_id: BlockStateId`
  and a `const fn new(pos, block, new_state_id)` (`crates/pumpkin-core/src/plugin/api/events/block/block_fade.rs:8-32`);
  `random_tick` passes `base_block.default_state.id` (`spreading_snowy_block.rs:127-131`), matching
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:71`.
- R2 `is_full_fluid` hardcoded: FIXED. `is_full_fluid` returns `!fluid_state.is_empty && fluid_state.level == 8`
  (`spreading_snowy_block.rs:58-63`), matching `getFluidState().isFull()` (`getAmount() == 8`) at
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:37`.
- R3 opacity not position-aware: VERIFIED. `can_stay_alive_with_above` passes `above_state.opacity`
  into `LightEngine::get_light_dampening_into` (`:87-92`), matching vanilla's `getLightDampeningInto`
  call at `SpreadingSnowyBlock.java:41` and the helper in
  `.vendor/vanilla-decompiled/net/minecraft/world/level/lighting/LightEngine.java:51-60`; the
  heuristic (`engine.rs:1084-1099`) yields the same 16/0 result for every block that can sit above grass.
- S2 `can_propagate` argument: FIXED. `random_tick` passes `default_block_state` (`:152`), matching
  `SpreadingSnowyBlock.java:95`.
- Fade gate: FIXED. The fade branch fires on every `!can_stay_alive` with no brightness/opacity gate
  (`:123-138`), matching `SpreadingSnowyBlock.java:68-75`; the `>= 9` gate stays only on the spread
  branch (`SpreadingSnowyBlock.java:77`).
- S3 `MyceliumBlock.animateTick` particles: not a blocker (client-only visual effect).

Unit row 2 - `own/taudit-crates-pumpkin-core-src-block-bloc`:
- Row 1 on_place on grass/mycelium/podzol: VERIFIED. `SnowyBlock::on_place` / `get_state_for_neighbor_update`
  (`:29-49`, used by Podzol/Mycelium at `:191-223`) mirror `SnowyBlock.getStateForPlacement`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/SnowyBlock.java:41-44`) and `updateShape`
  (`:25-38`).
- Row 2 fluid amount generic: VERIFIED. `!is_empty && level == 8` matches `FluidState.isFull()` for any
  fluid (`SpreadingSnowyBlock.java:37`); normalized fluid state derives `level` from any liquid block.
- Row 3 `LAYERS == 1` short-circuit and light block: VERIFIED. `can_stay_alive_with_above` (`:71-94`)
  matches `canStayAlive` at `.vendor/vanilla-decompiled/net/minecraft/world/level/block/SpreadingSnowyBlock.java:31-43`.
- Row 4 randomTick coverage: VERIFIED. Fade-on-`!canStayAlive`, `getMaxLocalRawBrightness(above) >= 9`,
  4-offset loop, `default_block_state` argument and SNOWY payload match `SpreadingSnowyBlock.java:68-96`;
  live-world coverage at `crates/pumpkin-core/src/server/builtin_game_tests/plants_and_support_blocks.rs:316-361`
  (registered as `pumpkin:grass_random_tick_fade_and_survival`) plus four pure unit tests (`:248-289`).

Files touched this pass: this doc only. No `.rs` edits, so nothing to rustfmt.
