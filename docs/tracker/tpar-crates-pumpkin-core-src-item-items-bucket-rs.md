# Parity rows for crates/pumpkin-core/src/item/items/bucket.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-it-0d4e6b** [partial/OWN-OPEN] own audit: 13 open, 7 fixed, 15 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md) docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (pass 3 verification, 2026-11-18)

The unit's single row is the own-audit summary for
docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md, whose seven parity
rows were all addressed by earlier passes. Re-verified every row against
decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/ and the current
bucket.rs. No source edits needed this pass: all fixes are already present in
bucket.rs and match vanilla. No cargo/git run (forbidden for this worker). No
Rust touched, so rustfmt not needed this pass.

- Row 1 (waterlogged pickup, bucket.rs:311-333): VERIFIED. Dried-state
  set_waterlogged(false) + can_place_at + break_block with drops matches
  SimpleWaterloggedBlock.pickupBlock destroyBlock(pos, true).
- Row 2 (sneak waterlogging, bucket.rs:442): VERIFIED, no change. BucketItem.java
  emptyContents:146 gates canPlaceFluidInsideBlock on
  (!shiftKeyDown || hitResult == null) and recurses at :190; Pumpkin's
  !is_sneaking matches. The doc's proposed removal would be WRONG.
- Row 3 (ultrawarm mob bucket, bucket.rs:555-598): VERIFIED. BucketItem.java:158-170
  takes the fire-extinguish branch and returns true without playEmptySound;
  use():64 still runs checkExtraContent so MobBucketItem spawns the mob. Pumpkin
  skips the empty sound on evaporation (:555) and still spawns (:570). Correct.
- Row 4 (mayInteract / spawn protection, bucket.rs:662/751): VERIFIED.
  BucketItem.java use():60 gates on level.mayInteract(player, pos); Pumpkin's
  is_in_spawn_protection gates match. mayUseItemAt is region/spectator only.
- Row 5 (hardcoded Hand::Right, bucket.rs:836-840): VERIFIED. hand_holding
  (item/mod.rs:28) derives the used hand; off-hand no longer edits the main
  hand slot, matching UseOnContext.getHand().
- Row 6 (event payload face/stack, bucket.rs:659/675/770): VERIFIED. Events carry
  bucket registry key + block_face_name(direction), matching CraftBukkit
  PlayerBucketEmptyEvent/FillEvent getFace()/getItemStack(). Unit test
  block_face_names_match_bukkit (:920) covers all six BlockDirection variants.
- Row 7 (dispenser break_block drops, redstone/dispenser.rs): VERIFIED, out of
  scope (basename not permitted) and no fix needed. Vanilla filled-bucket
  dispensing runs BucketItem.emptyContents:177 destroyBlock(pos, true), so
  rolling loot for replaceable blocks is correct; SKIP_DROPS would break parity.

Compile-pitfall re-check: no Rust edited this pass. Existing code uses
unwrap_or (not unwrap), hand_holding imported from crate::item, block_face_name
is a const fn with an exhaustive six-variant match, and the test uses assert_eq!
on &str (Debug ok). No new imports, helpers, structs, clones, or pedantic issues.

All seven rows are FIXED/VERIFIED; no STILL OPEN or BLOCKED-DATA rows remain.
This unit's one row is therefore VERIFIED/closed. Files changed this pass: none.
