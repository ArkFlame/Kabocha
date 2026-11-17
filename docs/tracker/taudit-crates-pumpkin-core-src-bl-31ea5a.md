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
| entities/jukebox.rs:107-119 `tick` | `JukeboxSongPlayer.tick` + `spawnMusicParticles` | Particles spawn before the increment (so also on the first tick, tick 0), and the payload is `count 1`, offset `(rand(0..4)/24, 0, 0)`, speed `0`. Vanilla increments first (ticks 20, 40, ...), sends `count 0` with `cos/sin(f*PI*0.04)` horizontal offsets around `pos+0.5,+1.2`, random y offset, speed `1.0`. Client-only particle. | Move the spawn after `fetch_add` and use the vanilla cos/sin offsets. | S |
| entities/jukebox.rs:139 `chunk_data_nbt` | `BlockEntity.saveWithoutMetadata` | Always writes `ticks_since_song_started`, even with no song (vanilla writes it only when the song is set); also bypasses `write_nbt`. | Return `Some` only with the record, or reuse `write_nbt`. | S |
| entities/jukebox.rs:230 `set_record` | `JukeboxBlockEntity.setTheItem` | Replaces the stack without stopping a song or updating `HAS_RECORD`; safe for today's callers (they only set it on an empty jukebox) but a plugin can desync. | Route through the same stop/state logic as `set_stack`. | S |
| entities/jukebox.rs:428 `can_transfer_to` | `SingleStackInventory` / hopper pull | Returns true whenever the target hopper has any empty slot, regardless of the record slot; semantics of the other block entities' impls differ. | Compare with `hopper.rs` peers and expose the record slot only. | S |
| blocks/jukebox.rs:192 `on_state_replaced` | `JukeboxBlock.affectNeighborsAfterRemoval` | Only updates the output signal; record dropping is delegated to `JukeboxBlockEntity::on_block_replaced`, which also covers structure placement. Verified no double drop. | None. | - |
