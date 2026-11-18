# Final rows from docs/tracker/tentities-nautilus.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: natural_spawner.rs nautilus.rs sound.rs tag.rs

| 6 | `nautilus.rs` (absent) | `AbstractNautilus#tick` riding sound | `Sound::EntityNautilusRiding` (`pumpkin-data/src/generated/sound.rs:120`) is never played. **verify** the vanilla interval/volume/condition (ridden by a player). | Play it from the nautilus tick. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 7 | `crates/pumpkin-core/src/world/natural_spawner.rs` (absent) | nautilus spawn placement | Neither `Nautilus` nor `ZombieNautilus` appears in the natural spawner, although biome tags exist (`tag.rs:30248 spawns_coral_variant_zombie_nautilus`). | Register both with spawn group/weight/biome tags (verify vanilla weights). | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 8 | `nautilus.rs:258-278` | `TamableAnimal#addAdditionalSaveData` | Hand-rolled taming state: writes non-vanilla `IsTame`/`DashCooldown` keys and duplicates the owner, while `ZombieNautilusEntity` uses `TamableData`. Two representations of one vanilla concept. **verify** whether vanilla persists the dash cooldown at all. | Move to `TamableData` (tame derived from owner) and drop the extra keys. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 9 | `nautilus.rs:442-456` | `Animal#mobInteract` food healing | Heal is `2.0 * nutrition`. **verify** the vanilla heal value for the nautilus food set before changing. | Match the vanilla heal amount. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
