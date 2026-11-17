# Item and inventory sweep (titem-sweep)

Status legend: OK = matches vanilla 1.21.x per source read, UNVERIFIED = accepted from vanilla knowledge without a mechanical check, GAP = divergence noted, not changed.

Scope: `crates/pumpkin-core/src/item/items/*`, `crates/pumpkin-inventory/src/*`. Excluded by brief: dispenser behaviour, experience bottle, mace, fishing.

## Durations and charge

| Item | Check | Source | Status |
|---|---|---|---|
| Bow use duration | 72000 max | `items/bow.rs` `USE_DURATION` | OK |
| Bow power curve | ticks/20, (p*p+2p)/3, clamp 1; min draw 3 ticks | `bow.rs` `get_power_for_time`, `release_bow` | OK |
| Bow durability | 1 per shot | `bow.rs` `damage_item_in_slot(.., 1)` | OK |
| Crossbow charge | 25 ticks base, quick charge via enchantment helper | `items/crossbow.rs` | OK (UNVERIFIED for the quick charge reduction, lives in `EnchantmentHelper`) |
| Crossbow arrow/firework power | 3.15 / 1.6 | `crossbow.rs` consts | OK |
| Trident throw threshold | 10 ticks | `items/trident.rs` `THROW_THRESHOLD_TIME` | OK |
| Trident use duration | 72000 | `trident.rs` | OK |
| Trident spin attack | 20 ticks | `trident.rs` `SPIN_ATTACK_TICKS` | UNVERIFIED |
| Spyglass use duration | 1200 | `spyglass.rs` | OK |
| Brush use duration | 200 | `brush.rs` | OK |
| Bundle use duration | 200, used by per-tick insertion timing | `bundle.rs` | UNVERIFIED, kept as is |
| Goat horn use duration | from `Instrument::PonderGoatHorn` data | `goat_horn.rs` | OK (data driven) |

## Cooldowns

| Item | Check | Source | Status |
|---|---|---|---|
| Use cooldown seconds to ticks | `(seconds * 20) as i32`, matches `UseCooldown` | `item/registry.rs` | OK |
| Cooldown group default | item registry key when the component has no group (ender pearl has `cooldown_group: None` in generated data, 1.0 s = 20 ticks) | `item/registry.rs` | OK |
| Shield disable on axe hit | duration from `disable_blocking_for_seconds` data | `entity/living.rs` | OK (data driven) |

## Projectile throw constants

| Item | POWER | ROLL | Status |
|---|---|---|---|
| Ender pearl | 1.5, roll 0 | `items/ender_pearl.rs` | OK |
| Egg | 1.5 | `items/egg.rs` | OK |
| Snowball | 1.5 | `items/snowball.rs` | OK |
| Splash/lingering potion | 0.5, roll -20 | `items/potions.rs` | OK |

## Stack sizes

Stack sizes (ender pearl 16, egg 16, snowball 16, sign 16, banner 16, honey bottle 16, potion 1, water bucket 1, boat 1) come from generated data in `pumpkin-data`. Not re-checked against a mechanical extract in this pass. UNVERIFIED.

## Tool damage

| Action | Check | Status |
|---|---|---|
| Axe and hoe block use | `item_damage_per_use` from block data | OK (data driven) |
| Bow shot | 1 | OK |

## Bone meal

Sea grass, coral and underwater bone meal selection in `items/bone_meal.rs` follows vanilla structure (wall coral on attempt 0, 1-in-4 underwater tag, sea grass 1-in-10 on test). Random ranges not compared line by line. UNVERIFIED.

## Inventory quick move

| Handler | Check | Status |
|---|---|---|
| Generic container (chest, barrel, shulker) | Container to player (end first), player to container (start first), returns empty when nothing moves | OK |
| Stonecutter | Result to inventory, input to inventory, from inventory by recipe | UNVERIFIED (read partially) |
| Grindstone, loom, smithing, crafter, cartography, beacon | Present, not compared to vanilla slot ranges in this pass | UNVERIFIED |
| Lectern | Quick move empty, matches vanilla | OK |
| Furnace, brewing stand, anvil, enchanting, villager trade | Not read in this pass | UNVERIFIED |

## Gaps

No confident divergence found in the sampled set, so no code changed in this pass. Remaining sweep: stack sizes from the extracted data, slot ranges per handler, armor equip swap, bucket fill and empty rules, flint and steel and fire charge behaviour, and the rest of the quick move table. Checks were not run (worker policy: no cargo).
