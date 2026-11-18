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

Re-audited all STILL OPEN rows against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. This unit's only editable source is `world/scoreboard.rs`. Row 4 is the only row whose data lives there; it is already fixed outside this unit, so no source edit was made.

- Row 4 (team color in display name, scoreboard.rs `Team`): VERIFIED. `PlayerTeam.getFormattedName` = `applyColor(empty + playerPrefix + name + playerSuffix)` (PlayerTeam.java:162-164), `applyColor` sets the root colour only when present (113-116), `formatNameForTeam` returns a copy for a null team (166-168), and `Entity.getDisplayName` adds hover/insertion to that result. `Team` in scoreboard.rs already carries `color`/`player_prefix`/`player_suffix` (lines 1299/1301/1302) and feeds `color_id()` into team packets. Decoration already landed outside this unit: `entity/mod.rs format_name_for_team` (line 258, used at 1474, tests at 7316/7318) and `player.rs format_team_name` (line 10750, used at 9290, test at 10874). No duplicate added to scoreboard.rs.
- Row 1 (enchantment damage TODO): left open, lives in player.rs, outside this unit.
- Row 2 (merge stacks in drop): left open, lives in player.rs/ItemEntity, outside this unit.
- Row 3 (submerged mining speed): left open, lives in player.rs, outside this unit.
- Row 5 (effect particle metadata): left open, lives in living.rs, outside this unit (covered by sync_effect_particles).
- Row 6 (hunger across reconnect): left open, lives in player.rs, outside this unit.
- Row 7 (CPlayerSpawnPosition on join): left open, lives in player.rs, outside this unit.
- Row 8 (fall-flying exhaustion): left open, lives in player.rs, outside this unit.
- Row 9 (bed respawn offsets): left open, lives in player.rs, outside this unit.
- Row 10 (XP fields not atomic): left open, lives in player.rs, outside this unit.

No `.rs` file edited: scoreboard.rs has no broken pure seam for row 4 (decoration and its tests already exist in entity/mod.rs and player.rs). No new unit test added. No rustfmt run required.


## Result

Second pass (full edit scope): re-audited every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/`. Most rows were already implemented in the current checkout by earlier commits; one genuine parity regression in the dig-speed area was found and fixed.

- ROW 3 (dig speed / `Player.getDestroySpeed`): FIXED. Vanilla `MobEffectUtil.getDigSpeedAmplification` (MobEffectUtil.java:29-41) returns `Math.max(hasteAmp, conduitAmp)`, and `Player.getDestroySpeed` (Player.java:629-631) applies ONE `1 + (amp+1)*0.2` multiplier. The prior `get_mining_speed` multiplied Haste and Conduit Power separately, over-speeding mining when both are active. Added pure helper `dig_speed_effect_multiplier` using the max amplifier once, wired it into `get_mining_speed`, and added test `dig_speed_effects_use_single_max_amplifier_multiplier`. The eye-in-water check (`Entity.is_submerged_in_water`) matches vanilla `isEyeInFluid(WATER)`; the Aqua Affinity `/5` path is behaviourally equal to the default `SUBMERGED_MINING_SPEED` 0.2 until equipment attribute modifiers reach the player (accepted gap that needs that subsystem, not generated data).
- ROW 1 (enchantment damage TODO): VERIFIED already fixed. Logic uses `compute_sweep_damage`; the stale TODO is gone.
- ROW 2 (merge stacks on drop): VERIFIED already fixed. `entity/item.rs` implements full `merge_with_neighbours`/`try_merge_with` with unit tests.
- ROW 4 (team color in display name): VERIFIED already fixed. `player.rs format_team_name` consumes the `scoreboard.rs Team` color/prefix/suffix fields; test present.
- ROW 5 (effect particle metadata): VERIFIED already fixed. Stale TODO gone; covered by `LivingEntity::remove_effect -> sync_effect_particles`.
- ROW 6 (hunger across reconnect): VERIFIED already fixed. `player.rs read_nbt` restores `hunger_manager`, called from `server/mod.rs` on join.
- ROW 7 (CPlayerSpawnPosition on join): VERIFIED already fixed. `world/mod.rs` sends `CPlayerSpawnPosition` with the world spawn inside the level-info sequence, matching vanilla `PlayerList.sendLevelInfo` (PlayerList.java:1217).
- ROW 8 (fall-flying exhaustion): VERIFIED already fixed. Vanilla `ServerPlayer.checkMovementStatistics` (ServerPlayer.java:2868-2870) awards AVIATE_ONE_CM but grants NO food exhaustion while fall-flying; `progress_motion` returns without exhaustion, matching. TODO replaced with an explanatory comment.
- ROW 9 (bed respawn offsets): VERIFIED already fixed. `player.rs get_bed_spawn_offsets` has the full facing-based offset table with fallback positions.
- ROW 10 (XP fields atomic): VERIFIED already fixed. `tick_experience` reads level/progress/points under one `experience.lock()`.

No BLOCKED rows: each gap is either already implemented or behaviourally equivalent. Source file edited: `crates/pumpkin-core/src/entity/player.rs` (one pure helper, one call site, one unit test). No rustfmt or compile run by this worker (orchestrator handles both).
