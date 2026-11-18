# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All seven rows from docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md verified
end to end against the decompiled 1.21 source and the current Pumpkin code. No source
edits this pass: every in-scope fix was landed by earlier passes and matches vanilla.

- Row 1 (waterlogged pickup, bucket.rs:311-333): VERIFIED.
  .vendor/vanilla-decompiled/net/minecraft/world/level/block/SimpleWaterloggedBlock.java:39-50
  sets WATERLOGGED=false, then destroyBlock(pos, true) when the dried state cannot
  survive. Pumpkin's set_waterlogged(false) + can_place_at(dried state) + break_block
  with drops reproduces it. No change.
- Row 2 (sneak waterlogging, bucket.rs:443): VERIFIED, no change.
  .vendor/vanilla-decompiled/net/minecraft/world/item/BucketItem.java:143-146 gates
  canPlaceFluidInsideBlock on (!shiftKeyDown || hitResult == null), so sneaking recurses
  to the adjacent position (line 190). Pumpkin's !is_sneaking matches; removing it
  would break parity.
- Row 3 (ultrawarm mob bucket, bucket.rs:555-568): FIXED (prior pass), VERIFIED.
  BucketItem.java:158-170 takes the fire-extinguish branch and returns true without
  playEmptySound; use():64 still runs checkExtraContent and
  MobBucketItem.java:33-39 spawns the mob. Pumpkin skips the empty sound on
  evaporation and still spawns. Matches.
- Row 4 (mayInteract / spawn protection, bucket.rs:661-664, 750-753): FIXED (prior
  pass), VERIFIED. BucketItem.java:60 gates the interaction on level.mayInteract.
  Pumpkin's is_in_spawn_protection gates on both pickup and place match. mayUseItemAt
  is region/spectator-only and is covered by plugin events.
- Row 5 (hardcoded Hand::Right, bucket.rs:834-840): FIXED (prior pass), VERIFIED.
  Vanilla reads UseOnContext.getHand() (BucketItem.java:47,63). use_on_block now
  derives the hand via hand_holding(player, stack.item) (item/mod.rs:28) and writes
  the result back to that hand; off-hand no longer edits the main-hand slot.
- Row 6 (event payload face/stack, bucket.rs:659/675, 770; plugin player_bucket.rs;
  event.wit; both wasm hosts): FIXED (prior pass), VERIFIED end to end. Core
  PlayerBucketEmptyEvent / PlayerBucketFillEvent carry bucket + block_face;
  bucket.rs fills them from block_face_name(hit.direction/direction) and the bucket
  registry key; v0.1/v0.2 event.wit records declare bucket and block-face
  (v0.2/event.wit:1053-1067, v0.1/event.wit:948-963); host-v0_2/src/events/player.rs:1125-1142
  and host-v0_1 map both fields. Unit test block_face_names_match_bukkit (bucket.rs:919)
  covers all six BlockDirection variants. Bukkit's getFace()/getItemStack() are
  represented by the block_face/bucket strings.
- Row 7 (dispenser break_block drops, redstone/dispenser.rs:910-923): VERIFIED, no
  change (file outside this unit). Vanilla filled-bucket dispensing runs
  BucketItem.emptyContents, which does destroyBlock(pos, true) at BucketItem.java:176-177
  for replaceable blocks, so rolling loot is correct; adding SKIP_DROPS would break parity.

## Result
All seven rows closed: FIXED (3, 4, 5, 6) or VERIFIED parity (1, 2, 7). No BLOCKED-DATA.
No cargo/git run (forbidden for this worker); no Rust edited this pass, so no rustfmt
needed. Compile-pitfall re-check: no new imports, helpers, struct initializers, clones,
or trait-impl placements; bucket.rs's existing test uses assert_eq! on &str (Debug ok)
and block_face_name is exhaustive over the six BlockDirection variants.
