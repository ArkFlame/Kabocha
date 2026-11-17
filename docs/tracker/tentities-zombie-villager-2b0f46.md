**FIXED** `cdaaff7ca` (feat(entity): give cured zombie villager its binding gear as equipment). Matched by commit subject on master; code not re-diffed in this sweep.

# Tracker: entities / Zombie Villager (RE-REVIEW-UNTRACKED)

Re-reviewed: actor still does not exist upstream; this is the tracker row "entities / Zombie Villager".
Source: crates/pumpkin-core/src/entity/mob/zombie/zombie_villager.rs (full, current branch), plus
entity/mob/conversion.rs and entity/mob/mod.rs (equipment/drop-chance helpers).
Reference: vanilla net.minecraft.world.entity.monster.ZombieVillager. No decompiled tree in this
worktree (.vendor/ absent), so vanilla means the class as known for the current protocol target;
every claim below is cross-checked against the current code path, not taken from a previous pass.

## Fixed in this pass

1. **Curse-of-Binding gear was handed to the cured villager's 8-slot inventory, not its equipment
   slots, and kept the default drop chance** - finish_conversion (was :309-320).
   Vanilla finishConversion does, per slot: `villager.setItemSlot(slot, stack)` and
   `villager.setDropChance(slot, 2.0F)` (Mob.GUARANTEED_DROP_CHANCE). Pumpkin wrote the stack into
   `VillagerEntity.inventory` (INVENTORY_SIZE = 8, passive/villager/mod.rs:319) at
   `slot.get_slot_index()`, i.e. FEET..HEAD landed in food/trade inventory slots, were never worn,
   never synced as equipment, and stayed droppable. Now the kept stack goes through
   `villager.mob_entity.set_item_slot(&slot, stack)` and
   `set_guaranteed_drop(&slot)` (mob/mod.rs:597-603, 2.0). Vanilla class:
   ZombieVillager#finishConversion. Commit: see this branch.

## Remaining gaps

1. **Strength amplifier during curing is hardcoded 0** - zombie_villager.rs:206-215
   (start_converting). Vanilla: `Math.min(this.level().getDifficulty().getId() - 1, 0)`, i.e. -1
   on Peaceful and 0 on Easy+. Not a one-liner in Pumpkin: `Effect.amplifier` is i32 but
   `entity/living.rs:5151` only forwards the effect to clients via `u8::try_from`, so a -1 would be
   dropped instead of synced, and `effect/mod.rs:112` reads it back with `as u8`. Suggested fix:
   widen the amplifier plumbing to signed (packet field is a VAR_INT) or clamp at 0 and document
   the divergence - **M**.

2. **Carried villager save layout needs a vanilla check** - zombie_villager.rs:511-541
   (mob_write_nbt) / :543-568 (mob_read_nbt). Pumpkin stores VillagerData (type/profession/level)
   plus root Offers, Gossips and Xp. If 1.20.5+ vanilla nests the whole remembered villager compound
   inside VillagerData, a vanilla-written world loses trades/gossips and vice versa. Verify against
   decompiled addAdditionalSaveData/readAdditionalSaveData before changing; data-compat sensitive -
   **M**.

3. **Reading a pre-flag save marks villager data finalized whenever VillagerData exists** -
   zombie_villager.rs:546-552. Vanilla reads only the boolean `VillagerDataFinalized`; Pumpkin adds
   `|| has_villager_data`, so a save written before the flag existed keeps its stored biome type
   instead of re-rolling on finalizeSpawn. Only reachable for legacy saves, and the fallback looks
   deliberate; confirm vanilla then drop the `has_villager_data` branch or comment it as compat -
   **S**.

4. **No gametest for the cure equipment path** - finish_conversion is only reachable after a full
   3600-5999 tick cure with Weakness, so the transfer above is verified by reading vanilla, not by a
   test. Suggested fix: pumpkin-gametest that cures a zombie villager wearing binding armour and
   asserts the villager wears it with a 2.0 drop chance - **M**.

## Checked and matching (no action)

Equipment drop predicate for non-binding gear (`drop_chance(slot) > 1.0` -> spawnAtLocation) and the
PREVENT_ARMOR_CHANGE test; conversion gating in mob_tick, conversion time bounds [3600, 5999] and the
Weakness requirement; Weakness removal + Strength of the drawn duration; entity status 16; the
special-block scan now lets fifteen beds/bars through (guard `> 14` before the roll, matching
vanilla's `j > 14`); defaults (plains / unemployed / level 1) and biome type on spawn; the finalized
flag and synched data ids; NBT keys ConversionTime (-1 when idle), ConversionPlayer, Xp; advancement
CuredZombieVillager + ZOMBIE_VILLAGER_CURED reputation; Nausea 200 on the cured villager; level event
1027 silent-gated; ambient/hurt/death/step sounds and the 0.2 voice pitch spread (+2.0 baby);
removeWhenFarAway (!converting && xp == 0); baby metadata; creative-mode golden-apple consumption;
effects cleared before convert_to so the cured villager starts clean; `set_villager_data` clearing
only Offers on a profession change, which matches vanilla ZombieVillager#setVillagerData.
