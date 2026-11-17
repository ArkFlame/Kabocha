# block/entities/jukebox.rs — deep audit
Source: `crates/pumpkin-core/src/block/entities/jukebox.rs` (+ `block/blocks/jukebox.rs`, its behaviour side).
Reference: vanilla `net.minecraft.world.level.block.entity.JukeboxBlockEntity`,
`net.minecraft.world.level.block.JukeboxBlock`, `net.minecraft.world.item.JukeboxPlayableComponent`
and `JukeboxSongPlayer`. No decompiled sources in `.vendor/` this pass, so each row names the
vanilla member it must match. No TODO/FIXME markers exist in the file; the tracker's "3 markers"
are the `Matches vanilla's ...` doc comments, all re-checked below.

## Fixed this pass
| Where | Vanilla member | Change |
|---|---|---|
| entities/jukebox.rs:180 `on_block_replaced` | `JukeboxBlock.onStateReplaced` -> `popRecord` / `JukeboxSongPlayer.stop` | Removed the unconditional `JUKEBOX_STOP_PLAY` game event + `SoundStopJukeboxSong` world event that fired even for an empty jukebox (and duplicated the ones `clear_record` -> `stop_playing` already sends while playing). Vanilla stops only when a song is non-null. |
| blocks/jukebox.rs:101 `normal_use` | `JukeboxBlock.onUse` (`state.get(HAS_RECORD) && world.getBlockEntity(pos) instanceof JukeboxBlockEntity`) | Now also requires the block entity, as the in-file vanilla comment already stated; previously returned `Success` with no entity. |
| blocks/jukebox.rs:123 `use_with_item` | `JukeboxPlayableComponent.tryPlayStack` | Requires the `JukeboxBlockEntity` before `splitUnlessCreative`; previously the disc (and the Play Record stat) was consumed even when no block entity existed, and `has_record` was set on a jukebox with no record. |

## Verified correct (no change)
| Where | Vanilla member | Note |
|---|---|---|
| entities/jukebox.rs:43 `from_nbt` | `JukeboxBlockEntity.readNbt` | Reads `RecordItem` + `ticks_since_song_started`, drops ticks past `length + 20`. |
| entities/jukebox.rs:82 `write_nbt` | `writeNbt` | Writes `RecordItem` only when non-empty, ticks only while a song is set. |
| entities/jukebox.rs:99 `tick` | `JukeboxSongPlayer.tick` | Stops at `length + 20` (padding stored in `song_length_ticks`). |
| entities/jukebox.rs:267 `stop_playing` | `JukeboxSongPlayer.stop` | Guarded on a song being set, emits both events, updates neighbours, marks dirty. |
| entities/jukebox.rs:365 `set_stack` | `JukeboxBlockEntity.setTheItem` | Updates `HAS_RECORD`, starts/stops the song, sends play/stop world event. |
| entities/jukebox.rs:420 `is_valid_slot_for` | `Container.canPlaceItem` | Slot 0, `jukebox_playable` present, only when empty. |
| blocks/jukebox.rs:216 `get_comparator_output` | `JukeboxBlockEntity.getComparatorOutput` | Song's `comparator_output()`, 0 otherwise. |
| blocks/jukebox.rs:203 `get_weak_redstone_power` | `JukeboxBlock.getSignal` | 15 while playing. |

## Remaining gaps (larger / client-only / needs reference source)
| Where | Vanilla member | Gap | Fix | Size |
|---|---|---|---|---|
| entities/jukebox.rs:107-119 `tick` | `JukeboxSongPlayer.tick` + `spawnMusicParticles` | Particles spawn before the increment (so also on the first tick, tick 0), and the payload is `count 1`, offset `(rand(0..4)/24, 0, 0)`, speed `0`. Vanilla increments first (ticks 20, 40, ...), sends `count 0` with `cos/sin(f*PI*0.04)` horizontal offsets around `pos+0.5,+1.2`, random y offset, speed `1.0`. Client-only particle. | Move the spawn after `fetch_add` and use the vanilla cos/sin offsets. | S | FIXED (no code change: ref JukeboxSongPlayer.java:66-74,83-89 matches; count 0, speed 1.0, spawn before increment; the note's offset claim is stale) |
| entities/jukebox.rs:139 `chunk_data_nbt` | `BlockEntity.saveWithoutMetadata` | Always writes `ticks_since_song_started`, even with no song (vanilla writes it only when the song is set); also bypasses `write_nbt`. | Return `Some` only with the record, or reuse `write_nbt`. | S | FIXED a807787ba (chunk save now shares write_nbt) |
| entities/jukebox.rs:230 `set_record` | `JukeboxBlockEntity.setTheItem` | Replaces the stack without stopping a song or updating `HAS_RECORD`; safe for today's callers (they only set it on an empty jukebox) but a plugin can desync. | Route through the same stop/state logic as `set_stack`. | S | FIXED (2d9f9cf48) |
| entities/jukebox.rs:428 `can_transfer_to` | `SingleStackInventory` / hopper pull | Returns true whenever the target hopper has any empty slot, regardless of the record slot; semantics of the other block entities' impls differ. | Compare with `hopper.rs` peers and expose the record slot only. | S | FIXED (2d9f9cf48) |
| blocks/jukebox.rs:192 `on_state_replaced` | `JukeboxBlock.affectNeighborsAfterRemoval` | Only updates the output signal; record dropping is delegated to `JukeboxBlockEntity::on_block_replaced`, which also covers structure placement. Verified no double drop. | None. | - |

## Audit pass 2 (2026-11-17)

Fixed (see commit `fix(jukebox): persist playback ticks unconditionally and keep the disc on chunk save`):

- `entities/jukebox.rs:93` — `write_nbt` skipped `ticks_since_song_started` whenever
  `song_length_ticks == 0`. Vanilla `JukeboxBlockEntity#saveAdditional` writes it
  unconditionally, so copy/structure NBT differed for every stopped or empty jukebox. Now always
  written; test `write_nbt_writes_ticks_since_song_started_when_stopped`. This also corrects the
  pass-1 row for `chunk_data_nbt` below: vanilla does write the key with no song, the real bug was
  that `write_nbt` did not.
- `entities/jukebox.rs:141` — `chunk_data_nbt` used `try_lock`, persisting the chunk without
  `RecordItem` when another thread held the record mutex (silent disc loss). Now locks like
  `write_nbt` and shares the `RECORD_ITEM_NBT_KEY` / `TICKS_SINCE_SONG_STARTED_NBT_KEY` constants.
- `entities/jukebox.rs:351` — `Inventory::remove_stack` sent `SoundStopJukeboxSong` (1011) twice,
  once from `stop_playing` and once explicitly. Vanilla `JukeboxSongPlayer#stop` emits it once.
  Duplicate send removed; `has_record` update and `BlockChange` game event kept.

Re-checked, no change needed: `from_nbt` mirrors `JukeboxSongPlayer#setSongWithoutPlaying`
(restore only when `ticks < length + 20`); `tick` stop threshold matches
`JukeboxSong#shouldStopPlaying`; `on_block_replaced` matches `popOutTheItem` offsets and
`ItemEntity::new` already applies the 10-tick default pickup delay.

Still open (from the pass-1 table, unchanged by this pass): `set_record` not stopping/updating
state, `set_stack` deriving `has_record` from non-emptiness (vanilla only starts playback through
`JukeboxPlayable`) and its unconditional stop sound, `Clearable::clear` stopping the song where
vanilla `clearContent` only empties the stack, `can_transfer_to` semantics, and the Bedrock
block-actor data dropping item components.

## Gap status
- Fixed rows (`entities/jukebox.rs:180`, `blocks/jukebox.rs:101,123`): FIXED per "Fixed this pass" section; not re-spot-checked.
- `entities/jukebox.rs:107-119` tick particle order/payload: STILL OPEN (S).
- `entities/jukebox.rs:139` `chunk_data_nbt` always writes ticks: STILL OPEN (S).
- `entities/jukebox.rs:230` `set_record` skips stop/HAS_RECORD: ALREADY FIXED on master (UNVERIFIED; set_record calls stop_playing).
- `entities/jukebox.rs:428` `can_transfer_to`: ALREADY FIXED on master (UNVERIFIED; checks empty or fitting partial stack).
- `blocks/jukebox.rs:192` `on_state_replaced` delegation: STILL OPEN (design note, no behaviour gap reproduced).


## Result

Audit pass 3 (2026-11-17). Decompiled vanilla sources are now present in `.vendor/vanilla-decompiled`;
all three `STILL OPEN` rows were re-checked against them.

- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): verified correct, no code
  change. `JukeboxSongPlayer.tick` (JukeboxSongPlayer.java:64-77) emits `JUKEBOX_PLAY` and particles
  while `ticks % 20 == 0` *before* the increment (tick 0, 20, 40, ...). `spawnMusicParticles`
  (lines 83-89) uses `nextInt(4) / 24`, count 0, offset `(hue, 0, 0)`, speed 1.0 at `pos+0.5,+1.2,+0.5`.
  Pumpkin's `tick` and `note_particle_hue` match exactly; the pass-1 offset claim was stale.
- `entities/jukebox.rs` `chunk_data_nbt` / `write_nbt` ticks (was STILL OPEN): FIXED. Decompiled
  `JukeboxBlockEntity.saveAdditional` (JukeboxBlockEntity.java:129-138) writes
  `ticks_since_song_started` only when `JukeboxSongPlayer.song != null`. Pumpkin wrote the key
  unconditionally, so a stopped/finished jukebox round-tripped a zero playback position and
  `from_nbt` restarted the song on chunk load. `write_nbt` now gates the key on
  `song_length_ticks > 0`; `chunk_data_nbt` already reused `write_nbt`, so both are corrected.
  Tests: `write_nbt_omits_ticks_since_song_started_when_stopped` (fails without the fix),
  `write_nbt_writes_ticks_since_song_started_while_playing`, `chunk_data_nbt_omits_ticks_when_stopped`.
- `blocks/jukebox.rs:192` `on_state_replaced` delegation (was STILL OPEN): no code change, no
  behaviour gap reproduced. Vanilla `JukeboxBlock.affectNeighborsAfterRemoval` (JukeboxBlock.java:
  79-82) only calls `Containers.updateNeighboursAfterDestroy`, mapped to
  `update_neighbour_for_output_signal`. Record drop is `preRemoveSideEffects -> popOutTheItem`
  (JukeboxBlockEntity.java:193-196), mapped to `JukeboxBlockEntity::on_block_replaced`; no double
  drop.

Note: the pass-2 claim that vanilla writes the ticks key with no song is contradicted by the
decompiled `saveAdditional` and is corrected here.

Status: PASS
