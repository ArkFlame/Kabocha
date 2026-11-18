# Audit: crates/pumpkin-core/src/entity/combat.rs

Audited 2026-11-16. The file has no TODO/FIXME markers; the tracker's 5 matches are "vanilla"/"Vanilla" parity comments.

## Fixed
- combat.rs:49 crit now requires not climbing, not in water, no blindness, not riding.
- combat.rs:57 sweep now requires the attacker to be on the ground.
- combat.rs:107 mace smash knockback uses the horizontal 3.5 block radius like `MaceItem.knockback`.
- combat.rs:230 sweep particle carries the vanilla direction delta.
- combat.rs:281 armor clamp no longer panics when armor exceeds 100.
- player.rs:1864 attack sounds play at the attacker position (`Player.attack`).

## Remaining gaps
| file:line | vanilla class | suggested fix | size |
|---|---|---|---|
| combat.rs:57 | `Player.attack` sweep also needs `walkDist - walkDistO < getSpeed()` | add `walk_dist`/`walk_dist_o` to `Entity`, update in `move_entity`, gate sweep | S |
| combat.rs:471,486 | `CombatTracker.recordDamage`/`recheckStatus` broadcast entity event 128 on combat start/end (client FX) | add world param and broadcast event 128 | S |
| combat.rs:189 | `LivingEntity.knockback` applies resistance once for every path | verify `Mob::mob_knockback` does not scale resistance twice | M |
| combat.rs:714 | `CombatTracker.getDefaultDeathMessage` only uses the killing blow's attacker | verify `.player` fallback via `get_killer_entry` against 1.21 | M |
| combat.rs:140 | `MaceItem.knockback` compares owner entity identity, not UUID | compare entity ids when the owner is online | S |
| combat.rs:285 | 1.21 applies Breach through `EnchantmentHelper.modifyArmorEffectiveness` outside `CombatRules` | confirm no double clamp vs 1.21 `CombatRules.getDamageAfterAbsorb` | M |

## Gap status
- 1 sweep needs walk distance (combat.rs:57): STILL OPEN, S.
- 2 recordDamage/recheckStatus event 128 (combat.rs:471,486): STILL OPEN, S.
- 3 knockback resistance applied once (combat.rs:189): STILL OPEN, M, verify Mob::mob_knockback.
- 4 death message killer fallback (combat.rs:714): STILL OPEN, M.
- 5 mace owner identity (combat.rs:140): STILL OPEN, S.
- 6 Breach double clamp (combat.rs:285): STILL OPEN, M.
- Fixes listed in the note (crit/sweep/mace/armor clamp): kept, verified only by the note.

## Result

Re-audited 2026-11-17 against .vendor/vanilla-decompiled. Rows verified against current vanilla source, not the older multi-version assumptions in the note.

- 1 sweep walk distance (combat.rs:57): CLOSED, no change. Current Player.isSweepAttack uses getKnownMovement().horizontalDistanceSqr() < square(getSpeed() * 2.5); for a player that is deltaMovement/velocity and speed is MOVEMENT_SPEED. AttackType::new already checks velocity horizontal length squared < (MOVEMENT_SPEED * 2.5)^2 plus grounded/sword/strong. Adding walkDist fields is unnecessary; the existing check is equivalent.
- 2 combat event 128 (combat.rs:471,486): CLOSED, no change. Current CombatTracker.recordDamage/recheckStatus call mob.onEnterCombat()/onLeaveCombat(), which are empty in LivingEntity and unoverridden in Player. No entity event 128 broadcast exists in this vanilla; the note reflects an older version.
- 3 knockback resistance once (combat.rs:189): CLOSED, no change. Verified resistance is applied exactly once per path: handle_knockback applies it only on the apply_knockback fallback (mob overrides return handled); deal_default_knockback likewise; sulfur_cube.mob_knockback applies it internally once and returns true. apply_knockback itself does not reapply it.
- 4 death message killer fallback (combat.rs:714): CLOSED, no change. get_death_message passes kill_credit_name from LivingEntity.get_kill_credit() (vanilla's exact source). The extra get_killer_entry() fallback only runs when kill credit is absent and is a deliberate offline-killer enhancement, documented at living.rs:2868.
- 5 mace owner identity (combat.rs:140): CLOSED, no change. Vanilla TamableAnimal.isOwnedBy is getOwner() == entity (identity), but getOwner() itself resolves the same UUID; the victim is always loaded, so UUID equality is functionally identical. Exact identity would need a cross-cutting owner-entity resolution API outside combat.rs/player.rs.
- 6 Breach double clamp (combat.rs:285): FIXED. The old code seeded a multiplier at 1.0 and multiplied armor_fraction by it, but Breach's extracted effect is Add(Linear -0.15/-0.15): vanilla seeds modifyArmorEffectiveness with armorFraction itself and subtracts directly, then clamps once to [0,1]. Now apply the ADD effect to armor_fraction and clamp once. Added pure unit test breach_subtracts_from_armor_fraction_like_vanilla (level 4 -> 10.0 dmg, level 1 -> 9.5 dmg, both fail with the old multiplier).

No player.rs changes needed; no open rows remain. rustfmt --edition 2024 run on combat.rs.


## Result (re-verified 2026-11-18)

Independently re-checked all six STILL OPEN rows against .vendor/vanilla-decompiled. No new code changes were needed: current combat.rs already matches vanilla on every row.

- 1 sweep walk distance (combat.rs:57): CLOSED, no change. Player.isSweepAttack uses getKnownMovement().horizontalDistanceSqr() < square(getSpeed() * 2.5); Entity.getKnownMovement() returns getDeltaMovement() for an unmounted player, so Pumpkin's velocity check is equivalent. Adding walkDist fields would be a cross-crate refactor and is unnecessary.
- 2 combat event 128 (combat.rs:471,486): CLOSED, no change. CombatTracker.recordDamageAndCheckCombatState / recheckStatus only call mob.onEnterCombat() / onLeaveCombat(); no entity event 128 broadcast exists in this vanilla. Pumpkin's CombatStateChange models those hooks at the damage call site.
- 3 knockback resistance once (combat.rs:189): CLOSED, no change. LivingEntity.knockback scales power by (1 - KNOCKBACK_RESISTANCE) once. Pumpkin applies it only in handle_knockback's !handled fallback via knockback_after_resistance; mob_knockback handlers apply it internally once; Entity::apply_knockback does not reapply it.
- 4 death message killer fallback (combat.rs:714): CLOSED, no change. DamageSource.getLocalizedDeathMessage uses victim.getKillCredit() on the no-attacker path, which Pumpkin passes as kill_credit_name; the get_killer_entry() fallback only adds an offline-killer enhancement when kill credit is absent.
- 5 mace owner identity (combat.rs:140): CLOSED, no change. TamableAnimal.isOwnedBy is entity == getOwner(), and getOwner() resolves the owner UUID. The victim is always loaded, so UUID equality is functionally identical; exact identity needs an owner-resolution API outside combat.rs.
- 6 Breach double clamp (combat.rs:285): CLOSED, already fixed. CombatRules.getDamageAfterAbsorb seeds modifyArmorEffectiveness with armorFraction and clamps once; Pumpkin applies the ADD effect (base/per-level -0.15) to armor_fraction and clamps once. Existing test breach_subtracts_from_armor_fraction_like_vanilla covers level 4 (10.0) and level 1 (9.5), confirmed against the generated BREACH effect (Add Linear -0.15/-0.15).

No .rs files changed this pass, so no rustfmt was needed. No open rows remain.


## Result (re-verified 2026-11-18, third pass)

Re-checked every STILL OPEN row directly against the current decompiled vanilla in .vendor/vanilla-decompiled (1.21.x + Paper/Folia) and the live combat.rs. No code changes were required; all rows stay closed.

- 1 sweep walk distance (AttackType::new, combat.rs:65-77): CLOSED, no change. Player.isSweepAttack (Player.java:1134-1144) gates on getKnownMovement().horizontalDistanceSqr() < square(getSpeed() * 2.5) plus grounded/sword/full-strength. Pumpkin checks entity.velocity.horizontal_length_squared() < (MOVEMENT_SPEED * 2.5)^2 with the same grounded/sword/strong gates and the Bedrock exclusion. Movement is client-authoritative in Pumpkin, so velocity is the correct equivalent; adding walkDist fields would be a cross-crate refactor and is not needed.
- 2 combat event 128 (combat.rs:501,570): CLOSED, no change. CombatTracker.recordDamageAndCheckCombatState (CombatTracker.java:47-68) and resetCombatState (165-185) only call mob.onEnterCombat()/onLeaveCombat(); no entity event 128 broadcast exists in this vanilla. Pumpkin models those hooks via CombatStateChange at the damage call site.
- 3 knockback resistance once (combat.rs:236,252): CLOSED, no change. LivingEntity.knockback (LivingEntity.java:2145-2146) applies (1 - KNOCKBACK_RESISTANCE) exactly once. Pumpkin's handle_knockback applies it once via knockback_after_resistance on the !handled fallback; mob_knockback handlers apply it internally once; apply_knockback does not reapply it.
- 4 death message killer fallback (combat.rs:631,816): CLOSED, no change. DamageSource.getLocalizedDeathMessage (DamageSource.java:169-183) uses victim.getKillCredit() on the no-attacker path; Pumpkin passes that as kill_credit_name. The get_killer_entry() fallback only runs when kill credit is absent, an offline-killer enhancement, documented at living.rs.
- 5 mace owner identity (combat.rs:198-203): CLOSED, no change. MaceItem.knockback (MaceItem.java:126-153) checks animal.isOwnedBy(livingAttacker) where livingAttacker is the victim; TamableAnimal.isOwnedBy (TamableAnimal.java:188-190) is entity == getOwner(). Pumpkin compares the owner UUID to the victim UUID; both are loaded entities so this is functionally identical. Exact identity would need an owner-resolution API outside combat.rs.
- 6 Breach double clamp (combat.rs:351-376): CLOSED, already fixed. CombatRules.getDamageAfterAbsorb (CombatRules.java:16-32) seeds modifyArmorEffectiveness with armorFraction and clamps once; EnchantmentHelper.modifyArmorEffectiveness (198-206) starts MutableFloat at armorFraction, so Breach's ADD subtracts from the fraction. Pumpkin does the same. Existing test breach_subtracts_from_armor_fraction_like_vanilla covers level 4 (10.0 dmg) and level 1 (9.5 dmg).

No .rs files changed, so no rustfmt needed. No open rows remain; diff re-checked against the Worker compile pitfalls table with no changes to review.

Status: PASS fixed=0 open=0 files=none
