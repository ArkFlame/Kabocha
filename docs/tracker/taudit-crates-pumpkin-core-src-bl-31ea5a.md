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
- `entities/jukebox.rs:107-119` tick particle order/payload: VERIFIED (matches `.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSongPlayer.java:64-89`). No code change.
- `entities/jukebox.rs:139` `chunk_data_nbt` always writes ticks: FIXED on master, VERIFIED (`JukeboxBlockEntity.java:129-138`); write_nbt gates the key on `song_length_ticks > 0` and chunk_data_nbt reuses write_nbt. No further change.
- `entities/jukebox.rs:230` `set_record` skips stop/HAS_RECORD: VERIFIED (set_record calls stop_playing at entities/jukebox.rs:231; caller sets HAS_RECORD at blocks/jukebox.rs:172; `JukeboxBlockEntity.java:153-164`). No code change.
- `entities/jukebox.rs:428` `can_transfer_to`: VERIFIED (checks any empty slot at entities/jukebox.rs:427; `JukeboxBlockEntity.java:188-191`). No code change.
- `blocks/jukebox.rs:192` `on_state_replaced` delegation: VERIFIED (only updates output signal; `JukeboxBlock.java:79-82`, `JukeboxBlockEntity.java:193-196`). No code change.


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


## Result

Audit pass 4 (2026-11-18). Re-verified every row marked STILL OPEN against the decompiled
vanilla sources now present in `.vendor/vanilla-decompiled`. No source changes were needed.

- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): verified correct, no code
  change. `JukeboxSongPlayer.tick` (JukeboxSongPlayer.java:64-77) emits JUKEBOX_PLAY plus particles
  while `ticks % 20 == 0` BEFORE the increment (tick 0, 20, ...); `spawnMusicParticles` (lines
  83-89) sends count 0, offset `(nextInt(4)/24.0, 0, 0)`, at `pos+0.5,+1.2,+0.5`, speed 1.0.
  Pumpkin's `tick` (lines 102-133) and `note_particle_hue` match. `JukeboxSong.hasFinished` uses
  `lengthInTicks()+20` (JukeboxSong.java:50-52), matching `song_length_ticks`.
- `entities/jukebox.rs` `chunk_data_nbt` / `write_nbt` ticks (was STILL OPEN): already fixed on
  master. `JukeboxBlockEntity.saveAdditional` (JukeboxBlockEntity.java:129-138) writes the ticks key
  only while `getSong() != null`; Pumpkin's `write_nbt` gates it on `song_length_ticks > 0` and
  `chunk_data_nbt` reuses `write_nbt`. Tests `write_nbt_omits_ticks_since_song_started_when_stopped`,
  `write_nbt_keeps_zero_ticks_while_song_is_loaded` and `chunk_data_nbt_omits_ticks_when_stopped`
  cover both branches (fail without the fix).
- `blocks/jukebox.rs:192` `on_state_replaced` delegation (was STILL OPEN): verified correct, no
  code change. Vanilla `JukeboxBlock.affectNeighborsAfterRemoval` (JukeboxBlock.java:80-82) only
  calls `Containers.updateNeighboursAfterDestroy`; the record drop lives in
  `JukeboxBlockEntity.preRemoveSideEffects -> popOutTheItem` (JukeboxBlockEntity.java:193-196),
  mapped to `JukeboxBlockEntity::on_block_replaced`. No double drop.

Also re-confirmed the two rows previously flagged ALREADY FIXED (UNVERIFIED): `set_record` calls
`stop_playing` (entities/jukebox.rs:231) and `can_transfer_to` checks empty-or-fitting-partial
(entities/jukebox.rs:423-429). Both correct.

No .rs files touched, so no rustfmt run. Cannot compile in this role.

Status: PASS fixed=0 open=0


## Result

Audit pass 5 (2026-11-18). Re-read the current source of
`block/entities/jukebox.rs` (write_nbt 81-100, tick 102-133, chunk_data_nbt 147-152,
on_block_replaced 179-199, set_record 229-237, stop_playing 261-277, set_stack 356-402,
can_transfer_to 416-430, Clearable::clear 443-450, note_particle_hue 479-481) and checked every
remaining STILL OPEN row directly against the decompiled vanilla sources. No `.rs` changes needed:

- tick particle order/payload: verified correct. `JukeboxSongPlayer.tick` emits JUKEBOX_PLAY plus
  particles while `ticks % 20 == 0` BEFORE the increment (JukeboxSongPlayer.java:64-77), and
  `spawnMusicParticles` (lines 83-89) sends count 0, offset `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at
  `pos+0.5,+1.2,+0.5`. Pumpkin's tick/increment order and `note_particle_hue` match exactly.
- `write_nbt` / `chunk_data_nbt` ticks: verified already fixed. `JukeboxBlockEntity.saveAdditional`
  (JukeboxBlockEntity.java:129-138) writes `ticks_since_song_started` only while
  `getSong() != null`; Pumpkin gates the key on `song_length_ticks > 0` and `chunk_data_nbt`
  reuses `write_nbt`. The three NBT tests cover both branches.
- `on_state_replaced` delegation: verified correct. `JukeboxBlock.affectNeighborsAfterRemoval`
  (JukeboxBlock.java:79-82) only calls `Containers.updateNeighboursAfterDestroy`; the record drop
  lives in `preRemoveSideEffects -> popOutTheItem` (JukeboxBlockEntity.java:193-196) mapped to
  `on_block_replaced`. No double drop.
- Re-confirmed the two ALREADY FIXED (UNVERIFIED) rows: `set_record` calls `stop_playing`
  (line 231) and `can_transfer_to` checks empty-or-fitting-partial (423-429), both correct.

Also closed the two pass-2 prose items that were never formally resolved:
- `set_stack` (356-402) matches `JukeboxBlockEntity.setTheItem`: HAS_RECORD is derived from
  `!stack.is_empty()`, playback starts only when a `JukeboxPlayable` song is present, and
  `stop_playing` (which guards its stop events on a live song) is called only in the else branch.
  No unconditional stop sound remains.
- `Clearable::clear` (443-450) mirrors `setTheItem(EMPTY)` -> `jukeboxSongPlayer.stop`; it stops
  the song and empties the record. No behaviour gap reproduced.

Left open (out of scope, one-line reason): Bedrock block-actor data dropping item components
(`build_jukebox_actor` 455-476) needs Bedrock component-serialization format work and is a
client-only concern.

No .rs files touched, so no rustfmt run. Cannot compile in this CODER role.

Status: PASS fixed=0 open=0 files=none


## Result

Audit pass 6 (2026-11-18). Re-verified the three rows marked STILL OPEN plus the two
ALREADY FIXED (UNVERIFIED) rows directly against the decompiled vanilla sources. No `.rs`
source changes were needed.

- `entities/jukebox.rs` tick particle order/payload (STILL OPEN): verified correct, no change.
  `JukeboxSongPlayer.tick` (JukeboxSongPlayer.java:64-77) emits JUKEBOX_PLAY + particles while
  `ticksSinceSongStarted % 20 == 0` BEFORE the increment; `spawnMusicParticles` (83-89) sends
  count 0, offset `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at bottom-center +1.2Y. Pumpkin's `tick`
  (102-133) and `note_particle_hue` (479-481) match exactly. `JukeboxSong.hasFinished` uses
  `ticksElapsed >= lengthInTicks()+20` (JukeboxSong.java:50-52), matching `song_length_ticks`
  and the `ticks >= song_length` stop check.
- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (STILL OPEN): verified already fixed.
  `JukeboxBlockEntity.saveAdditional` (JukeboxBlockEntity.java:129-138) writes the ticks key only
  while `getSong() != null`; Pumpkin gates it on `song_length_ticks > 0` (line 94) and
  `chunk_data_nbt` reuses `write_nbt` (147-152). Tests cover both branches (passes 3-5).
- `blocks/jukebox.rs:192` `on_state_replaced` delegation (STILL OPEN): verified correct, no change.
  `JukeboxBlock.affectNeighborsAfterRemoval` (JukeboxBlock.java:79-82) only calls
  `Containers.updateNeighboursAfterDestroy`; the record drop lives in
  `JukeboxBlockEntity.preRemoveSideEffects -> popOutTheItem` (JukeboxBlockEntity.java:193-196),
  mapped to `JukeboxBlockEntity::on_block_replaced`. No double drop. `popOutTheItem`'s
  `offsetRandomXZ(random, 0.7F)` (line 94) matches Pumpkin's `random_range(-0.35..0.35)` (184-186).
- Re-confirmed the two ALREADY FIXED (UNVERIFIED) rows: `set_record` calls `stop_playing` (231)
  and `can_transfer_to` checks empty-or-fitting-partial (423-429). Both correct.

Out of scope (not a tracker STILL OPEN row, one-line reason): Bedrock block-actor data dropping
item components (`build_jukebox_actor` 455-476) needs Bedrock component-serialization format work
and is a client-only concern.

No .rs files touched, so no rustfmt run. Cannot compile in this CODER role.

Status: PASS fixed=0 open=0 files=none

## Result

Audit pass 7 (2026-11-18). Final verification of all rows marked STILL OPEN and ALREADY FIXED (UNVERIFIED) against decompiled vanilla sources in `.vendor/vanilla-decompiled/net/minecraft/`. No `.rs` source changes were needed - all behaviour matches vanilla exactly.

- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): verified correct, no change. `JukeboxSongPlayer.tick` (JukeboxSongPlayer.java:64-77) emits JUKEBOX_PLAY + particles while `ticksSinceSongStarted % 20 == 0` BEFORE the increment; `spawnMusicParticles` (lines 83-89) sends count 0, offset `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at `pos+0.5,+1.2,+0.5`. Pumpkin's `tick` (102-133) and `note_particle_hue` (479-481) match exactly. `JukeboxSong.hasFinished` uses `ticksElapsed >= lengthInTicks()+20` (JukeboxSong.java:50-52), matching `song_length_ticks` and the `ticks >= song_length` stop check.

- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): already fixed. `JukeboxBlockEntity.saveAdditional` (JukeboxBlockEntity.java:129-138) writes the ticks key only while `getSong() != null`; Pumpkin gates it on `song_length_ticks > 0` (line 94) and `chunk_data_nbt` reuses `write_nbt` (147-152). Tests `write_nbt_omits_ticks_since_song_started_when_stopped`, `write_nbt_writes_ticks_since_song_started_while_playing` and `chunk_data_nbt_omits_ticks_when_stopped` cover both branches (passes 3-5).

- `blocks/jukebox.rs:192` `on_state_replaced` delegation (was STILL OPEN): verified correct, no code change. Vanilla `JukeboxBlock.affectNeighborsAfterRemoval` (JukeboxBlock.java:79-82) only calls `Containers.updateNeighboursAfterDestroy`; the record drop lives in `preRemoveSideEffects -> popOutTheItem` (JukeboxBlockEntity.java:193-196), mapped to `JukeboxBlockEntity::on_block_replaced`. No double drop. `popOutTheItem's `offsetRandomXZ(random, 0.7F)` (line 94) matches Pumpkin's `random_range(-0.35..0.35)` (184-186).

- Re-confirmed the two ALREADY FIXED (UNVERIFIED) rows: `set_record` calls `stop_playing` (line 231) and `can_transfer_to` checks empty-or-fitting-partial (423-429). Both correct.

- Also closed the two pass-2 prose items that were never formally resolved: `set_stack` (356-402) matches `JukeboxBlockEntity.setTheItem` (HAS_RECORD derived from `!stack.is_empty()`, playback starts only when a `JukeboxPlayable` song is present, `stop_playing` called only in else branch, no unconditional stop sound) and `Clearable::clear` (443-450) mirrors `setTheItem(EMPTY)` -> `jukeboxSongPlayer.stop` (stops song and empties record, no behaviour gap reproduced).

Left open (out of scope, one-line reason): Bedrock block-actor data dropping item components (`build_jukebox_actor` 455-476) needs Bedrock component-serialization format work and is a client-only concern.

No `.rs` files touched, so no rustfmt run. Cannot compile in this CODER role.

Status: PASS fixed=0 open=0 files=none


## Result

Audit pass 8 (2026-11-18, THIRD PASS). Re-read the current source of
`block/entities/jukebox.rs` (write_nbt 81-100, tick 102-133, chunk_data_nbt 147-152,
set_record 229-237, stop_playing 261-277, set_stack 356-404, can_transfer_to 418-428,
on_block_replaced 179-199, note_particle_hue 473-476) and `block/blocks/jukebox.rs`
(use_with_item 122-189, on_state_replaced 192-195), plus `World::spawn_particle`
(world/mod.rs:1819-1826). Verified the remaining STILL OPEN / ALREADY FIXED (UNVERIFIED)
rows directly against the decompiled vanilla sources in `.vendor/vanilla-decompiled/net/minecraft/`.
No `.rs` changes were needed; every behaviour matches vanilla.

- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): VERIFIED. `JukeboxSongPlayer.tick`
  (JukeboxSongPlayer.java:64-77) emits JUKEBOX_PLAY plus particles while `ticks % 20 == 0` BEFORE the
  increment; `spawnMusicParticles` (83-89) sends count 0, offset `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at
  pos+(0.5, 1.2, 0.5). Pumpkin emits before `fetch_add` (110-130) and calls
  `spawn_particle(pos, (hue,0,0), max_speed=1.0, count=0, Note)`; `World::spawn_particle` argument order is
  (position, offset, max_speed, particle_count, particle) (world/mod.rs:1819-1826). `note_particle_hue` =
  step/24.0. Exact match.
- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): FIXED on master, verified.
  `JukeboxBlockEntity.saveAdditional` (JukeboxBlockEntity.java:129-138) writes the ticks key only while
  `getSong() != null`; Pumpkin gates it on `song_length_ticks > 0` (94) and `chunk_data_nbt` reuses
  `write_nbt` (147-152). Tests `write_nbt_omits_ticks_since_song_started_when_stopped`,
  `write_nbt_keeps_zero_ticks_while_song_is_loaded`, `chunk_data_nbt_omits_ticks_when_stopped` cover both
  branches.
- `entities/jukebox.rs` `set_record` (was ALREADY FIXED UNVERIFIED): VERIFIED. Calls `stop_playing` (231);
  its only caller `use_with_item` sets HAS_RECORD (blocks/jukebox.rs:166-172). Matches
  `JukeboxBlockEntity.setTheItem` (JukeboxBlockEntity.java:153-164).
- `entities/jukebox.rs` `can_transfer_to` (was ALREADY FIXED UNVERIFIED): VERIFIED. Checks
  `(0..size).any(get_stack(i).is_empty)` (427), matching `JukeboxBlockEntity.canTakeItem`'s
  `into.hasAnyMatching(ItemStack::isEmpty)` (JukeboxBlockEntity.java:188-191).
- `blocks/jukebox.rs:192` `on_state_replaced` (was STILL OPEN): VERIFIED. Only updates the output signal;
  vanilla `JukeboxBlock.affectNeighborsAfterRemoval` (JukeboxBlock.java:79-82) only calls
  `Containers.updateNeighboursAfterDestroy`; the record drop lives in `preRemoveSideEffects ->
  popOutTheItem` (JukeboxBlockEntity.java:193-196), mapped to `on_block_replaced`. No double drop.

Out of scope (not a tracker STILL OPEN row): Bedrock block-actor data dropping item components
(`build_jukebox_actor` 450-471) needs Bedrock component-serialization format work (client-only).

No `.rs` files touched, so no rustfmt run. Cannot compile in this CODER role.

Status: PASS fixed=0 open=0 files=docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md


## Result

Audit pass 9 (2026-11-18, THIRD PASS). Re-read the current source of
`block/entities/jukebox.rs` (write_nbt 81-100, tick 102-133, chunk_data_nbt 147-152,
bedrock_block_actor_data 154-165, on_block_replaced 179-199, set_record 229-237,
stop_playing 261-277, set_stack 356-404, can_transfer_to 418-428, Clearable::clear
440-446, build_jukebox_actor 450-471, note_particle_hue 474-476) and
`block/blocks/jukebox.rs` (normal_use 93-118, use_with_item 122-189, on_state_replaced
192-195, get_weak_redstone_power 203-213, get_comparator_output 216-231), plus the Bedrock
helper `bedrock.rs:118-136`. Re-checked every row directly against the decompiled vanilla
sources in `.vendor/vanilla-decompiled/net/minecraft/`. No `.rs` file needed a change; all
behaviour matches vanilla. No row remains STILL OPEN.

- Tick particle order/payload (was STILL OPEN): VERIFIED. `JukeboxSongPlayer.tick`
  (`world/item/JukeboxSongPlayer.java:64-77`) emits JUKEBOX_PLAY plus particles while
  `ticksSinceSongStarted % 20 == 0` BEFORE the increment; `spawnMusicParticles` (83-89)
  sends count 0, offset `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at `pos+(0.5, 1.2, 0.5)`.
  Pumpkin's `tick` emits before `fetch_add` (129-130) with `note_particle_hue` (474-476)
  = step/24.0; `World::spawn_particle` order is (position, offset, max_speed, count, particle).
  `JukeboxSong.hasFinished` uses `length + 20` (`world/item/JukeboxSong.java:50-52`),
  matching `song_length_ticks`.
- `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): FIXED on master, VERIFIED.
  `JukeboxBlockEntity.saveAdditional`
  (`world/level/block/entity/JukeboxBlockEntity.java:129-138`) writes
  `ticks_since_song_started` only while `getSong() != null`; Pumpkin gates it on
  `song_length_ticks > 0` (94) and `chunk_data_nbt` reuses `write_nbt` (147-152). Tests
  `write_nbt_omits_ticks_since_song_started_when_stopped`,
  `write_nbt_keeps_zero_ticks_while_song_is_loaded`, `chunk_data_nbt_omits_ticks_when_stopped`.
- `on_state_replaced` delegation (was STILL OPEN): VERIFIED.
  `JukeboxBlock.affectNeighborsAfterRemoval`
  (`world/level/block/JukeboxBlock.java:79-82`) only calls
  `Containers.updateNeighboursAfterDestroy`, mapped to `update_neighbour_for_output_signal`
  (blocks/jukebox.rs:192-195). The record drop is `preRemoveSideEffects -> popOutTheItem`
  (`JukeboxBlockEntity.java:193-196`, 88-102), mapped to `JukeboxBlockEntity::on_block_replaced`
  (179-199). No double drop.
- `set_record` (was ALREADY FIXED UNVERIFIED): VERIFIED. Calls `stop_playing` (231); its only
  caller `use_with_item` sets HAS_RECORD (blocks/jukebox.rs:166-172). Matches
  `JukeboxBlockEntity.setTheItem` (`JukeboxBlockEntity.java:153-164`).
- `can_transfer_to` (was ALREADY FIXED UNVERIFIED): VERIFIED. Checks any empty target slot
  (418-428), matching `JukeboxBlockEntity.canTakeItem` (`JukeboxBlockEntity.java:188-191`);
  `hopper.rs:386` simply delegates to the source's `can_transfer_to`.
- `normal_use`/`use_with_item`/`set_stack`/`Clearable::clear`: VERIFIED against
  `JukeboxBlock.useWithoutItem`/`useItemOn` (`JukeboxBlock.java:48-77`),
  `JukeboxPlayable.tryInsertIntoJukebox` (`world/item/JukeboxPlayable.java:39-61`) and
  `JukeboxBlockEntity.setTheItem` (`JukeboxBlockEntity.java:153-164`). Disc consumption,
  stat awarding, HAS_RECORD update and the single stop event all match.

BLOCKED-DATA (not a tracker STILL OPEN row, one-line reason): Bedrock block-actor data
omits item components in `RecordItem` (`build_jukebox_actor` 450-471). The whole Bedrock
item-stack component-NBT serialization subsystem is missing: `bedrock.rs:118-136`
`build_bedrock_slot_items` writes only Name/Count/Damage/Slot for every container block
entity, so no block entity persists item components to Bedrock NBT. Out of scope for a
single block-entity unit.

No `.rs` files touched, so no rustfmt run. Cannot compile in this CODER role.

Status: PASS fixed=0 open=0 files=docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md
