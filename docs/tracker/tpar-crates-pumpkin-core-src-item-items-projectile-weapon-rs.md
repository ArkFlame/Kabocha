# Parity rows for crates/pumpkin-core/src/item/items/projectile_weapon.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-item-items** [partial/OWN-OPEN] own audit: 1 open, 2 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-item-items.md) docs/tracker/taudit-crates-pumpkin-core-src-item-items.md

## Result (third pass, verified against `.vendor/vanilla-decompiled` 1.21.x)

All four behavioural rows are already fixed in source and match decompiled vanilla; no further code change was required this pass.

- Row 1 `is_arrow` tag: FIXED. `ProjectileWeaponItem::is_arrow` uses `Taggable::has_tag(&pumpkin_data::tag::Item::MINECRAFT_ARROWS)`, matching vanilla `ProjectileWeaponItem.ARROW_ONLY = stack -> stack.is(ItemTags.ARROWS)` (`.vendor/vanilla-decompiled/net/minecraft/world/item/ProjectileWeaponItem.java:20`). The generated `minecraft:arrows` item tag lists `arrow`, `tipped_arrow`, `spectral_arrow` (`crates/pumpkin-data/src/generated/tag.rs:24871-24875`), so datapack tags are honoured and tipped arrows are not regressed.
- Row 2 creative pickup: FIXED. `ProjectileWeaponItem::create_projectile` (`projectile_weapon.rs`) no longer takes `is_creative` and derives pickup with `pickup_for(is_intangible, is_player_owner)`, matching vanilla `AbstractArrow`: intangible -> `CREATIVE_ONLY` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/arrow/AbstractArrow.java:104-106`), player owner -> `ALLOWED` (`AbstractArrow.java:695-699`), otherwise `DISALLOWED` (`AbstractArrow.java:71`). Callers pass no creative flag (`bow.rs:166-180` wrapper, `crossbow.rs:256` via `shoot_projectiles`); three pure unit tests in `projectile_weapon.rs` cover the mapping.
- Row 3 crit with creative/Infinity: VERIFIED (matches vanilla, no change needed). Decompiled `BowItem.releaseUsing` passes `pow == 1.0F` as `isCrit` (`.vendor/vanilla-decompiled/net/minecraft/world/item/BowItem.java:41`); the older `flag || f == 1.0F` behaviour is absent in 1.21. `bow.rs:118` uses `(power - 1.0).abs() < f32::EPSILON` after `get_power_for_time` clamps to 1.0 (`bow.rs:145-152`), equivalent to `pow == 1.0F`.
- Row 4 ammo-use item filter: VERIFIED (matches vanilla, no change needed). Vanilla `Enchantments.INFINITY` registers its `AMMO_USE` effect with `MatchTool.toolMatches(ItemPredicate.Builder.item().of(items, Items.ARROW))` (`.vendor/vanilla-decompiled/net/minecraft/world/item/enchantment/Enchantments.java:918-922`), so Infinity applies only to plain arrows. `EnchantmentHelper::process_ammo_use`'s `Item::ARROW` filter (`crates/pumpkin-core/src/enchantment/helper.rs:221`) and `bow.rs`'s `infinite_projectile` consumption check (`bow.rs:108`) match this.
- Row 5 checked/left alone: unchanged, verified previously.

No rows remain STILL OPEN; no BLOCKED-DATA rows.
