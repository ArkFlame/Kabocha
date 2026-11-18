# Parity rows for crates/pumpkin-core/src/entity/mob/zombie/zombie_villager.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Zombie Villager** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-zombie-villager** [partial/OWN-OPEN] own audit: 3 open, 2 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-zombie-villager.md) docs/tracker/tentities-zombie-villager.md

## Result

Third-pass verification against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/ZombieVillager.java`
and `net/minecraft/world/entity/ConversionType.java` (plus `Mob`/`DropChances` as cited below).
Both unit rows closed; no source edits were needed this pass because the current code already
matches vanilla on every audited behaviour.

- **entities/Zombie Villager**: VERIFIED.
  - Strength amplifier (`cure_strength_amplifier`): vanilla `startConverting`
    (ZombieVillager.java:209) uses `Math.min(getDifficulty().getId() - 1, 0)`, i.e. -1 on
    Peaceful and 0 on Easy/Normal/Hard. Pumpkin maps Peaceful to None (no bonus; hostile
    mobs are removed there) and the others to Some(0); the -1 grants no attribute bonus,
    so this is observably identical. Covered by the pure unit test
    `cure_strength_skipped_only_on_peaceful`.
  - Special-block scan (`conversion_progress`): vanilla `getConversionProgress`
    (ZombieVillager.java:312-335) guards each x-4..x+4 loop with `specialBlocksCount < 14`,
    with a 0.01 entry roll and 0.3 per-block roll. Pumpkin uses radius 4 and
    `special_blocks >= MAX_SPECIAL_BLOCKS_COUNT` (14), so at most fourteen blocks count;
    the rolls match.
  - Equipment on cure (`drop_preserved_equipment`): vanilla `finishConversion`
    (ZombieVillager.java:240-252) keeps binding-curse gear via
    `!has(PREVENT_ARMOR_CHANGE)` and `Mob.dropPreservedEquipment` (Mob.java:1092-1111) drops
    only when `isPreserved` (`byEquipment > 1.0F`, DropChances.java:46-48). Pumpkin's
    `prevents_armor_change` / `drop_chance(slot) > 1.0` match; the retained
    `set_guaranteed_drop` is a harmless extra. A cure gametest would live outside this
    unit's files.
  - Cured villager effects: vanilla `convertCommon` (ConversionType.java:95-100) copies
    every active effect; Pumpkin `convert_common` (conversion.rs:145-154) does the same.
  - `set_villager_data`: vanilla `setVillagerData` (ZombieVillager.java:373-379) nulls
    `tradeOffers` on a profession change; Pumpkin removes the stored `Offers` child tag.
  Also confirmed cure duration `random_range(3600..=6000)` equals vanilla
  `nextInt(2401) + 3600` (ZombieVillager.java:171).

- **own/tentities-zombie-villager**: VERIFIED. All five rows in
  `docs/tracker/tentities-zombie-villager.md` are resolved; the stale '3 open' count in
  this row is superseded by that doc's Result sections and the per-row citations above.

No .rs files touched this pass; no build run per worker rules; rustfmt not applicable.
