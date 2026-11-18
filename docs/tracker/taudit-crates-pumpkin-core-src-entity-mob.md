# taudit: crates/pumpkin-core/src/entity/mob/equipment.rs

Audited 2026-11-16 against vanilla `Mob.populateDefaultEquipmentSlots`,
`Mob.populateDefaultEquipmentEnchantments`, `Mob.equipItemIfPossible`,
`DifficultyInstance.calculateDifficulty` and `AbstractSkeleton`/`Piglin`
overrides. No decompiled source is present in this worktree (`.vendor/` absent),
so items marked "needs vanilla source" were not edited.

The tracker note "33 TODO/FIXME markers" is a false positive: all 33 grep hits
are the word "vanilla" in doc comments; the file contains no TODO/FIXME.

Already faithful (no change): armor tier table and upgrade rolls, partial-armor
break chance (0.1 Hard / 0.25 else), drop chance 0.085, `DifficultyInstance`
formula including `clamp(moonBrightness * 0.25, 0, globalScale)`, moon phase
brightness table, weighted weapon tables, `is_equippable_in_slot`,
`can_replace_equal_item` (enchant count, damage, custom name), binding-curse
check in `compare_armor`, raid buff providers and wave thresholds, old-item drop
`random - 0.1 < dropChance`.

Fixed in this pass:
- equipment.rs:378 `RegionalDifficulty::at` read the day time with
  `level_time.try_lock()` and fell back to `0` when the mutex was busy, which
  drops `specialMultiplier` to 0 and silently suppresses all spawn equipment.
  Vanilla reads `getDayTime()` unconditionally. Now uses `World::get_time_of_day`.

Remaining gaps:
1. equipment.rs:217 `EQUIPMENT_REGISTRY` + equipment.rs:591
   `equip_mob_from_def` duplicate the vanilla dispatch in a second system:
   `Mob::populate_default_equipment_slots` (mod.rs:2570) and the per-species
   overrides (skeleton/mod.rs:108, piglin.rs:914, zombie/mod.rs:685) implement
   the same rules again, and the skeleton/piglin overrides have no caller at
   all (grep shows only the zombie family calls the trait method). Vanilla has
   one virtual `populateDefaultEquipmentSlots` dispatch. Suggested fix: keep
   the trait methods, delete registry weapon/armor generation (registry only
   for `can_pick_up_loot`), or delete the dead overrides. Size L (multi-file,
   behaviour-visible).
2. equipment.rs:296 wither_skeleton `enchanted: false` and equipment.rs:303
   `can_pick_up_loot: false`. Vanilla `AbstractSkeleton.finalizeSpawn` calls
   `populateDefaultEquipmentEnchantments` for every skeleton subclass after
   `populateDefaultEquipmentSlots`, so a wither skeleton's stone sword can be
   enchanted; the pick-up flag is likewise not cleared by `WitherSkeleton`.
   Suggested fix: set `enchanted: true` (and verify the pick-up flag) after
   checking decompiled `AbstractSkeleton`/`WitherSkeleton`. Size S, needs
   vanilla source.
3. equipment.rs:564 `select_vanilla_armor` and :598 weapon placement never
   check `get(slot).is_empty()`, while vanilla `populateDefaultEquipmentSlots`
   only fills empty slots (`getItemBySlot(slot).isEmpty()`). Harmless today
   because `finalize_spawn` runs on fresh mobs, but a converted/re-finalized
   mob would have its existing armor overwritten. Suggested fix: skip occupied
   slots. Size S.
4. equipment.rs:375 `RegionalDifficulty::at` returns inhabited time 0 when the
   chunk is absent from `world.level.loaded_chunks`; vanilla
   `Level.getCurrentDifficultyAt` uses `hasChunkAt ? chunk.getInhabitedTime() : 0`,
   so mobs spawned while the chunk is still generating should still see the
   inhabited time. Suggested fix: read inhabited time from the chunk provider,
   not only the loaded map. Size M.
5. equipment.rs:456 `should_happen` draws from the process-wide `rand::rng()`;
   vanilla uses the level's `RandomSource`. Spawn equipment is not reproducible
   for gametests/replays. Suggested fix: thread `RandomSource` through
   `equip_mob_from_def`. Size M.
6. equipment.rs:475 `apply_vanilla_enchantments` ignores the slot argument;
   vanilla `populateDefaultEquipmentEnchantments` only enchants HUMANOID_ARMOR
   slots plus the main hand. Callers already respect it, so this is an API trap
   only. Suggested fix: drop the parameter or assert the slot. Size S.
7. equipment.rs:711 `equip_mob_on_spawn` applies no `0.55 * specialMultiplier`
   can-pick-up-loot roll itself; it is rolled in the caller (spawn.rs:99) and in
   mod.rs:2542, both gated on `EQUIPMENT_REGISTRY` membership, whereas vanilla
   `Mob.finalizeSpawn` rolls it for every mob. Mobs in neither list (and the
   zombie family, which rolls elsewhere) are inconsistent. Suggested fix: one
   shared roll in `finalize_spawn_base`. Size M.

## Gap status
- Rows: STILL OPEN. Note body not recoverable in this pass; nothing verified.


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/` (Mob.java,
world/entity/monster/skeleton/{AbstractSkeleton,WitherSkeleton}.java,
server/level/{ServerLevel,WorldGenRegion}.java). Smallest rows first.

- **Row 2 (S) FIXED.** `AbstractSkeleton.finalizeSpawn` (lines 131-143) rolls
  `setCanPickUpLoot` for every skeleton, but `WitherSkeleton.populateDefaultEquipmentEnchantments`
  (lines 78-80) is an empty override. So `enchanted: false` was already correct;
  `can_pick_up_loot: false` was wrong. Set `can_pick_up_loot: true` for the
  wither_skeleton registry entry and added `wither_skeleton_spawn_flags_match_vanilla`.
- **Row 3 (S) FIXED.** `Mob.populateDefaultEquipmentSlots` (lines 1146-1158) guards
  armor with `if (itemStack.isEmpty())`. `equip_mob_on_spawn` now skips any armor
  slot whose current stack is non-empty (weapon slots still match the skeleton
  overrides, which set MAINHAND unconditionally).
- **Row 6 (S) OPEN.** Dropping the unused `_slot` parameter would break the caller
  at `mod.rs:2727`, a file outside this unit; change reverted.
- **Row 4 (M) NO CHANGE - verified faithful.** `ServerLevel.getCurrentDifficultyAt`
  (lines 2703-2712) reads `getChunk(..., ChunkStatus.FULL, false)` and uses 0
  inhabited time when no such chunk exists; `WorldGenRegion.getCurrentDifficultyAt`
  (lines 525-531) explicitly passes `0L` during generation. The current
  `loaded_chunks` lookup with a 0 fallback matches vanilla.
- **Row 7 (M) NO CHANGE - verified faithful.** Base `Mob.finalizeSpawn`
  (lines 1256-1269) does not roll can-pick-up-loot; each species does so in its
  own override (`AbstractSkeleton.java:139`). Pumpkin's per-definition roll is
  already species-scoped.
- **Row 1 (L) OPEN.** Duplicate dispatch / dead overrides require a multi-file
  refactor across `skeleton/mod.rs`, `zombie/mod.rs` and `mod.rs:2570`, all outside
  this unit.
- **Row 5 (M) OPEN.** Threading a level `RandomSource` through every random draw
  spans `equip_mob_from_def`, `select_vanilla_armor`, `should_happen`,
  `equip_mob_on_spawn` and `spawn.rs`; too large to type-check without a compile.

Files edited: `crates/pumpkin-core/src/entity/mob/equipment.rs` (rustfmt --edition 2024 applied).
`piglin.rs` untouched: no row maps to it. No compile/test run (forbidden for this role).

## Result (2026-11-18 re-audit, supersedes stale rows in the section above)

Re-verified against .vendor/vanilla-decompiled/net/minecraft/ with source present. Smallest rows first. The earlier Result section wrongly left rows 5 and 6 open; the code in this checkout already fixes both, so their status is corrected here.

- **Row 2 (S) FIXED (verified).** WitherSkeleton.populateDefaultEquipmentEnchantments (WitherSkeleton.java:78-80) is empty, so enchanted: false is correct; AbstractSkeleton.finalizeSpawn (AbstractSkeleton.java:131-139) rolls setCanPickUpLoot for every skeleton, so can_pick_up_loot: true is correct. Both are set in EQUIPMENT_REGISTRY and asserted by wither_skeleton_spawn_flags_match_vanilla.
- **Row 3 (S) FIXED (verified).** Mob.populateDefaultEquipmentSlots (Mob.java:1153) guards armor with if (itemStack.isEmpty()); equip_mob_on_spawn now skips non-empty armor slots.
- **Row 6 (S) FIXED (verified, prior section stale).** apply_vanilla_enchantments takes (stack, special_multiplier) only; the unused slot parameter was dropped and the caller at mob/mod.rs:2736 passes two args. Vanilla's slot check lives in enchantSpawnedEquipment (Mob.java:1246-1254), which the caller mirrors.
- **Row 5 (M) FIXED (verified, prior section stale).** equip_mob_from_def, select_vanilla_armor, should_happen and weighted_select_item take &mut impl RandomImpl; equip_mob_on_spawn supplies world.loot_random, matching vanilla finalizeSpawn drawing from level.getRandom() (Mob.java:1259).
- **Row 4 (M) NO CHANGE - verified faithful.** getCurrentDifficultyAt uses 0 inhabited time when no FULL chunk exists (WorldGenRegion passes 0L during generation); the loaded_chunks lookup with 0 fallback matches.
- **Row 7 (M) NO CHANGE - verified faithful.** Mob.finalizeSpawn does not roll can-pick-up-loot; species overrides do (AbstractSkeleton.java:139). Pumpkin's roll in spawn.rs:populate_spawn_equipment is gated per definition.
- **Row 1 (L) OPEN.** Deleting the duplicate EQUIPMENT_REGISTRY dispatch or the dead per-species overrides spans skeleton/mod.rs, zombie/mod.rs and mod.rs:2570, all outside this unit; behaviour-visible, needs a compile.

Also replaced the two .expect() calls in equipment.rs tests with ok_or(..)? and Result<(), &'static str> returns (expect_used = deny); rustfmt applied. No new behaviour change in this pass beyond that lint fix; no compile run (forbidden).

## Result (verification pass, source present)

Re-verified all seven rows against .vendor/vanilla-decompiled/net/minecraft/. No in-scope code changes were needed; the fixes below are already present in equipment.rs and spawn.rs and match vanilla. No code was compiled (forbidden for this role).

- **Row 2 (S) FIXED, verified.** WitherSkeleton.java:79-80 populateDefaultEquipmentEnchantments is empty; enchanted: false is correct. AbstractSkeleton.java:139 rolls setCanPickUpLoot for every skeleton; can_pick_up_loot: true is correct. Both set in EQUIPMENT_REGISTRY and asserted by wither_skeleton_spawn_flags_match_vanilla (equipment.rs:1201).
- **Row 3 (S) FIXED, verified.** Mob.java:1146-1158 guards armor with if (itemStack.isEmpty()). equip_mob_on_spawn (equipment.rs:776) skips non-empty armor slots; MAIN_HAND stays unconditional as in the skeleton overrides.
- **Row 5 (M) FIXED, verified.** Mob.java:1259 draws from level.getRandom(). should_happen, select_vanilla_armor, equip_mob_from_def, weighted_select_item take &mut impl RandomImpl and equip_mob_on_spawn supplies world.loot_random. Residual: apply_vanilla_enchantments still uses process-wide rand::rng() for the provider cost roll, but its signature has a caller in mob/mod.rs:2736 (outside this unit), so it was not changed.
- **Row 6 (S) FIXED, verified.** Vanilla's slot filter lives in enchantSpawnedEquipment (Mob.java:1246-1254), which the caller mirrors; apply_vanilla_enchantments takes (stack, special_multiplier) only.
- **Row 4 (M) NO CHANGE, verified faithful.** getCurrentDifficultyAt uses 0 inhabited time when no FULL chunk exists; the loaded_chunks lookup with a 0 fallback matches.
- **Row 7 (M) NO CHANGE, verified faithful.** Mob.finalizeSpawn (Mob.java:1256-1269) does not roll can-pick-up-loot; species overrides do (AbstractSkeleton.java:139). Pumpkin's roll in spawn.rs:populate_spawn_equipment is gated per definition.
- **Row 1 (L) OPEN.** Duplicate EQUIPMENT_REGISTRY dispatch and dead per-species overrides span skeleton/mod.rs, zombie/mod.rs, piglin.rs and mod.rs:2570, files outside this unit; behaviour-visible and needs a compile.

Files edited this pass: none (only this doc). piglin.rs and spawn.rs need no change: no in-scope row maps to them beyond the already-fixed spawn roll.


## Result (CODER re-audit 2026-11-18, source present)

Re-verified every row against .vendor/vanilla-decompiled/net/minecraft/. No in-scope code changes were needed: the current checkout already contains every in-scope fix and vanilla confirms them. No code was compiled (forbidden for this role).

- **Row 2 (S) FIXED, verified.** AbstractSkeleton.finalizeSpawn (AbstractSkeleton.java:139) rolls setCanPickUpLoot for every skeleton; WitherSkeleton.populateDefaultEquipmentEnchantments (WitherSkeleton.java:79-80) is empty. equipment.rs wither_skeleton entry has enchanted: false, can_pick_up_loot: true, asserted by wither_skeleton_spawn_flags_match_vanilla (equipment.rs:1201).
- **Row 3 (S) FIXED, verified.** Mob.populateDefaultEquipmentSlots (Mob.java:1153) guards armor with if (itemStack.isEmpty()); equip_mob_on_spawn (equipment.rs:776) skips non-empty armor slots.
- **Row 5 (M) FIXED, verified (residual out of scope).** Mob.java:1259 draws from level.getRandom(); should_happen, select_vanilla_armor, equip_mob_from_def, weighted_select_item take &mut impl RandomImpl and equip_mob_on_spawn supplies world.loot_random. Residual: apply_vanilla_enchantments still uses rand::rng() for the provider cost roll; fixing it needs its caller at mob/mod.rs:2736, outside this unit.
- **Row 6 (S) FIXED, verified.** Vanilla's slot filter lives in enchantSpawnedEquipment (Mob.java:1246-1254), which the caller mirrors; apply_vanilla_enchantments takes (stack, special_multiplier) only.
- **Row 4 (M) NO CHANGE, verified faithful.** getCurrentDifficultyAt uses 0 inhabited time when no FULL chunk exists; the loaded_chunks lookup with 0 fallback matches vanilla.
- **Row 7 (M) NO CHANGE, verified faithful.** Mob.finalizeSpawn (Mob.java:1256-1269) does not roll can-pick-up-loot; species overrides do, and spawn.rs:populate_spawn_equipment rolls it per EQUIPMENT_REGISTRY definition.
- **Row 1 (L) OPEN.** Removing the duplicate EQUIPMENT_REGISTRY dispatch or the per-species overrides spans skeleton/*, zombie/* and mod.rs:2570, outside this unit; behaviour-visible and needs a compile. Note: the registry has no zombie entries and piglin.rs has no populate_default_equipment_slots override, so no live double-application exists in this checkout.

Files edited this pass: this doc only. equipment.rs, piglin.rs and spawn.rs need no change: no in-scope row remains unfixed. No new test seam (all fixes already covered by existing tests).
