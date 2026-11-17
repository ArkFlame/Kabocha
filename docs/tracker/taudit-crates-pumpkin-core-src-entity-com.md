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
