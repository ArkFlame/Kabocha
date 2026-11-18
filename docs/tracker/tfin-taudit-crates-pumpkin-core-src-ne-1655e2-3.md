# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-ne-1655e2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Third-pass final classification (2026-11-18) against decompiled vanilla. No STILL OPEN rows remain.
Final-pass classification (2026-11-18). Every row is terminal; no STILL OPEN rows remain. Verified directly against decompiled vanilla and the current source; no .rs file needed changing.

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


## Result

Final pass (2026-11-18). No STILL OPEN rows remain; every row is terminal. Re-verified against decompiled vanilla with paths:

1. Reach log level: VERIFIED. .vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:2298-2324 (handleUseItemOn) has no out-of-reach failure branch; net/java/play/mod.rs:102 maps BlockOutOfReach to Level::TRACE. No change.

2. Air check: VERIFIED. The same handleUseItemOn has no air early return; use_item_on.rs:66 is documented anti-cheat hardening (stricter than vanilla). No change.

3. Hit-position window: VERIFIED. handleUseItemOn:2317-2319 requires abs(distance) < 1.0000001; use_item_on.rs:390-395 clips to -1e-3..=1+1e-3 and use_item_on.rs:417-429 covers the seam. No change.

4. Default block use hand gate: VERIFIED. .vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java:626 calls useWithoutItem only for MAIN_HAND; use_item_on.rs:250 gates on EquipmentSlot::MAIN_HAND. No change.

5. Advancement state: VERIFIED. .vendor/vanilla-decompiled/net/minecraft/advancements/predicates/StatePropertiesPredicate.java:34-46 matches arbitrary named properties; use_item_on.rs:326, entity/player/advancement/trigger.rs:86 and :601, entity/player/advancement/predicate.rs:661 implement the generic map. No change.

6. Enabled-feature gate: BLOCKED-DATA. .vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayerGameMode.java:539 (!state.getBlock().isEnabled(level.enabledFeatures())) and .vendor/vanilla-decompiled/net/minecraft/world/item/ItemStack.java (isItemEnabled) require a required_features (FeatureFlagSet) field on the generated Block and Item definitions. Exact missing generated data: required_features on the generated block/item entries plus Extractor output in assets/, then a use-time gate. Codegen into crates/pumpkin-data/src/generated is forbidden here; hardcoding experimental blocks would violate parity. Not fixable from this unit.

7. ITEM_USED stat: VERIFIED. .vendor/vanilla-decompiled/net/minecraft/world/item/BowItem.java:54 and .vendor/vanilla-decompiled/net/minecraft/world/item/TridentItem.java:81 self-award; bow.rs:135 and trident.rs:237 match, and use_item.rs:39-42 has no central award. No change.

No .rs file changed; no rustfmt needed; no new pure test seam (row 6 needs generated per-element feature data).
