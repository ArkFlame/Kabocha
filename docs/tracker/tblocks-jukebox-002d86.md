# blocks/Jukebox - re-review (2026-11-17)

Sources: crates/pumpkin-core/src/block/blocks/jukebox.rs, crates/pumpkin-core/src/block/entities/jukebox.rs.
Reference: vanilla JukeboxBlock, JukeboxBlockEntity, JukeboxSongPlayer, JukeboxPlayableComponent,
SingleStackInventory. No decompiled sources in .vendor/ this pass.

Since the previous audit (taudit-crates-pumpkin-core-src-bl-31ea5a.md) the source changed in
f25ac931a (single JUKEBOX_PLAY event), cee6f6d37 (stop events only while playing) and 1d83cf3d0
(unconditional ticks NBT, no try_lock disc loss). All three match the vanilla members named there;
no regression found.

## Fixed this pass

- blocks/jukebox.rs:103 normal_use (JukeboxBlock.onUse -> JukeboxBlockEntity.popOutTheItem /
  JukeboxSongPlayer.stop): after drop_record the block-level stop_playing sent
  SoundStopJukeboxSong (1011) a second time while a song was playing, and sent it wrongly for a
  jukebox whose song had already finished (song player stopped, record still present). Vanilla
  sends that world event once, from the song player, and only while playing. Removed the helper;
  the eject path now only clears HAS_RECORD, emits BLOCK_CHANGE and updates neighbours.
- entities/jukebox.rs:399 Inventory::set_stack (Container.setItem -> JukeboxBlockEntity.setTheItem):
  the empty-stack branch also sent SoundStopJukeboxSong explicitly after stop_playing, doubling
  1011 while playing and sending it with nothing playing. Removed the extra send.

## Remaining gaps

| Where | Vanilla member | Gap | Fix | Size |
|---|---|---|---|---|
| entities/jukebox.rs:107-121 tick | JukeboxSongPlayer.tick | Particle fires when ticks % 20 == 0 before the increment (so also at tick 0) with payload count 1, offset (rand(0..4)/24, 0, 0). Vanilla's exact ordering/payload (cos/sin offsets, count, random y) could not be confirmed without a decompiled reference. Client-only particle. | Confirm against decompiled JukeboxSongPlayer.tick and align offsets/count. | S |
| entities/jukebox.rs:236 set_record | JukeboxBlockEntity.setTheItem | Replaces the stack without stopping a song or updating HAS_RECORD; only safe because current callers set it on an empty jukebox. A plugin calling it can desync state/sound. | Route through the same stop/HAS_RECORD logic as Inventory::set_stack. | S | FIXED (2d9f9cf48) |
| entities/jukebox.rs:368-417 set_stack | JukeboxBlockEntity.setTheItem + JukeboxPlayableComponent | HAS_RECORD is derived from stack non-emptiness instead of the jukebox_playable component; a plugin/datapack insert of a non-disc item marks the block as holding a record with no song. Vanilla only starts playback through the playable component. | Gate state change and playback on the component being present, mirroring use_with_item. | S |
| entities/jukebox.rs:451 Clearable::clear | JukeboxBlockEntity.clearContent | Stops the song (game event, 1011, neighbour update) where vanilla clearContent only empties the stack; clearing a playing jukebox through a plugin differs. | Clear the stack only; let the song player tick/stop handle playback. Needs reference confirmation. | S |
| entities/jukebox.rs:463 build_jukebox_actor | Bedrock jukebox block actor | Bedrock RecordItem NBT writes only Name/Count, dropping item components (custom name, damage, enchants). | Serialise the full item tag. Bedrock-only. | M |
| entities/jukebox.rs:431 can_transfer_to | SingleStackInventory.canTakeItem | Only checks the hopper has an empty slot; does not accept a matching partial stack like chiseled_bookshelf. No observable effect today because discs max-stack at 1. | Mirror chiseled_bookshelf.rs:335. | S | FIXED (2d9f9cf48) |
| blocks/jukebox.rs:76 placed | BlockEntityType.create / Block update shape | Unconditionally creates a new block entity; if one already exists at pos (setblock over a jukebox, structure placement) the previous instance may be orphaned instead of its contents dropped/kept. Needs a world.add_block_entity check. | Only create when no block entity exists at pos. | M |
| blocks/jukebox.rs:82 player_placed | none | Not a vanilla hook; harmless today (new BE is empty) but can force HAS_RECORD true whenever the BE already holds a disc. | Drop or keep as defensive no-op. | S |
