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
