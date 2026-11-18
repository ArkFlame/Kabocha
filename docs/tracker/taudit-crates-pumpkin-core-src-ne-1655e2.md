# net / use_item_on (`crates/pumpkin-core/src/net/java/play/use_item_on.rs`)

Status: AUDITED (2026-11-17). Coverage row: `audit` / `crates/pumpkin-core/src/net/java/play/use_item_on.rs`.
Vanilla classes: `ServerGamePacketListenerImpl#handleUseItemOn`, `ServerPlayerGameMode#useItemOn`,
`ServerPlayerGameMode#useItem`, `ItemStack#useOn`, `BlockState#useItemOn`, `ItemUsedOnBlockTrigger`,
`PlayerInteractEvent` (Paper `CraftEventFactory`).
Tracker note "5 TODO/FIXME/vanilla markers": 3 TODOs and 2 `vanilla` prose comments; triaged below.

## Fixed in this pass
- `use_item_on.rs:95` (old) awarded `StatisticCategory::Used` unconditionally for the hand item before the
  interaction was known to do anything, so a cancelled event, an out-of-cooldown use, a block-state interaction
  (chest/sign) or a whiff all counted as "times used". Vanilla `ServerPlayerGameMode#useItemOn` awards
  `Stats.ITEM_USED` only after `stack.useOn(...)` consumes the action (block placement included), and
  `ServerPlayerGameMode#useItem` does the same for item use. The statistic is now incremented after the result,
  for `item_result.consumes_action() || should_try_decrement` (a successful placement consumes even in creative,
  where the stack count does not change). Commit `b6dac6141`.

## Remaining gaps
1. `use_item_on.rs:26` / `net/java/play/mod.rs:105`: out-of-reach clicks return `BlockPlacingError::BlockOutOfReach`
   with `Level::WARN`. Vanilla `ServerGamePacketListenerImpl#handleUseItemOn` only debug-logs
   ("Rejecting UseItemOnPacket ... too far away"); every lagging/rubber-banding client spams the Pumpkin log at
   warn. Fix: map `BlockOutOfReach` to `Level::DEBUG`. Size: S.
2. `use_item_on.rs:50`: `world.get_block(&position) == &Block::AIR` returns early. Vanilla has no air check in
   `handleUseItemOn`/`useItemOn`; a block state always exists and placement/`useWithoutItem` decide. This is an
   anti-cheat hardening (only modified clients send a hit on air) and is stricter than vanilla, not looser.
   Vanilla class: `ServerGamePacketListenerImpl#handleUseItemOn`. Fix: none needed for parity; document. Size: S.
3. `use_item_on.rs:31-36` + `is_valid_hit_position` (`:358`): vanilla validates the packet hit position with
   `abs(hit - blockPos) < 1.0000001` on the relative position and then uses it; Pumpkin checks a `-0.5..1.5`
   window around the block center at `1e-6` and then clips the relative hit to `-1e-3..1+1e-3`. Stricter than
   vanilla (intentional, commit `6a7175af0`, #4547); a vanilla client never exceeds `0..1`. No change.
4. `use_item_on.rs:243`: `PassToDefaultBlockAction` is only resolved through `block_registry.on_use` when
   `equipment_slot == MAIN_HAND`. Vanilla `BlockState#useItemOn` resolves `InteractionResult.TRY_WITH_EMPTY_HAND`
   from `BlockBehaviour#useItemOn` through `BlockBehaviour#useWithoutItem` regardless of the hand that produced
   the packet. With an off-hand packet (only sent when the main-hand interaction passed) Pumpkin skips the block's
   default use and falls through to placement. Fix: drop the hand check (verify the 1.21 `BlockState#useItemOn`
   hand handling first). Size: S.
5. `use_item_on.rs:302`: the `minecraft:item_used_on_block` advancement payload hardcodes
   `CopperBulbLikeProperties.lit` as the `state` field. Vanilla `ItemUsedOnBlockTrigger#trigger` matches the
   advancement's `StatePropertiesPredicate` against the clicked block's own properties, so predicates on any
   other block property never match and copper-bulb `lit` is reported for every clicked block. Fix: serialize the
   clicked block state's property map generically (reuse the existing advancement predicate/property code in
   `pumpkin-data`/`entity/player/advancement`). Size: M.
6. Missing feature gate: vanilla `ServerPlayerGameMode#useItemOn` starts with
   `if (!blockState.getBlock().isEnabled(level.enabledFeatures())) return InteractionResult.FAIL;`. Pumpkin never
   consults `enabled` features in `use_item_on.rs` or `place_block`, so disabled blocks (experimental datapacks)
   can still be interacted with and placed. Fix: add the gate at the top of `handle_use_item_on`, next to the
   reach check. Size: S/M (depends on enabled-feature data being generated).
7. Sibling in `use_item.rs:40`: `Stats.ITEM_USED` is also incremented before the cooldown check and without
   knowing whether `on_use` consumed; vanilla `ServerPlayerGameMode#useItem` awards it only when `stack.use`
   returns a consuming result. Fixing this properly needs `ItemBehaviour::normal_use*` to return `BlockActionResult`
   (or a bool) and `ItemRegistry::on_use*` to return it, then award on consume. Size: M (API change across item
   behaviours). Out of this row's scope but same vanilla rule.

## TODO triage (no gap)
- `:26` `// TODO: maybe log?` - reach failure is already logged via `BlockPlacingError` (gap 1 covers the wrong
  level; vanilla itself only debug-logs).
- `:152` `// TODO item cool down` - stale. Vanilla `useItemOn` has no item-cooldown gate; Pumpkin enforces the
  cooldown in `ItemRegistry::use_on_block` (`item/registry.rs:150`) and `prepare_hand_item_for_use`
  (`use_item.rs:93`), and item use-on-block results are Pass-gated by it. No action.
- `:171` `// TODO: Config` - vanilla `BlockItem#useOn` always shrinks the stack outside creative; Pumpkin does
  that in `ItemBehaviour::on_placed_as_block`. No config exists in vanilla. No action.

## Verified matching (no change)
- Sneak bypass gate `!(sneaking && (!main_empty || !off_empty))` matches vanilla `useItemOn`'s
  `isSecondaryUseActive() && (main || offhand non-empty)`.
- Order block state use -> item useOn -> empty-stack `PASS` -> block placement matches `ServerPlayerGameMode#useItemOn`.
- Block placement runs after the item layer returns `PASS` and reuses the packet's `inside_block`,
  `CanPlaceOn` (adventure), spawn protection, height limit and `updatePlacementContext`; matches `BlockPlaceContext`
  / `BlockItem#updatePlacementContext`.
- `ITEM_USED_ON_BLOCK` is triggered with the pre-use stack copy, as vanilla does, and only on consuming results.
- Break entity status is broadcast before the hand slot sync (particle texture), and the item-used-on-block
  advancement/stat uses the pre-use item id.
- `InteractionResult.FAIL` resets predicted block changes at the clicked and placed positions, which is the
  Paper/Pumpkin prediction compensation.

## Gap status
- Rows: STILL OPEN. Note body not recoverable in this pass; nothing verified.


## Result

Re-verified each STILL OPEN row against `.vendor/vanilla-decompiled` (2026-11-17), smallest first:

1. Reach log level: RESOLVED, no change in this unit. `net/java/play/mod.rs:104` already maps
   `BlockOutOfReach` to `Level::TRACE` (not WARN); the doc claim is stale. Vanilla
   `ServerGamePacketListenerImpl#handleUseItemOn:2316` silently ignores out-of-reach hits. Nothing to
   change in `use_item_on.rs`.
2. Air check: NO CHANGE. `handleUseItemOn` has no air check; Pumpkin's
   `world.get_block(&position) == &Block::AIR` early return (`use_item_on.rs:62`) is deliberate
   anti-cheat hardening (stricter than vanilla, not looser). Documented only.
3. Hit-position window: NO CHANGE. `handleUseItemOn:2317-2319` uses
   `abs(hit - blockCenter) < 1.0000001`, which a real client always satisfies; Pumpkin's
   `-0.5..1.5` window plus `1e-3` clip is intentionally stricter (commit `6a7175af0`). The pure
   `is_valid_hit_position` test already covers this seam.
4. Default block use hand gate: NO CHANGE, premise disproven. `ServerPlayerGameMode#useItemOn:625`
   resolves `TryEmptyHandInteraction` -> `useWithoutItem` only when `hand == MAIN_HAND`; Pumpkin's
   `equipment_slot == MAIN_HAND` check in `call_use_item_on` (`use_item_on.rs:252`) matches vanilla.
   Dropping it would break off-hand placement parity.
5. Advancement `state`: STILL OPEN. `AdvancementTrigger::ItemUsedOnBlock.state` is
   `Option<(String, String)>` (`entity/player/advancement/trigger.rs:62`); vanilla matches a generic
   `StatePropertiesPredicate` against the clicked block's own properties. Needs a map type plus
   generic serialization across `entity/player/advancement` (file outside this unit).
6. Enabled-feature gate: STILL OPEN. Vanilla `useItemOn:539` checks `block.isEnabled(features)` and
   `handleUseItemOn:2307` / `handleUseItem:2397` check `itemStack.isItemEnabled(features)`. Pumpkin's
   `Block`/`Item` data has no feature-flag field and no `is_item_enabled` exists; needs generated data
   plus datapack feature resolution (outside this unit).
7. `ITEM_USED` stat in `use_item.rs:40`: STILL OPEN. Vanilla awards `Stats.ITEM_USED` inside individual
   item implementations (e.g. `BowItem:54`, `CauldronInteractions`), never centrally, and only after
   the cooldown gate in `ServerPlayerGameMode#useItem:478`. `ItemRegistry::on_use_with_rotation`
   returns `()`, so a correct fix needs a consume-result API change across item behaviours (outside
   this unit).

No `.rs` file was changed, so no rustfmt run; no new pure test seam exists in either handler beyond
the existing `is_valid_hit_position` test.


## Result

Re-verified the three STILL OPEN rows (5, 6, 7) against `.vendor/vanilla-decompiled` (2026-11-18), smallest first. The prior pass's fixes for rows 5 and 7 are present in the current files; no further in-unit code change was needed.

5. Advancement `state`: RESOLVED (already fixed in current code). `use_item_on.rs:326-337` now serializes `block.property_values(state_id)` into a generic `Vec<(String, String)>` instead of hardcoding copper-bulb `lit`; `trigger.rs:82` holds `Option<Vec<(String, String)>>` and `trigger.rs:581` passes `state.as_deref().unwrap_or(&[])` to `CriterionConditions::ItemUsedOnBlock(pred).matches`, whose `ItemUsedOnBlockCheck.matches` (`advancement/predicate.rs:643-671`) looks up `state_key` by name in that map. Vanilla `StatePropertiesPredicate.matches` (`net/minecraft/advancements/predicates/StatePropertiesPredicate.java:34-46`) matches arbitrary named properties against the clicked block's own state, so the generic map restores parity. No pure test seam in this unit (serialization needs `World`/`Block` state; the matcher lives in `predicate.rs`, outside this unit).

6. Enabled-feature gate: STILL OPEN. Vanilla `ServerPlayerGameMode#useItemOn:539` returns FAIL when `!block.isEnabled(level.enabledFeatures())` and `ServerGamePacketListenerImpl#handleUseItemOn:2307` / `handleUseItem:2397` check `itemStack.isItemEnabled(features)`. Pumpkin's generated `Block`/`Item` data has no feature-flag field (`pumpkin-data/src/blocks.rs:21` has none) and no enabled-feature resolution exists (`grep isEnabled|enabledFeatures` finds only the `CFeatureFlags` protocol packet), so the gate needs generated data plus datapack feature resolution, both outside this unit.

7. `ITEM_USED` stat in `use_item.rs`: RESOLVED within this unit (already fixed in current code), residual needs out-of-scope work. `use_item.rs:39-42` removed the central award and documents that vanilla `handleUseItem` never awards it centrally; `use_item.rs:168-169` awards only for `Equippable.swapAndEquip`. Vanilla `ServerPlayerGameMode#useItem:473-524` confirms no central award and item implementations self-award (Pumpkin `bow.rs`/`trident.rs` already call `increment_stat(Used, ...)`). A consume-result API across item behaviours would be needed to gate any remaining generic award, which is outside this unit.

No `.rs` file was changed this pass (both fixable rows were already applied in the current files), so no rustfmt run; no new pure test seam exists in these three files.


## Result

Re-verified the sole remaining STILL OPEN row (6) against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java` (2026-11-18). Spot-checked that rows 5 and 7 are still resolved in the current files (no regression): `use_item_on.rs:326` serializes `property_values(state_id)`, `trigger.rs:82/581` carries and passes the generic `Vec<(String, String)>` state map, `predicate.rs:661` looks up `state_key` by name, and `use_item.rs:39-42` has no central `ITEM_USED` award.

6. Enabled-feature gate: STILL OPEN. Vanilla `ServerPlayerGameMode#useItemOn:539` returns `FAIL` when `!state.getBlock().isEnabled(level.enabledFeatures())`. Feature-flag *resolution* now exists in this crate (`data/datapack/mod.rs:1182` `get_enabled_features`, `:1227` `is_feature_enabled`, `:2293` `known_feature_flag`), but the generated `Block`/`Item` definitions (`pumpkin-data/src/blocks.rs:21`) carry no `requiredFeatures` field and there is no per-element `is_enabled` check, so the block/item-to-feature mapping is unavailable. Implementing the gate needs a codegen change to `crates/pumpkin-data/src/generated` (forbidden for workers) plus a cross-crate gate at use time; left open with that reason.

No `.rs` file changed this pass, so no rustfmt run; no new pure test seam exists in these files.



## Result

Re-verified all rows against decompiled vanilla (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java`, 2026-11-18), smallest first. Only row 6 remains STILL OPEN.

5. Advancement `state`: RESOLVED (no regression). `use_item_on.rs:325-333` serializes `block.property_values(state_id)` into a generic `Vec<(String, String)>`; `trigger.rs` carries it and `predicate.rs:661` matches `state_key` by name. Matches vanilla `StatePropertiesPredicate.matches`. No change.
7. `ITEM_USED` stat in `use_item.rs`: RESOLVED (no regression). `use_item.rs:39-42` has no central award; vanilla awards per-item only (bow/trident self-award). No change.
6. Enabled-feature gate: STILL OPEN. Vanilla `useItemOn:539` returns `FAIL` when `!state.getBlock().isEnabled(level.enabledFeatures())`, and `handleUseItemOn`/`handleUseItem` check `itemStack.isItemEnabled(features)`. Feature resolution exists (`Server::is_feature_enabled`), but `Block`/`Item` carry no `requiredFeatures` field in `pumpkin-data/src/blocks.rs` or `assets/`, so the block/item-to-feature mapping is unavailable. Fix needs a codegen change to the forbidden `pumpkin-data/src/generated` plus a cross-crate gate at use time, outside this unit; left open with that reason.

No `.rs` file changed this pass, so no rustfmt run; no new pure test seam exists in these files (serialization needs `World`/`Block` state; the feature gate needs generated per-element data).


## Result

Re-verified every row against decompiled vanilla (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java`, 2026-11-18), smallest first. No `.rs` file in this unit was changed.

5. Advancement `state`: RESOLVED (no regression). `use_item_on.rs:325-333` serializes `block.property_values(state_id)` into a generic `Vec<(String, String)>` instead of the old hardcoded copper-bulb `lit`; `trigger.rs:82` carries `Option<Vec<(String, String)>>`, `trigger.rs:581` passes `state.as_deref().unwrap_or(&[])`, and `predicate.rs:661-671` looks up `state_key` by name in that map. Matches vanilla `StatePropertiesPredicate.matches` (arbitrary named properties of the clicked state). No pure test seam here (serialization needs `World`/`Block` state; the matcher lives in `predicate.rs`).

7. `ITEM_USED` stat: RESOLVED (no regression). `use_item.rs:39-42` has no central award and documents that vanilla `handleUseItem` never awards it centrally; the only award in the file is `use_item.rs:169` for `Equippable.swapAndEquip`, matching vanilla. `bow.rs:135` and `trident.rs:237` self-award `StatisticCategory::Used` in their use paths, as vanilla item implementations do. A generic consume-result award would need an API change across item behaviours (outside this unit) and would double-count self-awarding items.

6. Enabled-feature gate: STILL OPEN (out of scope). Vanilla `ServerPlayerGameMode#useItemOn:539` returns `FAIL` when `!state.getBlock().isEnabled(level.enabledFeatures())`; `handleUseItemOn`/`handleUseItem` also check `itemStack.isItemEnabled(features)`. Feature resolution exists in this crate (`server.is_feature_enabled`, `Server::get_enabled_features`), but generated `Block`/`Item` data carries no `requiredFeatures` field (`grep` for `requiredFeatures|required_features` in `pumpkin-data/src/blocks.rs` finds none), so no block/item-to-feature mapping is available. Reason left open: the fix needs a codegen change to the forbidden `crates/pumpkin-data/src/generated` plus a cross-crate gate at use time, both outside this unit; hardcoding a few experimental blocks would violate vanilla parity and the no-guessing rule.

No rustfmt run (no `.rs` changed); no new pure test seam exists (the feature gate needs generated per-element data).


## Result

Third-pass final classification (2026-11-18) against decompiled vanilla. No STILL OPEN rows remain.

5. Advancement `state`: VERIFIED (no regression). `use_item_on.rs:322-333` serializes `block.property_values(state_id)` into a generic `Vec<(String, String)>` instead of the old hardcoded copper-bulb `lit`; `trigger.rs:82` carries `Option<Vec<(String, String)>>`, `trigger.rs:581` passes `state.as_deref().unwrap_or(&[])`, and `predicate.rs:661-671` looks up each `state_key` by name in that map. Matches vanilla `StatePropertiesPredicate.matches` (`.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/StatePropertiesPredicate.java:34-46`), which matches arbitrary named properties of the clicked state. No pure test seam here (serialization needs `World`/`Block` state; matcher lives in `predicate.rs`). No change.

7. `ITEM_USED` stat: VERIFIED (no regression). `use_item.rs:39-42` has no central award, with a comment recording that vanilla `ServerGamePacketListenerImpl#handleUseItem` never awards it centrally; item implementations self-award. `bow.rs:135` and `trident.rs:237` self-award `StatisticCategory::Used` in their use paths, as vanilla item implementations do. Matches vanilla. No change.

6. Enabled-feature gate: BLOCKED-DATA. Vanilla `ServerPlayerGameMode#useItemOn` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java:539`) returns `InteractionResult.FAIL` when `!state.getBlock().isEnabled(level.enabledFeatures())`; item use is gated by `ItemStack#isItemEnabled(FeatureFlagSet)` (`.vendor/vanilla-decompiled/net/minecraft/world/item/ItemStack.java:338-339`). Both derive from the `requiredFeatures: FeatureFlagSet` field (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:99`, exposed at `:253-254`, populated by `Properties.requiredFeatures(...)` at `:1514-1515`). Pumpkin's generated `Block` definition (`crates/pumpkin-data/src/blocks.rs:21-53`) HAS NO `requiredFeatures` field; the `assets/` source data carries no per-block/per-item feature flags, and no `is_enabled`/`is_item_enabled` helper exists. Exact missing generated data: a `required_features` (`FeatureFlagSet`) field on the generated `Block` and `Item` definitions (codegen into forbidden `crates/pumpkin-data/src/generated` plus Extractor output in `assets/`), followed by a cross-crate gate at use time. Cannot be fixed without that generated data; hardcoding a few experimental blocks would violate vanilla parity and the no-guessing rule.

No `.rs` file changed this pass, so no rustfmt run; no new pure test seam exists (row 6 needs generated per-element feature data).


## Result

Final-pass classification (2026-11-18). Every row is terminal; no STILL OPEN rows remain. Verified directly against decompiled vanilla and the current source; no .rs file needed changing.

1. Reach log level: VERIFIED. net/java/play/mod.rs maps BlockOutOfReach to Level::TRACE; vanilla ServerGamePacketListenerImpl#handleUseItemOn silently ignores out-of-reach hits. No change.

2. Air check: VERIFIED. use_item_on.rs:66 early-returns on Block::AIR, deliberate anti-cheat hardening (stricter than vanilla), documented in place. No change.

3. Hit-position window: VERIFIED. is_valid_hit_position clips the relative hit into -1e-3..1+1e-3; vanilla handleUseItemOn:2317-2319 checks abs(hit - center) < 1.0000001, which a real client always satisfies. Stricter than vanilla by design (commit 6a7175af0); a pure test already covers the seam. No change.

4. Default block use hand gate: VERIFIED, premise disproven. use_item_on.rs resolves the empty-hand interaction only when equipment_slot == MAIN_HAND, matching ServerPlayerGameMode#useItemOn:625 which calls useWithoutItem only for MAIN_HAND. No change.

5. Advancement state: VERIFIED (no regression). use_item_on.rs:322-333 serializes block.property_values(state_id) into a generic Vec<(String, String)> instead of hardcoded copper-bulb lit; trigger.rs:85 carries Option<Vec<(String, String)>> and trigger.rs:594-597 passes state.as_deref().unwrap_or(&[]); predicate.rs:661-671 matches each state_key by name in that map. Matches vanilla StatePropertiesPredicate.matches (.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/StatePropertiesPredicate.java:34-46), which matches arbitrary named properties of the clicked state. No change.

6. Enabled-feature gate: BLOCKED-DATA. Vanilla ServerPlayerGameMode#useItemOn (.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java:539) returns FAIL when !state.getBlock().isEnabled(level.enabledFeatures()); item use is gated by ItemStack#isItemEnabled(FeatureFlagSet). Both derive from the requiredFeatures: FeatureFlagSet field (.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:99, exposed :253-254, populated by Properties.requiredFeatures(...) at :1514-1515). Feature resolution exists in Pumpkin (Server::is_feature_enabled / Server::get_enabled_features, used for minecraft:minecart_improvements in net/java/play/client_command.rs:170), but the generated Block/Item definitions carry NO requiredFeatures field (grep required_features|requiredFeatures in crates/pumpkin-data/src finds nothing) and assets/ has no per-element feature flags. Exact missing generated data: a required_features (FeatureFlagSet) field on the generated Block and Item definitions, plus Extractor output in assets/ and a cross-crate gate at use time. Codegen into forbidden crates/pumpkin-data/src/generated is required; hardcoding a few experimental blocks would violate vanilla parity and the no-guessing rule. Not fixable from this unit.

7. ITEM_USED stat: VERIFIED (no regression). use_item.rs:39-42 has no central award, with a comment recording that vanilla handleUseItem never awards it centrally; items self-award (bow.rs:135, trident.rs:237 call increment_stat(StatisticCategory::Used, ...), matching vanilla item implementations). A generic consume-result award would need an API change across item behaviours and would double-count self-awarding items. No change.

No rustfmt run (no .rs changed); no new pure test seam exists (row 6 needs generated per-element feature data).
## Result (coder pass 2026-11-18, batch r2_013)
- No STILL OPEN rows in the latest pass result above. Every row is FIXED, VERIFIED or BLOCKED-DATA per that result.
- This pass did not re-verify each row against vanilla (time budget); status carried from the latest prior pass, unchanged.
- Files edited this pass: none. No rustfmt run. Cargo and git not run (forbidden).
