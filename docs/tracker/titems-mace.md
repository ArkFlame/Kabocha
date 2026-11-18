# items / Mace (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/item/items/mace.rs`, `crates/pumpkin-core/src/entity/combat.rs:61-170`, `crates/pumpkin-core/src/entity/player.rs:1787-1862`, `crates/pumpkin-core/src/entity/player/client_move.rs:375-387`, `crates/pumpkin-core/src/entity/living.rs:337-387,2730-2755`, data `crates/pumpkin-data/src/generated/item.rs:64050`.

The player smash attack matches vanilla `MaceItem`: damage bonus curve (`4*f`, `12+2*(f-3)`, `22+(f-8)`), eligibility, heavy/ground/air sound choice, impact position + post-impulse fall-damage immunity, knockback with radius 3.5 / power 0.7 / x2 heavy, smash particles world event 750, durability cost from the `Weapon` component, fall distance reset after a living smash, Density (`smash_damage_per_fallen_block`), Breach (armor effectiveness), Wind Burst gating in `EnchantmentHelper::on_post_attack`, and the overkill advancement. The item's `Tool` component (`can_destroy_blocks_in_creative: false`) already covers the creative mining restriction, so `MaceItem::can_mine` is redundant but harmless. No small certain fix was needed this pass.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| `crates/pumpkin-core/src/entity/mob/mod.rs:1139` | `Mob.doHurtTarget` runs the weapon's `getAttackDamageBonus`/`postHurtEnemy`, so a mob wielding a mace performs the smash (bonus, knockback, fall reset) | share the smash path with `Mob::try_attack` | M, FIXED + VERIFIED: smash block ported; Density (`modify_fall_based_damage`) bonus now mirrored in mob/mod.rs against `MaceItem.getAttackDamageBonus` (MaceItem.java:100-124); `spawn_extra_particles_on_fall` is player-only (MaceItem.java:72-74), so N/A for mobs; scenario gametest BLOCKED-DATA as row 16 |
| `crates/pumpkin-core/src/entity/player.rs:1787` | `MaceItem` is identified by item class/behaviour, not a hardcoded item id | gate on the mace `ItemBehaviour` | S, VERIFIED: `MaceItem::is_mace` resolves through the registry behaviour class (mace.rs:13-17) with a unit test |
| `crates/pumpkin-core/src/entity/combat.rs:62` | `MaceItem.canSmashAttack` is `fallDistance > 1.5 && !isFallFlying` | confirm against 26.3 source and drop the extra on-ground gate | S, VERIFIED: no on-ground gate; `smash_fall_ready` matches MaceItem.java:160-162 |
| `crates/pumpkin-core/src/entity/player.rs:1837` | attacker velocity pinned to y=0.01 unconditionally in vanilla | move `set_velocity` into the on-ground branch | S, VERIFIED (no change): vanilla pins y=0.01 before the `onGround` branch (MaceItem.java:65 vs :71), matching player.rs:1982 |
| `crates/pumpkin-core/src/entity/combat.rs:108-112` | `getKnockbackPower` scales by `(1 - resistance)`; horizontal vector only | drop the resistance factor if vanilla uses `push` | S, VERIFIED (no change): resistance is baked into `knockbackPower` and the vertical 0.7 lift is fixed (MaceItem.java:126-158), matching `mace_smash_knockback_power` |
| `crates/pumpkin-core/src/entity/player.rs:1787-1862` | smash rules are behavioural and testable | add a pumpkin-gametest | M, BLOCKED-DATA: needs a player-attack + entity-health/fall-grace inspection harness absent from pumpkin-gametest GameTestWorld; deterministic rules covered by pure-unit seams in combat.rs |

## h9 status
- `combat.rs` `can_smash_attack` (on-ground gate): FIXED, UNVERIFIED. Dropped `!on_ground`; the gate is now vanilla `MaceItem.canSmashAttack` (`fallDistance > 1.5 && !isFallFlying`) via `smash_fall_ready`, with a unit test. No gametest; stale-fall-distance after ground-only packets is the risk to watch.

## Result

Re-review against decompiled net/minecraft/world/item/MaceItem.java (26.3, Paper-patched):
- combat.rs can_smash_attack / smash_fall_ready (row 13): CONFIRMED matches vanilla canSmashAttack = fallDistance > 1.5 && !isFallFlying; no on-ground gate in current code. No change.
- combat.rs mace_smash_knockback_power (row 15): CONFIRMED matches vanilla getKnockbackPower = (3.5 - dist) * 0.7 * heavy * (1 - resistance) with a fixed 0.7 vertical push and full 3D distance. No change.
- Rows 11, 12, 14 target files outside this unit (mob/mod.rs, player.rs); already recorded FIXED/UNVERIFIED, not touched.
- Row 16 STILL OPEN: asks for a pumpkin-gametest (drop player onto cow, assert base+bonus damage, knockback and no fall damage). A gametest fixture lives outside this unit's editable files (client_move.rs, combat.rs, mace.rs), so it cannot be added here. Added the smallest pure-unit seam instead: mace_smash_damage_curve_matches_vanilla_branches in combat.rs verifies the three linear segments at their 3/8-block boundaries against vanilla getAttackDamageBonus.


## Result (current re-review)

Re-verified every remaining row against decompiled net/minecraft/world/item/MaceItem.java (26.3, Paper-patched):
- Row 13 (combat.rs can_smash_attack / smash_fall_ready): CONFIRMED. Vanilla canSmashAttack is fallDistance > 1.5 && !isFallFlying (MaceItem.java:160-161); current code matches, no on-ground gate. No change.
- Row 15 (combat.rs mace_smash_knockback_power / mace_smash_knockback): CONFIRMED. Vanilla getKnockbackPower is (3.5 - dist) * 0.7 * heavy * (1 - resistance) (MaceItem.java:156-158) with a fixed 0.7F vertical push, full 3D distance and the same predicate filters; matches. No change.
- Row 14 (player.rs velocity pin): CONFIRMED no change needed. Vanilla setDeltaMovement(y=0.01F) is unconditional, outside the mob.onGround branch (MaceItem.java:65 vs :71); file is outside this unit anyway.
- Rows 11, 12 (mob/mod.rs, player.rs): outside this unit's editable files (client_move.rs, combat.rs, mace.rs); unchanged.
- Row 16 STILL OPEN (pumpkin-gametest: drop a player ~6 blocks onto a cow, assert base+bonus damage, knockback and no fall damage afterwards): the gametest fixture lives outside this unit's editable files, so it cannot be added here. The pure-unit seam mace_smash_damage_curve_matches_vanilla_branches in combat.rs (three linear segments meeting at the 3/8-block boundaries) is present and verified against MaceItem.getAttackDamageBonus (MaceItem.java:100-124); a new test would duplicate it, so none was added.

No source file required a fix this pass.


## Result (third pass, final)

Re-verified the smash implementation directly against the full decompiled class at .vendor/vanilla-decompiled/net/minecraft/world/item/MaceItem.java (26.3, Paper-patched):

- MaceItem.canSmashAttack (MaceItem.java:160-162) is fallDistance > 1.5 && !isFallFlying. Pumpkin combat.rs smash_fall_ready (fall_distance > 1.5 && !fall_flying) matches exactly, with no on-ground gate.
- MaceItem.getAttackDamageBonus three linear segments (MaceItem.java:106-120): 4*f / 12+2*(f-3) / 22+(f-8). Pumpkin combat.rs mace_smash_damage_bonus matches; the Density smash_damage_per_fallen_block term is added on top in player.rs:1943-1944 via smash_bonus_per_block.mul_add(fall_distance, base_curve), matching MaceItem.java:118-120.
- MaceItem.getKnockbackPower (MaceItem.java:156-158) is (3.5 - dist3D) * 0.7 * (heavy ? 2 : 1) * (1 - res) with a fixed 0.7F vertical push (MaceItem.java:133). Pumpkin mace_smash_knockback_power and mace_smash_knockback match (0.7 f32 constant, full 3D distance, fixed 0.7 lift, resistance scaling). Predicate filters (MaceItem.java:141-153) match mace_smash_can_knockback.
- MaceItem.hurtEnemy (MaceItem.java:56-84): unconditional y=0.01 velocity pin, impact-position fall-damage grace, ground/heavy/air sound chosen by victim.onGround() and fallDistance > 5, spawnExtraParticlesOnFall only in the on-ground branch, plus postHurtEnemy fall reset (MaceItem.java:92-97) and the overkill trigger. All present in player.rs:1978-2007. The vanilla source also confirms row 14 (y=0.01 pin is unconditional) and row 13 (no on-ground eligibility gate).

No source file required a fix: every smash rule matches vanilla.

Row 16 FINAL: BLOCKED-DATA for the gametest deliverable; behavior itself VERIFIED against the MaceItem.java path above.

- The full scenario (drop a player ~6 blocks onto a cow, assert base+bonus damage, knockback and no fall damage) cannot be built with the current pumpkin-gametest entity subsystem. crates/pumpkin-gametest/src/world.rs GameTestWorld exposes only block-state mutation, spawn_entity/spawn_item, entity_count_in_box, set_entity_velocity, entity_fire_ticks and execute_command. There is no API to equip a mace, trigger a player melee attack, read entity health or damage taken, or read fall-damage/impulse grace. Adding that means extending the GameTestWorld trait and all its adapters (the in-memory test world and the server adapter) with entity-state inspection and player-attack simulation, a whole missing subsystem and not a smallest test.
- The deterministic smash rules this gametest would assert are already covered by pure-unit seams: combat.rs smash_needs_a_fall_over_one_and_a_half_blocks_and_no_glide (eligibility), combat.rs mace_smash_damage_curve_matches_vanilla_branches (base+bonus curve), combat.rs smash_knockback_power_uses_full_3d_distance (knockback, heavy, resistance), and mace.rs mace_behaviour_is_identified_by_registry_class. Another unit test would duplicate them, which the MVP test rules forbid.

Rows 11-15 stay in their prior terminal states (FIXED / CONFIRMED against vanilla). No STILL OPEN rows remain.


## Result (final, third pass)

Source fix applied this pass: `crates/pumpkin-core/src/entity/mob/mod.rs` `try_attack` now adds the Density / fall-based enchantment bonus to a mace smash, mirroring the player path and vanilla `MaceItem.getAttackDamageBonus` (`.vendor/vanilla-decompiled/net/minecraft/world/item/MaceItem.java:100-124`). It reuses the existing `EnchantmentHelper::modify_fall_based_damage(&weapon, 0.0)` helper (`enchantment/helper.rs:354-362`), previously without a caller. Fully-qualified paths, so no new imports; no new helper fn; no unwrap/expect/panic/println. No pure-unit seam exists for the mob attack damage path (needs a world); the helper and base curve already have pure-unit coverage in combat.rs and mace.rs, so no test was added. rustfmt is run by the orchestrator.

Terminal states, all verified against `.vendor/vanilla-decompiled/net/minecraft/world/item/MaceItem.java`:
- Row 11 (mob smash): FIXED + VERIFIED. Density bonus now applied. `spawn_extra_particles_on_fall` is set only for `ServerPlayer` (MaceItem.java:72-74), so its absence on mobs is correct, not a gap. The full gameplay scenario remains BLOCKED-DATA as recorded for row 16.
- Row 12 (mace detection): VERIFIED via mace.rs:13-17 and unit test `mace_behaviour_is_identified_by_registry_class`.
- Row 13 (smash eligibility): VERIFIED vs MaceItem.java:160-162; `smash_fall_ready` has no on-ground gate.
- Row 14 (y=0.01 velocity pin): VERIFIED vs MaceItem.java:65, unconditional and before the `onGround` branch at :71.
- Row 15 (knockback): VERIFIED vs MaceItem.java:126-158; resistance folded into horizontal power, fixed 0.7 vertical lift.
- Row 16 (gametest): BLOCKED-DATA. Exact missing subsystem: `crates/pumpkin-gametest/src/world.rs` `GameTestWorld` exposes only block-state mutation, spawn_entity/spawn_item, entity_count_in_box, set_entity_velocity, entity_fire_ticks and execute_command; there is no API to equip a mace, trigger a player melee attack, read entity health/damage taken, or read fall-damage/impulse grace. Implementing it means extending the GameTestWorld trait and both its in-memory and server adapters. No generated pumpkin-data field is missing.

No STILL OPEN rows remain.


## Result (final pass, complete)

No source file required a fix this pass. Every rule was re-verified against .vendor/vanilla-decompiled/net/minecraft/world/item/MaceItem.java and .vendor/vanilla-decompiled/net/minecraft/world/entity/Mob.java. All six rows are terminal; no STILL OPEN rows remain.

- Row 11 (mob smash): FIXED + VERIFIED. crates/pumpkin-core/src/entity/mob/mod.rs try_attack applies the base smash curve plus the Density term (EnchantmentHelper::modify_fall_based_damage) and mirrors hurtEnemy impact grace, sound, smash knockback and fall reset. Vanilla Mob.doHurtTarget calls weaponItem.getItem().getAttackDamageBonus (Mob.java:1609) then weaponItem.hurtEnemy (Mob.java:1615); MaceItem.getAttackDamageBonus adds Density scaled by fallDistance (MaceItem.java:118-120). spawn_extra_particles_on_fall is ServerPlayer-only (MaceItem.java:72-74), so its absence on mobs is correct.
- Row 12 (mace detection): VERIFIED. crates/pumpkin-core/src/item/items/mace.rs MaceItem::is_mace resolves the behaviour through the item registry (mace.rs:13-17), with unit test mace_behaviour_is_identified_by_registry_class.
- Row 13 (smash eligibility): VERIFIED. combat.rs smash_fall_ready is fall_distance > 1.5 && !fall_flying, matching MaceItem.canSmashAttack (MaceItem.java:160-162); no on-ground gate.
- Row 14 (velocity pin): VERIFIED (no change). player.rs pins y=0.01 unconditionally within the smash branch, matching the unconditional setDeltaMovement in MaceItem.hurtEnemy (MaceItem.java:65) before the onGround branch (MaceItem.java:71).
- Row 15 (knockback): VERIFIED (no change). combat.rs mace_smash_knockback_power is (3.5 - dist3D) * 0.7 * heavy * (1 - resistance) with a fixed 0.7 vertical push and the MaceItem.knockback predicate filters, matching MaceItem.java:126-158.
- Row 16 (gametest): BLOCKED-DATA. Missing subsystem, not a missing generated field: crates/pumpkin-gametest/src/world.rs GameTestWorld exposes only block-state mutation, spawn_entity/spawn_item, entity_count_in_box, set_entity_velocity, entity_fire_ticks and execute_command. It has no API to equip a mace, trigger a player melee attack, read entity health/damage taken, or read fall-damage/impulse grace, so the drop-onto-cow scenario cannot be asserted. Implementing it means extending the GameTestWorld trait and both its in-memory and server adapters. No pumpkin-data generated field is missing. The deterministic smash rules are already covered by pure-unit seams in combat.rs (mace_smash_damage_curve_matches_vanilla_branches, smash_knockback_power_uses_full_3d_distance, smash_needs_a_fall_over_one_and_a_half_blocks_and_no_glide) and mace.rs.

Verified against a fresh read of the decompiled source this pass; code was type-checked by inspection against sibling patterns, not compiled (worker shell/build tools are denied).
