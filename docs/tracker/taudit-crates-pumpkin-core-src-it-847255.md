# item / bow - deep audit

Source: `crates/pumpkin-core/src/item/items/bow.rs` (plus the paths it drives:
`crates/pumpkin-core/src/item/items/projectile_weapon.rs`,
`crates/pumpkin-core/src/entity/player.rs#find_arrow/consume_arrow`,
`crates/pumpkin-core/src/net/java/play/set_held_item.rs`).
Reference: vanilla `BowItem` / `ProjectileWeaponItem` (1.21-era flow, matching the
`bl`/`bl2` comment already in the file), Paper adds no bow-specific patch.
No `TODO`/`FIXME` markers remain; the two "Vanilla" mentions are explanatory comments.

## Fixed in this audit
- Hotbar switch mid-draw cancelled the shot instead of firing it. Vanilla
  `ServerGamePacketListenerImpl.handleSetCarriedItem` calls `player.stopUsingItem()`, and
  `LivingEntity.stopUsingItem` runs `ItemStack.releaseUsing` before clearing the active item;
  `BowItem.releaseUsing` therefore fires the arrow. Pumpkin cleared the active hand without
  calling `on_stopped_using`, so the draw was silently dropped. `set_held_item.rs` now releases
  the in-use stack through `item_registry.on_stopped_using` first (no-op for food/potions,
  trident/spyglass/crossbow implementations are empty, so only the bow path changes).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | `entity/living.rs:1176-1200` (`clear_active_hand`) and its callers | `LivingEntity.stopUsingItem` calls `ItemStack.releaseUsing` then `clearActiveItem` | `clear_active_hand` only drops `item_in_use`; every caller that is not the release packet cancels a draw without firing, without durability and without the used stat. Affected: `entity/player.rs:8313 stop_using_item` (used from `player.rs:6859`), `net/bedrock/play/inventory_action.rs:27`, death/respawn/item-break paths in `living.rs:3527,3553,3589,4986`. Needs a registry/server handle at those sites, hence M not S. | M |
| 2 | `bow.rs:126-136` | damage slot comes from the used hand, which vanilla always has (`getUsedItemHand`) | if `active_hand` is `None` at release the code silently charges `MAIN_HAND`; with the ReleaseItemInUse packet the hand is still set, so no known delta, but the fallback would damage the wrong hand for a future caller that clears first | XS |
| 3 | `bow.rs:24-47` | item use is rejected for spectators by `ServerPlayerGameMode.useItem` | `normal_use_with_rotation` gates only on ammo and `GameMode::Creative`, so a spectator holding arrows can start a draw and burn ammo/durability if the packet path ever lets the action through; verify the java use-item gate before adding a guard here | S |

## Checked, no change
- Draw start: `has_arrows || creative` matches `BowItem.use` (`!getAbilities().instabuild && !bl -> fail`).
  Infinity does not bypass this, same as vanilla: an Infinity bow with no arrows cannot start a draw.
- `get_power_for_time` = `(i/20)^2 + 2(i/20)) / 3` clamped at 1.0, and the `use_ticks < 3` guard is
  exactly `f < 0.1` (i=2 -> 0.07, i=3 -> 0.1075). Critical flag `power == 1.0` matches `f == 1.0F`.
- Infinity/creative folding: `arrow_slot.is_none() && !creative && !infinity -> return` plus
  `!(has_infinity && projectile is ARROW)` reproduces vanilla `bl` / `bl2`; tipped and spectral
  arrows are still consumed by an Infinity bow, also matching vanilla.
- Ammo draw (`ProjectileWeaponItem::draw` -> `use_ammo`) sets `IntangibleProjectileImpl` when no ammo
  is used, and `EnchantmentHelper::process_ammo_use` only zeroes ammo for plain `ARROW`; the fired
  arrow then gets `ArrowPickup::CreativeOnly`, matching 1.21 `INTANGIBLE_PROJECTILE`.
- Arrow spawn point is `shooter eye height - 0.1` (`entity/projectile/arrow.rs:117-119`), the same
  offset `BowItem.getArrow` uses; the multi-shot spread loop, firework branch and velocity call in
  `projectile_weapon.rs` mirror `ProjectileWeaponItem.shoot`.
- Durability: 1 point to the used hand through `damage_item_in_slot`, creative/spectator skipped and
  `PlayerItemDamageEvent`/break stat/sound handled there (`player.rs:2118-2132`), matching
  `hurtAndBreak`; `Stats.ITEM_USED` is awarded once per shot after the arrow is fired.
- Ammo lookup priority: `find_arrow` checks offhand, then hotbar/main inventory, exactly like
  `ProjectileWeaponItem.getHeldProjectile` then `Player.getProjectile`; `consume_arrow` shrinks the
  real slot and removes empty stacks.
- Shoot sound: `1.0 / (rand * 0.4 + 1.2) + power * 0.5`, `PLAYERS` category, broadcast after the
  projectile spawn, matching `Level.playSound(... ARROW_SHOOT ...)`.

## h3 wave status
- `bow.rs:24-47` spectator item use (S): FIXED 5610b37a9. `normal_use_with_rotation` returns early for spectators, matching `ServerPlayerGameMode.useItem` (UNVERIFIED in game).
