# blocks / Note Block (re-review)

Re-review 2026-11-17 after the source changed. Prior audit: 2026-11-16 (branch
`kab/tblocks-note-block`). Vanilla reference: Paper 1.21.9 decompiled
`net.minecraft.world.level.block.NoteBlock` (read during the prior audit; that
paperweight cache path no longer exists, no `.vendor` copy in this worktree, no
network here, so vanilla line numbers are cited from the prior reading).

Source: `crates/pumpkin-core/src/block/blocks/note.rs`.

## What changed since the reviewed source
Commit `0a9ddfd22 fix(block): align block behaviours with vanilla`:
- `note.rs:160` pitch/instrument classification now includes `Pling` and all four
  copper `Trumpet*` instruments in `is_base_block` (vanilla `Type.BASE_BLOCK` /
  `isTunable()`).
- `note.rs:200` sound position switched from `to_f64()` to `to_centered_f64()`
  (vanilla plays at block center `x+0.5, y+0.5, z+0.5`).

Checked against `assets/blocks.json`: the 20 non-head instruments are
`banjo, basedrum, bass, bell, bit, chime, cow_bell, didgeridoo, flute, guitar,
harp, hat, iron_xylophone, pling, snare, trumpet, trumpet_exposed,
trumpet_oxidized, trumpet_weathered, xylophone`; `is_base_block` now covers
exactly these. Head/`custom_head` instruments are all outside the list, which is
what vanilla's `!isTunable()` (`worksAboveNoteBlock()`) means for the
`play_note` gate at `note.rs:33` and the above/below derivation at
`note.rs:65-78`. No instruments are missing.

## Verified correct (no change)
- `note.rs:102-125` tune: cycle `NOTE` mod 25, set block, play, `TUNE_NOTEBLOCK`.
- `note.rs:127-140` attack: play, `PLAY_NOTEBLOCK`.
- `note.rs:33` silence when a non-tunable instrument has a block above;
  head instruments always play (`worksAboveNoteBlock() || above.isAir()`).
- `note.rs:220` Y-axis-only instrument refresh (above head and below block both
  feed `setInstrument`).
- `note.rs:84-100` redstone `POWERED` toggle: note on rising edge, state set
  afterwards with flag 3 (`BlockFlags::NOTIFY_ALL == 0b11`), matching vanilla.
- `note.rs:186` `SoundEvent { range: None }` is the variable-range event: the
  packet encoder (`sound_effect.rs:48`) writes the optional fixed range, so a
  missing range lets the client resolve it from the sound registry. The prior
  audit's gap #1 (asserting `None` was wrong) is withdrawn.

## Fixed in this pass
1. `note.rs:205-212` every playback now spawns the vanilla note particle
   (`triggerEvent`: `addParticle(NOTE, x+0.5, y+1.2, z+0.5, note/24, 0, 0)`) for
   tunable, mob-head and custom-head instruments; the head branch was converted
   from `return true` to `else` so the particle runs for it too.
2. `note.rs:36` `NotePlayEvent` now reports `instrument.to_value()`
   (`trumpet_exposed`) instead of the Rust `Debug` name (`TrumpetExposed`).
   Plugin-API-visible string only.

## Remaining gaps
None found in this pass.

Known untestable-by-agent detail: the particle position/velocity is only
observable by a client, so it is covered by the vanilla source rather than by an
automated test. Everything else has existing gametest coverage under
`server/builtin_game_tests/{note_and_comparators,lamps_bulbs_and_noteblocks_extended}.rs`.

Structural note (size S, not a parity bug): `is_base_block` is a hand-maintained
mirror of vanilla's `NoteblockInstrument.Type.BASE_BLOCK`; each new instrument
(e.g. 1.21.5 copper trumpets) must be added here or it silently loses pitch and
loses the "block above" gate. A `Type`/`is_tunable` field in the generated
`NoteblockInstrument` (codegen from vanilla enum) would remove the class of bug.
