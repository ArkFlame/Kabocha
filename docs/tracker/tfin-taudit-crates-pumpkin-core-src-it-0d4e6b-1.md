# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bucket.rs dispenser.rs

- 5 hardcoded Hand::Right (bucket.rs:783-798): STILL OPEN, M, trait change.
- 6 event payload lacks face/stack (bucket.rs:642-654, 726-736): STILL OPEN, M, additive WIT change.
- 7 dispenser break_block without SKIP_DROPS (redstone/dispenser.rs:910-923): STILL OPEN, S.
Verified all seven STILL OPEN rows against .vendor/vanilla-decompiled (BucketItem,

## Result
FINAL PASS results. All three rows were already fixed or verified by earlier passes; no source edits were needed this pass. Vanilla reference: .vendor/vanilla-decompiled/net/minecraft/world/item/BucketItem.java.

- Row 5 (hardcoded Hand::Right, bucket.rs:824-842): FIXED. `use_on_block` derives the used hand via `hand_holding(player, stack.item)` (crates/pumpkin-core/src/item/mod.rs:28) and threads it into `place_filled_bucket`, so emptying an off-hand bucket no longer edits the main-hand slot. Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/item/BucketItem.java:47` reads `player.getItemInHand(hand)`. No trait signature change required.

- Row 6 (event payload lacks face/stack, bucket.rs:659/674/770): FIXED. Core `PlayerBucketEmptyEvent` / `PlayerBucketFillEvent` (crates/pumpkin-core/src/plugin/api/events/player/player_bucket.rs) carry `bucket: String` and `block_face: String`. bucket.rs builds them with `block_face_name(hit.direction)` (:659/:674) and `block_face_name(direction)` (:770), passing the bucket registry key (:674/:769). WIT v0.1/v0.2 `event.wit` declare `bucket` and `block-face`; both wasm hosts map the fields. Pure unit test `block_face_names_match_bukkit` (bucket.rs:920) covers all six `BlockDirection` variants. Vanilla Bukkit `callPlayerBucketEmptyEvent`/`callPlayerBucketFillEvent` pass direction + itemStack (BucketItem.java:79, :150); the string fields are Pumpkin's equivalent representation.

- Row 7 (dispenser break_block without SKIP_DROPS, redstone/dispenser.rs:916-937): VERIFIED, no change. Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/item/BucketItem.java:176-177` runs `level.destroyBlock(pos, true)` for replaceable blocks in `emptyContents`, so dispensing a filled bucket rolling loot is correct; adding SKIP_DROPS would break parity. Row is outside this unit's permitted basenames but resolves without a fix.

Compile-pitfall recheck: no Rust touched this pass, so no new imports, helpers, struct initializers, clones or pedantic issues. No cargo/git run (forbidden for this worker). No open rows remain; all three rows end FIXED or VERIFIED.
