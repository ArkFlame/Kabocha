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
