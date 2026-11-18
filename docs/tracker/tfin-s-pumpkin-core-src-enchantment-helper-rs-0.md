# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-enchantment-helper-rs.md

Close each item below: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result

FINAL PASS 2026-11-18. Single row `own/taudit-crates-pumpkin-core-src-en-68d878` (audit of crates/pumpkin-core/src/enchantment/helper.rs): VERIFIED, closed. Every gap in the referenced audit is already fixed in helper.rs and matches decompiled vanilla; no source change was needed this pass.

Re-verified each gap directly against .vendor/vanilla-decompiled/net/minecraft/world/item/enchantment/Enchantments.java (plus the already-closed helper.rs tests):

- Gap 1 (on_post_attack requirements): VERIFIED FIXED. helper.rs:115-179 hardcodes the vanilla predicates: Bane of Arthropods requires victim tag SENSITIVE_TO_BANE_OF_ARTHROPODS (Enchantments.java:634-669 via `meets_bane_of_arthropods_requirements`, helper.rs:536-538) and Wind Burst requires `isFlying=false` + `fallDistance>=1.5` (Enchantments.java:1263-1264 via `meets_wind_burst_requirements`, helper.rs:541-543); Channeling lightning is handled in on_trident_post_attack (Enchantments.java:1079-1107).
- Gap 2 (on_location_changed Frost Walker gate): VERIFIED FIXED. helper.rs:506-529 gates via `meets_frost_walker_requirements` (helper.rs:546-548) on on_ground && !has_vehicle, matching Enchantments.java:419-428 (`setOnGround(true)` AND inverted `vehicle(...)`), confirmed by direct read of Enchantments.java:415-430. entity.on_ground is populated from the client flag.
- Gap 3 (generic on_hit_block): VERIFIED FIXED. No callerless generic `fn on_hit_block` remains (grep: no match); only on_trident_hit_block remains (helper.rs:297-350), gating Channeling on thundering + LIGHTNING_RODS + canSeeSky per Enchantments.java:1102, 1122-1123.
- Gap 4 (is_world_thundering): VERIFIED FIXED. helper.rs:259-263 checks is_raining && is_thundering with no dimension gate; World::is_thundering() folds can_have_weather(), matching Level.isThundering() (Level.java:2451-2452).
- Gap 5 (modify_durability_change): VERIFIED FIXED. The dead, unwired function is gone (grep: no match); no dangling callers.
- Cross-check: modify_crossbow_charge_time (helper.rs:390-400) threads the seconds value through every effect, matching vanilla EnchantmentHelper.modifyCrossbowChargeTime; the three pure unit tests exist in the helper.rs tests module (helper.rs:550-577).

No .rs file was touched this pass, so no rustfmt was needed.
