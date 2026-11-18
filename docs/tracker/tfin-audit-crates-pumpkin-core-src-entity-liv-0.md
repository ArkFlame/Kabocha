# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-liv.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: living.rs

| 1 | living.rs:1242 | `LivingEntity.heal` | TODO: negative-amount heal is clamped with no config switch; vanilla `heal` returns the amount healed and skips dead entities. Design/config gap. | Return the healed amount, early-return for dead entities, expose a config toggle for the clamp behaviour. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 2 | living.rs:1907 | `LivingEntity` / `Entity` fall-damage prevention (`Block.onLandedUpon`, `FALL_DAMAGE_RESETTING` tag) | FIXME: current code does a 3x3x1 block scan with a `SAFE_FALL_DISTANCE` distance test instead of checking the block state below and the fall-damage-resetting tag. | Use the block state under the entity plus the `FALL_DAMAGE_RESETTING` block tag and `Block.onLandedUpon` semantics. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | living.rs:2257 | `Entity.getGravity` | TODO: effective gravity when the entity world is not loaded is unhandled; vanilla falls back to -0.1 below the world bottom and 0.0 otherwise. | Implement the vanilla void fallback for unloaded or out-of-bounds positions. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | living.rs:2540 | `LivingEntity.travel` water branch | TODO: scaffolding descent clamp missing (sneaking, `velo.y < 0`, scaffolding at position, then `velo.y = 0`). | Add the clamp in the water travel path, gated on sneaking and scaffolding at the entity position. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
