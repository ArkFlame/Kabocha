# Tracker: entities / Zombie Villager (RE-REVIEW-UNTRACKED)

Re-reviewed: actor does not exist upstream; this is the tracker row "entities / Zombie Villager".
Source: crates/pumpkin-core/src/entity/mob/zombie/zombie_villager.rs (full), plus
entity/mob/conversion.rs and entity/mob/zombie/mod.rs (zombify/cure entry points).
Reference: vanilla net.minecraft.world.entity.monster.ZombieVillager (no decompiled tree in this
worktree: .vendor/ is absent, so vanilla means the class as known for the current protocol target).

Reviewed because the file changed since the previous assessment (commits 4304228f5, ee4bfd657,
c48fa2510, 57a1bff0c, 8368ff257, 794ad553f).

## Fixed in this pass

1. **Cured villager inherited the zombie's effects** - finish_conversion (was :285-287).
   Vanilla convertTo() spawns a fresh EntityType.VILLAGER, which starts with no effects; the cure
   only adds Nausea 200. Pumpkin's convert_common() copies every active effect
   (conversion.rs:145-154), so a cured villager kept the leftover Strength (and any potion
   effects the zombie had). Now the zombie's effects are cleared before convert_to. Vanilla
   class: ZombieVillager#finishConversion / Entity#convertTo. Commit 4ed5eede9.
2. **Golden apple consumed the interaction without Weakness** - mob_interact (was :457-478).
   Vanilla only returns sidedSuccess when the stack is a golden apple AND the zombie has
   Weakness; otherwise it falls through to super.mobInteract. Pumpkin returned true for any
   golden apple, swallowing the fall-through (e.g. leash handling) and forcing a swing.
   Now the Weakness test gates the whole branch. Vanilla: ZombieVillager#mobInteract.
   Commit bc603483c.
3. **Special-block scan stopped one candidate short** - conversion_progress:243.
   Vanilla's loop guard is j <= 14, so fifteen beds/iron bars can roll the 30% +1. The early
   return used >= 14 and stopped at fourteen. Changed to > MAX_SPECIAL_BLOCKS_COUNT.
   Vanilla: ZombieVillager#getConversionProgress. Commit d261bca4b.

## Remaining gaps

1. **Equipment handling on cure is a heuristic, needs vanilla check** -
   zombie_villager.rs:261-282 (drop_preserved_equipment) and :285 (finish_conversion).
   Pumpkin transfers slots whose stack has an enchantment with prevent_armor_change
   (Curse of Binding) to the villager and drops the rest only when drop_chance(slot) > 1.0.
   Vanilla ZombieVillager#finishConversion iterates EquipmentSlot.values() and decides per slot,
   also raising the villager-side drop chance to 2.0 for the transferred slot. Suggested fix:
   read the decompiled finishConversion body, align the predicate, and cover it with a gametest
   that cures a zombie villager wearing binding armour - **M**.
2. **Transferred binding gear never gets the villager-side drop chance** -
   zombie_villager.rs:301-310. Vanilla calls villager.setDropChance(slot, 2.0F) next to
   setItemSlot, so the item can never be dropped by the cured villager. Pumpkin writes the stack
   into villager.inventory but leaves the drop chance at the villager default. Suggested fix:
   set the matching equipment_drop_chances entry on the villager - **S**.
3. **Strength amplifier during curing is hardcoded 0** - zombie_villager.rs:210
   (start_converting). Vanilla: Math.min(this.level().getDifficulty().getId() - 1, 0), i.e.
   -1 on Peaceful and 0 elsewhere; the code comment already says so. Effectively identical on
   Peaceful (negative amplifier is clamped), so cosmetic. Suggested fix: pass the difficulty id
   and use min(id - 1, 0) - **S**.
4. **Carried villager save layout needs a vanilla check** - zombie_villager.rs:511-541
   (mob_write_nbt) / :543-560 (mob_read_nbt). Pumpkin stores the remembered villager data as a
   root VillagerData compound (type/profession/level) plus separate root Offers, Gossips and Xp.
   The 1.20.5+ vanilla ZombieVillager keeps the whole villager save compound under
   VillagerData (so Offers/Gossips/Xp are nested inside it). If that recollection is right, a
   world written by vanilla would load with the zombie villager's trades/gossips lost and vice
   versa. Verify against the decompiled addAdditionalSaveData/readAdditionalSaveData before
   changing; converting the layout is data-compat sensitive - **M**.
5. **set_villager_data erases only Offers on a profession change** -
   zombie_villager.rs:101-118. Vanilla's setVillagerData (1.20.5+) appears to discard the whole
   carried villager compound when the profession changes, whereas Pumpkin removes only Offers and
   leaves Gossips/Xp stale. No current caller changes a zombie villager's profession (only the
   villager type in finalize_villager_type / mob_set_variant_name), so latent. Suggested fix:
   confirm the vanilla effect, then clear the whole carried compound - **S**.
6. **Reading a pre-flag save marks villager data finalized whenever VillagerData exists** -
   zombie_villager.rs:543-552. Vanilla reads only the VillagerDataFinalized boolean; Pumpkin
   treats the presence of a VillagerData compound as finalized, so such a zombie villager keeps
   the saved biome type instead of re-rolling it on finalizeSpawn. Only reachable for saves
   written before the flag existed. Suggested fix: verify vanilla's readAdditionalSaveData and
   mirror it - **S**.

## Checked and matching (no action)

Conversion gating in mob_tick (alive && converting, progress subtracted from the remaining time,
finish at <= 0), conversion time bounds [3600, 5999] and the Weakness requirement, Weakness
removal + Strength of the drawn duration, entity status 16, defaults (plains / unemployed /
level 1) and biome villager type on spawn, finalize flag + synched data ids (converting,
villager data, villager data finalized), NBT keys and -1 ConversionTime when idle, ConversionPlayer
uuid, Xp, advancement CuredZombieVillager + ZOMBIE_VILLAGER_CURED reputation, Nausea 200 on the
cured villager, level event 1027 silent-gated, ambient/hurt/death/step sounds and the 0.2 voice
pitch spread (+2.0 baby), removeWhenFarAway (!converting && xp == 0), baby flag in metadata, and
creative-mode golden-apple consumption.
