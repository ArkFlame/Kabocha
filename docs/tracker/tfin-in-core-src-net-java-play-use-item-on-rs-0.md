# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-net-java-play-use-item-on-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: predicate.rs use_item_on.rs

- **own/taudit-crates-pumpkin-core-src-ne-1655e2**: all 12 sub-rows now classified, 0 STILL OPEN. Row 5 (advancement `state`) is VERIFIED: `use_item_on.rs:325-333` serializes every clicked block-state property into a generic `Vec<(String, String)>` and `predicate.rs` matches each named key, as vanilla `StatePropertiesPredicate.matches` does (`.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/StatePropertiesPredicate.java:34-46`). Row 7 (`ITEM_USED` stat) is VERIFIED: `use_item_on.rs:192-194` awards only when the use consumes the action, matching the item self-award paths. The one hard sub-row (enabled-feature gate) is BLOCKED-DATA below. No code change needed.

## Result (final pass, 2026-11-18)

Re-verified every row against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/` and the current source. All in-unit source fixes are
already present in `use_item_on.rs` and `predicate.rs`; no `.rs` file needed changing this pass.

- **own/taudit-crates-pumpkin-core-src-ne-1655e2**: terminal, all rows closed, 0 STILL OPEN.
  1. Reach log level: VERIFIED. `BlockOutOfReach` maps to `Level::TRACE`; vanilla
     `ServerGamePacketListenerImpl#handleUseItemOn` silently drops out-of-reach hits.
  2. Air check: VERIFIED. `use_item_on.rs:66` early-returns on `Block::AIR`, deliberate
     anti-cheat hardening documented in place.
  3. Hit-position window: VERIFIED. `is_within_block_center_limit` (`use_item_on.rs:381-386`)
     and `is_valid_hit_position` (`:390-395`) enforce the vanilla clip, locked by the pure tests
     `block_center_hit_limit_matches_vanilla` and `hit_position_must_be_inside_clicked_block`
     (`use_item_on.rs:402-429`).
  4. Default block use hand gate: VERIFIED, premise disproven. `call_use_item_on` runs the
     default `on_use` only for `EquipmentSlot::MAIN_HAND` (`use_item_on.rs:249-251`), matching
     vanilla `ServerPlayerGameMode#useItemOn` which calls `useWithoutItem` only for MAIN_HAND.
  5. Advancement `state`: VERIFIED. `use_item_on.rs:325-333` serializes every clicked block-state
     property into a generic `Vec<(String, String)>`;
     `crates/pumpkin-core/src/entity/player/advancement/predicate.rs:661-671` matches each named
     `state_key`/`state_value` by lookup, as vanilla `StatePropertiesPredicate.matches` does
     (`.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/StatePropertiesPredicate.java:34-46`).
  6. Enabled-feature gate: BLOCKED-DATA (exact missing field below).
  7. `ITEM_USED` stat: VERIFIED. `use_item_on.rs:192-194` awards `StatisticCategory::Used` only
     when the item's use consumes the action or a block was placed, matching vanilla's
     item self-award paths.

- **Enabled-feature gate / `isItemEnabled(level.enabledFeatures())`: BLOCKED-DATA.** Vanilla
  `ServerPlayerGameMode#useItemOn`
  (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java:539`) returns
  `InteractionResult.FAIL` when `!state.getBlock().isEnabled(level.enabledFeatures())`; item use
  is gated by `ItemStack#isItemEnabled(FeatureFlagSet)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/ItemStack.java:338-339`), both derived from
  the `requiredFeatures: FeatureFlagSet` field
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:99`,
  accessor `:253-254`). Pumpkin's generated `Item`
  (`crates/pumpkin-data/src/generated/item.rs:22-27`) and `Block`
  (`crates/pumpkin-data/src/blocks.rs`) definitions carry no feature-flag field, and
  `grep required_features|requiredFeatures|FeatureFlag` across `crates/pumpkin-data/src` returns
  nothing. Exact missing generated data: a `required_features` (`FeatureFlagSet`) field on the
  generated `Block` and `Item` definitions (codegen into worker-forbidden
  `crates/pumpkin-data/src/generated`, plus per-element flags in `assets/`), followed by a
  cross-crate gate at use time. Hardcoding a few experimental blocks would break vanilla parity
  and violate the no-guessing rule, so this cannot be fixed from this unit.

No new pure test seam exists: the blocked row needs generated feature data plus a live
`World`/`Player`; the two pure helpers in `use_item_on.rs` already have unit tests. Nothing to
rustfmt (no `.rs` file edited this pass).
