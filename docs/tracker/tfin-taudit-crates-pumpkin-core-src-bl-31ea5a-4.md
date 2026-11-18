# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: jukebox.rs mod.rs

- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): already fixed. `JukeboxBlockEntity.saveAdditional` (JukeboxBlockEntity.java:129-138) writes the ticks key only while `getSong() != null`; Pumpkin gates it on `song_length_ticks > 0` (line 94) and `chunk_data_nbt` reuses `write_nbt` (147-152). Tests `write_nbt_omits_ticks_since_song_started_when_stopped`, `write_nbt_writes_ticks_since_song_started_while_playing` and `chunk_data_nbt_omits_ticks_when_stopped` cover both branches (passes 3-5).
- `blocks/jukebox.rs:192` `on_state_replaced` delegation (was STILL OPEN): verified correct, no code change. Vanilla `JukeboxBlock.affectNeighborsAfterRemoval` (JukeboxBlock.java:79-82) only calls `Containers.updateNeighboursAfterDestroy`; the record drop lives in `preRemoveSideEffects -> popOutTheItem` (JukeboxBlockEntity.java:193-196), mapped to `JukeboxBlockEntity::on_block_replaced`. No double drop. `popOutTheItem's `offsetRandomXZ(random, 0.7F)` (line 94) matches Pumpkin's `random_range(-0.35..0.35)` (184-186).
(world/mod.rs:1819-1826). Verified the remaining STILL OPEN / ALREADY FIXED (UNVERIFIED)
- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): VERIFIED. `JukeboxSongPlayer.tick`

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
