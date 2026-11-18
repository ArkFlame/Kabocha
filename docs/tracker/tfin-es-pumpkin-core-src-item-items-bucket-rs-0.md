# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-item-items-bucket-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All seven rows are FIXED/VERIFIED; no STILL OPEN or BLOCKED-DATA rows remain.
All seven rows are FIXED/VERIFIED; no STILL OPEN or BLOCKED-DATA rows remain.

## Result
FINAL PASS per-row results. Each row re-verified against decompiled 1.21 vanilla at
.vendor/vanilla-decompiled/net/minecraft/ and the current bucket.rs. All fixes are already
present; no source edits needed. No cargo/git run (forbidden for this worker).

- Row 1 (waterlogged pickup, bucket.rs:311-333): FIXED/VERIFIED. SimpleWaterloggedBlock.java:39-50
  pickupBlock sets WATERLOGGED=false then destroyBlock(pos, true) when state.canSurvive is
  false; Pumpkin set_waterlogged(false) + can_place_at + break_block with drops matches.
- Row 2 (sneak waterlogging, bucket.rs:443): FIXED/VERIFIED. BucketItem.java:146 gates
  canPlaceFluidInsideBlock on (!shiftKeyDown || hitResult == null) and recurses at :190;
  Pumpkin is_water && !is_sneaking matches. Removing it would be wrong.
- Row 3 (ultrawarm mob bucket, bucket.rs:555/570-599): FIXED/VERIFIED. BucketItem.java:158-170
  takes the fire-extinguish branch and returns true without playEmptySound; use():64 still runs
  checkExtraContent and MobBucketItem.java:34-39 spawns the mob. Pumpkin skips the empty sound
  on evaporation and still spawns.
- Row 4 (mayInteract / spawn protection, bucket.rs:662/751): FIXED/VERIFIED. BucketItem.java:60
  gates use() on level.mayInteract(player, pos); Pumpkin is_in_spawn_protection gates match.
- Row 5 (hardcoded Hand::Right, bucket.rs:836-840): FIXED/VERIFIED. UseOnContext.getHand()
  is mirrored by hand_holding(player, stack.item) (item/mod.rs:28); the used hand edits its own slot.
- Row 6 (event payload face/stack, bucket.rs:659/675/770): FIXED/VERIFIED. player_bucket.rs
  PlayerBucketFillEvent/EmptyEvent carry bucket registry key + block_face_name(direction), matching
  CraftBukkit PlayerBucketEmptyEvent/FillEvent getFace()/getItemStack(). Test
  block_face_names_match_bukkit (:920) covers all six BlockDirection variants.
- Row 7 (dispenser break_block drops, redstone/dispenser.rs): FIXED/VERIFIED (out of scope
  basename, no fix needed). BucketItem.java:176-177 destroyBlock(pos, true) for replaceable
  blocks makes rolling loot correct; SKIP_DROPS would break parity.

Compile-pitfall re-check: no Rust edited this pass. Existing code uses unwrap_or, hand_holding
imported from crate::item, block_face_name is a const fn with exhaustive six-variant match, and
the test uses assert_eq! on &str (Debug ok). No new imports, helpers, structs, clones, or
pedantic issues.

All seven rows FIXED/VERIFIED; no STILL OPEN or BLOCKED-DATA rows remain. Files changed: none.
