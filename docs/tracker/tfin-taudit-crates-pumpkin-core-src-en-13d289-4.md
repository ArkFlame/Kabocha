# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: damage_type.rs move_vehicle.rs statistics.rs

All 12 rows resolved; none remain open. Re-verified the current code against vanilla 1.21.4 decompiled sources under `.vendor/vanilla-decompiled/`. No `.rs` file needed an edit this pass: the three unit files (`statistics.rs`, `move_vehicle.rs`, `damage_type.rs`) already carry the row 4 and row 5 fixes, and row 8 is upstream parity rather than a bug. `rustfmt` not rerun (no `.rs` touched); compilation intentionally left to the orchestrator.
No `## Result` section left any row open, and re-reading the current source confirms every row is FIXED or VERIFIED against vanilla 1.21.4 decompiled sources under `.vendor/vanilla-decompiled/`. No `.rs` file was changed this pass, so rustfmt was not needed; compilation is left to the orchestrator.
Files edited this pass: none. No remaining open rows; no BLOCKED-DATA dependency outstanding.
Files edited this pass: `docs/tracker/taudit-crates-pumpkin-core-src-en-13d289.md` only (stale gap status resolved + this Result). No source file needed an edit; no remaining open or BLOCKED-DATA rows.

## Result
All 12 rows end FIXED / VERIFIED or NO CHANGE against vanilla 1.21.4 decompiled sources under `.vendor/vanilla-decompiled/`. No row remains open; no BLOCKED-DATA dependency is outstanding (the row 5 generated field is present). No `.rs` file needed an edit this pass: this unit's two editable files already carry the row 4 and row 5 fixes plus their pure unit tests, so rustfmt was not rerun; compilation is left to the orchestrator.

- Row 1 (respawn FoodData/breath carry-over): VERIFIED FIXED. Respawn reuses the same `Player` instance via `World::respawn_player`, so hunger/saturation/air persist; equivalent to the state vanilla's `ServerPlayer(Server, ..., ServerPlayer that)` copy constructor preserves (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:1020`).
- Row 2 (stale spawn-position TODO): VERIFIED FIXED. TODO removed; the join path sends `CPlayerSpawnPosition`.
- Row 3 (attack cooldown / enchant damage order): VERIFIED NO CHANGE. Vanilla cooldown-scales the enchantment bonus; Pumpkin's `extra_ench_damage * attack_cooldown_progress` matches (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1036,1345`).
- Row 4 (gliding movement statistics): VERIFIED FIXED in this unit's file. `crates/pumpkin-core/src/entity/player/statistics.rs::movement_statistic` orders branches exactly as vanilla `ServerPlayer.checkMovementStatistics` (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:2830-2877`): swimming, eye-in-water, in-water, climbable, on-ground (sprint/crouch/walk), then fall-flying -> `AviateOneCm`, else `FlyOneCm`. Pure unit test `movement_statistic_follows_vanilla_check_order` covers the ordering. No edit needed this pass.
- Row 5 (difficulty damage scaling): VERIFIED FIXED. Generated `crates/pumpkin-data/src/generated/damage_type.rs:9` already has `scaling: DamageScaling` (enum at :28), so there is NO missing generated field / BLOCKED-DATA dependency. `player.rs::damage_scales_with_difficulty` mirrors vanilla `DamageSource.scalesWithDifficulty` (`.vendor/vanilla-decompiled/net/minecraft/world/damagesource/DamageSource.java:190-196`); `difficulty_scaled_damage` mirrors vanilla `Player.hurtServer` Peaceful -> None, Easy -> min(amount/2+1, amount), Hard -> amount*1.5, Normal unchanged (`.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:727-739`). Unit tests exist (`difficulty_scaling_matches_vanilla_hurt_server`). No edit needed this pass.
- Row 6 (duplicate effect-removal packets): VERIFIED FIXED (prior pass). `LivingEntity::remove_effect` broadcasts once; player.rs no longer double-sends.
- Row 7 (team colour on display name): VERIFIED FIXED (prior pass). Team formatting in `get_display_name` with pure `format_team_name` helper + unit test.
- Row 8 (dimension-change duplication): VERIFIED NO CHANGE. Vanilla keeps parallel respawn/transfer paths (`.vendor/vanilla-decompiled/net/minecraft/server/level/ServerPlayer.java:1817`, `.vendor/vanilla-decompiled/net/minecraft/server/network/ServerGamePacketListenerImpl.java:1942`); the duplication is upstream parity, not a bug.
- Row 9 (relative teleport flags): VERIFIED NO CHANGE (prior audit). `request_teleport_with_flags` builds the `PositionFlag` list.
- Row 10 (drop-time merging TODO): VERIFIED FIXED in commit 11d974fc7.
- Row 11 (experience state atomicity): VERIFIED FIXED (`ExperienceState` mutex, prior audit).
- Row 12 (`previous_gamemode` on plain setGameMode): VERIFIED NO CHANGE (vanilla parity).

Files edited this pass: this doc only. Rows 4 and 5 re-checked against the vanilla sources cited above and confirmed already implemented in `statistics.rs` and generated `damage_type.rs`; no source file needed an edit.
