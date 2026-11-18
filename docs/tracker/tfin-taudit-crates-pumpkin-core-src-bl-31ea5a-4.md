# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: jukebox.rs mod.rs

- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): already fixed. `JukeboxBlockEntity.saveAdditional` (JukeboxBlockEntity.java:129-138) writes the ticks key only while `getSong() != null`; Pumpkin gates it on `song_length_ticks > 0` (line 94) and `chunk_data_nbt` reuses `write_nbt` (147-152). Tests `write_nbt_omits_ticks_since_song_started_when_stopped`, `write_nbt_writes_ticks_since_song_started_while_playing` and `chunk_data_nbt_omits_ticks_when_stopped` cover both branches (passes 3-5).
- `blocks/jukebox.rs:192` `on_state_replaced` delegation (was STILL OPEN): verified correct, no code change. Vanilla `JukeboxBlock.affectNeighborsAfterRemoval` (JukeboxBlock.java:79-82) only calls `Containers.updateNeighboursAfterDestroy`; the record drop lives in `preRemoveSideEffects -> popOutTheItem` (JukeboxBlockEntity.java:193-196), mapped to `JukeboxBlockEntity::on_block_replaced`. No double drop. `popOutTheItem's `offsetRandomXZ(random, 0.7F)` (line 94) matches Pumpkin's `random_range(-0.35..0.35)` (184-186).
(world/mod.rs:1819-1826). Verified the remaining STILL OPEN / ALREADY FIXED (UNVERIFIED)
- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): VERIFIED. `JukeboxSongPlayer.tick`

## Result
Audit pass 11 (2026-11-18, FINAL PASS). Re-read the current source of
`block/entities/jukebox.rs` (`write_nbt` 81-100, `tick` 102-133, `chunk_data_nbt` 147-152,
`note_particle_hue` 474-476) and `block/blocks/jukebox.rs` (`on_state_replaced` 192-195), then
checked each row directly against the decompiled vanilla sources in
`.vendor/vanilla-decompiled/net/minecraft/`. Every row is closed; no `.rs` change was needed.

- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): FIXED on master,
  VERIFIED. `write_nbt` gates `ticks_since_song_started` on `song_length_ticks > 0` (line 94) and
  `chunk_data_nbt` reuses `write_nbt` (147-152). Vanilla `JukeboxBlockEntity.saveAdditional`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java:129-138`)
  writes the key only while `getSong() != null`. Tests
  `write_nbt_omits_ticks_since_song_started_when_stopped`,
  `write_nbt_writes_ticks_since_song_started_while_playing`,
  `write_nbt_keeps_zero_ticks_while_song_is_loaded`, `chunk_data_nbt_omits_ticks_when_stopped`
  cover both branches.
- `blocks/jukebox.rs:192` `on_state_replaced` delegation (was STILL OPEN): VERIFIED, no code
  change. Pumpkin only calls `update_neighbour_for_output_signal`. Vanilla
  `JukeboxBlock.affectNeighborsAfterRemoval`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/JukeboxBlock.java:79-82`) only calls
  `Containers.updateNeighboursAfterDestroy`; the record drop lives in
  `preRemoveSideEffects -> popOutTheItem` (`JukeboxBlockEntity.java:193-196`), mapped to
  `JukeboxBlockEntity::on_block_replaced`. No double drop.
- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): VERIFIED, no code change.
  `JukeboxSongPlayer.tick`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSongPlayer.java:64-77`) emits
  JUKEBOX_PLAY plus particles while `ticks % 20 == 0` BEFORE the increment; `spawnMusicParticles`
  (lines 83-89) sends count 0, offset `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at `pos+(0.5,1.2,0.5)`.
  Pumpkin emits before `fetch_add` (110-130) with `note_particle_hue` = step/24.0 (474-476) and
  `World::spawn_particle` args (pos, offset, max_speed, count, particle). Exact match. `JukeboxSong.hasFinished`
  uses `length + 20` (`world/item/JukeboxSong.java:50-52`), matching `song_length_ticks`.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-taudit-crates-pumpkin-core-src-bl-31ea5a-4.md
