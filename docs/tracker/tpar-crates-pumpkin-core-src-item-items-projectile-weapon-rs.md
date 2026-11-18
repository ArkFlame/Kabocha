# Parity rows for crates/pumpkin-core/src/item/items/projectile_weapon.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-item-items** [partial/OWN-OPEN] own audit: 1 open, 2 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-item-items.md) docs/tracker/taudit-crates-pumpkin-core-src-item-items.md

## Result (FINAL PASS, verified against `.vendor/vanilla-decompiled` 1.21.x)

All five rows are closed. No source change was required this pass: each row was re-verified
against the decompiled vanilla sources under `.vendor/vanilla-decompiled`. No row is STILL OPEN
and no row is BLOCKED-DATA.

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
  `projectile_weapon.rs` cover the mapping; callers in `bow.rs` and `crossbow.rs` pass no creative flag.
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
  so Infinity applies only to plain arrows. `processAmmoUse` itself has no item filter
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/enchantment/EnchantmentHelper.java:101-105`),
  so Pumpkin correctly places the `Item::ARROW` filter at the enchantment-effect level in
  `EnchantmentHelper::process_ammo_use` (`crates/pumpkin-core/src/enchantment/helper.rs:221`) and
  mirrors it with the `infinite_projectile` consumption check (`bow.rs:108`). The first-pass
  audit claim that vanilla has no filter was incorrect; the third-pass resolution stands.
- Row 5 checked/left alone: unchanged, verified previously.

No rows remain STILL OPEN; no BLOCKED-DATA rows.
