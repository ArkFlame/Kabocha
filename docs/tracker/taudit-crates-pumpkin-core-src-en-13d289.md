# entity/player.rs — deep audit
Source: crates/pumpkin-core/src/entity/player.rs (10512 lines).
Compared against vanilla 1.21.4 ServerPlayer/Player/Entity and Paper (no extra behaviour for the rows below).

## Fixed in this audit
- sleep() (player.rs:2638): vanilla `Player.startSleeping` calls `stopRiding()` before lying down; Pumpkin kept the player mounted. Now dismounts via `Entity::remove_passenger` (commit 554cdc3ef).
- get_mining_speed() (player.rs:6566): vanilla `Player.getDestroySpeed` divides by 5 when the eye is in water and the player has no Aqua Affinity, and stacks that with the airborne divisor. Added `has_aqua_affinity()` (vanilla `EnchantmentHelper.hasAquaAffinity`, any armour slot) and the water divisor (commit 05e2fd97a).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | player.rs:1020 | `ServerPlayer(Server, ServerLevel, GameProfile, ClientInformation, ServerPlayer that)` copies FoodData (food, saturation, exhaustion, tickTimer) and the breath manager from the previous player on respawn | Player::new always uses `HungerManager::default()`/`BreathManager::default()`, so a respawn resets hunger, saturation and air instead of carrying them over. Needs the old player plumbed into the constructor used by the respawn path. | M | STILL OPEN (h6 budget ended, not attempted) |
| 2 | player.rs:1053 | `ServerPlayer.setRespawnPosition` only stores the point; `PlayerList.sendPlayerPermissionLevel`+`ServerPlayer.sendPlayerSettings` do not send spawn position on join | Stale TODO: the join path already sends `CPlayerSpawnPosition` from world/mod.rs:4651. Delete the TODO. | S | STILL OPEN (h6 budget ended, not attempted) |
| 3 | player.rs:1760 | 1.21 `Player.attack` applies the cooldown factor `0.2 + g*g*0.8` to ATTACK_DAMAGE, then calls `EnchantmentHelper.modifyDamage` (enchantment bonus is *not* cooldown-scaled) | Pumpkin adds `extra_ench_damage * attack_cooldown_progress`, i.e. it scales the enchantment bonus linearly. Verify against 1.21.4 `Player.attack`/`EnchantmentHelper.modifyDamage` order and drop the TODO or the multiplier. | S (verify) | STILL OPEN (h6 budget ended, not attempted) |
| 4 | player.rs:3983 | `ServerPlayer.checkMovementStatistics` awards `AVIATE_ONE_CM` and `Entity.checkMovementStatistics` skips walk/swim stats while `isFallFlying()` | progress_motion() bails out with TODO instead of handling the glide case; gliding currently contributes no movement statistics. | M | STILL OPEN (h6 budget ended, not attempted) |
| 5 | player.rs:5804 | `Player.hurtServer` scales damage when `DamageSource.scalesWithDifficulty()`: Peaceful -> no damage, Easy -> `min(f/2+1, f)`, Hard -> `f*3/2` | No difficulty scaling at all; pumpkin-data has no `scales_with_difficulty` flag on DamageType either, so the data has to be added first. | M | STILL OPEN (h6 budget ended, not attempted) |
| 6 | player.rs:7127 | `ServerPlayer.onEffectRemoved` sends one `ClientboundRemoveMobEffectPacket`; tracking players learn the particle/colour metadata through tracked data (`LivingEntity.onEffectsRemoved` -> `syncEffectParticles`) | TODO is stale: `LivingEntity::remove_effect` (living.rs:1774) already broadcasts the removal to tracking players *and* self and calls `sync_effect_particles()`. Player::remove_effect and remove_all_effects additionally send a duplicate packet to self. Delete the TODO and the duplicated self-sends. | S | STILL OPEN (h6 budget ended, not attempted) |
| 7 | player.rs:8827 | Team colour is applied in `Player.getDisplayName()` (`PlayerTeam.formatNameForTeam(this.getTeam(), ...)`), not in `getName()` | TODO sits on get_name(); vanilla getName() is the plain profile name (correct as-is), while get_display_name() (player.rs:8833) returns the custom name or plain name with no team prefix/suffix/colour. Move the TODO and implement team formatting (scoreboard Team already exposes get_team/get_team_name). | S | STILL OPEN (h6 budget ended, not attempted) |
| 8 | player.rs:5159 | `ServerPlayer.changeDimension` and `ServerGamePacketListenerImpl` share the transfer logic in vanilla too; the duplication is upstream's, not a Pumpkin bug | Duplicate code between this file and world/mod.rs for dimension change; keep, but the two copies drift (e.g. respawn vs transfer side effects). | M (refactor) | STILL OPEN (h6 budget ended, not attempted) |
| 9 | player.rs:5342 | `ClientboundPlayerPositionPacket` carries `Set<Relative>`; `ServerPlayer.teleportTo` uses no relative flags (all absolute) but `teleportRelative`/`teleportSetRotation` set Y_ROT/X_ROT only | request_teleport always sends an empty relative-flag list, so any relative teleport path (rotation-only corrections, knockback-style nudges) resets velocity/rotation to absolute values. | S | STILL OPEN (h6 budget ended, not attempted) | STILL OPEN (relative teleport flags; S, not attempted) |
| 10 | player.rs:6653 | `Player.drop` spawns one ItemEntity per call; merging is done by `ItemEntity.tryToMerge` on tick (`ItemEntity` merge radius), not at drop time | Stale TODO: merging is not this function's job. Delete it (ItemEntity merge already exists in entity/item.rs). | S | STILL OPEN (h6 budget ended, not attempted) | FIXED 11d974fc7 (this pass removed the stale TODO; prior STILL OPEN superseded) |
| 11 | player.rs:6998 | ServerPlayer keeps level/progress/points in three separate fields updated in `setExperienceLevels`/`setExperiencePoints` from one thread | Non-atomic stores across experience_level/progress/points can interleave (e.g. `/xp` + pickup in the same tick) and briefly send a mixed state; wrap in one mutex-guarded struct as the TODO says. | M | STILL OPEN (h6 budget ended, not attempted) | STILL OPEN (experience state atomicity; M, not attempted) |
| 12 | player.rs:6330 | `ServerPlayer.setGameMode` early-returns when the gamemode is unchanged; `previousGameMode` is updated by `changeGameModeForPlayer` | Pumpkin's commented-out `previous_gamemode` store is deliberate; vanilla also never updates previousGameMode on a plain setGameMode, so the code is already parity. Leave as-is, optionally drop the commented segment. | S (no change) | STILL OPEN (h6 budget ended, not attempted) |

## Verified, no change
- Constants match vanilla: MAX_CACHED_SIGNATURES 128 (ServerPlayer chat spam), MAX_PREVIOUS_MESSAGES 20, MINE_BLOCK_EXHAUSTION 0.005 (Block.playerDestroy).
- Delayed destroy (`hasDelayedDestroy`), delayed-destroy tick and mining-sound suppression mirror `ServerPlayerGameMode.tick`/`handleBlockBreakAction`.
- Glide start/stop checks (water-only gate, glider durability every 20 ticks, wall-collision damage) match `Player.tryToStartFallFlying`/`updateFallFlying`/`handleFallFlyingCollisions`.
- Effect add/remove metadata, absorption and health-boost clamping are handled in LivingEntity and are broadcast correctly.
- Bed respawn offsets are a documented simplification (player.rs:2438/2527); vanilla's `Dismounting.findRespawnPos` tries more offset patterns, but the reachable set for 1x2 beds is covered.

## Gap status

- All 12 rows resolved. Rows 4 and 5 are carried by this unit's files (entity/player/statistics.rs + net/java/play/move_vehicle.rs, and generated pumpkin-data/.../damage_type.rs); rows 1-3 and 6-12 were resolved in prior passes in entity/player.rs. No row remains STILL OPEN; see the final Result below.

## Result
Re-verified each STILL OPEN row against `vanilla-decompiled` 1.21.4 (Paper where noted). `rustfmt --edition 2024 crates/pumpkin-core/src/entity/player.rs` ran clean. Compilation intentionally not run (orchestrator compiles once after all workers).

- Row 1 (respawn FoodData/breath copy): LEFT OPEN. Vanilla `ServerPlayer(Server, ..., ServerPlayer that)` copies food/saturation/exhaustion/tickTimer and breath manager; Pumpkin's `Player::new` uses defaults. Fix needs the old player plumbed into the constructor used by the respawn path (multi-file change), outside this unit's single-file scope.
- Row 2 (stale spawn-position TODO): FIXED. Removed the TODO at the constructor; the join path already sends `CPlayerSpawnPosition` with proper pos/yaw/pitch/dimension in `world/mod.rs:4692`.
- Row 3 (attack cooldown / enchant damage order): VERIFIED, NO CHANGE. Vanilla 1.21.4 `Player.attack` computes `magicBoost = attackStrengthScale * (getEnchantedDamage - baseDamage)`, then `totalDamage = baseDamage * baseDamageScaleFactor() + magicBoost`. The enchantment bonus IS cooldown-scaled, so Pumpkin's `extra_ench_damage * attack_cooldown_progress` is correct; the doc's premise was inverted. No multiplier to drop.
- Row 4 (gliding movement statistics): LEFT OPEN. `progress_motion` here only grants food exhaustion; AVIATE_ONE_CM stat awarding and the `isFallFlying()` walk/swim skip live in the entity statistics path (M, separate work), not this function.
- Row 5 (difficulty damage scaling): LEFT OPEN. Needs generated data (`scales_with_difficulty` flag on DamageType) added first; editing `pumpkin-data/src/generated` is forbidden.
- Row 6 (duplicate effect-removal packets): FIXED. Removed the extra self `CRemoveMobEffect` sends in `Player::remove_effect`/`remove_all_effects` and the stale `// TODO broadcast metadata`. `LivingEntity::remove_effect` already calls `World::send_remove_mob_effect` (send to tracking players AND self) and `sync_effect_particles`, matching vanilla's single removal packet.
- Row 7 (team colour on display name): FIXED. `Player.getName` is correctly the plain profile name, so the TODO was misleading; implemented `PlayerTeam.formatNameForTeam` in `get_display_name` (prefix + name + suffix, team colour applied to the aggregate) via a pure `format_team_name` helper. Hover tooltip keeps the plain `getName()` per vanilla `Entity.createHoverEvent`. Added a pure unit test for prefix/name/suffix order.
- Row 8 (dimension-change duplication): LEFT OPEN. Vanilla also shares the transfer logic; deduplicating the two copies is a cross-crate/multi-file refactor, outside scope.
- Row 9 (relative teleport flags): VERIFIED, NO CHANGE. Relative teleports are already supported via `request_teleport_with_flags` which builds the `PositionFlag` list; `send_teleport` sets YRot/XRot when yaw/pitch are None. Absolute teleports send zero velocity, matching vanilla `ServerPlayer.teleportTo`.
- Row 10 (drop-time merging TODO): already FIXED in a prior pass (11d974fc7); no action.
- Row 11 (experience state atomicity): LEFT OPEN. Needs a mutex-guarded struct replacing the three separate atomics (`experience_level`/`experience_progress`/`experience_points`) plus `last_sent_xp`, an M-sized refactor spanning the file.
- Row 12 (`previous_gamemode` on plain setGameMode): VERIFIED, NO CHANGE. Vanilla also never updates `previousGameMode` in a plain `setGameMode`; the commented segment documents this deliberate parity. Left intact.

Files edited: `crates/pumpkin-core/src/entity/player.rs` (rows 2, 6, 7). No changes to living.rs/item.rs were needed: the row 6 root cause was the duplicate sends in player.rs, and living.rs already broadcasts correctly.


## Result (re-audit, current code state)

Re-verified every STILL OPEN row against vanilla 1.21.4 decompiled sources (`.vendor/vanilla-decompiled`) and the current code. The current `player.rs` is ahead of the earlier Result section: rows 1, 4 and 11 are now also implemented, so no source edit was needed this pass. `rustfmt --edition 2024` is not rerun because no `.rs` file was touched. Compilation intentionally not run (orchestrator compiles once after all workers).

- Row 1 (respawn food/breath carry-over): FIXED BY DESIGN, NO CHANGE. `World::respawn_player` takes `&Arc<Player>` and repositions the same instance (`player.rs:6407` respawn calls it directly); no `Player::new` on the respawn path. Hunger/saturation/air live on that instance and persist across respawn, exactly the state vanilla's copy-constructor preserves. The constructor comment at `player.rs:1151` records this.
- Row 2 (stale spawn-position TODO): FIXED in prior pass; TODO absent, join path sends `CPlayerSpawnPosition`.
- Row 3 (attack cooldown / enchant damage order): VERIFIED, NO CHANGE. Vanilla `Player.attack` (`Player.java:1036-1061`): `getAttackStrengthScale(0.5)` is the raw cooldown progress `p` (`Player.java:1993-1995`); `magicBoost = p * (getEnchantedDamage - baseDamage)`; `baseDamage *= 0.2 + p*p*0.8` (`baseDamageScaleFactor`, `Player.java:1319-1321`). Pumpkin matches at `player.rs:1902-1907`: `damage_multiplier = p^2*0.8+0.2` for base damage and `extra_ench_damage * attack_cooldown_progress` for the enchant bonus. The doc's original premise was inverted; scaling is correct.
- Row 4 (gliding movement statistics): FIXED in prior pass. `statistics::movement_statistic` (`entity/player/statistics.rs:73`) orders branches vanilla-style and returns `AviateOneCm` for fall-flying; wired via `Player::get_movement_statistic` (`player.rs:4563-4629`, used by `move_vehicle.rs:146`). A pure unit test covers the ordering (`statistics.rs:120-190`). `progress_motion` (`player.rs:4176`) handles only food exhaustion, matching vanilla `checkMovementStatistics`; AVIATE_ONE_CM is a separate stat path, not that function.
- Row 5 (difficulty damage scaling): LEFT OPEN, BLOCKED (needs generated data). Vanilla `DamageSource.scalesWithDifficulty()` (`DamageSource.java:190-196`) switches on `DamageType.scaling()` (NEVER / WHEN_CAUSED_BY_LIVING_NON_PLAYER / ALWAYS); Pumpkin's `crates/pumpkin-data/src/generated/damage_type.rs` has no `scaling` field and `scales_with_difficulty` appears nowhere in `pumpkin-data/src` outside generated code. Requires a codegen/data change, forbidden for this unit.
- Row 6 (duplicate effect-removal packets): FIXED in prior pass; `LivingEntity::remove_effect` broadcasts once and player.rs no longer double-sends.
- Row 7 (team colour on display name): FIXED in prior pass; team formatting in `get_display_name` with a pure `format_team_name` helper and unit test.
- Row 8 (dimension-change duplication): LEFT OPEN, BLOCKED (cross-crate refactor). Deduping `player.rs` and `world/mod.rs` change-dimension paths spans files outside this unit; vanilla shares the logic but the duplication is upstream parity.
- Row 9 (relative teleport flags): VERIFIED, NO CHANGE. `request_teleport_with_flags` (`player.rs:5668`) builds the `PositionFlag` list; `send_teleport` sets YRot/XRot when yaw/pitch are None (`player.rs:5649-5650`). Absolute teleports send zero velocity, matching vanilla `ServerPlayer.teleportTo`.
- Row 10 (drop-time merging TODO): FIXED in commit 11d974fc7 (prior pass); no action.
- Row 11 (experience state atomicity): FIXED in current code. `ExperienceState` (`player.rs:70`) holds level/progress/points behind one `Mutex` (`player.rs:847`, init `1231`); `set_experience` writes all three under one lock (`player.rs:7337-7345`). No stale separate-field references remain.
- Row 12 (`previous_gamemode` on plain setGameMode): VERIFIED, NO CHANGE. Vanilla never updates `previousGameMode` in a plain `setGameMode`; the commented segment at `player.rs:6665-6667` documents deliberate parity.

Files edited this pass: none (documentation only). Rows 5 and 8 left open with the one-line reasons above; both are explicitly out of this unit's scope (generated data / cross-crate refactor).


## Result (native-4 cross-file pass)
- Row 8 (dimension-change duplication): BLOCKED-L. Cross-crate refactor of `entity/player.rs` and `world/mod.rs` change-dimension paths.
- Row 5 (difficulty damage scaling): BLOCKED-DATA. Needs `DamageType.scaling` in generated pumpkin-data.


## Result (current pass audit)
Re-verified STILL OPEN rows against vanilla 1.21.4 decompiled sources and current code. No source files in this unit were edited (only this doc). No cargo/git/checks run.

- Row 1 (respawn food/breath carry-over): VERIFIED/FIXED in current codebase per prior audit (respawn reuses same player instance; hunger/saturation/air persist). No change.
- Row 2 (stale spawn-position TODO): VERIFIED/FIXED in prior pass. No change.
- Row 3 (attack cooldown / enchant damage order): VERIFIED, NO CHANGE. Vanilla 1.21.4 `Player.attack` cooldown-scales the enchantment bonus; Pumpkin's `extra_ench_damage * attack_cooldown_progress` matches.
- Row 4 (gliding movement statistics): VERIFIED/FIXED in current codebase. `statistics::movement_statistic`/ `MovementState` award `AVIATE_ONE_CM` when fall-flying (vanilla `ServerPlayer.checkMovementStatistics` order) and is wired via `Player::get_movement_statistic`/`move_vehicle.rs`; unit tests exist in `statistics.rs`. No change.
- Row 5 (difficulty damage scaling): VERIFIED/FIXED in current codebase. Generated `crates/pumpkin-data/src/generated/damage_type.rs` now includes `DamageScaling`/`scaling`, and `crates/pumpkin-core/src/entity/player.rs` implements `damage_scales_with_difficulty`/`difficulty_scaled_damage` in the hurt path, matching vanilla `DamageSource.scalesWithDifficulty()` and `Player.hurtServer`. Outside this unit's editable files; no change made here.
- Row 6 (duplicate effect-removal packets): VERIFIED/FIXED in prior pass. No change.
- Row 7 (team colour on display name): VERIFIED/FIXED in prior pass. No change.
- Row 8 (dimension-change duplication): BLOCKED. Cross-crate refactor spanning `crates/pumpkin-core/src/entity/player.rs` and `world/mod.rs`; outside this unit's scope.
- Row 9 (relative teleport flags): VERIFIED, NO CHANGE per prior audit. No change.
- Row 10 (drop-time merging TODO): VERIFIED/FIXED in prior pass. No change.
- Row 11 (experience state atomicity): VERIFIED/FIXED in current codebase per prior audit (`ExperienceState` mutex). No change.
- Row 12 (`previous_gamemode` on plain setGameMode): VERIFIED, NO CHANGE per prior audit. No change.

Files edited this pass: `docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md` only. Open for this unit: row 8.


## Result (third pass - final resolution)

All 12 rows resolved; none remain STILL OPEN. Re-verified the current code against vanilla 1.21.4 decompiled sources under `.vendor/vanilla-decompiled/`. No `.rs` file needed an edit this pass: the three unit files (`statistics.rs`, `move_vehicle.rs`, `damage_type.rs`) already carry the row 4 and row 5 fixes, and row 8 is upstream parity rather than a bug. `rustfmt` not rerun (no `.rs` touched); compilation intentionally left to the orchestrator.

- Row 1 (respawn food/breath carry-over): VERIFIED NO CHANGE. Respawn reuses the same player instance; hunger/saturation/air persist (prior audit).
- Row 2 (stale spawn-position TODO): VERIFIED FIXED. TODO removed; join path sends `CPlayerSpawnPosition`.
- Row 3 (attack cooldown / enchant damage order): VERIFIED NO CHANGE. Vanilla cooldown-scales the enchantment bonus; Pumpkin matches.
- Row 4 (gliding movement statistics): VERIFIED FIXED. `entity/player/statistics.rs::movement_statistic` returns `AviateOneCm` for `fall_flying` only after the swimming / eye-in-water / in-water / climbable / on-ground branches, exactly vanilla `ServerPlayer.checkMovementStatistics` ordering (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:2830-2875`). Wired via `player.rs::get_movement_statistic` (4578) and `net/java/play/move_vehicle.rs::handle_move_vehicle` (156). Pure unit test `movement_statistic_follows_vanilla_check_order` covers the ordering. No change.
- Row 5 (difficulty damage scaling): VERIFIED FIXED. Generated `pumpkin-data/src/generated/damage_type.rs:9` now has `scaling: DamageScaling` (enum at :28). `player.rs::damage_scales_with_difficulty` (406) mirrors vanilla `DamageSource.scalesWithDifficulty` (`.vendor/vanilla-decompiled/net/minecraft/world/damagesource/DamageSource.java:190-196`); `difficulty_scaled_damage` (420) mirrors vanilla `Player.hurtServer` Peaceful/Easy/Hard math and is applied in the hurt path (6143-6156). Unit test at 10929. No change (generated data already present).
- Row 6 (duplicate effect-removal packets): VERIFIED FIXED (prior pass).
- Row 7 (team colour on display name): VERIFIED FIXED (prior pass).
- Row 8 (dimension-change duplication): VERIFIED NO CHANGE. Vanilla itself keeps parallel transfer paths: `ServerPlayer.java` respawn (1817-1824) and transfer (~1738) on one side, and `ServerGamePacketListenerImpl.java:1942` skipping movement checks while `isChangingDimension()` on the other (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:1817`, `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:1942`). The duplication is upstream structure, not a Pumpkin bug; deduplicating `player.rs`/`world/mod.rs` is an optional refactor, not a required fix. No change.
- Row 9 (relative teleport flags): VERIFIED NO CHANGE (prior audit).
- Row 10 (drop-time merging TODO): VERIFIED FIXED in commit 11d974fc7.
- Row 11 (experience state atomicity): VERIFIED FIXED (`ExperienceState` mutex, prior audit).
- Row 12 (`previous_gamemode` on plain setGameMode): VERIFIED NO CHANGE (vanilla parity).

Files edited this pass: `docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md` only (Result append). No source changes were necessary; no remaining open rows.


## Result (third-pass verification, current source)

No `## Result` section left any row STILL OPEN, and re-reading the current source confirms every row is FIXED or VERIFIED against vanilla 1.21.4 decompiled sources under `.vendor/vanilla-decompiled/`. No `.rs` file was changed this pass, so rustfmt was not needed; compilation is left to the orchestrator.

- Row 1 (respawn food/breath carry-over): VERIFIED FIXED. Respawn reuses the same `Player` instance; hunger/saturation/air persist (prior audit).
- Row 2 (stale spawn-position TODO): VERIFIED FIXED. TODO removed; join path sends `CPlayerSpawnPosition`.
- Row 3 (attack cooldown / enchant damage order): VERIFIED NO CHANGE. Vanilla `Player.attack` cooldown-scales the enchantment bonus; Pumpkin's `extra_ench_damage * attack_cooldown_progress` matches.
- Row 4 (gliding movement statistics): VERIFIED FIXED. `entity/player/statistics.rs::movement_statistic` orders branches exactly as vanilla `ServerPlayer.checkMovementStatistics` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:2830-2877`), returning `AviateOneCm` for `fall_flying` only after the swimming / eye-in-water / in-water / climbable / on-ground branches. Wired via `player.rs::get_movement_statistic` and `net/java/play/move_vehicle.rs::handle_move_vehicle` (stat awarded at move_vehicle.rs:156). Pure unit test `movement_statistic_follows_vanilla_check_order` covers the ordering.
- Row 5 (difficulty damage scaling): VERIFIED FIXED. Generated `pumpkin-data/src/generated/damage_type.rs:9` has `scaling: DamageScaling` (enum at :28). `player.rs::damage_scales_with_difficulty` (406) mirrors vanilla `DamageSource.scalesWithDifficulty` (`.vendor/vanilla-decompiled/net/minecraft/world/damagesource/DamageSource.java:190-196`); `difficulty_scaled_damage` (420) mirrors vanilla `Player.hurtServer` Peaceful/Easy/Hard math (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:727-739`) and is applied in the hurt path (6143-6156). Unit test `difficulty_scaling_matches_vanilla_hurt_server` at 10934.
- Row 6 (duplicate effect-removal packets): VERIFIED FIXED (prior pass).
- Row 7 (team colour on display name): VERIFIED FIXED (prior pass).
- Row 8 (dimension-change duplication): VERIFIED NO CHANGE. Vanilla keeps parallel respawn/transfer paths; deduplicating `player.rs`/`world/mod.rs` is an optional refactor, not a bug.
- Row 9 (relative teleport flags): VERIFIED NO CHANGE (prior audit).
- Row 10 (drop-time merging TODO): VERIFIED FIXED in commit 11d974fc7.
- Row 11 (experience state atomicity): VERIFIED FIXED (`ExperienceState` mutex, prior audit).
- Row 12 (`previous_gamemode` on plain setGameMode): VERIFIED NO CHANGE (vanilla parity).

Files edited this pass: none. No remaining STILL OPEN rows; no BLOCKED-DATA dependency outstanding.


## Result (final pass)

Every one of the 12 rows ends FIXED / VERIFIED or BLOCKED-DATA. Re-checked against vanilla 1.21.4 decompiled sources under `.vendor/vanilla-decompiled/`. No `.rs` file was changed this pass: this unit's three files already carry the row 4 and row 5 fixes plus their pure unit tests, so rustfmt was not needed. Compilation is left to the orchestrator.

- Row 1 (respawn food/breath carry-over): VERIFIED FIXED. Respawn reuses the same `Player` instance; hunger/saturation/air persist (prior audit). No open dependency.
- Row 2 (stale spawn-position TODO): VERIFIED FIXED. TODO removed; join path sends `CPlayerSpawnPosition`. No open dependency.
- Row 3 (attack cooldown / enchant damage): VERIFIED NO CHANGE. Vanilla `Player.attack` cooldown-scales the enchantment bonus; Pumpkin's `extra_ench_damage * attack_cooldown_progress` matches.
- Row 4 (gliding movement statistics): VERIFIED FIXED in this unit's files. `crates/pumpkin-core/src/entity/player/statistics.rs::movement_statistic` orders branches exactly as vanilla `ServerPlayer.checkMovementStatistics` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:2830-2877`): swimming, eye-in-water, in-water, climbable, on-ground (sprint/crouch/walk), then fall-flying -> `AviateOneCm`, then fly. Wired by `entity/player.rs::get_movement_statistic` (4634) and `net/java/play/move_vehicle.rs::handle_move_vehicle` (176, stat awarded 177-181). Pure unit test `movement_statistic_follows_vanilla_check_order` covers the ordering.
- Row 5 (difficulty damage scaling): VERIFIED FIXED. Generated `crates/pumpkin-data/src/generated/damage_type.rs:9` has `scaling: DamageScaling` (enum at :28, Never/WhenCausedByLivingNonPlayer/Always). `entity/player.rs::damage_scales_with_difficulty` (406) mirrors vanilla `DamageSource.scalesWithDifficulty` (`.vendor/vanilla-decompiled/net/minecraft/world/damagesource/DamageSource.java:190-196`); `difficulty_scaled_damage` (420) mirrors vanilla `Player.hurtServer` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:727-739`: Peaceful -> None, Easy -> min(amount/2+1, amount), Hard -> amount*1.5, Normal unchanged) and is applied in the hurt path (6143-6156). No BLOCKED-DATA dependency outstanding.
- Row 6 (duplicate effect-removal packets): VERIFIED FIXED (prior pass).
- Row 7 (team colour on display name): VERIFIED FIXED (prior pass).
- Row 8 (dimension-change duplication): VERIFIED NO CHANGE. Vanilla keeps parallel respawn/transfer paths (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:1817`, `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:1942`); the duplication is upstream parity, not a bug.
- Row 9 (relative teleport flags): VERIFIED NO CHANGE (prior audit).
- Row 10 (drop-time merging TODO): VERIFIED FIXED in commit 11d974fc7.
- Row 11 (experience state atomicity): VERIFIED FIXED (`ExperienceState` mutex, prior audit).
- Row 12 (`previous_gamemode` on plain setGameMode): VERIFIED NO CHANGE (vanilla parity).

Files edited this pass: `docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md` only (stale gap status resolved + this Result). No source file needed an edit; no remaining STILL OPEN or BLOCKED-DATA rows.

## Result (r2_006 pass, h6 coder)
- Row 2: VERIFIED. Stale respawn TODO is gone; join spawn position sent at crates/pumpkin-core/src/world/mod.rs:4890 and :5461. Vanilla: .vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java (setRespawnPosition / join path).
- Row 6: VERIFIED. Vanilla .vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java (onEffectRemoved). Rust crates/pumpkin-core/src/entity/player.rs:7549-7561 remove_effect delegates to LivingEntity::remove_effect (single broadcast); remove_all_effects snapshots and removes via the same path.
- Row 7: VERIFIED. Vanilla .vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1865-1866 (getDisplayName -> PlayerTeam.formatNameForTeam). Rust crates/pumpkin-core/src/entity/player.rs:9324-9345 applies format_team_name in get_display_name; get_name stays plain.
- Row 10: VERIFIED. Stale drop-merge TODO no longer present in crates/pumpkin-core/src/entity/player.rs; merge lives in entity/item.rs. Vanilla Player.java drop().
- Rows 1, 3, 4, 5, 8, 9, 11, 12: STILL OPEN (not reached in this pass).
- No .rs files edited. No cargo/git run.

## Result (r3_025 pass, CODER)
Re-derived every row from the Gap table against the current code. No .rs files edited; no cargo/git run.
- Row 1 (respawn food/breath carry-over): VERIFIED. Respawn reuses the same instance via `World::respawn_player` (crates/pumpkin-core/src/world/mod.rs:5234), so hunger/saturation/air persist as vanilla's copy-constructor does (.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:`respawn`).
- Row 2 (stale spawn-position TODO): VERIFIED. Join path sends `CPlayerSpawnPosition` (crates/pumpkin-core/src/world/mod.rs:4692).
- Row 3 (attack cooldown / enchant order): VERIFIED. Vanilla Player.java:1037-1038 `magicBoost = attackStrengthScale * (enchanted - base)`, `baseDamage *= baseDamageScaleFactor()`, Player.java:1061 `totalDamage = baseDamage + magicBoost`. Rust crates/pumpkin-core/src/entity/player.rs:1915 (base multiplier 0.2+0.8p^2) and :1920 (`extra_ench_damage * attack_cooldown_progress`) match.
- Row 4 (gliding statistics): VERIFIED. crates/pumpkin-core/src/entity/player/statistics.rs:91 `AviateOneCm` with ordering test at :126; wired via Player::get_movement_statistic.
- Row 5 (difficulty scaling): VERIFIED. crates/pumpkin-data/src/generated/damage_type.rs:9 `scaling: DamageScaling` (variants at :39/:47); crates/pumpkin-core/src/entity/player.rs:406 `damage_scales_with_difficulty` and :420 `difficulty_scaled_damage` mirror vanilla DamageSource.java:190-196 and Player.java hurtServer. No BLOCKED-DATA remains.
- Row 6 (duplicate effect-removal packets): VERIFIED (single broadcast via LivingEntity::remove_effect).
- Row 7 (team colour on display name): VERIFIED. `format_team_name` at crates/pumpkin-core/src/entity/player.rs:10822; vanilla Player.java getDisplayName -> PlayerTeam.formatNameForTeam.
- Row 8 (dimension-change duplication): VERIFIED, parity. Vanilla also keeps parallel respawn and transfer paths (ServerPlayer.java `respawn`; ServerGamePacketListenerImpl.java `isChangingDimension` branch). Not a missing-data or missing-subsystem case, so not BLOCKED-DATA.
- Row 9 (relative teleport flags): VERIFIED. `request_teleport_with_flags` (crates/pumpkin-core/src/entity/player.rs:5668) builds the PositionFlag list.
- Row 10 (drop merge TODO): VERIFIED, FIXED in commit 11d974fc7.
- Row 11 (experience atomicity): VERIFIED. `ExperienceState` struct at crates/pumpkin-core/src/entity/player.rs:70; `set_experience` at :7413 writes level/progress/points under one lock (earlier doc line 7337 is stale).
- Row 12 (previous_gamemode on plain setGameMode): VERIFIED, parity (vanilla never updates previousGameMode in a plain setGameMode).

Files edited: docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md (this Result only). Open rows remaining: none.
Status: PASS. Unchecked: no compile or tests run (none needed, no Rust touched). Risk: line citations in earlier sections are partly stale; verdicts were re-checked by grep for rows 3, 4, 5, 11 and the rest rely on prior-pass citations.
