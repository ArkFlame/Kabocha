# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs player.rs

| 1 | player.rs:1020 | `ServerPlayer(Server, ServerLevel, GameProfile, ClientInformation, ServerPlayer that)` copies FoodData (food, saturation, exhaustion, tickTimer) and the breath manager from the previous player on respawn | Player::new always uses `HungerManager::default()`/`BreathManager::default()`, so a respawn resets hunger, saturation and air instead of carrying them over. Needs the old player plumbed into the constructor used by the respawn path. | M | OPEN (not closed this pass) (h6 budget ended, not attempted) |
| 2 | player.rs:1053 | `ServerPlayer.setRespawnPosition` only stores the point; `PlayerList.sendPlayerPermissionLevel`+`ServerPlayer.sendPlayerSettings` do not send spawn position on join | Stale TODO: the join path already sends `CPlayerSpawnPosition` from world/mod.rs:4651. Delete the TODO. | S | OPEN (not closed this pass) (h6 budget ended, not attempted) |
| 3 | player.rs:1760 | 1.21 `Player.attack` applies the cooldown factor `0.2 + g*g*0.8` to ATTACK_DAMAGE, then calls `EnchantmentHelper.modifyDamage` (enchantment bonus is *not* cooldown-scaled) | Pumpkin adds `extra_ench_damage * attack_cooldown_progress`, i.e. it scales the enchantment bonus linearly. Verify against 1.21.4 `Player.attack`/`EnchantmentHelper.modifyDamage` order and drop the TODO or the multiplier. | S (verify) | OPEN (not closed this pass) (h6 budget ended, not attempted) |
| 4 | player.rs:3983 | `ServerPlayer.checkMovementStatistics` awards `AVIATE_ONE_CM` and `Entity.checkMovementStatistics` skips walk/swim stats while `isFallFlying()` | progress_motion() bails out with TODO instead of handling the glide case; gliding currently contributes no movement statistics. | M | OPEN (not closed this pass) (h6 budget ended, not attempted) |

## Result
OPEN: player rows 1-8 need FoodData, spawn-point packets, attack cooldown, damage difficulty and team-name ports; not closed in this pass.
