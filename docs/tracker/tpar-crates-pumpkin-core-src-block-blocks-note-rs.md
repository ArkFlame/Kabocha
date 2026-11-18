# Parity rows for crates/pumpkin-core/src/block/blocks/note.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Note Block** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tblocks-note-block** [partial/OWN-OPEN] own audit: 2 open, 3 fixed, 0 verified, 0 blocked-data (docs/tracker/tblocks-note-block.md) docs/tracker/tblocks-note-block.md

## Result
THIRD PASS 2026-11-18 (final). Full re-review against .vendor/vanilla-decompiled; all prior fixes are present and correct, so no source edits were needed this pass.

- Row "blocks/Note Block" (full re-review): VERIFIED, matches vanilla net/minecraft/world/level/block/NoteBlock.java. setInstrument above/below (note.rs:65-79 vs NoteBlock.java:47-56): vanilla worksAboveNoteBlock() == !is_base_block() and isTunable() == is_base_block() (NoteBlockInstrument.java:56-66), so the Pumpkin mapping is correct. playNote gate (note.rs:33 vs :104), note cycle 0..24 (note.rs:105 vs :128), top-instrument passthrough (note.rs:144-149 vs :120), pitch 2^((note-12)/12) (note.rs:55-56 vs :145-146) all match. Custom-head sound uses play_custom_sound volume 3.0 (note.rs:184-190 vs NoteBlock.java:166-179); audible radius = ceil(3.0) = 3 chunks = 48 blocks (world/mod.rs:9543-9545, 2027), matching SoundEvent.getRange(3.0) = 16*3 = 48 (SoundEvent.java:43-45). Encoded range: None matches createVariableRangeEvent = Optional.empty() (SoundEvent.java:35-37). NOTE particle correctly absent: Level.addParticle is a server no-op with no override (Level.java:1712-1714), so it is client-only.

- Row "own/tblocks-note-block": VERIFIED. Referenced doc docs/tracker/tblocks-note-block.md is fully closed (fixed=2, open=0); both rows re-confirmed here: custom-head sound audible range FIXED (note.rs:184-190), NotePlayEvent instrument serialized id FIXED (note.rs:36 to_value().to_owned()).

- No BLOCKED-DATA rows: no generated pumpkin-data field or missing subsystem is involved. No test added: the only pure helpers are the instrument/sound tables (a test would merely mirror them) and get_note_pitch (formula mirroring), and every behaviour path needs a live World. Rows: fixed=0 this pass, verified=2, open=0.
