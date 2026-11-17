# Audit: crates/pumpkin-core/src/entity/living.rs (coverage AUDIT)

Vanilla references: `net.minecraft.world.entity.LivingEntity`, `net.minecraft.world.entity.Entity`, `net.minecraft.world.effect.MobEffectInstance`.

Fixed in this pass:
- living.rs effect load: no longer forces `blend = true` on NBT load; blend is derived from fade ticks and OR-ed into the packet flag (commit: stop forcing blend flag when loading effects from nbt).
- effect/mod.rs `create_from_nbt`: missing `show_icon` now defaults to true like vanilla `MobEffectInstance.load` instead of dropping the effect (commit: default show_icon to true when loading effect nbt), with a unit test.

## Remaining open items

| # | Site | Vanilla class / method | Issue | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | living.rs:1242 | `LivingEntity.heal` | TODO: negative-amount heal is clamped with no config switch; vanilla `heal` returns the amount healed and skips dead entities. Design/config gap. | Return the healed amount, early-return for dead entities, expose a config toggle for the clamp behaviour. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 2 | living.rs:1907 | `LivingEntity` / `Entity` fall-damage prevention (`Block.onLandedUpon`, `FALL_DAMAGE_RESETTING` tag) | FIXME: current code does a 3x3x1 block scan with a `SAFE_FALL_DISTANCE` distance test instead of checking the block state below and the fall-damage-resetting tag. | Use the block state under the entity plus the `FALL_DAMAGE_RESETTING` block tag and `Block.onLandedUpon` semantics. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | living.rs:2257 | `Entity.getGravity` | TODO: effective gravity when the entity world is not loaded is unhandled; vanilla falls back to -0.1 below the world bottom and 0.0 otherwise. | Implement the vanilla void fallback for unloaded or out-of-bounds positions. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | living.rs:2540 | `LivingEntity.travel` water branch | TODO: scaffolding descent clamp missing (sneaking, `velo.y < 0`, scaffolding at position, then `velo.y = 0`). | Add the clamp in the water travel path, gated on sneaking and scaffolding at the entity position. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 5 | living.rs:5457 | `LivingEntity` consumable-effect application | TODO: consumable-effect probability hardcoded; needs datapack support. | Move the probability table into datapack data and read it at apply time. | L, STILL OPEN: size L, skipped per wave brief |

## Result

| # | Result | Notes |
|---|---|---|
| 1 | RESOLVED, no change | Vanilla 1.21.11 `LivingEntity.heal(float)` returns `void` and skips only `health <= 0`; there is no config switch and negative amounts clamp at `setHealth`. Pumpkin already matches, so the TODO was removed and the doc comment now states it. |
| 2 | FIXED | `should_prevent_fall_damage_in_area` no longer scans the 3x3 below with a `SAFE_FALL_DISTANCE` radius: it reads the single block state under the feet and returns true for `PREVENT_AREA_FALL_DAMAGE_BLOCKS` or the `#minecraft:fall_damage_resetting` tag, which is vanilla `Block.fallOn` on the landed state. Scaffolding still needs sneaking via `should_prevent_fall_damage`. |
| 3 | RESOLVED, no change | Vanilla picks the `-0.1` / `0.0` fallback only in the client branch (`!level.isClientSide() || level.hasChunkAt(posBelow)`); the server always subtracts effective gravity. Pumpkin is server-only with no chunkless travel, so the TODO was replaced by that note. |
| 4 | FIXED | `apply_climbing_speed` now zeroes a negative Y velocity for a sneaking player on a climbable that is not `minecraft:scaffolding`, matching vanilla `handleOnClimbable` plus `isSuppressingSlidingDownLadder`; extracted as `climbing_descent_blocked` with a unit test. |
| 5 | STILL OPEN | Consumable-effect probability stays hardcoded; needs the datapack component data pipeline (`ConsumableImpl` is generated from assets), which is outside this unit. |
