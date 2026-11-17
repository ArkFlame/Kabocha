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
