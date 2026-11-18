# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: jukebox.rs

- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): FIXED on master, verified.
- `blocks/jukebox.rs:192` `on_state_replaced` (was STILL OPEN): VERIFIED. Only updates the output signal;
Out of scope (not a tracker STILL OPEN row): Bedrock block-actor data dropping item components
behaviour matches vanilla. No row remains STILL OPEN.

## Result
Audit pass 11 (2026-11-18, FINAL PASS). Re-read the current source of block/entities/jukebox.rs (write_nbt 81-100, tick 102-133, chunk_data_nbt 147-152, note_particle_hue 474-476) and block/blocks/jukebox.rs (on_state_replaced 192-195) plus the cited decompiled vanilla classes. All three tracker STILL OPEN rows end closed; no .rs change was needed because each already matches vanilla exactly.

- entities/jukebox.rs tick particle order/payload: VERIFIED. JukeboxSongPlayer.tick (.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSongPlayer.java:64-81) emits JUKEBOX_PLAY plus particles while ticksSinceSongStarted % 20 == 0 BEFORE incrementing; spawnMusicParticles (83-89) sends count 0, offset (nextInt(4)/24.0F, 0, 0), speed 1.0 at pos+0.5,+1.2,+0.5. Pumpkin's tick (102-133) emits before fetch_add, and note_particle_hue (474-476) is step/24.0; World::spawn_particle (world/mod.rs:1819-1826) takes (position, offset, max_speed, particle_count, particle), so the call passes speed 1.0 and count 0. Exact match; unit test note_particle_hue_uses_vanilla_quarter_steps covers the hue.

- entities/jukebox.rs chunk_data_nbt/write_nbt ticks: FIXED on master, VERIFIED. JukeboxBlockEntity.saveAdditional (.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java:129-138) writes ticks_since_song_started only while getSong() != null. Pumpkin's write_nbt gates the key on song_length_ticks > 0 (line 94) and chunk_data_nbt (147-152) reuses write_nbt. Tests: write_nbt_omits_ticks_since_song_started_when_stopped (504), write_nbt_writes_ticks_since_song_started_while_playing (513), write_nbt_keeps_zero_ticks_while_song_is_loaded (523), chunk_data_nbt_omits_ticks_when_stopped (534). The pass-2 prose claim that vanilla writes the key with no song is contradicted by the cited source and is discarded.

- blocks/jukebox.rs:192 on_state_replaced: VERIFIED. Only updates the output signal; vanilla JukeboxBlock.affectNeighborsAfterRemoval (.vendor/vanilla-decompiled/net/minecraft/world/level/block/JukeboxBlock.java:79-82) only calls Containers.updateNeighboursAfterDestroy. The record drop lives in JukeboxBlockEntity.preRemoveSideEffects -> popOutTheItem (JukeboxBlockEntity.java:88-102, 193-196), mapped to JukeboxBlockEntity::on_block_replaced. No double drop.

Out of scope (not a tracker STILL OPEN row): Bedrock block-actor data omitting item components in build_jukebox_actor needs Bedrock component-serialization format work (client-only). No row remains STILL OPEN.


## Result (x_01)
- Jukebox `write_nbt` / `tick` / `chunk_data_nbt` (`block/entities/jukebox.rs:81,102,147`): VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java`; the tests at `jukebox.rs:504-534` cover the stopped/playing ticks.
- `on_state_replaced` (`block/blocks/jukebox.rs:192-195`): VERIFIED, output-signal update only.
- Bedrock block-actor item components: out of scope (client format work), not a STILL OPEN row.
Files edited: none. Checks: none run (CODER cannot compile).
