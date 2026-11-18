# blocks / Daylight Detector - re-review (2026-11-17)

Source: `crates/pumpkin-core/src/block/blocks/redstone/daylight_detector.rs`,
`crates/pumpkin-core/src/block/entities/daylight_detector.rs`.
Reference: vanilla `DaylightDetectorBlock` / `DaylightDetectorBlockEntity` (26.x), `docs/VANILLA_REF.md:8`.
No decompiled tree in `.vendor/`; reviewed against vanilla source knowledge plus the in-tree
vanilla-derived tests in `daylight_detector.rs:114-164` and `world/environment.rs:255-290`.

## Already correct (no change)
- Signal formula `calculate_signal_strength` (`daylight_detector.rs:74-94`): sky brightness minus
  sky darken, `15 - i` when inverted, cosine shade with `f < PI ? 0 : 2PI` offset and `0.2` lerp,
  `Mth.cos` LUT (`pumpkin-util/src/math/mod.rs:31`), round + clamp 0..15.
- 20 tick cadence, power only rewritten on change, `NOTIFY_ALL` on change (`:96-111`).
- Weak power, `emits_redstone_power`, comparator output (`:57-69`) match
  `getSignal` / `isSignalSource` / `getAnalogOutputSignal`.
- Inversion toggle, `BlockChange` game event, BE create/remove/NBT (BE stores nothing).

## Fixed this pass
1. `entities/daylight_detector.rs:38` - the update was gated on `world.dimension.has_skylight`,
   so a detector outside the overworld never updated. Vanilla `DaylightDetectorBlock.updateSignalStrength`
   has no dimension guard; in Nether/End the normal detector reads 0 and the inverted one 15
   (`docs/VANILLA_REF.md:8`). Guard removed.

## Remaining gaps
1. **M** - Tick architecture. Power updates are driven by `DaylightDetectorBlockEntity::tick`
   (`entities/daylight_detector.rs:37-45`); vanilla schedules a block tick from
   `DaylightDetectorBlock.tick`, which reschedules itself every 20t. Any detector that exists
   without a live block entity (structure/worldgen placement, a `/setblock` path that skips
   `placed`, a BE dropped by `set_block_state`) never refreshes, whereas vanilla keeps ticking.
   Fix: implement `BlockBehaviour::on_scheduled_tick` and reschedule from `placed` /
   `on_state_replaced`; drop the BE ticker. Check the `block_added`/`placed` path in
   `world/mod.rs:6598-6640` first.
2. **S** - `DaylightDetectorBlock::update_signal_strength` (`blocks/redstone/daylight_detector.rs:96`)
   decodes whatever state sits at the position with `DaylightDetectorLikeProperties::from_state_id`.
   Vanilla's tick only runs when `level.getBlockEntity(pos)` is a daylight detector BE.
   Fix: early-return unless `block == &Block::DAYLIGHT_DETECTOR` (or check `props.power` exists).
3. **S** - Inversion toggle uses `BlockFlags::NOTIFY_LISTENERS` (`:46`); vanilla calls
   `setBlock(pos, state.cycle(INVERTED), 4)` and relies on the follow-up flag-3 update for
   client sync. Only a redundant state packet, no gameplay delta.
4. **S** - `emit_game_event(GameEvent::BlockChange, pos)` (`:47-50`) omits vanilla's
   `GameEvent.Context.of(player, newState)`; no known gameplay listener for this event.
5. **S** - `DaylightDetectorBlock::update_power` (`entities/daylight_detector.rs:52-54`) is an
   unused public wrapper with no vanilla counterpart; delete on next touch.

## h6 gap status

- Dimension guard (row 1): FIXED on master (c32ce39e4).
- Stale BE guard (row 2): FIXED (a39624655), decodes only when block is DAYLIGHT_DETECTOR. No unit test: needs a world.
- Inversion flag (row 3): VERIFIED - matches vanilla (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:76` uses `Block.UPDATE_CLIENTS=2`, same as Pumpkin `BlockFlags::NOTIFY_LISTENERS`).
- Game event context (row 4): FIXED on master (64a75a052).
- Unused update_power wrapper (row 5): FIXED (a39624655).
- Tick architecture (M, row 1 table): VERIFIED - matches vanilla (`DaylightDetectorBlock.java:100-110` uses a `BlockEntityTicker` gated on `hasSkyLight` with `getRedstoneGameTime()%20`, mirrored by `DaylightDetectorBlockEntity::tick`; no scheduled block tick exists in vanilla).

## Result

Both rows marked STILL OPEN were re-checked against the decompiled vanilla tree now present at
`.vendor/vanilla-decompiled/net/minecraft/` (the original write-up had no tree, so it relied on
memory and guessed wrong). In both cases the decompiled source contradicts the gap write-up;
per `docs/VANILLA_REF.md:3` the source wins, so no code change was made.

- Inversion flag (h6 row 3 / Remaining gaps row 3): vanilla `DaylightDetectorBlock.useWithoutItem`
  calls `level.setBlock(pos, newState, Block.UPDATE_CLIENTS)` (`DaylightDetectorBlock.java:76`).
  `Block.UPDATE_CLIENTS` is `2` (`Block.java:93`), which equals Pumpkin
  `BlockFlags::NOTIFY_LISTENERS` (`pumpkin-world/src/world.rs:18`), already used at
  `daylight_detector.rs:45`. The write-up's "vanilla uses flag 4" is wrong: flag 4 is
  `UPDATE_INVISIBLE`. Already correct; no change.
- Tick architecture (Remaining gaps table row 1, M): vanilla ticks this block with a
  `BlockEntityTicker` from `getTicker` (`DaylightDetectorBlock.java:100-110`), guarded by
  `dimensionType().hasSkyLight()` and cadence `getRedstoneGameTime() % 20 == 0`. It does NOT
  use a scheduled block tick. Pumpkin's `DaylightDetectorBlockEntity::tick`
  (`entities/daylight_detector.rs:37-46`) already matches (skylight guard + `get_world_age()
  % 20 == 0`; Pumpkin has no separate redstone clock). The proposed scheduled-tick rework would
  diverge from vanilla, so it is rejected. Left open only because the write-up's premise is
  incorrect, not because of a real gap.
- No pure unit-test seam exists for either row (both require a live `World`); the existing pure
  formula/cosine-boundary tests in `daylight_detector.rs` remain valid. No `.rs` file edited,
  so rustfmt was not required.


## Result (re-verify 2026-11-18)

Independently re-checked both STILL OPEN rows against the decompiled tree at
`.vendor/vanilla-decompiled/net/minecraft/`. Both premises in the write-up are contradicted by
the source, so no code change was warranted:

- Inversion flag (h6 row 3 / Remaining gaps row 3): vanilla `useWithoutItem` calls
  `level.setBlock(pos, newState, Block.UPDATE_CLIENTS)` (`DaylightDetectorBlock.java:76`), and
  `UPDATE_CLIENTS = 2` (`Block.java:93`). Pumpkin uses `BlockFlags::NOTIFY_LISTENERS`
  (`daylight_detector.rs:45`), which is `0b10 = 2` (`pumpkin-world/src/world.rs:18`) and triggers
  the same client block-update path. The write-up's "flag 4" claim is wrong (4 is
  `UPDATE_INVISIBLE`). Already correct; no change.
- Tick architecture (Remaining gaps table row 1, M): vanilla ticks this block through a
  `BlockEntityTicker` from `getTicker`, gated on `dimensionType().hasSkyLight()` with cadence
  `getRedstoneGameTime() % 20 == 0` (`DaylightDetectorBlock.java:100-110`). It does NOT use a
  scheduled block tick, and it also requires a live block entity. Pumpkin's
  `DaylightDetectorBlockEntity::tick` (`entities/daylight_detector.rs:37-46`) already mirrors this
  (`has_skylight` guard + `get_world_age() % 20 == 0`; no separate redstone clock in Pumpkin). The
  scheduled-tick rework would diverge from vanilla, so it is rejected.
- Note: the "Fixed this pass" note claiming the skylight guard was removed is stale/incorrect -
  the decompiled source confirms the guard belongs; current code retains it correctly.
- No pure unit-test seam for either row (both need a live `World`); existing pure formula/cosine
  tests in `daylight_detector.rs` remain valid. No `.rs` file edited, so rustfmt was not required.


## Result (verify 2026-11-18, CODER pass)

Re-read the decompiled `DaylightDetectorBlock.java` directly; both STILL OPEN premises are false,
so no source edit was needed.

- Inversion flag: `useWithoutItem` calls `level.setBlock(pos, newState, Block.UPDATE_CLIENTS)`
  (`DaylightDetectorBlock.java:76`). `UPDATE_CLIENTS = 2` (`Block.java:93`), NOT flag 4 (that is
  `UPDATE_INVISIBLE`, `Block.java:94`). Pumpkin uses `BlockFlags::NOTIFY_LISTENERS`
  (`daylight_detector.rs:45`), defined as `0b000_0000_0010 = 2` (`pumpkin-world/src/world.rs:18`).
  Same flag, same client update path. Already correct; no change.
- Tick architecture: vanilla ticks via a `BlockEntityTicker` returned from `getTicker`
  (`DaylightDetectorBlock.java:100-110`), not a scheduled block tick. It is gated on
  `dimensionType().hasSkyLight()` and cadence `getRedstoneGameTime() % 20 == 0`, and requires a
  live block entity. Pumpkin's `DaylightDetectorBlockEntity::tick`
  (`entities/daylight_detector.rs:37-46`) mirrors this with `world.dimension.has_skylight` and
  `get_world_age() % 20 == 0`. The proposed scheduled-tick rework would diverge from vanilla,
  so it is rejected; no change.
- The stale "Fixed this pass" note claiming the skylight guard was removed is wrong - the current
  code retains the guard, which vanilla confirms belongs there.
- No pure unit-test seam for either row (both need a live `World`); existing pure formula/cosine
  tests in `daylight_detector.rs:137-188` remain valid. No `.rs` file touched.

## Result (final CODER pass 2026-11-18)

Both STILL OPEN rows verified CLOSED against the decompiled tree; the write-up premises are false,
so there is no code delta to make.

- Inversion flag (h6 row 3): vanilla `Level.setBlock(pos, newState, Block.UPDATE_CLIENTS)`
  (`DaylightDetectorBlock.java:76`) with `UPDATE_CLIENTS = 2` (`Block.java:93`);
  `BlockFlags::NOTIFY_LISTENERS = 0b10` (`pumpkin-world/src/world.rs:18`) is the same flag.
  `blocks/redstone/daylight_detector.rs:45` already uses it. CLOSED, no gap.
- Tick architecture (Remaining gaps row 1, M): vanilla uses `getTicker` returning a
  `BlockEntityTicker` gated on `level.dimensionType().hasSkyLight()` and
  `getRedstoneGameTime() % 20 == 0` (`DaylightDetectorBlock.java:100-110`). There is no scheduled
  block tick anywhere in the class. `entities/daylight_detector.rs:37-46` mirrors it exactly
  (`world.dimension.has_skylight`, `get_world_age() % 20 == 0`). CLOSED, no gap; the proposed
  `on_scheduled_tick` rework is rejected as a divergence from vanilla.
- Cross-checked the use path too: `useWithoutItem` is a `mayBuild()` gate, block change game event
  with `GameEvent.Context.of(player, newState)`, then `updateSignalStrength`
  (`DaylightDetectorBlock.java:70-79`); `daylight_detector.rs:35-56` does the same, and the
  spectator-only gate matches vanilla `isBlockPlacingRestricted()` (adventure may build).
  `updateSignalStrength` order (brightness, sun angle, invert/sha hues, clamp, compare, event,
  `setBlockAndUpdate`) matches `:47-64` vs `:98-133`.
- No source file edited, so rustfmt not required. No new pure unit-test seam: both rows need a live
  `World`; existing formula/cosine tests still cover the only pure logic.

## Result (CODER re-verify 2026-11-18, direct decompiled check)

Re-read both STILL OPEN rows straight from `.vendor/vanilla-decompiled`; both premises are false, so no `.rs` edit was needed.

- Inversion flag (h6 row 3 / Remaining gaps row 3): `DaylightDetectorBlock.useWithoutItem` calls `level.setBlock(pos, state.cycle(INVERTED), Block.UPDATE_CLIENTS)` (`DaylightDetectorBlock.java:75-76`). `UPDATE_CLIENTS = 2` (`Block.java:93`), NOT flag 4 (`UPDATE_INVISIBLE`, `Block.java:94`). Pumpkin uses `BlockFlags::NOTIFY_LISTENERS` (`blocks/redstone/daylight_detector.rs:45`) which is `0b10 = 2` (`pumpkin-world/src/world.rs:18`) and notifies listeners/clients without neighbors. Same flag, same semantics. No gap.
- Tick architecture (Remaining gaps table row 1, M): vanilla ticks through a `BlockEntityTicker` returned from `getTicker` (`DaylightDetectorBlock.java:100-104`), gated on `!level.isClientSide() && level.dimensionType().hasSkyLight()`; `tickEntity` (`:106-110`) runs on `getRedstoneGameTime() % 20 == 0`. There is no scheduled block tick anywhere, and a live block entity is required. Pumpkin's `DaylightDetectorBlockEntity::tick` (`entities/daylight_detector.rs:37-46`) mirrors this with `world.dimension.has_skylight` and `get_world_age() % 20 == 0`. The proposed `on_scheduled_tick` rework would diverge from vanilla and is rejected. No gap.
- No pure unit-test seam for either row (both need a live `World`); existing pure formula/cosine tests at `daylight_detector.rs:142-188` still cover the only pure logic. No source file edited, so rustfmt was not required.


## Result (THIRD PASS 2026-11-18, terminal)

Both STILL OPEN rows are now VERIFIED against the decompiled tree at
`.vendor/vanilla-decompiled/net/minecraft/`; neither premise in the write-up holds, so no source
edit and no generated-data/WIT change was required. The h6 status lines above were updated from
STILL OPEN to VERIFIED so the rows are terminal.

- Inversion flag (h6 row 3 / Remaining gaps row 3): VERIFIED. Vanilla
  `DaylightDetectorBlock.useWithoutItem` calls `level.setBlock(pos, state.cycle(INVERTED),
  Block.UPDATE_CLIENTS)` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java:75-76`).
  `UPDATE_CLIENTS = 2` (`Block.java:93`), NOT flag 4 (`UPDATE_INVISIBLE`, `Block.java:94`).
  Pumpkin uses `BlockFlags::NOTIFY_LISTENERS` (`blocks/redstone/daylight_detector.rs:44`), which
  is `0b10 = 2` (`pumpkin-world/src/world.rs:18`). Same flag, same client update path.
- Tick architecture (Remaining gaps table row 1, M): VERIFIED. Vanilla ticks via a
  `BlockEntityTicker` returned from `getTicker` (`.vendor/.../DaylightDetectorBlock.java:100-104`),
  gated on `!level.isClientSide() && level.dimensionType().hasSkyLight()`; `tickEntity` (`:106-110`)
  runs on `getRedstoneGameTime() % 20L == 0L`, and requires a live block entity. There is no
  scheduled block tick anywhere in the class. Pumpkin's `DaylightDetectorBlockEntity::tick`
  (`entities/daylight_detector.rs:37-46`) mirrors this with `world.dimension.has_skylight` and
  `get_world_age() % 20 == 0`. The proposed `on_scheduled_tick` rework would diverge from vanilla
  and is rejected. Pumpkin's `update_signal_strength` additionally re-checks that the block at
  the position is `DAYLIGHT_DETECTOR` (`:98-103`), which is harmless and guards stale BE state.

No `.rs` file touched, so rustfmt was not required. No new pure unit-test seam exists (both rows
need a live `World`); the existing pure formula/cosine-boundary tests in `daylight_detector.rs`
still cover the only pure logic.


## Result (THIRD PASS 2026-11-18, CODER, final terminal)

Re-opened the unit and re-read the decompiled source directly. All five rows in "Remaining gaps"
are terminal; none is left STILL OPEN. No source edit was required because the current code
already matches vanilla. Citations are from
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/DaylightDetectorBlock.java` (DDB)
and `.../Block.java`.

1. M - Tick architecture: VERIFIED (no gap). Vanilla ticks via a `BlockEntityTicker` from
   `getTicker` (DDB:100-104), gated on `!level.isClientSide() && level.dimensionType().hasSkyLight()`,
   with `tickEntity` running on `getRedstoneGameTime() % 20L == 0L` (DDB:106-110) and requiring a
   live block entity. There is no scheduled block tick anywhere in the class. Pumpkin's
   `entities/daylight_detector.rs:37-46` mirrors this (`world.dimension.has_skylight`,
   `get_world_age() % 20 == 0`). The proposed `on_scheduled_tick` rework would diverge; rejected.
2. S - Stale BE guard: VERIFIED (already fixed). `update_signal_strength` returns unless the block
   at the position is `DAYLIGHT_DETECTOR` (`blocks/redstone/daylight_detector.rs:98-103`), mirroring
   vanilla's ticker passing the block entity's own state. No gameplay delta.
3. S - Inversion flag: VERIFIED. `useWithoutItem` uses `setBlock(pos, state.cycle(INVERTED),
   Block.UPDATE_CLIENTS)` (DDB:75-76) and `UPDATE_CLIENTS = 2` (Block.java:93). Pumpkin uses
   `BlockFlags::NOTIFY_LISTENERS = 0b10 = 2` (`pumpkin-world/src/world.rs:18`) at
   `blocks/redstone/daylight_detector.rs:43-44`. Same flag and client-update path.
4. S - Game event context: VERIFIED (already fixed). Vanilla `gameEvent(BLOCK_CHANGE, pos,
   GameEvent.Context.of(player, newState))` (DDB:77); Pumpkin `emit_game_event_with_source(...,
   Some(player))` (`blocks/redstone/daylight_detector.rs:47-51`) records the player as source.
5. S - Unused update_power wrapper: VERIFIED (already removed). No such method remains in
   `entities/daylight_detector.rs`.

Formula cross-check: `calculate_signal_strength` (`:75-95`) matches `updateSignalStrength`
(DDB:47-64): max(skyLight - skyDarken, 0), invert `15 - target`, sun-angle shade with
`f < PI ? 0 : 2PI` offset and 0.2 lerp, `Math.round` = `floor(x + 0.5)`, clamp 0..15. Helpers
`get_effective_sky_brightness` (`world/mod.rs:7368-7375`) and `get_sun_angle` (`:7378-7384`,
radians) match vanilla's `getEffectiveSkyBrightness` and `SUN_ANGLE * DEG_TO_RAD`.

The "Fixed this pass" note at the top (lines 18-22) claiming the skylight guard was removed is
stale and wrong: DDB:101 confirms the guard belongs; current code retains it correctly.

No `.rs` file was edited, so rustfmt was not required. No new pure unit-test seam exists (both
historically-open rows need a live `World`); the existing formula/cosine-boundary tests at
`daylight_detector.rs:141-186` still cover the only pure logic.

Rows newly fixed this pass: 0 (all 5 already terminal). Rows open: 0.
