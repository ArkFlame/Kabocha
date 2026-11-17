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
