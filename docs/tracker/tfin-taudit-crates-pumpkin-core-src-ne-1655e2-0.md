# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-1655e2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Rows: STILL OPEN. Note body not recoverable in this pass; nothing verified.
Re-verified each STILL OPEN row against `.vendor/vanilla-decompiled` (2026-11-17), smallest first:
5. Advancement `state`: FIXED (already in code). `use_item_on.rs` `trigger_item_used_on_block_advancement` serializes every property of the clicked block state via `block.property_values(state_id)`, matching vanilla `ItemUsedOnLocationTrigger` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java` path and advancement trigger class).
6. Enabled-feature gate: BLOCKED-DATA. Vanilla `ServerPlayerGameMode.java:539` checks `state.getBlock().isEnabled(level.enabledFeatures())`. Missing generated field: per-block `required_features` in `crates/pumpkin-data/src/generated/block.rs` (count of `required_feature`/`enabled_features` = 0). Gate needs that field first.

## Result
- Gap 5 (advancement state): FIXED (verified existing code, no edit).
- Gap 6 (enabled-feature gate): BLOCKED-DATA, missing generated `Block.required_features` in `pumpkin-data`.
- Earlier-pass gap 1 (out-of-reach log level): FIXED in `crates/pumpkin-core/src/net/java/play/mod.rs` (`BlockOutOfReach` now `Level::DEBUG`, vanilla debug-logs).
- Earlier-pass gap 4 (off-hand block default use): VERIFIED no change needed; `ServerPlayerGameMode.java:625` gates `TryEmptyHandInteraction` on `hand == InteractionHand.MAIN_HAND`, same as Pumpkin.
- Gap 7 (`use_item.rs` stat before consume): OPEN, size M (needs `ItemBehaviour` return-type change), not closed in this pass.
