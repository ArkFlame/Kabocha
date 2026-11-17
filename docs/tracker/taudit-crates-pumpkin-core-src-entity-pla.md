# DEEP AUDIT: crates/pumpkin-core/src/entity/player.rs

Remaining TODO/FIXME and known vanilla gaps after this pass.

| file:line | kind / TODO text | vanilla class + method | suggested fix | size |
|---|---|---|---|---|
| 1761 | TODO enchantment damage reduction | `Player.attack`: extra enchantment damage is `bonus * attackStrength` (no square) | already implemented at 1768; delete stale TODO | XS |
| 6620 | TODO merge stacks together | vanilla `Player.drop` does not merge; `ItemEntity.tryToMerge` does | implement in ItemEntity, not here | S |
| 6556 | TODO handle in water | `Player.getDigSpeed`: `f *= getAttributeValue(SUBMERGED_MINING_SPEED)` when `isEyeInFluid(WATER)` | needs the attribute + an eye-in-water check on Entity; BLOCK_BREAK_SPEED multiply is also missing from this method | M |
| 8842 | TODO team color | `Entity.getDisplayName` = `Team.decorateName(getTeam(), getName())`; `PlayerTeam.getFormattedName` applies color to prefix+name+suffix | `Team` (crates/pumpkin-core/src/world/scoreboard.rs:834) already has color/player_prefix/player_suffix; decorate get_display_name | S |
| 7142 | TODO broadcast metadata | `LivingEntity.onEffectRemoved` updates `DATA_EFFECT_PARTICLES` | already covered by `LivingEntity::remove_effect` -> `sync_effect_particles` (living.rs:1821); delete stale TODO | XS |
| 1020 | TODO load from previous instance | `ServerPlayer` keeps hunger across reconnect (PlayerList reuses the saved player data) | persist/restore HungerManager with player data | M |
| 1053 | TODO send CPlayerSpawnPosition on connect | `ServerPlayer` sends `SetSpawnPosition` with the respawn point on join | send in join sequence once respawn_point is loaded | S |
| 3976 | TODO gliding in progress_motion | `ServerPlayer.checkMovementStatistics` fall-flying/water exhaustion branches | re-check the fall-flying branch against current vanilla and add it | S |
| 2431, 2479, 2520 | simplified bed offsets | `Dismounting.findRespawnPos` / `BedBlock.getBedOrientation` offsets | port the full offset table | M |
| 7013 | TODO not atomic | XP level/progress/points written independently | single lock/struct like vanilla's field triple | M |

## Verified in this pass

- Dig speed: Haste and Conduit Power now multiply separately by their own amplifier (`Player.getDigSpeed`), replacing the max-amplifier helper.
- Sleep: `Player::sleep` dismounts the vehicle first (`Player.startSleeping`).
- Accepted parity as-is: enchantment bonus damage at 1768, effect particle metadata sync at living.rs:1821.
