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
