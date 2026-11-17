# items / Mace (RE-REVIEW after source change)

Re-reviewed: `crates/pumpkin-core/src/item/items/mace.rs`, `crates/pumpkin-core/src/entity/combat.rs:61-170`, `crates/pumpkin-core/src/entity/player.rs:1775-1852`, `crates/pumpkin-core/src/entity/living.rs:383-389`, `crates/pumpkin-core/src/enchantment/helper.rs:115-155`, `crates/pumpkin-core/src/entity/mob/mod.rs:1139-1160,2287-2289`, data `crates/pumpkin-data/src/generated/item.rs:64050-64158`.

Why re-review: commit 55b85ffa4 ("entity motion, forced chunks, attributes and combat knockback") landed after the previous review (a1d8fecb4) and rewrote `Player::attack`'s damage/knockback plumbing: Strength/Weakness are now ATTACK_DAMAGE attribute modifiers folded into `base_damage` (player.rs:1766-1770), enchantment knockback is `f64` and uncapped (player.rs:1694,1721), and extra knockback/hit velocity moved. The mace smash block itself was not edited, but its line anchors moved (1787 -> 1775).

Verified unchanged and matching vanilla `MaceItem`:
- Smash gate `fallDistance > 1.5 && !fall_flying && !on_ground` (combat.rs:62-66); the on-ground clause is deliberate (comment: stale fall distance after a ground-only packet) and mirrors the client-authoritative movement design, see gap 3.
- Damage bonus curve `4*f`, `12+2*(f-3)`, `22+(f-8)` (combat.rs:69-77), Density added per fallen block before the crit multiplier (player.rs:1778-1792). Strength/Weakness no longer double-apply because the bonus is added after the attribute-derived base damage.
- `MACE_SMASH` damage type whenever the smash gate is true (player.rs:1798), overkill advancement >= 100 (player.rs:1817).
- Post-hit effects: y velocity pinned to 0.01, impact position + post-impulse fall-damage immunity, ground/heavy/air sound, extra landing particles flag, smash knockback, Wind Burst after these, fall distance reset last (player.rs:1821-1850, enchantment/helper.rs:132-153, living.rs:383-389).
- Knockback: world-event 750 particles, radius 3.5, power 0.7, x2 when fall > 5, y 0.7, allied/spectator/marker/tamed-by-victim filtering, creative-flying filter (combat.rs:80-170).
- Data: 5 attack damage + 1 base = 6, attack speed -3.4, 500 durability, Weapon x1, Tool `can_destroy_blocks_in_creative: false`, Enchantable 15, Repairable = breeze rod (item.rs:64050-64158), so `MaceItem::can_mine` (mace.rs:14-16) is redundant but harmless.
- `combat_weapon_durability_cost` (player.rs:1995-2000) charges 1 durability per hit including smash hits.

Dropped from the previous review, not a real gap:
- "gate the smash on the mace behaviour instead of `Item::MACE.id`" (player.rs:1775): Pumpkin keys item behaviour by id everywhere (`MaceItem::ids` returns `Item::MACE.id`), so the id test is the idiomatic equivalent of `instanceof MaceItem`; datapack items cannot swap the behaviour.
- "attacker velocity pinned to y=0.01 only when the victim is on the ground": the code already applies it for every living victim, before the ground/air sound branch (player.rs:1825), matching `MaceItem.postHurtEnemy`.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| `crates/pumpkin-core/src/entity/mob/mod.rs:1139-1160` | `Mob.doHurtTarget` calls `EnchantmentHelper.modifyDamage`, the weapon's `getAttackDamageBonus` and `ItemStack.postHurtEnemy`, so a mob holding a mace performs the smash (bonus damage, MACE_SMASH type, knockback, fall-damage immunity, fall reset) and hatchet/mace durability is charged | extract the smash block from `Player::attack` into `combat::apply_smash_attack`/`apply_post_hurt_weapon` and call it from `Mob::try_attack` too | M |
| `crates/pumpkin-core/src/entity/mob/mod.rs:1139-1160` | `Mob.doHurtTarget` also runs post-attack enchantment effects (Fire Aspect, Wind Burst, Breach) held by mobs | call `EnchantmentHelper::on_post_attack` from the mob path once the shared helper exists | S |
| `crates/pumpkin-core/src/entity/combat.rs:62-66` | `MaceItem.canSmashAttack` is `fallDistance > 1.5 && !isFallFlying`; Pumpkin adds `!on_ground` to reject stale fall distance from a ground-only move packet | confirm against the 26.3 decompiled `MaceItem`; if the extra clause has no vanilla counterpart keep it but document the deviation in one comment | S (verify) |
| `crates/pumpkin-core/src/entity/combat.rs:108-112` | `MaceItem.knockback` (26.3) either uses `Entity.push`, which ignores knockback resistance, or a fixed 0.7 strength; Pumpkin applies `(3.5 - distance)` falloff and `(1 - resistance)` to the horizontal component | compare with the decompiled method: drop the `3.5 - distance` falloff and/or the resistance factor if absent, keeping y at 0.7 | S (verify) |
| `crates/pumpkin-core/src/entity/player.rs:1821-1850` | smash outcome is behavioural | add a `pumpkin-gametest`: drop a player ~6 blocks onto a cow, assert base+bonus damage, MACE_SMASH type, knockback on a bystander, no fall damage after landing, and fall distance reset | M |
