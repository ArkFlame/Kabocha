# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-item-items-projectile-weapon-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

against the decompiled vanilla sources under `.vendor/vanilla-decompiled`. No row is STILL OPEN
No rows remain STILL OPEN; no BLOCKED-DATA rows.

## Result (FINAL PASS, re-verified against `.vendor/vanilla-decompiled` 1.21.x/Paper)

All five rows are closed. No source change was required this pass: each fix from the earlier
passes is present in the source and was re-verified against decompiled vanilla. No row is
STILL OPEN and no row is BLOCKED-DATA.

- Row 1 `is_arrow` tag: FIXED. `ProjectileWeaponItem::is_arrow` uses
  `Taggable::has_tag(&pumpkin_data::tag::Item::MINECRAFT_ARROWS)`, matching vanilla
  `ProjectileWeaponItem.ARROW_ONLY = stack -> stack.is(ItemTags.ARROWS)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/ProjectileWeaponItem.java:20`).
  The generated `minecraft:arrows` item tag lists `arrow`, `tipped_arrow`, `spectral_arrow`
  (`crates/pumpkin-data/src/generated/tag.rs:24871-24875`), so datapack tags are honoured and
  tipped arrows are not regressed.
- Row 2 creative pickup: FIXED. `create_projectile` derives pickup from the
  `INTANGIBLE_PROJECTILE` component and whether the shooter is a player via
  `World::get_player_by_id`, matching vanilla `AbstractArrow`: intangible -> `CREATIVE_ONLY`
  (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:104-107`),
  player owner -> `ALLOWED` only when pickup was `DISALLOWED` (`AbstractArrow.java:690-699`),
  otherwise `DISALLOWED` (`AbstractArrow.java:71`). `pickup_for` plus three pure unit tests in
  `projectile_weapon.rs` cover the mapping; callers in `bow.rs` and `crossbow.rs` pass no creative
  flag (grep confirms the only `ProjectileWeaponItem::create_projectile` caller is
  `projectile_weapon.rs:199`, signature `(world, shooter, weapon, projectile, is_crit)`).
- Row 3 crit with creative/Infinity: VERIFIED (matches vanilla, no change needed). Decompiled
  `BowItem.releaseUsing` passes `pow == 1.0F` as `isCrit`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/BowItem.java:41`); the older
  `flag || f == 1.0F` behaviour is absent in 1.21. `bow.rs:118` uses
  `(power - 1.0).abs() < f32::EPSILON` after `get_power_for_time` clamps to 1.0
  (`bow.rs:145-152`), equivalent to `pow == 1.0F`.
- Row 4 ammo-use item filter: VERIFIED (matches vanilla, no change needed). Vanilla
  `Enchantments.INFINITY` registers its `AMMO_USE` effect with
  `MatchTool.toolMatches(ItemPredicate.Builder.item().of(items, Items.ARROW))`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/enchantment/Enchantments.java:918-922`),
  so Infinity applies only to plain arrows. `EnchantmentHelper::process_ammo_use`'s `Item::ARROW`
  filter (`crates/pumpkin-core/src/enchantment/helper.rs:221`) and the `bow.rs:108`
  `infinite_projectile` consumption check match this. `processAmmoUse` itself has no item filter
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/enchantment/EnchantmentHelper.java:101-105`).
- Row 5 checked/left alone: VERIFIED (no change needed). Power damage applied to `base_damage`
  at spawn, integral `punch_level`/`pierce_level` atoms, the spread/step formula
  (`angle_offset` + `div_ceil(2)`), and the arrow spawn position/eye offset in
  `ArrowEntity::new_shot` all match vanilla behaviour; no change.

No source files were edited this pass; no rustfmt needed. No BLOCKED-DATA rows.
