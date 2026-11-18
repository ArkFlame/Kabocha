# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: jukebox.rs

- `blocks/jukebox.rs:192` `on_state_replaced` delegation (STILL OPEN): verified correct, no change.
Out of scope (not a tracker STILL OPEN row, one-line reason): Bedrock block-actor data dropping
Audit pass 7 (2026-11-18). Final verification of all rows marked STILL OPEN and ALREADY FIXED (UNVERIFIED) against decompiled vanilla sources in `.vendor/vanilla-decompiled/net/minecraft/`. No `.rs` source changes were needed - all behaviour matches vanilla exactly.
- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): verified correct, no change. `JukeboxSongPlayer.tick` (JukeboxSongPlayer.java:64-77) emits JUKEBOX_PLAY + particles while `ticksSinceSongStarted % 20 == 0` BEFORE the increment; `spawnMusicParticles` (lines 83-89) sends count 0, offset `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at `pos+0.5,+1.2,+0.5`. Pumpkin's `tick` (102-133) and `note_particle_hue` (479-481) match exactly. `JukeboxSong.hasFinished` uses `ticksElapsed >= lengthInTicks()+20` (JukeboxSong.java:50-52), matching `song_length_ticks` and the `ticks >= song_length` stop check.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
