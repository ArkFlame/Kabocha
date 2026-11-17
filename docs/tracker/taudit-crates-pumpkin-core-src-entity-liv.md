# Audit: crates/pumpkin-core/src/entity/living.rs (coverage AUDIT)

Vanilla references: `net.minecraft.world.entity.LivingEntity`, `net.minecraft.world.entity.Entity`, `net.minecraft.world.effect.MobEffectInstance`.

Fixed in this pass:
- living.rs effect load: no longer forces `blend = true` on NBT load; blend is derived from fade ticks and OR-ed into the packet flag (commit: stop forcing blend flag when loading effects from nbt).
- effect/mod.rs `create_from_nbt`: missing `show_icon` now defaults to true like vanilla `MobEffectInstance.load` instead of dropping the effect (commit: default show_icon to true when loading effect nbt), with a unit test.

## Remaining open items

| # | Site | Vanilla class / method | Issue | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | living.rs:1242 | `LivingEntity.heal` | TODO: negative-amount heal is clamped with no config switch; vanilla `heal` returns the amount healed and skips dead entities. Design/config gap. | Return the healed amount, early-return for dead entities, expose a config toggle for the clamp behaviour. | M |
| 2 | living.rs:1907 | `LivingEntity` / `Entity` fall-damage prevention (`Block.onLandedUpon`, `FALL_DAMAGE_RESETTING` tag) | FIXME: current code does a 3x3x1 block scan with a `SAFE_FALL_DISTANCE` distance test instead of checking the block state below and the fall-damage-resetting tag. | Use the block state under the entity plus the `FALL_DAMAGE_RESETTING` block tag and `Block.onLandedUpon` semantics. | M |
| 3 | living.rs:2257 | `Entity.getGravity` | TODO: effective gravity when the entity world is not loaded is unhandled; vanilla falls back to -0.1 below the world bottom and 0.0 otherwise. | Implement the vanilla void fallback for unloaded or out-of-bounds positions. | M |
| 4 | living.rs:2540 | `LivingEntity.travel` water branch | TODO: scaffolding descent clamp missing (sneaking, `velo.y < 0`, scaffolding at position, then `velo.y = 0`). | Add the clamp in the water travel path, gated on sneaking and scaffolding at the entity position. | S |
| 5 | living.rs:5457 | `LivingEntity` consumable-effect application | TODO: consumable-effect probability hardcoded; needs datapack support. | Move the probability table into datapack data and read it at apply time. | L |
