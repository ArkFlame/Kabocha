# Parity rows for crates/pumpkin-core/src/net/java/play/use_item_on.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-ne-1655e2** [partial/OWN-OPEN] own audit: 12 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-ne-1655e2.md) docs/tracker/taudit-crates-pumpkin-core-src-ne-1655e2.md
- **own/taudit-crates-pumpkin-core-src-net-java-p** [partial/OWN-OPEN] own audit: 1 open, 5 fixed, 7 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-net-java-p.md) docs/tracker/taudit-crates-pumpkin-core-src-net-java-p.md

## Result
Final classification is in the Result section below.

## Result (third pass, 2026-11-18, decompiled vanilla present)

Both rows in this unit reduce to one parity gap: the feature-flag use gate. Everything else was already fixed and verified in earlier passes against `.vendor/vanilla-decompiled`, and the fixes are present in `use_item_on.rs`.

- **own/taudit-crates-pumpkin-core-src-ne-1655e2**: all 12 sub-rows now classified, 0 STILL OPEN. Row 5 (advancement `state`) is VERIFIED: `use_item_on.rs:325-333` serializes every clicked block-state property into a generic `Vec<(String, String)>` and `predicate.rs` matches each named key, as vanilla `StatePropertiesPredicate.matches` does (`.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/StatePropertiesPredicate.java:34-46`). Row 7 (`ITEM_USED` stat) is VERIFIED: `use_item_on.rs:192-194` awards only when the use consumes the action, matching the item self-award paths. The one hard sub-row (enabled-feature gate) is BLOCKED-DATA below. No code change needed.
- **own/taudit-crates-pumpkin-core-src-net-java-p**: 5 fixed, 7 verified; sole remaining open row `isItemEnabled(level.enabledFeatures())` is BLOCKED-DATA below. No code change needed.
- **Enabled-feature gate / `isItemEnabled(level.enabledFeatures())`: BLOCKED-DATA.** Vanilla rejects the interaction before any action when the block or held item requires a flag absent from the level: `ServerPlayerGameMode.java:539-540` returns `InteractionResult.FAIL` when `!state.getBlock().isEnabled(level.enabledFeatures())`, and `ServerGamePacketListenerImpl.java:2307` and `:2397` gate on `itemStack.isItemEnabled(level.enabledFeatures())` (`ItemStack.java:338`, driven by `requiredFeatures: FeatureFlagSet`). Pumpkin's generated `Item` (`crates/pumpkin-data/src/generated/item.rs:22-27`) has only `id`/`registry_key`/`crafting_remainder`/`components`, and `Block` (`crates/pumpkin-data/src/blocks.rs:21-53`) has no feature field; a grep for `required_features|requiredFeatures` across `crates/pumpkin-data/src` returns nothing. Exact missing generated data: a `required_features` (`FeatureFlagSet`) field on the generated `Block` and `Item` definitions, emitted by `pumpkin-codegen` from per-block/per-item feature flags in `assets/` (both outside this unit; `crates/pumpkin-data/src/generated` is worker-forbidden), plus a cross-crate gate at use time. No conservative subset check is possible without that mapping; hardcoding experimental blocks would break parity and violate the no-guessing rule.

No source files were edited this pass: all in-unit fixes were already present and verified, and the only remaining row has no in-unit fix. Nothing to rustfmt. No new pure test seam exists for the blocked row (it needs generated feature data plus a live `World`/`Player`); the two existing helpers in `use_item_on.rs` are already covered by their unit tests.


## Result (final pass, 2026-11-18)

All rows in this unit are terminal. No `.rs` file needed editing this pass: the in-unit
fixes are already present and verified in `use_item_on.rs` and `predicate.rs`, and the
sole hard row is genuinely blocked on generated data.

- **own/taudit-crates-pumpkin-core-src-ne-1655e2**: VERIFIED / FIXED. All 12 sub-rows
  closed. Build-height + `mayInteract` + teleport guards and the hit-center epsilon check
  are in `use_item_on.rs:30-68` and locked by `block_center_hit_limit_matches_vanilla` /
  `hit_position_must_be_inside_clicked_block`. Advancement `state` is VERIFIED:
  `use_item_on.rs:325-333` serializes every clicked block-state property and
  `predicate.rs:661-668` matches each named key, matching vanilla
  `StatePropertiesPredicate.matches`
  (`.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/StatePropertiesPredicate.java:34-46`).
  The `ITEM_USED` stat is VERIFIED gated on a consuming result at `use_item_on.rs:192-194`.
  The one hard sub-row (enabled-feature gate) is BLOCKED-DATA below.
- **own/taudit-crates-pumpkin-core-src-net-java-p**: VERIFIED / FIXED (5 fixed, 7 verified);
  its sole remaining row `isItemEnabled(level.enabledFeatures())` is BLOCKED-DATA below.
- **Enabled-feature gate / `isItemEnabled(level.enabledFeatures())`**: BLOCKED-DATA.
  Vanilla rejects the interaction before any action when the block or held item requires a
  flag absent from the level: `ServerPlayerGameMode.java:539` returns `InteractionResult.FAIL`
  when `!state.getBlock().isEnabled(level.enabledFeatures())`
  (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java:539`), and
  item use is gated by `ItemStack.isItemEnabled(FeatureFlagSet)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/ItemStack.java:338`). Both derive from
  the `requiredFeatures: FeatureFlagSet` field
  (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/state/BlockBehaviour.java:99`,
  accessor `:253-254`, populated by `Properties.requiredFeatures(...)` at `:1514-1515`).
  Exact missing generated data: a `required_features` (`FeatureFlagSet`) field on the generated
  `Block` (`crates/pumpkin-data/src/blocks.rs:21-53`) and `Item`
  (`crates/pumpkin-data/src/generated/item.rs`) definitions, emitted by `pumpkin-codegen` from
  per-element feature flags in `assets/` (both outside this unit; `crates/pumpkin-data/src/generated`
  is worker-forbidden), plus a cross-crate gate at use time. `grep required_features|requiredFeatures`
  across `crates/pumpkin-data/src` returns nothing. No conservative subset check exists without
  that mapping; hardcoding a few experimental blocks would break vanilla parity and violate the
  no-guessing rule.

No source files were edited this final pass; nothing to rustfmt. No new pure test seam exists for
the blocked row (it needs generated feature data plus a live `World`/`Player`); the existing pure
helpers in `use_item_on.rs` are already covered by unit tests.
