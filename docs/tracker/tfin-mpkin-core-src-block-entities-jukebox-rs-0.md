# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-entities-jukebox-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Every sub-row is FIXED or VERIFIED against vanilla and the source; no row remains open.
BLOCKED-DATA (not a sub-row of this unit; out-of-scope subsystem): Bedrock block-actor

## Result

Final pass (2026-11-18). Re-read `crates/pumpkin-core/src/block/entities/jukebox.rs`
(tick 102-133, write_nbt 81-100, chunk_data_nbt 147-152) and the vanilla classes directly,
then closed every row. No `.rs` change was needed: the source already matches vanilla, so no
rustfmt and no new test (existing NBT/comparator/can_transfer_to tests cover the fixed branches).

Per-row dispositions:

- Tick particle order/payload: VERIFIED. `JukeboxSongPlayer.tick`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSongPlayer.java:64-89`) emits
  JUKEBOX_PLAY plus NOTE particles while `ticksSinceSongStarted % 20 == 0` BEFORE
  `ticksSinceSongStarted++`; `spawnMusicParticles` (83-89) sends count 0, offset
  `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at `pos+(0.5,1.2,0.5)`. Pumpkin emits before `fetch_add`
  (110-130), `note_particle_hue` = step/24.0 (474-476), pos+(0.5,1.2,0.5), offset (hue,0,0),
  count 0, speed 1.0. `JukeboxSong.hasFinished` = `lengthInTicks()+20`
  (`world/item/JukeboxSong.java:44-52`), matching `song_length_ticks` and the 20-tick padding.
- `chunk_data_nbt`/`write_nbt` ticks: FIXED on master, VERIFIED.
  `JukeboxBlockEntity.saveAdditional`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java:129-138`)
  writes `ticks_since_song_started` only while `getSong() != null`; Pumpkin gates the key on
  `song_length_ticks > 0` (94) and `chunk_data_nbt` reuses `write_nbt` (147-152). Tests
  `write_nbt_omits_ticks_since_song_started_when_stopped`,
  `write_nbt_writes_ticks_since_song_started_while_playing`,
  `write_nbt_keeps_zero_ticks_while_song_is_loaded`, `chunk_data_nbt_omits_ticks_when_stopped`.
- `set_record`/`can_transfer_to`: VERIFIED. `JukeboxBlockEntity.setTheItem`
  (`JukeboxBlockEntity.java:153-164`) plays or stops on insert; Pumpkin `set_record` (231) calls
  `stop_playing` and its only caller `use_with_item` sets HAS_RECORD. `canTakeItem`
  (`JukeboxBlockEntity.java:188-191`) matches Pumpkin's any-empty-target-slot check (418-428).
- `on_block_replaced`/`on_state_replaced`: VERIFIED. `JukeboxBlock.affectNeighborsAfterRemoval`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/JukeboxBlock.java:79-82`) only calls
  `Containers.updateNeighboursAfterDestroy`, mapped to `update_neighbour_for_output_signal`
  (blocks/jukebox.rs:192-195); the record drop is `preRemoveSideEffects -> popOutTheItem`
  (`JukeboxBlockEntity.java:88-102,193-196`), mapped to `on_block_replaced`. No double drop.
- `normal_use`/`use_with_item`/`set_stack`/`Clearable::clear`: VERIFIED against
  `JukeboxBlock.useWithoutItem`/`useItemOn` (`JukeboxBlock.java:48-77`) and
  `JukeboxPlayable.tryInsertIntoJukebox`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxPlayable.java:39-61`).

BLOCKED-DATA (not a sub-row; out-of-scope subsystem): Bedrock block-actor data omits item
components in `RecordItem` (`build_jukebox_actor` 450-471). Missing subsystem: Bedrock item-stack
component-NBT serialization (`entities/bedrock.rs:118-136` `build_bedrock_slot_items` writes only
Name/Count/Damage/Slot for every container). Client-only Bedrock concern.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-mpkin-core-src-block-entities-jukebox-rs-0.md
