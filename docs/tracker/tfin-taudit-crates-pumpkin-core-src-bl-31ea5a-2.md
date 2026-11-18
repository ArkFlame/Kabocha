# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: jukebox.rs

remaining STILL OPEN row directly against the decompiled vanilla sources. No `.rs` changes needed:
Audit pass 6 (2026-11-18). Re-verified the three rows marked STILL OPEN plus the two
- `entities/jukebox.rs` tick particle order/payload (STILL OPEN): verified correct, no change.
- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (STILL OPEN): verified already fixed.

## Result
Final pass (2026-11-18). Re-read the current source of
`crates/pumpkin-core/src/block/entities/jukebox.rs` (write_nbt 81-100, tick 102-133,
chunk_data_nbt 147-152, note_particle_hue 474-476) and checked both remaining STILL OPEN rows
directly against the decompiled vanilla sources in `.vendor/vanilla-decompiled/net/minecraft/`.
No `.rs` changes were needed: the behaviour already matches vanilla exactly.

- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): VERIFIED.
  `JukeboxSongPlayer.tick`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSongPlayer.java:64-81`) emits
  `JUKEBOX_PLAY` + particles while `ticksSinceSongStarted % 20 == 0` BEFORE the
  `ticksSinceSongStarted++` increment; `spawnMusicParticles` (lines 83-89) sends the NOTE
  particle at `Vec3.atBottomCenterOf(pos) + (0, 1.2, 0)` with count 0, offset
  `(nextInt(4)/24.0, 0, 0)`, speed 1.0. Pumpkin's `tick` (lines 102-133) checks
  `ticks.is_multiple_of(20)` before `fetch_add(1)`, emits `JukeboxPlay`, and calls
  `spawn_particle` (signature at `crates/pumpkin-core/src/world/mod.rs:1819-1830`:
  position/offset/max_speed/count/particle) with position `(x+0.5, y+1.2, z+0.5)`, offset
  `(note_particle_hue(random_range(0..4)), 0, 0)` = `step/24.0`, speed 1.0, count 0.
  `note_particle_hue` (474-476) matches `nextInt(4)/24.0F`. `JukeboxSong.hasFinished`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSong.java:50-52`) is
  `ticksElapsed >= lengthInTicks()+20`, matching Pumpkin's `song_length_ticks =
  length_in_ticks()+SONG_END_PADDING_TICKS(20)` and the `ticks >= song_length` stop check.
  No change.

- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): FIXED on master,
  VERIFIED. `JukeboxBlockEntity.saveAdditional`
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java:129-138`)
  stores `RecordItem` only when the item is non-empty and writes `ticks_since_song_started`
  only while `getSong() != null`. Pumpkin's `write_nbt` (81-100) writes `RecordItem` only
  when non-empty and gates the ticks key on `song_length_ticks > 0` (0 when no song is
  loaded/playing), and `chunk_data_nbt` (147-152) reuses `write_nbt` so chunk saves cannot
  drift. Tests `write_nbt_omits_ticks_since_song_started_when_stopped` (504-510),
  `write_nbt_writes_ticks_since_song_started_while_playing` (513-520),
  `write_nbt_keeps_zero_ticks_while_song_is_loaded` (523-531) and
  `chunk_data_nbt_omits_ticks_when_stopped` (534-539) cover both branches. No further change.

Neither row needs a generated pumpkin-data field or a missing subsystem, so no BLOCKED-DATA.
No .rs files touched, so no rustfmt run needed. Cannot compile in this CODER role.

Status: PASS fixed=0 open=0 files=none

## Result (x_18)

- `entities/jukebox.rs` tick particle order/payload: VERIFIED. Vanilla `world/item/JukeboxSongPlayer.java:64-81`; Rust `block/entities/jukebox.rs:102-133`, `note_particle_hue` 474-476.
- `entities/jukebox.rs` `write_nbt`/`chunk_data_nbt` ticks: VERIFIED (already matching). Vanilla `world/level/block/entity/JukeboxBlockEntity.java:129-138`; Rust `jukebox.rs:81-100`, `147-152`.
- No `.rs` edited, no cargo. Status: PASS.
