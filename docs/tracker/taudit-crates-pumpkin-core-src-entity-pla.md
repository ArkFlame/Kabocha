# DEEP AUDIT: crates/pumpkin-core/src/entity/player.rs

Remaining TODO/FIXME and known vanilla gaps after this pass.

| file:line | kind / TODO text | vanilla class + method | suggested fix | size |
|---|---|---|---|---|
| 1761 | TODO enchantment damage reduction | `Player.attack`: extra enchantment damage is `bonus * attackStrength` (no square) | already implemented at 1768; delete stale TODO | XS |
| 6620 | TODO merge stacks together | vanilla `Player.drop` does not merge; `ItemEntity.tryToMerge` does | implement in ItemEntity, not here | S |
| 6556 | TODO handle in water | `Player.getDigSpeed`: `f *= getAttributeValue(SUBMERGED_MINING_SPEED)` when `isEyeInFluid(WATER)` | needs the attribute + an eye-in-water check on Entity; BLOCK_BREAK_SPEED multiply: FIXED c02b709ac (UNVERIFIED order). SUBMERGED_MINING_SPEED: STILL OPEN, the aqua-affinity `/5` path stands in until item attribute modifiers reach the player | M |
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

## Gap status
- 1 stale TODO enchantment (player.rs:1761): STILL OPEN, XS, cleanup only; note says logic at 1768.
- 2 merge stacks in drop (player.rs:6620): STILL OPEN, S, belongs in ItemEntity.
- 3 submerged mining speed (player.rs:6556): STILL OPEN, M.
- 4 team color in display name (player.rs:8842): STILL OPEN, S.
- 5 effect particle metadata (player.rs:7142): FIXED by existing LivingEntity::remove_effect -> sync_effect_particles (living.rs:1821, per note); stale TODO remains, XS.
- 6 hunger across reconnect (player.rs:1020): STILL OPEN, M.
- 7 CPlayerSpawnPosition on join (player.rs:1053): STILL OPEN, S.
- 8 fall-flying exhaustion (player.rs:3976): STILL OPEN, S.
- 9 bed respawn offsets (player.rs:2431,2479,2520): STILL OPEN, M.
- 10 XP fields not atomic (player.rs:7013): STILL OPEN, M.


## Result

Audited all STILL OPEN rows against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. This unit's only editable source is `world/scoreboard.rs`; row 4 is the only row whose data lives there, and it is already fixed elsewhere.

- Row 4 (team color in display name, scoreboard.rs `Team`): VERIFIED against vanilla. `PlayerTeam.getFormattedName(Component)` = `applyColor(empty + playerPrefix + name + playerSuffix)` (PlayerTeam.java:162-164), `applyColor` sets root colour only when present (113-116); `PlayerTeam.formatNameForTeam` = `team == null ? name.copy() : team.getFormattedName(name)` (166-168); `Entity.getDisplayName` = `formatNameForTeam(getTeam(), getName())` + hover/insertion (Entity.java:6231-6234). `Team` in scoreboard.rs already carries `color`/`player_prefix`/`player_suffix` (lines 1016-1019), so no source change is needed here. The actual decoration already landed outside this unit: `entity/mod.rs:257 format_name_for_team` (test asserts order, null case and red colour at 7140-7144) and `player.rs:10481 format_team_name` (test at 10520-10528). No unused `decorate_name` duplicate added to scoreboard.rs.
- Row 1 (enchantment damage TODO): left open - lives in player.rs, outside this unit.
- Row 2 (merge stacks in drop): left open - lives in player.rs/ItemEntity, outside this unit.
- Row 3 (submerged mining speed): left open - lives in player.rs, outside this unit.
- Row 5 (effect particle metadata): left open - lives in living.rs, outside this unit (already covered by sync_effect_particles).
- Row 6 (hunger across reconnect): left open - lives in player.rs, outside this unit.
- Row 7 (CPlayerSpawnPosition on join): left open - lives in player.rs, outside this unit.
- Row 8 (fall-flying exhaustion): left open - lives in player.rs, outside this unit.
- Row 9 (bed respawn offsets): left open - lives in player.rs, outside this unit.
- Row 10 (XP fields not atomic): left open - lives in player.rs, outside this unit.

No `.rs` file edited; scoreboard.rs exposes no broken pure seam for row 4 (decoration and its tests already exist in entity/mod.rs and player.rs), and scoreboard/tests.rs is network-transport based, so no new unit test was added. No rustfmt run required.
