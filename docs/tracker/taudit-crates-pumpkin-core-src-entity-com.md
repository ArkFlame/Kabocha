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
