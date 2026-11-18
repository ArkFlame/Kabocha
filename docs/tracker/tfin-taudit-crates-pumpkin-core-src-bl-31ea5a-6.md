# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Tick particle order/payload (was STILL OPEN): VERIFIED. `JukeboxSongPlayer.tick`
- `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): FIXED on master, VERIFIED.
- `on_state_replaced` delegation (was STILL OPEN): VERIFIED.
BLOCKED-DATA (not a tracker STILL OPEN row, one-line reason): Bedrock block-actor data

## Result
FINAL PASS (2026-11-18). All rows re-checked against decompiled vanilla; no .rs change was needed (every fix is already on master).

- Tick particle order/payload: VERIFIED. `JukeboxSongPlayer.tick` (`.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSongPlayer.java:64-89`) emits JUKEBOX_PLAY + NOTE particles while `ticks % 20 == 0` BEFORE the increment; `spawnMusicParticles` (83-89) sends count 0, offset `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at `pos+0.5,+1.2,+0.5`. Pumpkin `tick` (`entities/jukebox.rs:102-133`) and `note_particle_hue` (473-481) match; `spawn_particle` arg order is (position, offset, max_speed, particle_count, particle) (`world/mod.rs:1819-1826`). Test `note_particle_hue_uses_vanilla_quarter_steps` covers the hue.

- `chunk_data_nbt`/`write_nbt` ticks: FIXED on master, VERIFIED. `JukeboxBlockEntity.saveAdditional` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/JukeboxBlockEntity.java:129-138`) writes `RecordItem` only when non-empty and the ticks key only while `getSong() != null`. Pumpkin gates the key on `song_length_ticks > 0` (line 94) and `chunk_data_nbt` reuses `write_nbt` (147-152). Tests `write_nbt_omits_ticks_since_song_started_when_stopped`, `write_nbt_writes_ticks_since_song_started_while_playing`, `write_nbt_keeps_zero_ticks_while_song_is_loaded`, `chunk_data_nbt_omits_ticks_when_stopped` cover both branches.

- `on_state_replaced` delegation: VERIFIED. `JukeboxBlock.affectNeighborsAfterRemoval` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/JukeboxBlock.java:79-82`) only calls `Containers.updateNeighboursAfterDestroy`; the record drop lives in `preRemoveSideEffects -> popOutTheItem` (`JukeboxBlockEntity.java:88-102,193-196`). Pumpkin `on_state_replaced` (`blocks/jukebox.rs:192-195`) only updates the output signal and delegates the drop to `on_block_replaced` (`entities/jukebox.rs:179-199`); no double drop.

- Bedrock block-actor data: BLOCKED-DATA (not a tracker STILL OPEN row). Missing subsystem: Bedrock item-component serialization for `RecordItem`. `build_jukebox_actor` (`entities/jukebox.rs:450-471`) and the shared helper `build_bedrock_slot_items` (`bedrock.rs:118-136`) write only Name/Count/Damage/Slot; there is no writer for a Bedrock `RecordItem.components` tag. Client-only Bedrock concern with no decompiled Java vanilla equivalent.
