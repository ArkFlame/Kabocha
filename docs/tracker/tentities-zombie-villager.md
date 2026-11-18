# Tracker: entities / Zombie Villager (RE-REVIEW-UNTRACKED)

Reviewed: `crates/pumpkin-core/src/entity/mob/zombie/zombie_villager.rs` (559 lines) plus
`entity/mob/conversion.rs`, `entity/mob/zombie/mod.rs` (cure/zombify entry points).
Reference: vanilla `net.minecraft.world.entity.monster.ZombieVillager` (no decompiled tree in
this worktree: `.vendor/` absent, so vanilla behaviour is from the class as known for the
current protocol target). Line numbers are pre-fix where noted.

## Fixed in this pass

- **Cure duration upper bound** (`mob_interact`, was :479-484). Vanilla draws
  `random.nextInt(MAX_CONVERSION_TIME - MIN_CONVERSION_TIME) + MIN_CONVERSION_TIME`, i.e.
  `[3600, 5999]`; Pumpkin drew `3600..=6000`. Changed to an exclusive range.
  Vanilla class: `ZombieVillager#mobInteract`. Commit `00bc9bdc1`.
- **Default villager data** (`initialize_zombie_villager_data`, was :86-93). Vanilla's synched
  default is `new VillagerData(VillagerType.PLAINS, VillagerProfession.NONE, 1)`; Pumpkin
  rolled a random profession 0..15, so every naturally spawned zombie villager healed into a
  employed level-1 villager with instant trades. Now unemployed; the brain's job-site goal
  (`villager/ai.rs` `create_assign_profession_from_job_site`) still lets the cured villager take
  a profession, matching vanilla. Commit `c7025b038`.

## Remaining gaps

1. **Strength amplifier during curing** — `zombie_villager.rs:210` (`start_converting`).
   Vanilla: `Math.min(this.level().getDifficulty().getId() - 1, 0)`, which is `-1` on Peaceful
   and `0` elsewhere. Pumpkin hardcodes `0`. Observable only on Peaceful, where hostile mobs
   are removed anyway, so it is cosmetic. Suggested fix: pass the level difficulty in and use
   `min(difficulty_id - 1, 0)` — **S**.

2. **Special-block scan allows one fewer block than vanilla** —
   `zombie_villager.rs:242` (`conversion_progress`). Vanilla loops with the guard `j <= 14`
   checked *before* each candidate, so up to 15 beds/iron bars are rolled; the early
   `special_blocks >= MAX_SPECIAL_BLOCKS_COUNT` return here stops at 14. Effect: at most one
   missing 30% chance of +1 progress tick in a dense cluster. Suggested fix: compare with
   `> MAX_SPECIAL_BLOCKS_COUNT` (vanilla's constant name still applies) — **S**.

3. **Equipment handling on cure is a heuristic, needs vanilla check** —
   `zombie_villager.rs:260-281` (`drop_preserved_equipment`). Pumpkin transfers slots whose stack
   has an enchantment with `prevent_armor_change` (Curse of Binding) to the villager and drops
   the rest only when `drop_chance(slot) > 1.0`. Vanilla `ZombieVillager#finishConversion`
   iterates `EquipmentSlot.values()` and decides per slot; the exact predicate should be read
   from the decompiled class before changing it. Suggested fix: obtain the decompiled
   `finishConversion` body, align the predicate, and cover it with a gametest that cures a
   zombie villager wearing binding armour — **M**.

4. **Cured villager inherits the zombie's active effects** —
   `entity/mob/conversion.rs:147-159` (`convert_common`), reached from
   `zombie_villager.rs:287` (`finish_conversion`). `convert_common` copies every active effect,
   so the cured villager keeps the leftover Strength that `start_converting` applied (duration
   equals the conversion time). Vanilla `Mob#convertTo` does not transfer mob effects; the cure
   only adds Nausea 200. Suggested fix: remove `StatusEffect::STRENGTH` from the zombie before
   `convert_to` (localised), or add a `copy_effects` flag to `ConversionParams` and clear it
   for the cure — **S** (localised) / **L** (flag, touches every conversion caller).

5. **`set_villager_data` drops stored offers on profession change** —
   `zombie_villager.rs:104-118`. Vanilla's `setVillagerData` is a plain synched-data write and
   never touches the carried `Offers`/`Gossips`. No current caller changes the profession of a
   zombie villager (only type changes happen in `finalize_villager_type` /
   `mob_set_variant_name`), so this is latent, not observable. Suggested fix: drop the
   `Offers` removal and let the villager side decide trade validity — **S**.

## Checked and matching (no action)

Tracked data ids (converting / villager data / villain data finalized), NBT keys
(`VillagerData` with codec field names `type`/`profession`/`level`, `ConversionTime` `-1` when
idle, `ConversionPlayer`, `Offers`, `Gossips`, `Xp`), conversion gating in `mob_tick`
(alive + converting, progress applied to the remaining time), ambient/hurt/death sounds and
voice pitch spread (0.2, +1.0 / +2.0 baby), step sound 0.15/1.0, entity status 16 +
`WorldEvent::SoundZombieConverted` (level event 1027, silent-gated), Nausea 200 on the cured
villager, cured-zombie-villager advancement and `ZOMBIE_VILLAGER_CURED` reputation, biome
villager type applied at spawn, and `removeWhenFarAway` (`!converting && villager_xp == 0`).
Golden apple consumption respects creative mode; the interaction is consumed with and without
Weakness, as in vanilla.

## h2 parity sweep (status)
- All S rows STILL OPEN - not attempted this pass; the difficulty clamp suggestion needs the vanilla formula confirmed first.


## Result

Verified against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/ZombieVillager.java`
and `net/minecraft/world/entity/{Mob,DropChances,ConversionType,ConversionParams}.java`.

1. Strength amplifier: checked. Vanilla `startConverting` uses
   `Math.min(difficulty.getId() - 1, 0)`; `cure_strength_amplifier` already maps
   Peaceful to None and Easy/Normal/Hard to Some(0). The Peaceful amplifier -1 in
   vanilla grants no attribute bonus and hostile mobs are removed there, so the
   difference is cosmetic. Existing pure unit test `cure_strength_skipped_only_on_peaceful`
   covers it. No change.
2. Special-block scan: FIXED. Vanilla's `getConversionProgress` guards every candidate
   with `specialBlocksCount < 14`, so at most fourteen blocks are counted. Pumpkin
   used `> MAX_SPECIAL_BLOCKS_COUNT`, allowing fifteen. Changed to
   `>= MAX_SPECIAL_BLOCKS_COUNT` and corrected the comment.
3. Equipment on cure: checked. Decompiled `finishConversion` calls
   `dropPreservedEquipment(level, stack -> !has(stack, PREVENT_ARMOR_CHANGE))` and
   `Mob.dropPreservedEquipment` drops only when `dropChances.isPreserved(slot)`,
   i.e. `byEquipment(slot) > 1.0F`. Pumpkin's `prevents_armor_change` predicate and
   `drop_chance(slot) > 1.0` already match. The retained `set_guaranteed_drop` is a
   harmless older-vanilla extra. A cure gametest would live outside this unit's
   files, so none added. No change.
4. Cured villager effects: FIXED. Vanilla `ConversionType.convertCommon` copies every
   active effect onto the converted mob, so the cured villager inherits the
   leftover Strength. Removed the block that stripped all effects before
   `convert_to`; `convert_common` now supplies the vanilla behaviour.
5. `set_villager_data` offers: checked. Vanilla `ZombieVillager.setVillagerData`
   clears `tradeOffers` when the profession changes. Pumpkin already drops the
   stored `Offers` on a profession change, so the doc's claim that vanilla never
   touches offers was wrong. No change.

Also confirmed the cure duration in `mob_interact` uses `nextInt(2401) + 3600`
(3600..=6000 inclusive), matching this decompiled vanilla; the earlier doc note
claiming an exclusive upper bound was wrong for this target.

Files touched: `crates/pumpkin-core/src/entity/mob/zombie/zombie_villager.rs`
(rustfmt --edition 2024 applied, exit 0). No build run per worker rules.


## Re-verification against decompiled vanilla (this pass)

Re-checked all five rows against `.vendor/vanilla-decompiled/net/minecraft/` now present in this checkout. All rows are already resolved in the current code, so no new source changes were needed.

1. Strength amplifier: matches. `cure_strength_amplifier` maps Peaceful to None (vanilla `min(id-1,0)` = -1, no bonus) and the other levels to Some(0); covered by `cure_strength_skipped_only_on_peaceful`. No change.
2. Special-block scan: already fixed. `>= MAX_SPECIAL_BLOCKS_COUNT` (14) matches vanilla's `< 14` guard, so at most 14 blocks count. No change.
3. Equipment on cure: matches. Vanilla `dropPreservedEquipment` uses predicate `!has(PREVENT_ARMOR_CHANGE)` and `DropChances.isPreserved` is `byEquipment(slot) > 1.0F`; Pumpkin uses `prevents_armor_change` and `drop_chance(slot) > 1.0`. No change.
4. Cured villager effects: already fixed. Vanilla `ConversionType.convertCommon` copies every active effect (ZombieVillager.java conversion lines 98-100) and Pumpkin `convert_common` does the same; the effect-stripping block is gone. No change.
5. `set_villager_data` offers: matches. Vanilla clears `tradeOffers` on a profession change (lines 373-380); Pumpkin removes the stored `Offers`. No change.

Also confirmed the cure duration `VILLAGER_CONVERSION_WAIT_MIN..=MAX` = 3600..=6000 inclusive matches vanilla `nextInt(2401) + 3600` (line 171).

No .rs files touched this pass; no build run per worker rules.


## Result (second-pass re-verification)

All five rows previously marked STILL OPEN were already fixed in the working tree by an
earlier pass; no new source edits were needed. Re-checked each against the current code:

1. Strength amplifier: `cure_strength_amplifier` maps Peaceful -> None and the other three
   difficulties -> Some(0), matching vanilla `min(id - 1, 0)` (only Peaceful differs, where -1
   grants no bonus). Covered by the existing `cure_strength_skipped_only_on_peaceful` test.
   No change.
2. Special-block scan: `conversion_progress` guards with `special_blocks >= 14` before each
   candidate, so exactly 14 special blocks get the 30% roll, matching vanilla's `j >= 14` cap.
   No change.
3. Equipment on cure: `prevents_armor_change` keeps binding-curse gear; other slots drop when
   `drop_chance(slot) > 1.0`, matching vanilla `isPreserved(slot)` (`byEquipment > 1.0F`).
   Kept gear gets `set_guaranteed_drop`, matching vanilla `setDropChance(slot, 2.0F)`. No change;
   a cure gametest would live outside this unit's files.
4. Cured villager effects: `convert_common` copies every active effect, matching vanilla
   `convertCommon`; the earlier stripping block is gone. No change.
5. `set_villager_data`: drops stored Offers when the profession changes, matching vanilla
   `setVillagerData` clearing tradeOffers on a profession change. No change.

Also confirmed the cure duration is `random_range(3600..=6000)` inclusive, i.e. vanilla
`nextInt(2401) + 3600`; the doc's earlier "exclusive upper bound" note was wrong.

No .rs files touched this pass; no build run per worker rules.


## Result (decompiled-vanilla verification, current pass)

Re-verified all five STILL OPEN rows against .vendor/vanilla-decompiled/net/minecraft/. Every row is already fixed in the current source; no new source edits were needed.

1. Strength amplifier: matches. startConverting (ZombieVillager.java:209) uses Math.min(getDifficulty().getId() - 1, 0) = -1 on Peaceful, 0 elsewhere. cure_strength_amplifier maps Peaceful -> None (no bonus, hostile mobs removed there) and the others -> Some(0). Pure unit test cure_strength_skipped_only_on_peaceful covers it. No change.
2. Special-block scan: matches. getConversionProgress (ZombieVillager.java:312-335) guards each of the three loops with specialBlocksCount < 14 and iterates X-4..X+4. conversion_progress uses special_blocks >= MAX_SPECIAL_BLOCKS_COUNT (14) and radius 4, so at most 14 blocks get the 30% roll. No change.
3. Equipment on cure: matches. finishConversion (ZombieVillager.java:240-252) calls dropPreservedEquipment(level, stack -> !has(PREVENT_ARMOR_CHANGE)); Mob.dropPreservedEquipment (Mob.java:1092-1113) keeps slots failing the shouldDrop predicate (binding curse) and drops only when isPreserved (byEquipment > 1.0F, DropChances.java:46). Pumpkin prevents_armor_change / drop_chance(slot) > 1.0 match. A cure gametest would live outside this unit files. No change.
4. Cured villager effects: matches. ConversionType.convertCommon (ConversionType.java:98-100) copies every active effect; Pumpkin convert_common (conversion.rs:145-154) does the same. No change.
5. set_villager_data offers: matches. setVillagerData (ZombieVillager.java:373-379) nulls tradeOffers when the profession changes; Pumpkin removes the stored Offers. No change.

Also confirmed cure duration nextInt(2401) + 3600 (ZombieVillager.java:171) = 3600..=6000 inclusive, matching random_range(3600..=6000).

No .rs files touched; no build run per worker rules.

## Result (third-pass terminal verification)

All five rows resolved against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/`. No source edits needed this pass.

1. Strength amplifier: VERIFIED. `startConverting` (ZombieVillager.java:202-210) uses
   `Math.min(getDifficulty().getId() - 1, 0)` = -1 on Peaceful, 0 elsewhere.
   `cure_strength_amplifier` maps Peaceful -> None (no attribute bonus; hostile mobs are
   removed there) and the other levels -> Some(0). Exact -1 cannot be stored because the
   generated `Effect.amplifier` is `u8`, but a -1 Strength grants no bonus, so the
   behaviour is observably identical. Covered by `cure_strength_skipped_only_on_peaceful`.
2. Special-block scan: FIXED (verified). `getConversionProgress` (ZombieVillager.java:312-335)
   guards each of the three x-4..x+4 loops with `specialBlocksCount < 14`, with a 0.01
   entry roll and 0.3 per-block roll. `conversion_progress` uses
   `>= MAX_SPECIAL_BLOCKS_COUNT` (14), radius 4 and the same rolls, so at most fourteen
   blocks count. Already fixed in source.
3. Equipment on cure: VERIFIED. `finishConversion` (ZombieVillager.java:240-252) and
   `Mob.dropPreservedEquipment` (Mob.java:1092-1111) keep slots failing
   `!has(PREVENT_ARMOR_CHANGE)` (binding curse) and drop only when
   `DropChances.isPreserved` (`byEquipment > 1.0F`, DropChances.java:46-48).
   `prevents_armor_change` / `drop_chance(slot) > 1.0` match. The retained
   `set_guaranteed_drop` (setDropChance 2.0F) is not in this decompiled vanilla but is
   harmless; a cure gametest would live outside this unit's two files.
4. Cured villager effects: FIXED (verified). `ConversionType.convertCommon`
   (ConversionType.java:95-100) copies every active effect; Pumpkin `convert_common`
   (conversion.rs:136-154) does the same. The earlier effect-stripping block is gone.
5. `set_villager_data` offers: VERIFIED. `setVillagerData` (ZombieVillager.java:373-380)
   nulls `tradeOffers` on a profession change; Pumpkin removes the stored `Offers`
   child tag on that change.

Also confirmed the cure duration `nextInt(2401) + 3600` = 3600..=6000 (inclusive)
(ZombieVillager.java:171) matches `random_range(3600..=6000)`.

No .rs files touched; no build run per worker rules; rustfmt not applicable.

