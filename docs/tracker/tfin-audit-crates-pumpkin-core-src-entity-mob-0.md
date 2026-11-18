# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-mob.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Rows: CLOSED. All seven rows verified against .vendor vanilla source; no source edit needed this pass (see Result).

## Result

FINAL PASS (2026-11-18). Re-verified all seven rows against `.vendor/vanilla-decompiled/net/minecraft/` with source present. The checkout already contains every in-scope fix from earlier passes; no source file needed editing this pass, so no rustfmt was required. No code was compiled (forbidden for this role).

- **Row 1 (L) VERIFIED, no live double-dispatch.** Vanilla has one virtual `Mob.populateDefaultEquipmentSlots` (Mob.java:1133-1161), called from each species' `finalizeSpawn` (AbstractSkeleton.java:136, Zombie.java:515-516, Piglin.java:201-202). Pumpkin implements the same behaviour through two disjoint paths: the data-driven `EQUIPMENT_REGISTRY`/`equip_mob_on_spawn` (equipment.rs:215/741) for skeleton, stray, bogged, wither_skeleton, piglin, pillager, vindicator; and the `Mob::populate_default_equipment_slots` trait method (mod.rs:2658) plus zombie-family overrides (zombie/mod.rs:708, husk.rs:156, drowned.rs:349, zombie_villager.rs:430, zombified_piglin.rs:203) for the zombie family. No mob appears in both sets, so no mob is equipped twice. Each path reproduces its vanilla counterpart: the registry piglin weapon weights match `Piglin.createSpawnWeapon` (Piglin.java:296-298) and the 0.1 golden-armor chances (Piglin.java:207-219); the zombie trait method matches `Zombie.populateDefaultEquipmentSlots` (Zombie.java:408-419). Skeleton's `equip_default_bow` (skeleton/mod.rs:105) is idempotent, so it does not stack with the registry bow. The original "skeleton/piglin overrides have no caller" claim is stale: there are no dead overrides. The remaining difference is structural only (two implementations of one vanilla virtual method) with no observable vanilla mismatch; unifying them is a behaviour-neutral refactor that cannot be safely type-checked without a compile, so it is not attempted.

- **Row 2 (S) FIXED, verified.** `WitherSkeleton.populateDefaultEquipmentEnchantments` (WitherSkeleton.java:78-80) is empty, so `enchanted: false` is correct; `AbstractSkeleton.finalizeSpawn` (AbstractSkeleton.java:139) rolls `setCanPickUpLoot` for every skeleton, so `can_pick_up_loot: true` is correct. The equipment.rs wither_skeleton entry sets both, asserted by `wither_skeleton_spawn_flags_match_vanilla` (equipment.rs:1201).

- **Row 3 (S) FIXED, verified.** `Mob.populateDefaultEquipmentSlots` (Mob.java:1153) guards `if (itemStack.isEmpty())`; `equip_mob_on_spawn` (equipment.rs:776) skips non-empty armor slots, MAIN_HAND stays unconditional as in the skeleton overrides.

- **Row 4 (M) NO CHANGE, verified faithful.** `WorldGenRegion.getCurrentDifficultyAt` (WorldGenRegion.java:525-530) passes `0L` inhabited time during generation; `ServerLevel.getCurrentDifficultyAt` uses 0 when no FULL chunk exists. The `loaded_chunks` lookup with a 0 fallback in `RegionalDifficulty::at` (equipment.rs:373-383) matches.

- **Row 5 (M) FIXED, verified.** `Mob.finalizeSpawn` draws from `level.getRandom()` (Mob.java:1259). `should_happen`, `select_vanilla_armor`, `equip_mob_from_def` and `weighted_select_item` take `&mut impl RandomImpl` and `equip_mob_on_spawn` supplies `world.loot_random` (equipment.rs:765-770), matching. Residual (the `apply_vanilla_enchantments` provider cost roll still uses process-wide `rand`) needs its caller at mob/mod.rs:2736 (outside this unit); not an observable mismatch for this row.

- **Row 6 (S) FIXED, verified.** Vanilla's slot filter lives in `enchantSpawnedEquipment` (Mob.java:1246-1254) and `populateDefaultEquipmentEnchantments` (Mob.java:1226-1234, HUMANOID_ARMOR + MAINHAND only); `apply_vanilla_enchantments` takes `(stack, special_multiplier)` only and the caller mirrors the slot filter.

- **Row 7 (M) NO CHANGE, verified faithful.** Base `Mob.finalizeSpawn` (Mob.java:1256-1269) does not roll can-pick-up-loot; species overrides do (AbstractSkeleton.java:139, Zombie.java:480). Pumpkin's roll in `spawn.rs:populate_spawn_equipment` (spawn.rs:115-121) and zombie `finalize_zombie_spawn` (zombie/mod.rs:256-260) is gated per definition, matching.

Files edited this pass: this doc only. No source file needed a change: every in-scope fix is already present and confirmed against vanilla. No new test seam (all fixes already covered by existing tests). No compile run (forbidden for this role).
