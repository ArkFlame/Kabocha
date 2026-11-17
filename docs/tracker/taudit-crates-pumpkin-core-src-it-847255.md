# Audit: crates/pumpkin-core/src/item/items/bow.rs

Tracker row: audit / bow.rs (AUDIT). Markers found: none real — the only "vanilla"
matches are two doc comments (bow.rs:156, bow.rs:175) documenting the
`ProjectileWeaponItem` ports. The item logic is a faithful port of
`BowItem.use`/`BowItem.releaseUsing` (1.20.1/1.21 line: `use` fails without
ammo unless instabuild; `releaseUsing` folds instabuild+Infinity into `flag`,
shrinks the ammo stack once, hurts the used stack via `getSlotForHand`, plays
`ARROW_SHOOT` with `1/(rand*0.4+1.2)+f*0.5`, awards `ITEM_USED`).

## Fixed in this pass
- bow.rs:92 (was :85): release ignored Infinity when the ammo stack vanished
  mid-draw; vanilla `flag = instabuild || Infinity` still fires a plain arrow.
- bow.rs:206 (was :206): shoot sound was sent only to players tracking the
  shooter's chunk; vanilla `Level.playSound(null, x, y, z, ...)` broadcasts to
  every player within 16 blocks (now `World::play_sound_raw`, matching every
  other in-tree sound call).

## Remaining gaps (not fixed)
1. **No use-action plumbing** — `ItemBehaviour` has no `get_use_action`/`UseAction`
   equivalent (grep: no `use_action` anywhere under `crates/pumpkin-core/src/item`),
   so the client never receives the vanilla `BOW` use action; the draw pose is
   inferred from `SetActiveHand`. Vanilla: `Item.getUseAnimation` returns
   `UseAnim.BOW` for bows, sent through item/equipment sync.
   File: crates/pumpkin-core/src/item/mod.rs (trait), bow.rs:23.
   Suggested fix: add a `use_action() -> Option<UseAnim>` to `ItemBehaviour`,
   add the animation to `CSyncEntityData`/equipment packets, set `UseAnim::Bow`
   in `BowItem`. Size: L (protocol + client prediction), client-only effect.
2. **Natural use-duration expiry never releases the bow** — when
   `item_use_time` hits 0, `LivingEntity::tick` clears the active hand without
   calling `ItemRegistry::on_stopped_using` (living.rs:4807-4943). Vanilla
   `LivingEntity.updateUsingItem` calls `releaseUsingItem()` ->
   `onStoppedUsing`, so a bow drawn for the full 72000 ticks (`USE_DURATION`
   bow.rs:64) still fires. Note `release_bow` derives elapsed ticks from the
   countdown, so any wiring must run before `item_use_time` is zeroed.
   File: crates/pumpkin-core/src/entity/living.rs:4807.
   Suggested fix: on expiry, call `on_stopped_using(&stack, player)` for players
   before `clear_active_hand()`. Size: S, but affects every item (food, potions,
   trident), so it needs the full item suite checked.
3. **Duplicate speed constant** — `BowItem::ARROW_SPEED_MULTIPLIER` (bow.rs:66) is
   now dead: `shoot` uses `ProjectileWeaponItem::ARROW_SPEED_MULTIPLIER`
   (bow.rs:190). Remove or re-export. Size: XS, cosmetic.
4. **Crit test is `f32::EPSILON`-based** — bow.rs:114 uses
   `(power - 1.0).abs() < f32::EPSILON` where vanilla uses `f == 1.0F`; today
   `get_power_for_time(>=20)` clamps to exactly 1.0 so behaviour matches, but a
   data-driven draw time would drift. Size: XS, no change recommended.
5. **Ammo count for multishot bows** — bow.rs:118 consumes exactly one arrow even
   when `ProjectileWeaponItem::draw` returned several (Multishot). Vanilla
   `BowItem` also shrinks the ammo stack once, so this is correct for bows;
   `fire_arrow_with_crit` (bow.rs:216) passes a raw `ItemStack` and does not
   consume at all, which is only used by mobs. No fix; logged for reviewers.

## Verified as already vanilla
- Power curve `(p*p+2p)/3` clamped at 1.0, minimum draw 3 ticks, crit at full draw.
- Ammo lookup order (offhand, then inventory) and creative fallback arrow.
- Creative no-consume/no-durability (via `damage_item_stack_internal`),
  `Stats.ITEM_USED` award, sound pitch formula, `ArrowPickup::CreativeOnly` for
  Infinity/creative shots, arrow spawn at `eyeY - 0.1` (arrow.rs:117).
