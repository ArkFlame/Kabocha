# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-1655e2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: blocks.rs predicate.rs trigger.rs use_item.rs use_item_on.rs

7. `ITEM_USED` stat in `use_item.rs:40`: STILL OPEN. Vanilla awards `Stats.ITEM_USED` inside individual
Re-verified the three STILL OPEN rows (5, 6, 7) against `.vendor/vanilla-decompiled` (2026-11-18), smallest first. The prior pass's fixes for rows 5 and 7 are present in the current files; no further in-unit code change was needed.
6. Enabled-feature gate: STILL OPEN. Vanilla `ServerPlayerGameMode#useItemOn:539` returns FAIL when `!block.isEnabled(level.enabledFeatures())` and `ServerGamePacketListenerImpl#handleUseItemOn:2307` / `handleUseItem:2397` check `itemStack.isItemEnabled(features)`. Pumpkin's generated `Block`/`Item` data has no feature-flag field (`pumpkin-data/src/blocks.rs:21` has none) and no enabled-feature resolution exists (`grep isEnabled|enabledFeatures` finds only the `CFeatureFlags` protocol packet), so the gate needs generated data plus datapack feature resolution, both outside this unit.
Re-verified the sole remaining STILL OPEN row (6) against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java` (2026-11-18). Spot-checked that rows 5 and 7 are still resolved in the current files (no regression): `use_item_on.rs:326` serializes `property_values(state_id)`, `trigger.rs:82/581` carries and passes the generic `Vec<(String, String)>` state map, `predicate.rs:661` looks up `state_key` by name, and `use_item.rs:39-42` has no central `ITEM_USED` award.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
