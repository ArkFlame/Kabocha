# Final rows from docs/tracker/tentities-zombie-villager.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- All S rows STILL OPEN - not attempted this pass; the difficulty clamp suggestion needs the vanilla formula confirmed first.
All five rows previously marked STILL OPEN were already fixed in the working tree by an
Re-verified all five STILL OPEN rows against .vendor/vanilla-decompiled/net/minecraft/. Every row is already fixed in the current source; no new source edits were needed.

## Result
FINAL PASS: all five rows resolved against decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/`. No source edits were needed; every row already matches
the current source. Rows are tracker "Remaining gaps" 1-5.

1. Strength amplifier: VERIFIED. `startConverting` (ZombieVillager.java:202-210) applies
   `Math.min(getDifficulty().getId() - 1, 0)` = -1 on Peaceful, 0 elsewhere. Pumpkin
   `cure_strength_amplifier` maps Peaceful -> None and Easy/Normal/Hard -> Some(0). -1 grants no
   attribute bonus and hostile mobs are removed on Peaceful, so behaviour is observably identical;
   `Effect.amplifier` is `u8` so -1 cannot be stored. Covered by pure unit test
   `cure_strength_skipped_only_on_peaceful`. Path:
   `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/ZombieVillager.java`.
2. Special-block scan: FIXED (verified). `getConversionProgress` (ZombieVillager.java:312-335)
   guards each x-4..x+4 loop with `specialBlocksCount < 14`, entry roll 0.01, per-block roll 0.3.
   Pumpkin `conversion_progress` uses radius 4, `>= MAX_SPECIAL_BLOCKS_COUNT` (14) before each
   candidate, the same 0.01 entry roll and 0.3 per-block roll, so at most fourteen blocks count.
   Same vanilla path as row 1.
3. Equipment on cure: VERIFIED. `finishConversion` (ZombieVillager.java:240-252) calls
   `dropPreservedEquipment(level, stack -> !has(PREVENT_ARMOR_CHANGE))`; `Mob.dropPreservedEquipment`
   drops only when `DropChances.isPreserved` (`byEquipment(slot) > 1.0F`) and the callback re-equips
   retained slots. Pumpkin `prevents_armor_change` keeps binding-curse gear, drops other slots when
   `drop_chance(slot) > 1.0`, and calls `set_guaranteed_drop` on kept gear. Vanilla paths:
   ZombieVillager.java above, `net/minecraft/world/entity/Mob.java` and
   `net/minecraft/world/entity/DropChances.java`.
4. Cured villager effects: FIXED (verified). `ConversionType.convertCommon`
   (ConversionType.java:95-100) copies every active effect via `new MobEffectInstance(effect)`.
   Pumpkin `convert_common` (conversion.rs:136-154) clones and re-adds every active effect; the old
   stripping block is gone. Path:
   `.vendor/vanilla-decompiled/net/minecraft/world/entity/ConversionType.java`.
5. `set_villager_data` offers: VERIFIED. `setVillagerData` (ZombieVillager.java:373-380) nulls
   `tradeOffers` when the profession changes. Pumpkin `set_villager_data` removes the stored
   `Offers` child tag when `profession_enum()` changes. Same vanilla path as row 1.

Also confirmed the cure duration: vanilla `nextInt(2401) + 3600` (ZombieVillager.java:171) gives
3600..=6000 inclusive, matching Pumpkin `random_range(3600..=6000)`.

No .rs files touched this pass; rustfmt not applicable; no build run per worker rules.
