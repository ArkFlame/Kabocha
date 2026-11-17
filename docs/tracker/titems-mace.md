# items / Mace (RE-REVIEW)

Reviewed: `crates/pumpkin-core/src/item/items/mace.rs`, `crates/pumpkin-core/src/entity/combat.rs:61-170`, `crates/pumpkin-core/src/entity/player.rs:1787-1862`, `crates/pumpkin-core/src/entity/player/client_move.rs:375-387`, `crates/pumpkin-core/src/entity/living.rs:337-387,2730-2755`, data `crates/pumpkin-data/src/generated/item.rs:64050`.

The player smash attack matches vanilla `MaceItem`: damage bonus curve (`4*f`, `12+2*(f-3)`, `22+(f-8)`), eligibility, heavy/ground/air sound choice, impact position + post-impulse fall-damage immunity, knockback with radius 3.5 / power 0.7 / x2 heavy, smash particles world event 750, durability cost from the `Weapon` component, fall distance reset after a living smash, Density (`smash_damage_per_fallen_block`), Breach (armor effectiveness), Wind Burst gating in `EnchantmentHelper::on_post_attack`, and the overkill advancement. The item's `Tool` component (`can_destroy_blocks_in_creative: false`) already covers the creative mining restriction, so `MaceItem::can_mine` is redundant but harmless. No small certain fix was needed this pass.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| `crates/pumpkin-core/src/entity/mob/mod.rs:1139` | `Mob.doHurtTarget` runs the weapon's `getAttackDamageBonus`/`postHurtEnemy`, so a mob wielding a mace performs the smash (bonus, knockback, fall reset) | move the smash block out of `Player::attack` into a shared `combat::apply_smash_attack` used by `Mob::try_attack` too | M |
| `crates/pumpkin-core/src/entity/player.rs:1787` | `MaceItem` is identified by the item class/behaviour, not a hardcoded item id, so component-overridden/datapack maces still smash | gate on the mace `ItemBehaviour` instead of `Item::MACE.id` | S |
| `crates/pumpkin-core/src/entity/combat.rs:62` | `MaceItem.canSmashAttack` is `fallDistance > 1.5 && !isFallFlying`; Pumpkin also requires `!on_ground`, which can reject a smash in the landing tick | confirm against the 26.3 `MaceItem` source and drop the extra on-ground gate if absent | S (verify) |
| `crates/pumpkin-core/src/entity/player.rs:1837` | attacker velocity is pinned to y=0.01 only when the victim is on the ground (an air smash keeps the attacker falling) | move `set_velocity` into the `victim.on_ground` branch already used for the sound choice | S (verify) |
| `crates/pumpkin-core/src/entity/combat.rs:108-112` | `MaceItem.knockback` pushes nearby entities with `push()`, which ignores knockback resistance | drop the `(1.0 - resistance)` factor if the decompiled source uses `push` | S (verify) |
| `crates/pumpkin-core/src/entity/player.rs:1787-1862` | smash rules are behavioural and testable | add a `pumpkin-gametest`: drop a player ~6 blocks onto a cow, assert base+bonus damage, knockback and that the attacker takes no fall damage afterwards | M |
