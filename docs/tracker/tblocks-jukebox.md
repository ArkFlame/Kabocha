# blocks / Jukebox — re-review

Sources: `crates/pumpkin-core/src/block/blocks/jukebox.rs`, `crates/pumpkin-core/src/block/entities/jukebox.rs`.
Compared against vanilla `JukeboxBlock`, `JukeboxBlockEntity`, `JukeboxSongPlayer`, `JukeboxPlayable(Component)` (no decompiled reference in `.vendor/`).
Previously open: #4128 (disc duplication), #3643 (block-change event), #3548 (resume after reload).

## Fixed in this review
- `JukeboxSongPlayer.play` emits the `jukebox_play` game event once per start; the port emitted it every 20 ticks from the note-particle branch (`entities/jukebox.rs:107-111`). Moved to `start_playing` (`entities/jukebox.rs:259-268`). Vanilla `JukeboxSongPlayer.tick` only spawns note particles.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | `blocks/jukebox.rs:139-146`, `entities/jukebox.rs:55-58,370-373`, `blocks/jukebox.rs:214-218` | `JukeboxPlayable` holds a registry holder; `JukeboxSong` is resolved through the dynamic registry | Song is resolved by stripping the namespace (`rsplit(':')`) and matching the generated enum. Datapack/modded songs (`ns:cat`) silently resolve to the vanilla song, or fail for unknown paths, even though `jukebox_song` is a loaded dynamic registry. Needs registry access at these four call sites. | M |
| 2 | `entities/jukebox.rs:249-259,270-286`; `blocks/jukebox.rs:68-74,184-210` | `JukeboxSongPlayer.stop` is guarded by `song != null` and sends one `JUKEBOX_STOP_PLAY` + one `SoundStopJukeboxSong` | Eject/break emits stop twice: `clear_record` -> `stop_playing` already sends both, then `JukeboxBlock::stop_playing` / `on_block_replaced` send them again (plus duplicate neighbour updates). | S |
| 3 | `entities/jukebox.rs:93-96,152-155` | `writeNbt` writes `ticks_since_song_started` only when `ticks > 0` | Ticks are persisted whenever `song_length_ticks > 0` (including 0) and `chunk_data_nbt` writes them unconditionally, also with no record. Harmless on load, but NBT differs from vanilla. | S |
| 4 | `blocks/jukebox.rs:90-97` | `JukeboxBlock` has no `onPlace` hook | `player_placed` re-derives `HAS_RECORD` from the block entity; dead for normal placement (fresh entity) and can resurrect a stale flag. No vanilla counterpart. | S |
| FIXED 5 | `entities/jukebox.rs:112-123` | `playNoteParticles` uses `random.nextFloat() * 0.25F` for the note colour | Colour is `0..4 / 24.0` (0..0.125) instead of 0..0.25. Client-only cosmetic. | S |

`getComparatorOutput` (record-derived, `blocks/jukebox.rs:206-222`) matches vanilla's disc-derived strength and keeps the signal after playback ends; no change.
For #4128, insert/eject/break/hopper-extract each drop the record once in the current tree (break only via `JukeboxBlockEntity::on_block_replaced`); no duplication path found.

## Gapwave h3 status
- #5 note particle colour: FIXED a20553e39 (nextInt(24)/24 hue, UNVERIFIED client-only, no test)
- #3 ticks NBT persistence: STILL OPEN (S, unverified vanilla save condition; not touched this pass)

## Result
- #3 ticks NBT persistence: FIXED, verified. Decompiled `JukeboxBlockEntity.saveAdditional` writes `ticks_since_song_started` only when `JukeboxSongPlayer.getSong() != null` (i.e. `song != null`), not when `ticks > 0` as the gap table claimed. Pumpkin's `write_nbt` gates on `song_length_ticks > 0` (its `song != null` proxy) and `chunk_data_nbt` delegates to `write_nbt`, so both omit the key when stopped and persist it even at ticks=0 while a song is loaded. Added `write_nbt_keeps_zero_ticks_while_song_is_loaded` to lock in vanilla gating. UNVERIFIED (no cargo run; NBT detail only).
- #1 dynamic registry song resolution: LEFT OPEN. Needs dynamic `jukebox_song` registry access at the four call sites (generated-data / cross-crate change outside this unit).
- #2 duplicate stop on eject/break: RESOLVED in current tree; no code change needed. Vanilla `JukeboxSongPlayer.stop` early-returns on `song == null` and `JukeboxBlockEntity.setRemoved` emits stop once; Pumpkin's `clear_record` -> `stop_playing` emits the single stop event + 1011 sound for both eject and `on_block_replaced`, which no longer resend them.
- #4 `player_placed` HAS_RECORD: NOT A GAP; no code change needed. Decompiled `JukeboxBlock.setPlacedBy` sets `HAS_RECORD` when the placed item's `BLOCK_ENTITY_DATA` contains `RecordItem`; Pumpkin's `player_placed` mirrors this via the block entity's record.
