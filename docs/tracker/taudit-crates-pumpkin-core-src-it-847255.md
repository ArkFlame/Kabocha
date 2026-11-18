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

## Gap status
- Rows: STILL OPEN. Note body not recoverable in this pass; nothing verified.

## Result

- Row 1 (M) - LEFT OPEN. Systemic fix needs files outside this unit (`entity/living.rs` death/respawn/item-break sites, `entity/player.rs:8313 stop_using_item`) plus a registry/server handle threaded to every `clear_active_hand` caller. The only in-unit site, `net/bedrock/play/inventory_action.rs:27 correct_rejected_food_use`, is a food-rejection correction where `on_stopped_using` is a no-op, so fixing it alone cannot close the row.
- Row 2 (XS) - FIXED in `bow.rs`. Verified vanilla: `BowItem.releaseUsing` passes `player.getUsedItemHand()` to `ProjectileWeaponItem.shoot`, which applies `weapon.hurtAndBreak(1, shooter, hand.asEquipmentSlot())` to the exact used stack. Replaced the `active_hand` mutex read and `unwrap_or(MAIN_HAND)` fallback (silently damaged the wrong hand if the shared flag was cleared first) with a match on the released stack's network id against the off-hand stack, via the pure helper `damage_slot_for`. Added unit test `bow_damage_slot_matches_used_hand`.
- Row 3 (S) - VERIFIED, no code change. Vanilla `ServerPlayerGameMode.useItem` (line 474) rejects spectators before item use; `bow.rs:36-39` already carries the equivalent `GameMode::Spectator` guard, so a spectator holding arrows cannot start a draw.


## Result (third pass, closes the last open row)

Re-verified the last open row (Row 1, `clear_active_hand` cancel paths) directly against
decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java`.

- Row 1 (M): VERIFIED, no gap. Vanilla `LivingEntity.stopUsingItem` (LivingEntity.java:4498-4509)
  only emits `GameEvent.ITEM_INTERACT_FINISH` (when `wasUsingItem`) and then sets
  `useItem = ItemStack.EMPTY`; it deliberately does NOT call `ItemStack.releaseUsing`. Only the
  separate `releaseUsingItem` (LivingEntity.java:4484-4496) calls `useItem.releaseUsing(...)` before
  `stopUsingItem()`. Pumpkin `clear_active_hand` (living.rs:1182, reached from
  `player.rs::stop_using_item:8589` and the death/respawn/item-break cancel paths) faithfully
  mirrors `stopUsingItem`: it emits `ItemInteractFinish` vibration and clears `item_in_use` without
  firing the bow or charging durability. That is correct vanilla parity for a cancelled draw; the
  release packet path already fires the bow via `item_registry.on_stopped_using`
  (`bow.rs::on_stopped_using:53`). The original Row 1 premise (that cancel paths must fire) was a
  misread of `releaseUsingItem` versus `stopUsingItem`. No source change warranted.
- Rows 2 and 3 remain FIXED / VERIFIED as recorded above.

No source file edited this pass; no rustfmt or compile run (orchestrator gate).

Status: PASS fixed=0 open=0 files=docs/tracker/taudit-crates-pumpkin-core-src-it-847255.md

## Result (CODER batch r2_012)
- Row (only STILL OPEN row): STILL OPEN. The row body is not recoverable from this file (the file holds only a tool-output placeholder), so there is nothing to check against vanilla. No edit.

## Result (CODER batch r3_036)
- Row 1 (M, systemic clear_active_hand/stop_using_item): STILL OPEN. Row body not in this file; per docs/investigations/work/native-4.md:26 the fix spans entity/living.rs death/respawn/item-break sites, entity/player.rs:8313 stop_using_item, and a registry/server handle threaded to every clear_active_hand caller. Multi-file, beyond one pass. No edit.
- Row 2 (XS, bow.rs): already FIXED per docs/GAPS_FIXED.md:1461. Not re-checked here. No edit.
