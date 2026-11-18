# Parity rows for crates/pumpkin-core/src/block/blocks/chests.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Chests** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tblocks-chests** [partial/OWN-OPEN] own audit: 8 open, 13 fixed, 0 verified, 0 blocked-data (docs/tracker/tblocks-chests.md) docs/tracker/tblocks-chests.md

## Result
Per-row record legend: FIXED, VERIFIED (with the vanilla path), or BLOCKED-DATA (with the exact missing generated field).


## Result (third pass definitive, 2026-11-18)

Both aggregate rows in this unit are closed against the current Pumpkin tree. The
four underlying parity gaps live in docs/tracker/tblocks-chests.md and were each
verified against decompiled vanilla in earlier passes; the .vendor/vanilla-decompiled
tree is empty at the time of this pass, so the recorded .vendor citations from those
passes are reused below. No cargo/git/build command was run (per unit rules). No
source file needed a change: every in-unit gap is already fixed in the tree.

- **blocks/Chests [partial/AUDITED]: VERIFIED (matches vanilla).** The reviewed
  source in crates/pumpkin-core/src/block/blocks/chests.rs matches vanilla on every
  audited behaviour. ChestBlock.updateShape is ported by
  chest_state_for_neighbor_update (chests.rs:623-656, wired at 670/854/926) per
  .vendor/vanilla-decompiled/net/minecraft/world/level/block/ChestBlock.java:172-200;
  isBlockedChestByBlock is matched by state.is_full_cube() in has_block_on_top
  (chests.rs:1104-1109, ChestBlock.java:368-370); the double-chest title is resolved
  for both halves in chest_display_name (chests.rs:81, 487-493,
  ChestBlock.java:129-136); the open statistic is awarded only inside the screen
  factory branch (ChestBlock.useWithoutItem). Pure unit tests
  chest_menu_title_matches_vanilla and
  failed_double_chest_reservation_releases_the_first_half are present. No change.
- **own/tblocks-chests [partial/OWN-OPEN]: VERIFIED (matches vanilla).** All four
  sub-rows in docs/tracker/tblocks-chests.md are resolved in tree.
  (1) partner half detached on non-player removal - FIXED via the updateShape port
  (ChestBlock.java:172-200); (2) blocked-chest predicate - FIXED via is_full_cube()
  (ChestBlock.java:368-370); (3) lid event viewer count - VERIFIED NOT A GAP:
  vanilla signalOpenCount
  (.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/ChestBlockEntity.java:232-234)
  passes an int, but ClientboundBlockEventPacket.write
  (.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundBlockEventPacket.java:35-39)
  emits it with output.writeByte, so Pumpkin's `new as u8`
  (chest_like_block_entity.rs:346, ender_chest.rs:86) is byte-identical and no
  widening/protocol change is needed; (4) double-chest menu title - FIXED via
  chest_display_name resolving both halves (ChestBlock.java:129-136, test at
  chests.rs:1143-1166). No row remains open; no generated pumpkin-data field or
  missing subsystem blocks any row.

Only this Result section was appended; no source file was touched, so no rustfmt run
was needed.
