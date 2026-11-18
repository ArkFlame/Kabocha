# blocks / Note Block

Audit 2026-11-16 against Paper 1.21.9 `net.minecraft.world.level.block.NoteBlock`
(reference source under `../fc-verify-worktree/.gradle/caches/paperweight/upstreams/server-work/paper/file/src/minecraft/java/net/minecraft/world/level/block/NoteBlock.java`;
no `.vendor` copy in this worktree).

Source: `crates/pumpkin-core/src/block/blocks/note.rs`.

Matched vanilla behaviour: `setInstrument` derivation (above head/instrument, below
fallback to harp), Y-axis-only shape update, redstone `POWERED` toggle with note
played on rising edge, tune/attack stats, note pitch (`2^((note-12)/12)`), head-on-top
item passthrough (`noteblock_top_instruments` + UP face), custom head sound lookup and
silent failure, all copper trumpet instruments treated as tunable base blocks.

## Fixed in this pass
- `note.rs:161` was missing the vanilla NOTE particle on every tunable playback
  (`NoteBlock#triggerEvent`: `level.addParticle(ParticleTypes.NOTE, x+0.5, y+1.2, z+0.5, note/24.0, 0.0, 0.0)`).
  Now sent with `World::spawn_particle`, velocity x = note/24, speed 0, count 1.

## Remaining gaps
1. `note.rs:193` custom head sound is sent as `SoundEvent { range: None }`; vanilla uses
   `SoundEvent.createVariableRangeEvent(id)`, which resolves the range from the sound
   registry (16 by default). Only affects client audible distance for a custom head
   sound registered with a non-default range. Vanilla class `NoteBlock#triggerEvent`.
   Suggested fix: resolve the registry range when building the `SoundEvent`. Size S.
2. `note.rs:37` `NotePlayEvent` reports the instrument as the Rust `Debug` name
   (`TrumpetExposed`) instead of the vanilla serialized id (`trumpet_exposed`).
   Plugin API surface only, no vanilla gameplay effect. Vanilla class
   `NoteBlock#playNote` / `NoteBlockInstrument#getSerializedName`.
   Suggested fix: add a serialized-id helper on `NoteblockInstrument`. Size S.

## h6 gap status
- Row 2 (`NotePlayEvent` instrument serialized id): FIXED in tree at HEAD (`note.rs:37` passes `to_value()`, the serialized id).
- Row 1 (custom head sound range): STILL OPEN, needs the sound registry range lookup.

## Result
- Row 1 (custom head sound range): STILL OPEN premise is factually wrong; verified NO CHANGE NEEDED.
  NoteBlock#triggerEvent wraps custom head sounds in Holder.direct(SoundEvent.createVariableRangeEvent(id))
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/NoteBlock.java:172).
  SoundEvent.createVariableRangeEvent(id) returns new SoundEvent(id, Optional.empty())
  (net/minecraft/sounds/SoundEvent.java:35-37): no fixed range and no sound-registry lookup.
  SoundEvent.getRange(volume) computes 16, or 16 * volume when volume > 1, at play time (:43-45).
  Pumpkin's range: None (note.rs:186) passes unchanged to the protocol as an absent fixed range
  (pumpkin-protocol/src/codec/data_component.rs:39), so it already matches vanilla. Setting a fixed
  range would diverge. Added a short comment so the quirk is not fixed incorrectly later.
- Rows: fixed=0, open=0. No test added: no behaviour change, and on_synced_block_event needs a live
  World so there is no pure seam in this file.
- Re-verified 2026-11-18 against `.vendor/vanilla-decompiled`: NoteBlock.java:172 uses
  `Holder.direct(SoundEvent.createVariableRangeEvent(soundId))`; SoundEvent.java:35-37 returns
  `Optional.empty()` with no registry lookup, range computed from volume at play time (:43-45).
  Pumpkin `range: None` (note.rs:188) writes an absent fixed range (sound_effect.rs:51-54), matching
  vanilla. No change needed; leaving the explanatory comment in place. data_component.rs sound
  converters copy `range` verbatim (:34-59), so no fix or pure test seam there.
