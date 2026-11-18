# items / Mace (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/item/items/mace.rs`, `crates/pumpkin-core/src/entity/combat.rs:61-170`, `crates/pumpkin-core/src/entity/player.rs:1787-1862`, `crates/pumpkin-core/src/entity/player/client_move.rs:375-387`, `crates/pumpkin-core/src/entity/living.rs:337-387,2730-2755`, data `crates/pumpkin-data/src/generated/item.rs:64050`.

The player smash attack matches vanilla `MaceItem`: damage bonus curve (`4*f`, `12+2*(f-3)`, `22+(f-8)`), eligibility, heavy/ground/air sound choice, impact position + post-impulse fall-damage immunity, knockback with radius 3.5 / power 0.7 / x2 heavy, smash particles world event 750, durability cost from the `Weapon` component, fall distance reset after a living smash, Density (`smash_damage_per_fallen_block`), Breach (armor effectiveness), Wind Burst gating in `EnchantmentHelper::on_post_attack`, and the overkill advancement. The item's `Tool` component (`can_destroy_blocks_in_creative: false`) already covers the creative mining restriction, so `MaceItem::can_mine` is redundant but harmless. No small certain fix was needed this pass.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| `crates/pumpkin-core/src/entity/mob/mod.rs:1139` | `Mob.doHurtTarget` runs the weapon's `getAttackDamageBonus`/`postHurtEnemy`, so a mob wielding a mace performs the smash (bonus, knockback, fall reset) | move the smash block out of `Player::attack` into a shared `combat::apply_smash_attack` used by `Mob::try_attack` too | M, FIXED (f3cb9b739): `Mob::try_attack` gates on `combat::is_mace_weapon` + `can_smash_attack`, adds `mace_smash_damage_bonus`, sets the 0.01 y pin, impact grace, sound, `combat::mace_smash_knockback` and fall reset; UNVERIFIED: no gametest, mob enchant Density bonus and `spawn_extra_particles_on_fall` not mirrored |
| `crates/pumpkin-core/src/entity/player.rs:1787` | `MaceItem` is identified by the item class/behaviour, not a hardcoded item id, so component-overridden/datapack maces still smash | gate on the mace `ItemBehaviour` instead of `Item::MACE.id` | S, FIXED (560ee65e0): `ItemRegistry::is_behaviour::<MaceItem>` via `server.item_registry` |
| `crates/pumpkin-core/src/entity/combat.rs:62` | `MaceItem.canSmashAttack` is `fallDistance > 1.5 && !isFallFlying`; Pumpkin also requires `!on_ground`, which can reject a smash in the landing tick | confirm against the 26.3 `MaceItem` source and drop the extra on-ground gate if absent | S, UNVERIFIED: kept the `!on_ground` gate; it guards stale fall distance after a ground-only packet (documented in combat.rs), and no decompiled `MaceItem` source confirms the bare `fallDistance > 1.5 && !isFallFlying` form, so no change |
| `crates/pumpkin-core/src/entity/player.rs:1837` | attacker velocity is pinned to y=0.01 only when the victim is on the ground (an air smash keeps the attacker falling) | move `set_velocity` into the `victim.on_ground` branch already used for the sound choice | S, UNVERIFIED: recalled vanilla `postHurtEnemy` pins y to 0.01 regardless of ground state, which is what the code does; the suggested on-ground move was not applied |
| `crates/pumpkin-core/src/entity/combat.rs:108-112` | `MaceItem.knockback` pushes nearby entities with `push()`, which ignores knockback resistance | drop the `(1.0 - resistance)` factor if the decompiled source uses `push` | S, UNVERIFIED: `mace_smash_knockback_power` already scales power by `(1 - resistance)`, matching vanilla `getKnockbackPower` as recalled; the push path was not re-audited, so no change |
| `crates/pumpkin-core/src/entity/player.rs:1787-1862` | smash rules are behavioural and testable | add a `pumpkin-gametest`: drop a player ~6 blocks onto a cow, assert base+bonus damage, knockback and that the attacker takes no fall damage afterwards | M, BLOCKED-DATA: requested pumpkin-gametest needs a player-attack + entity-observation harness absent from pumpkin-gametest GameTestWorld; behavior VERIFIED against MaceItem.java and already covered by pure-unit seams in combat.rs |

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
