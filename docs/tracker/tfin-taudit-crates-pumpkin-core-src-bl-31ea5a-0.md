# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: jukebox.rs

all three `STILL OPEN` rows were re-checked against them.
- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): verified correct, no code
- `entities/jukebox.rs` `chunk_data_nbt` / `write_nbt` ticks (was STILL OPEN): FIXED. Decompiled
- `blocks/jukebox.rs:192` `on_state_replaced` delegation (was STILL OPEN): no code change, no

## Result
Audit pass 11 (2026-11-18, FINAL PASS). Re-read the current source of
crates/pumpkin-core/src/block/entities/jukebox.rs (write_nbt 81-100, tick 102-133,
chunk_data_nbt 147-152, on_block_replaced 179-199, set_record 229-237, set_stack 356-404,
can_transfer_to 418-428, note_particle_hue 474-476) and block/blocks/jukebox.rs
(on_state_replaced 192-195), plus World::spawn_particle (world/mod.rs:1819-1826). Verified
all three rows against the decompiled vanilla sources in .vendor/vanilla-decompiled/net/minecraft/.
No .rs source change was needed; every behaviour matches vanilla exactly.

- entities/jukebox.rs tick particle order/payload: VERIFIED. JukeboxSongPlayer.tick
  (.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSongPlayer.java:64-77) emits
  JUKEBOX_PLAY plus particles while ticksSinceSongStarted % 20 == 0 BEFORE the increment
  (line 74). spawnMusicParticles (83-89) sends count 0, offset (nextInt(4)/24.0, 0, 0),
  speed 1.0 at bottom-center +1.2Y. Pumpkin emits before fetch_add (129-130) with
  note_particle_hue = step/24.0 (474-476); World::spawn_particle order is
  (position, offset, max_speed, particle_count, particle) (world/mod.rs:1819-1826). Exact match.
- entities/jukebox.rs chunk_data_nbt/write_nbt ticks: FIXED on master, VERIFIED.
  JukeboxBlockEntity.saveAdditional
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java:129-138)
  writes ticks_since_song_started only while getSong() != null; Pumpkin gates the key on
  song_length_ticks > 0 (94) and chunk_data_nbt reuses write_nbt (147-152).
- blocks/jukebox.rs:192 on_state_replaced delegation: VERIFIED. Only updates the output
  signal; vanilla JukeboxBlock.affectNeighborsAfterRemoval
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/JukeboxBlock.java:79-82) only
  calls Containers.updateNeighboursAfterDestroy; the record drop lives in
  preRemoveSideEffects -> popOutTheItem (JukeboxBlockEntity.java:193-196, 88-102), mapped
  to JukeboxBlockEntity::on_block_replaced (179-199). No double drop.

No .rs files touched, so no rustfmt run. Cannot compile in this CODER role.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-taudit-crates-pumpkin-core-src-bl-31ea5a-0.md
