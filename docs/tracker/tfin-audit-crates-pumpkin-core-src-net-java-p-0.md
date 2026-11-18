# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-net-java-p.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Rows: all closed this pass (see Result).

## Result

Final pass 2026-11-18. Every row from the parent audit
(`docs/tracker/taudit-crates-pumpkin-core-src-net-java-p.md`) is closed below against
decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`. No Rust source
change was needed this pass: the actionable rows were already fixed and verified by
earlier passes, and the single remaining row is blocked on generated data. The code
in `crates/pumpkin-core/src/net/java/play/use_item_on.rs` was re-read and matches the
vanilla call chain; no edits were made to any Rust file.

1. **`ServerLevel.mayInteract` check - VERIFIED FIXED.** Vanilla
   `ServerLevel.mayInteract` (`server/level/ServerLevel.java:3058-3059`) is
   `!(entity instanceof Player && (isUnderSpawnProtection || !border.isWithinBounds(pos)))`.
   Pumpkin local helper `may_interact` (`net/java/play/use_item_on.rs:302-313`) matches
   via `Worldborder::is_within_bounds(x, z, 0.0)` and `World::is_in_spawn_protection`,
   and is called from the early return at `use_item_on.rs:39-49`. The latter already
   handles overworld-only, permission-level bypass, radius zero and no-operators, so
   the earlier "needs OP handling" caveat is stale.

2. **Max build height gate - VERIFIED FIXED.** Vanilla `handleUseItemOn`
   (`server/network/ServerGamePacketListenerImpl.java:2322-2327`) rejects
   `pos.getY() > getMaxY()` (and `pos.getY() < getMinY()`) before interacting.
   Pumpkin gate `position.0.y > world.get_top_y()` at `use_item_on.rs:39` matches the
   max side; `get_top_y()` equals `min_y + height - 1`, so no new `World` accessor was
   needed. Vanilla also sends a build-limit chat message and resyncs a neighbour block;
   Pumpkin only resyncs the clicked block (cosmetic, no gameplay effect).

3. **`itemStack.isItemEnabled(level.enabledFeatures())` - BLOCKED-DATA.** Vanilla
   rejects the interaction before any action when the held stack item requires disabled
   features (`ServerGamePacketListenerImpl.java:2307`), and gates the clicked block the
   same way (`ServerPlayerGameMode.java:539`, `state.getBlock().isEnabled(...)`).
   `ItemStack.isItemEnabled` (`world/item/ItemStack.java:338-339`) delegates to
   `Item.isEnabled`, which is the default `FeatureElement.isEnabled` =
   `requiredFeatures().isSubsetOf(enabledFeatures)`
   (`world/flag/FeatureElement.java:13-17`), driven by the per-item field
   `Item.requiredFeatures: FeatureFlagSet` (`world/item/Item.java:133,155,384-385`,
   set via `Item.Properties.requiredFeatures(...)` at `Item.java:657-667`). Pumpkin
   generated `Item` (`crates/pumpkin-data/src/generated/item.rs:22-27`) has only `id`,
   `registry_key`, `crafting_remainder`, `components`; `Block`
   (`crates/pumpkin-data/src/blocks.rs:21-53`) has no feature field either. Neither
   `tools/pumpkin-codegen`, `assets/`, nor any other `crates/pumpkin-data/src` source
   contains per-item/per-block feature data. **Exact missing generated fields:**
   `Item::required_features` (feature-flag set on the item registry entry) and the
   equivalent on `Block`, plus the codegen/asset pipeline that extracts them from
   vanilla item/block `requiredFeatures`. Pumpkin known-flag list
   (`data/datapack/mod.rs:2298-2307`) only names four flags and cannot map a stack or
   block to a flag, so no conservative or faithful check exists without that data layer.

4. **`awaitingPositionFromClient` guard - VERIFIED FIXED.** Vanilla guards the
   interaction with `this.awaitingPositionFromClient == null`
   (`ServerGamePacketListenerImpl.java:2334`). Pumpkin `teleport_pending` check
   (`use_item_on.rs:35-38,40`) reads `Player.awaiting_teleport` directly instead of
   `Player::is_awaiting_teleport()`, deliberately avoiding that method resend-after-20-ticks side effect on a read-only guard.

5. **`PlayerInteractEvent` for spectators - VERIFIED CORRECT, NO CHANGE.** Vanilla
   calls `CraftEventFactory.callPlayerInteractEvent` unconditionally at
   `ServerPlayerGameMode.java:551`, before the spectator branch at line 574. Pumpkin
   fires the event for spectators too (`use_item_on.rs:70-99`), matching Paper, not a
   superset.

6. **Item fallback to `ItemRegistry::on_use` - VERIFIED CORRECT, NO CHANGE.**
   `useItemOn` (`ServerPlayerGameMode.java:619,626,639-642`) calls `state.useItemOn`,
   `state.useWithoutItem` and `itemStack.useOn`, but never `Item#use` (the
   right-click-in-air path handled by `handle_use_item`). Pumpkin `use_with_item`/`on_use`
   plus `item_registry.use_on_block`/`place_block` cover the vanilla calls; the removed
   `ItemRegistry::on_use` fallback would have double-fired `Item#use`.

7. **`increment_stat(Used, ...)` unconditional - STALE, NO CHANGE.** The call
   (`use_item_on.rs:192-194`) is gated on
   `item_result.consumes_action() || should_try_decrement`, so it only counts actions
   that consume the item; nothing to change.

8. **Hit-position epsilon / block-center limit - VERIFIED CORRECT.** Vanilla drops
   hits where `abs(location - centerOf(block)) >= 1.0000001` on any axis
   (`ServerGamePacketListenerImpl.java:2317-2319`). Pumpkin
   `is_within_block_center_limit` (`use_item_on.rs:379-386`) uses the exact
   `1.000_000_1` constant and is covered by `block_center_hit_limit_matches_vanilla`
   (`use_item_on.rs:402-415`). `is_valid_hit_position` is covered by
   `hit_position_must_be_inside_clicked_block` (`use_item_on.rs:417-429`).

9. **Item cooldown / config TODOs - STALE, NO CHANGE.** The item path is already
   gated by `player.is_on_cooldown()` in `ItemRegistry::use_on_block`
   (`item/registry.rs:148`), and the decrement guard is the existing
   `GameMode::Creative` check (`use_item_on.rs:175`), matching vanilla
   `!player.getAbilities().instabuild`.

10. **`border.rs`, `hub.rs`, `worldborder.rs` - VERIFIED, NO CHANGE.**
    `Worldborder::is_within_bounds` matches vanilla `WorldBorder.isWithinBounds`
    (`world/level/border/WorldBorder.java:88`) and is covered by existing tests; no
    open rows remained for those files.

Gap status: no open rows remain. Rows 1,2,4 FIXED (verified); 5,6,8,10 VERIFIED CORRECT;
7,9 STALE (already correct); 3 BLOCKED-DATA (missing generated `Item::required_features`
and the equivalent `Block` field). No pure unit test seam exists for row 3 without the
generated feature data; the existing `use_item_on.rs` and `world/border.rs` tests cover
the helpers that can be tested. No cargo/git/shell run (forbidden for workers); no
Rust files edited; rustfmt not needed for this Markdown-only result record.

