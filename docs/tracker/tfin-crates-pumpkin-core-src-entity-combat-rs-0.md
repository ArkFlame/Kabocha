# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-combat-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows closed (FINAL PASS, 2026-11-18). Re-checked the live source in
`crates/pumpkin-core/src/entity/{combat.rs,living.rs,player.rs,mob/mod.rs}` and
`crates/pumpkin-core/src/item/items/mace.rs` against the decompiled vanilla in
`.vendor/vanilla-decompiled/net/minecraft/`. Every code fix from the earlier passes is
present; no new source change was needed. No generated `pumpkin-data` field is missing
for any row.

## Result

Source: `docs/tracker/tpar-crates-pumpkin-core-src-entity-combat-rs.md`, which aggregates
three trackers. Per-row terminal states:

### taudit-crates-pumpkin-core-src-en-39d9d7 (combat.rs audit)
- mace_smash_knockback (S): FIXED. `mace_smash_knockback` routes the impulse through
  `Entity::push_by` (marks velocity dirty) and still sends a motion packet to each pushed
  player; radius/power use the full 3D offset. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/item/MaceItem.java:126-158`
  (`nearby.push(x, 0.7F, z, attacker)`). Test `smash_knockback_power_uses_full_3d_distance`.
- record_damage combat enter/leave hooks (M): FIXED. `record_damage` returns
  `CombatStateChange { entered, left, end_duration }`; `living.rs:4625-4629` fires
  `on_leave_combat` / `on_enter_combat`. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/damagesource/CombatTracker.java:47-68,165-185`
  (`shouldEnterCombat` = `getEntity() instanceof LivingEntity`). Test
  `combat_reset_clears_state_and_reports_duration`.
- get_message_for_assisted_fall live item (S): FIXED. `get_fall_message` resolves the
  attacker's current main hand by id via `live_main_hand_item_name` at message time.
  Matches `.vendor/vanilla-decompiled/net/minecraft/world/damagesource/CombatTracker.java:74-81`.
  Test `assisted_fall_uses_live_item_name_and_falls_back_without_it`.
- player.rs sweep damage (M): VERIFIED. `compute_sweep_damage` is
  `(1 + SWEEPING_DAMAGE_RATIO * base) * attackStrengthScale`, with box inflate
  `(1.0, 0.25, 1.0)`, ally/marker/3-block filters, 0.4 knockback and the sweep particle.
  Matches `.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1277-1305`.
  No sweep damage type is needed (vanilla reuses the attack DamageSource; Paper-only cause tag).
- causeExtraKnockback non-living push (S): FIXED. `non_living_knockback_impulse` returns
  `(-sin*strength, 0.1, cos*strength)`, used at `player.rs:2140`. Matches
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1213`.
  Test `non_living_knockback_uses_opposite_horizontal_sign_and_small_lift`.

### taudit-crates-pumpkin-core-src-entity-com (combat.rs audit)
All six rows VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/`:
1. Sweep movement gate: `Player.isSweepAttack`, `world/entity/player/Player.java:1123-1144`
   (velocity is the client-authoritative equivalent of `getKnownMovement()`).
2. Combat enter/leave event: `CombatTracker.java:47-68,165-185`; no event 128 in this vanilla.
3. Knockback resistance once: `world/entity/LivingEntity.java:2145-2146` (`*= 1 - resistance`).
4. Death message killer fallback: `world/damagesource/DamageSource.java:169-183` (`getKillCredit()`).
5. Mace owner identity: `world/item/MaceItem.java:141-157` + `world/entity/TamableAnimal.java:188-190`
   (owner UUID comparison is functionally identical).
6. Breach double clamp: FIXED + VERIFIED. `get_damage_after_absorb` calls
   `modify_armor_effectiveness` on `armor_fraction` and clamps once to `[0,1]`, matching
   `world/damagesource/CombatRules.java:16-32`. Test `breach_subtracts_from_armor_fraction_like_vanilla`.

### titems-mace (Mace)
All six rows terminal against `.vendor/vanilla-decompiled/net/minecraft/world/item/MaceItem.java`:
- Row 11 mob smash: FIXED + VERIFIED. `mob/mod.rs try_attack` applies the base curve plus the
  Density term (`modify_fall_based_damage`), mirroring `MaceItem.getAttackDamageBonus`
  (MaceItem.java:100-124) and `world/entity/Mob.java:1609-1615`. `spawn_extra_particles_on_fall`
  is ServerPlayer-only (MaceItem.java:72-74), so its absence on mobs is correct.
- Row 12 mace detection: VERIFIED via `mace.rs:13-17` (`MaceItem::is_mace` through the registry).
- Row 13 smash eligibility: VERIFIED. `smash_fall_ready` = `fall_distance > 1.5 && !fall_flying`,
  matching `MaceItem.canSmashAttack` (MaceItem.java:160-162); no on-ground gate.
- Row 14 velocity pin: VERIFIED (no change). `player.rs` pins y=0.01 unconditionally, matching
  MaceItem.java:65 before the `onGround` branch at :71.
- Row 15 knockback: VERIFIED (no change). `(3.5 - dist3D) * 0.7 * heavy * (1 - resistance)` with
  fixed 0.7 vertical lift, matching MaceItem.java:126-158.
- Row 16 drop-onto-cow gametest: BLOCKED-DATA. Missing subsystem (not a generated field):
  `crates/pumpkin-gametest/src/world.rs` `GameTestWorld` exposes only block-state mutation,
  spawn_entity/spawn_item, entity_count_in_box, set_entity_velocity, entity_fire_ticks and
  execute_command. There is no API to equip a mace, trigger a player melee attack, or read
  entity health/damage taken or fall-damage/impulse grace. Implementing it needs the
  GameTestWorld trait plus its in-memory and server adapters extended with entity-state
  inspection and player-attack simulation. No `pumpkin-data` generated field is missing. The
  deterministic smash rules are already covered by pure-unit seams in combat.rs and mace.rs.

### Kept intentional differences (no parity requirement)
`AttackType::new` skipping sweep on Bedrock (Bedrock cannot sweep); `can_smash_attack` relying
on client-driven fall distance; `get_killer_entry` offline-killer heuristic. No STILL OPEN rows remain.
