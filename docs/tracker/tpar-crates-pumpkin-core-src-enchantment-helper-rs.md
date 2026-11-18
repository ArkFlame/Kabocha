# Parity rows for crates/pumpkin-core/src/enchantment/helper.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-68d878** [partial/OWN-OPEN] own audit: 1 open, 6 fixed, 5 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-68d878.md) docs/tracker/taudit-crates-pumpkin-core-src-en-68d878.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result

THIRD PASS 2026-11-18. Closed the only row `own/taudit-crates-pumpkin-core-src-en-68d878` as VERIFIED; every gap in the referenced audit is already fixed in helper.rs and matches decompiled vanilla. No source change was needed this pass.

Re-verified each gap directly against .vendor/vanilla-decompiled/net/minecraft/:

- Gap 1 (on_post_attack requirements): VERIFIED FIXED. helper.rs:115-178 hardcodes the vanilla predicates: Bane of Arthropods requires victim tag SENSITIVE_TO_BANE_OF_ARTHROPODS (Enchantments.java:656-672), Wind Burst requires DIRECT_ATTACKER isFlying=false and fallDistance>=1.5 (Enchantments.java:1242-1266; EntityFlagsPredicate.java:60-65), Channeling is handled in on_trident_post_attack (Enchantments.java:1091-1107).
- Gap 2 (on_location_changed Frost Walker gate): VERIFIED FIXED. helper.rs:505-529 gates via meets_frost_walker_requirements(on_ground, has_vehicle), matching Enchantments.java:419-428 (setOnGround(true) AND inverted vehicle). entity.on_ground is populated from the client flag at entity/player/client_move.rs:339-340.
- Gap 3 (generic on_hit_block): VERIFIED FIXED. No callerless generic fn remains; only on_trident_hit_block, gating Channeling on thundering + LIGHTNING_RODS + canSeeSky per Enchantments.java:1109-1126.
- Gap 4 (is_world_thundering): VERIFIED FIXED. helper.rs:259-263 checks is_raining && is_thundering; World::is_thundering() folds can_have_weather(), matching Level.isThundering() (Level.java:2451-2452).
- Gap 5 (modify_durability_change): VERIFIED FIXED. The dead, unwired function is gone; no dangling callers.

Pure unit tests for the three requirement helpers already exist in the helper.rs tests module (frost_walker_needs_ground_and_no_vehicle, bane_of_arthropods_needs_arthropod_victim, wind_burst_needs_fall_while_not_flying). No new test seam exists without generated-data or cross-crate changes. No .rs file was touched, so no rustfmt was needed.
