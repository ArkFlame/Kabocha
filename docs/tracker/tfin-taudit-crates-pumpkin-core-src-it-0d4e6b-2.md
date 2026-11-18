# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-0d4e6b.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bucket.rs

- Row 5 (hardcoded Hand::Right, bucket.rs:783-798): STILL OPEN. Needs ItemBehaviour::use_on_block
- Row 6 (event payload lacks face/stack, bucket.rs:642-654/726-736): STILL OPEN. Additive WIT
Re-checked every STILL OPEN row against .vendor/vanilla-decompiled BucketItem.java
- Row 6 (event payload lacks face/stack): STILL OPEN, out of scope. PlayerBucketEmptyEvent /

## Result

- Row 5 (hardcoded Hand::Right, bucket.rs:823-841): FIXED. use_on_block derives the used hand via item::hand_holding(player, stack.item) (item/mod.rs:28) and threads it into place_filled_bucket, so an off-hand bucket no longer edits the main-hand slot. No trait signature change needed; verified against BucketItem.use (decompiled .vendor/.../BucketItem.java:45-106) where UseOnContext.getHand() is used. Pure unit test seam: none (requires a live Player/World).
- Row 6 (event payload lacks face/stack, bucket.rs:642-654/726-736): FIXED/VERIFIED. PlayerBucketFillEvent and PlayerBucketEmptyEvent (pumpkin-core/src/plugin/api/events/player/player_bucket.rs) carry block_face and bucket; bucket.rs builds them with block_face_name(hit.direction)/block_face_name(direction) and the bucket registry key. Matches Bukkit PlayerBucketEmptyEvent/PlayerBucketFillEvent getFace()/getItemStack() in Pumpkin's string representation. Unit test block_face_names_match_bukkit (bucket.rs:918) covers all six BlockDirection variants. WIT v0.1/v0.2 already declare bucket and block-face.

Files changed this pass: none (fixes already present; verified against decompiled BucketItem.java). No cargo/git run (forbidden).
