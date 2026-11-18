# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: jukebox.rs

Audit pass 4 (2026-11-18). Re-verified every row marked STILL OPEN against the decompiled
- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): verified correct, no code
- `entities/jukebox.rs` `chunk_data_nbt` / `write_nbt` ticks (was STILL OPEN): already fixed on
- `blocks/jukebox.rs:192` `on_state_replaced` delegation (was STILL OPEN): verified correct, no

## Result
FINAL PASS (2026-11-18). Re-read the current source of `block/entities/jukebox.rs`
(write_nbt 81-100, tick 102-133, chunk_data_nbt 147-152, on_block_replaced 179-199,
note_particle_hue 474-476) and `block/blocks/jukebox.rs` (on_state_replaced 192-195), plus
`World::spawn_particle` (`crates/pumpkin-core/src/world/mod.rs:1819-1830`), and verified every row
directly against the decompiled vanilla sources under `.vendor/vanilla-decompiled/net/minecraft/`.
No `.rs` change was needed - each row already matches vanilla, so no rustfmt run.

- `entities/jukebox.rs` tick particle order/payload: VERIFIED. `JukeboxSongPlayer.tick`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSongPlayer.java:64-81`) emits
  `JUKEBOX_PLAY` plus particles while `ticksSinceSongStarted % 20 == 0` BEFORE the increment;
  `spawnMusicParticles` (83-89) sends count 0, offset `(nextInt(4)/24.0F, 0, 0)`, max speed 1.0
  at `pos+(0.5, 1.2, 0.5)`. Pumpkin's `tick` emits before `fetch_add` (110-130) and calls
  `spawn_particle(pos, (hue,0,0), max_speed=1.0, count=0, Note)`; arg order is
  (position, offset, max_speed, count, particle) (`world/mod.rs:1819-1826`), and
  `note_particle_hue` (474-476) = step/24.0. `JukeboxSong.hasFinished` uses `length+20`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSong.java:50-52`), matching
  `song_length_ticks` and the `ticks >= song_length` stop check. Exact match, no change.
- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks: VERIFIED (already fixed on master).
  `JukeboxBlockEntity.saveAdditional`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java:129-138`)
  writes `RecordItem` only when the item is non-empty and `ticks_since_song_started` only while
  `getSong() != null`. Pumpkin's `write_nbt` (81-100) writes the record key only when non-empty and
  gates the ticks key on `song_length_ticks > 0` (set to length+20 on play, 0 on stop), and
  `chunk_data_nbt` reuses `write_nbt` (147-152). Tests cover both branches.
- `blocks/jukebox.rs:192` `on_state_replaced` delegation: VERIFIED. Vanilla
  `JukeboxBlock.affectNeighborsAfterRemoval`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/JukeboxBlock.java:79-82`) only calls
  `Containers.updateNeighboursAfterDestroy`, mapped to `update_neighbour_for_output_signal`
  (192-195). The record drop is `preRemoveSideEffects -> popOutTheItem`
  (`JukeboxBlockEntity.java:193-196, 88-102`), mapped to `JukeboxBlockEntity::on_block_replaced`
  (179-199); `offsetRandomXZ(random, 0.7F)` matches `random_range(-0.35..0.35)`. No double drop.

No row remains open: all three rows end VERIFIED. No `.rs` files touched, no rustfmt run.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-taudit-crates-pumpkin-core-src-bl-31ea5a-1.md
