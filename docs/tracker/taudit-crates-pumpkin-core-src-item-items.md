# Tracker: item / items / projectile_weapon

Reviewed: `crates/pumpkin-core/src/item/items/projectile_weapon.rs` on branch
`kab/taudit-crates-pumpkin-core-src-item-items`.
Vanilla reference: `net.minecraft.world.item.ProjectileWeaponItem` (1.21.x),
`AbstractArrow` (pickup derivation), `BowItem.releaseUsing`, `CrossbowItem`,
`EnchantmentHelper.processAmmoUse`. No `.vendor/` tree in this worktree, so vanilla
behaviour was compared from the known sources above.

The file has no `TODO`/`FIXME`; the five "vanilla" matches are doc comments naming the
vanilla methods each helper mirrors (`createProjectile`, `draw`, `useAmmo`, `shoot`,
the class itself). Every helper has an equivalent; the divergences below are behavioural.

## Fixed in this review

| Gap | Change |
|---|---|
| `draw` truncated the ammo stack to one item before passing it to the extra (multishot) `use_ammo` calls, so `use_ammo`'s `ammo_to_use > projectile.item_count` guard used `1` instead of the real stack size. Vanilla `ProjectileWeaponItem.draw` keeps `ItemStack.copy()` of the whole stack for every iteration. | `draw` now copies the ammo stack with its real count (`ammo_copy`). |

## Remaining gaps

### 1. `is_arrow` hardcodes three items instead of `ItemTags.ARROWS` - size S

`is_arrow` (`projectile_weapon.rs:18`). Vanilla `ARROW_ONLY = stack.is(ItemTags.ARROWS)`, so a
datapack that adds an item to `#minecraft:arrows` is accepted by bows/crossbows.
`pumpkin_data::tag::Item::MINECRAFT_ARROWS` exists but the generated tag only lists
`arrow` + `spectral_arrow` (`crates/pumpkin-data/src/generated/tag.rs:20243`); vanilla data also
has `tipped_arrow`. Switching to the tag today would regress tipped arrows, so the hardcoded list
is currently *more* correct. Fix the tag asset/codegen first, then use `has_tag`.

### 2. Creative forces `ArrowPickup::CreativeOnly` - size S

`create_projectile` (`projectile_weapon.rs:46`). Vanilla derives pickup from the projectile:
`ALLOWED` when the owner is a player, `CREATIVE_ONLY` when the pickup stack carries
`INTANGIBLE_PROJECTILE` (Infinity / extra multishot ammo), `DISALLOWED` otherwise. There is no
creative case. Pumpkin's `is_creative` branch is unreachable for bows and crossbows (creative
drawing already produces an intangible stack, see `use_ammo`), but it is wrong for a creative
player firing projectiles charged in survival. Fix: drop the `is_creative` argument, test the
shooter with `World::get_player_by_id`, and update `bow.rs:168` and the `crossbow.rs` caller.
Confirm the 1.21 `AbstractArrow` pickup derivation before changing it.

### 3. `is_crit` ignores creative/Infinity, so those shots never crit - size S

`bow.rs:117` sets `is_crit` from full charge only. Vanilla
`BowItem.releaseUsing`: `bl = player.hasInfiniteMaterials() || Infinity`, then
`shoot(..., bl || f == 1.0F, ...)`, so creative and Infinity shots always crit. Outside this
row's file but the same `shoot_projectiles` crit path.

### 4. Ammo-use effects only apply to plain arrows - size S

`enchantment/helper.rs:197` requires `projectile.item.id == Item::ARROW.id`, and `bow.rs:96`
mirrors it with `infinite_projectile`. Vanilla `EnchantmentHelper.processAmmoUse` has no item
filter, so Infinity applies to tipped and spectral arrows too. Verify against the 1.21 Infinity
enchantment JSON, then drop the filter from `process_ammo_use` and from the arrow consumption
check.

### 5. Checked and left alone

- Power damage is applied to `base_damage` at spawn (`projectile_weapon.rs:55`) instead of at hit
  (`EnchantmentHelper.modifyDamage` on the hit path). `0.5 * (level + 1)` added to base `2.0` is
  the same as vanilla's `+25% * (level + 1)` multiplier on base damage, and the hit path scales
  by velocity, so the final damage matches; no change.
- `punch_level` / `pierce_level` read into atoms at spawn matches the weapon snapshot vanilla
  stores on the arrow; `modify_knockback` values are integral.
- Spread/step formula (`shoot_projectiles`, `angle_offset`, `div_ceil(2)`) reproduces vanilla's
  integer `(j + 1) / 2` arrangement for 1-4 projectiles; verified by hand for 2, 3 and 4.
- Arrow spawn position and eye offset live in `ArrowEntity::new_shot` (`eye - 0.1`), matching
  vanilla `AbstractArrow`; mob arrows pass `ArrowPickup::Disallowed` from `bow_attack.rs:98`,
  matching `ProjectileUtil.getMobArrow`.

## Gap status
- Rows: RESOLVED. All four behavioural rows re-verified against .vendor/vanilla-decompiled (1.21.x); applied fixes are present in source; see Result.

## Result (third pass, verified against `.vendor/vanilla-decompiled` 1.21.x/Paper)

All four behavioural rows are resolved in source; no further code changes were needed.
Each was re-verified against the decompiled vanilla sources this pass.

- Row 1 `is_arrow` tag: FIXED. `ProjectileWeaponItem::is_arrow` uses
  `Taggable::has_tag(&pumpkin_data::tag::Item::MINECRAFT_ARROWS)`, matching vanilla
  `ProjectileWeaponItem.ARROW_ONLY = stack -> stack.is(ItemTags.ARROWS)`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/ProjectileWeaponItem.java:20`).
  The generated `minecraft:arrows` item tag lists `arrow`, `tipped_arrow`, `spectral_arrow`
  (`crates/pumpkin-data/src/generated/tag.rs:24871`), so no tipped-arrow regression.
- Row 2 creative pickup: FIXED. `ProjectileWeaponItem::create_projectile` no longer takes
  `is_creative`; it derives pickup from the `INTANGIBLE_PROJECTILE` component and whether the
  shooter is a player via `World::get_player_by_id`, matching vanilla `AbstractArrow`:
  intangible -> `CREATIVE_ONLY` (`.../world/entity/projectile/arrow/AbstractArrow.java:104-107`),
  player owner -> `ALLOWED` (`AbstractArrow.java:695-699`), otherwise `DISALLOWED`
  (`AbstractArrow.java:71`). `pickup_for` plus its three pure unit tests in
  `projectile_weapon.rs` cover the mapping; callers in `bow.rs` and `crossbow.rs` pass no
  creative flag.
- Row 3 crit with creative/Infinity: VERIFIED (matches vanilla, no change needed). Decompiled
  `BowItem.releaseUsing` passes `pow == 1.0F` as `isCrit`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/BowItem.java:41`); the older
  `flag || f == 1.0F` behaviour is absent in 1.21. `bow.rs` uses
  `(power - 1.0).abs() < f32::EPSILON` after `get_power_for_time` clamps to 1.0, equivalent to
  `pow == 1.0F`.
- Row 4 ammo-use item filter: VERIFIED (matches vanilla, no change needed). Vanilla
  `Enchantments.INFINITY` registers its `AMMO_USE` effect with
  `MatchTool.toolMatches(ItemPredicate.Builder.item().of(items, Items.ARROW))`
  (`.vendor/vanilla-decompiled/net/minecraft/world/item/enchantment/Enchantments.java:918-922`),
  so Infinity only applies to plain arrows. `EnchantmentHelper::process_ammo_use`'s
  `Item::ARROW` filter (`crates/pumpkin-core/src/enchantment/helper.rs:221`) and the `bow.rs`
  `infinite_projectile` consumption check match this.
- Row 5: unchanged, previously checked and left alone.

No rows remain open; no BLOCKED-DATA rows.
