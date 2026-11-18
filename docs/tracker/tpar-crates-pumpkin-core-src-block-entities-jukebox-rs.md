# Parity rows for crates/pumpkin-core/src/block/entities/jukebox.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-31ea5a** [partial/OWN-OPEN] own audit: 21 open, 14 fixed, 8 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

## Result
Row `own/taudit-crates-pumpkin-core-src-bl-31ea5a` (the single aggregate row of the jukebox
deep-audit doc): VERIFIED. Every jukebox sub-row is FIXED or VERIFIED against the decompiled
vanilla sources in `.vendor/vanilla-decompiled/net/minecraft/` (detail below); the source
`crates/pumpkin-core/src/block/entities/jukebox.rs` matches vanilla and needs no change.

Third pass (2026-11-18). The single row `own/taudit-crates-pumpkin-core-src-bl-31ea5a` is the
aggregate of the jukebox deep-audit doc. Re-read that audit's final pass and re-verified every
sub-row directly against the decompiled vanilla sources in `.vendor/vanilla-decompiled/net/minecraft/`.

Row `own/taudit-crates-pumpkin-core-src-bl-31ea5a`: VERIFIED. The referenced audit has no row left
STILL OPEN; every sub-row is FIXED or VERIFIED against vanilla and the source
`crates/pumpkin-core/src/block/entities/jukebox.rs` matches vanilla:

- Tick particle order/payload: VERIFIED. `JukeboxSongPlayer.tick`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/JukeboxSongPlayer.java:64-77`) emits
  JUKEBOX_PLAY plus NOTE particles while `ticks % 20 == 0` BEFORE the increment;
  `spawnMusicParticles` (83-89) sends count 0, offset `(nextInt(4)/24.0, 0, 0)`, speed 1.0 at
  pos+(0.5,1.2,0.5). Pumpkin `tick` (110-130) emits before `fetch_add` and `note_particle_hue`
  (474-476) is `step/24.0`; `World::spawn_particle` arg order confirmed (world/mod.rs:1819-1826).
  `JukeboxSong.hasFinished` = `length+20` (`world/item/JukeboxSong.java:50-52`) matches
  `song_length_ticks` / `SONG_END_PADDING_TICKS` 20.
- `write_nbt` / `chunk_data_nbt` ticks: FIXED and VERIFIED. `JukeboxBlockEntity.saveAdditional`
  (`world/level/block/entity/JukeboxBlockEntity.java:129-138`) writes the ticks key only while
  `getSong() != null`; Pumpkin gates it on `song_length_ticks > 0` (94) and `chunk_data_nbt`
  reuses `write_nbt` (147-152). Tests cover empty, playing and resumed-at-zero branches.
- `set_record` / `can_transfer_to` (were ALREADY FIXED UNVERIFIED): VERIFIED against
  `JukeboxBlockEntity.setTheItem` (`JukeboxBlockEntity.java:153-164`) and `canTakeItem`
  (188-191); Pumpkin `set_record` calls `stop_playing` (231), `can_transfer_to` checks any empty
  target slot (418-428).
- `on_block_replaced` / `on_state_replaced`: VERIFIED. `popOutTheItem`
  (`JukeboxBlockEntity.java:88-102`) drop offset `offsetRandomXZ(random,0.7F)` matches Pumpkin's
  `random_range(-0.35..0.35)` (184-186); `affectNeighborsAfterRemoval`
  (`world/level/block/JukeboxBlock.java:79-82`) only calls `updateNeighboursAfterDestroy`, mapped
  to `update_neighbour_for_output_signal` (blocks/jukebox.rs:192-195). No double drop.
- `normal_use` / `use_with_item` / `set_stack` / `Clearable::clear`: VERIFIED against
  `JukeboxBlock.useWithoutItem`/`useItemOn` (`JukeboxBlock.java:48-77`),
  `JukeboxPlayable.tryInsertIntoJukebox` (`world/item/JukeboxPlayable.java:39-61`) and
  `JukeboxBlockEntity.setTheItem` (`JukeboxBlockEntity.java:153-164`).

BLOCKED-DATA (not a STILL OPEN sub-row of this unit; out-of-scope subsystem): Bedrock block-actor
data omits item components in `RecordItem` (`build_jukebox_actor` jukebox.rs:450-471). The whole
Bedrock item-stack component-NBT serialization subsystem is missing: the Bedrock builder writes
only Name/Count/Damage/Slot for every container block entity, so no block entity persists item
components to Bedrock NBT. Needs that subsystem; not fixable in a single block-entity unit.

No `.rs` source file changed (all rows already match vanilla), so no rustfmt and no new test;
existing NBT/comparator/can_transfer_to tests already cover the fixed branches. Cannot compile in
this CODER role.

Status: PASS fixed=0 open=0 files=docs/tracker/tpar-crates-pumpkin-core-src-block-entities-jukebox-rs.md
