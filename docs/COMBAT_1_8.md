# 1.8 combat mode: design note

**DESIGN ONLY. Nothing here is implemented and there is no code in this commit.** The tracker row `1-8-combat-mode` is DESIGN, not a gap; treat this file as the spec a worker gets when someone decides to build it.

Scope: an opt-in server mode that makes player melee behave like Minecraft 1.8 - no attack cooldown, and sword blocking instead of a shield. Everything below assumes a config switch, not a protocol change: modern clients still connect, the server just answers differently.

## 1. What 1.21 combat does today, and where it lives

| Behaviour | Vanilla 1.21 | Pumpkin owner |
|---|---|---|
| Player melee entry point | `Player#attack(Entity)` | `crates/pumpkin-core/src/entity/player.rs:1657` |
| Attack strength scale | `Player#getAttackStrengthScale(float)`, `attackStrengthTicker / (1.0F / attackSpeed * 20.0F)` | `crates/pumpkin-core/src/entity/player.rs:4085` (`get_attack_cooldown_progress`), tick counter reset at `:1756` |
| Attack speed source | held item `minecraft:base_attack_speed` modifier, base 4.0 empty-handed | `crates/pumpkin-core/src/entity/player.rs:1696-1733` |
| Damage scaling by charge | `0.2 + progress^2 * 0.8` multiplier, enchantment bonus scaled by progress | `crates/pumpkin-core/src/entity/player.rs:1766-1774` |
| Minimum attack charge gate | `minecraft:minimum_attack_charge` (spear/mace family) | `crates/pumpkin-core/src/entity/player.rs:1736-1750` |
| Bedrock shortcut | n/a (Bedrock always full charge) | `crates/pumpkin-core/src/entity/player.rs:1751-1755` |
| Critical / sweep / knockback classification | `Player#canCriticalAttack`, `Player#isSweepAttack` | `crates/pumpkin-core/src/entity/combat.rs:33-80`; applied at `crates/pumpkin-core/src/entity/player.rs:1801-1803` (crit), `crates/pumpkin-core/src/entity/player.rs:1893-1944` (sweep) |
| Shield use start | `ItemStack#use` on a `blocks_attacks` item | `crates/pumpkin-core/src/item/items/shield.rs:16-33` (72000 ticks), generic path `crates/pumpkin-core/src/net/java/play/use_item.rs:86-117` |
| Use dispatch from the wire | `ServerboundUseItemPacket` -> `ItemStack#use` | `crates/pumpkin-core/src/net/java/play/use_item.rs:6-84`, call at `:81` |
| "Am I blocking" | `LivingEntity#isBlocking` | `crates/pumpkin-core/src/entity/living.rs:1217-1244` (`BlocksAttacksImpl` on the item in use, `block_delay_ticks`) |
| Incoming damage blocked | `LivingEntity#blockedByShield` / `applyItemBlocking` | `crates/pumpkin-core/src/entity/living.rs:3424` (body `:3433-3599`), called at `:4316` |
| Mob melee passes attacker position so the shield angle check runs | `Mob#doHurtTarget` | `crates/pumpkin-core/src/entity/mob/mod.rs:1151-1160` |

## 2. Planned deltas under the mode

### 2.1 Attack cooldown removed

- Force `attack_cooldown_progress = 1.0` for players in the mode, in one place, and feed that same value to `AttackType::new` (`crates/pumpkin-core/src/entity/player.rs:1780-1784`). Every consequence is then automatic: multiplier `1.0`, no enchant scaling loss, `is_strong` true.
- Skip the `minimum_attack_charge` gate (`crates/pumpkin-core/src/entity/player.rs:1736-1750`); 1.8 had no such component.
- Do not fake 1.8 client feel by sending cooldown packets: the 1.8 client has no attack indicator, and the modern client's indicator is client-side prediction we cannot suppress. Document that a modern client will still show its own cooldown bar; the damage is full regardless.
- `AttackType::Sweeping` must be unreachable in this mode (`crates/pumpkin-core/src/entity/combat.rs:70-77`): sweeping arrived in 1.9. Crit (`crates/pumpkin-core/src/entity/combat.rs:50-63`, `crates/pumpkin-core/src/entity/player.rs:1801-1803`) and sprint knockback (`crates/pumpkin-core/src/entity/combat.rs:45-47`) stay.
- `get_attack_cooldown_progress` itself stays untouched: spear/mace/other callers outside the mode keep 1.21 semantics.

### 2.2 Sword blocking

- `SwordItem` (`crates/pumpkin-core/src/item/items/swords.rs:14-22`) gains a `normal_use`/`normal_use_with_rotation` that starts an active hand exactly like `crates/pumpkin-core/src/item/items/shield.rs:24-29` does, and a `get_use_duration()`. The item-use plumbing already calls it: `crates/pumpkin-core/src/net/java/play/use_item.rs:81`.
- Blocking must not require a `BlocksAttacksImpl` component, because a plain 1.8 sword has none and `LivingEntity#isBlocking` (`crates/pumpkin-core/src/entity/living.rs:1217-1244`) currently gates on it. The mode needs a blocking-state predicate that accepts "mode on + sword in the active hand", evaluated before the component check.
- No block delay: the mode must report `required_time == 0` (compare the Bedrock branch at `crates/pumpkin-core/src/entity/living.rs:1226-1241`), because 1.8 blocking engages instantly rather than after 5 ticks.
- Damage reduction: 1.8 blocks roughly half of a melee hit and ignores it when the attack comes from behind. Both the fraction and the angle test must be taken from the decompiled 1.8 `EntityPlayer#damageEntity` / `EntityLivingBase#blockUsingShield` equivalent, not from memory - **UNVERIFIED until that citation exists**.
- Durability: 1.8 blocking does not consume shield durability. The mode must therefore bypass the `item_damage` / `damage_item_in_slot` branch (`crates/pumpkin-core/src/entity/living.rs:3509-3556`) and the `WeaponImpl#disable_blocking_for_seconds` branch (`crates/pumpkin-core/src/entity/living.rs:3558-3595`), both of which are 1.9+ concepts.
- Movement and item use: blocking in 1.8 slows the player and prevents other item use. Movement input is client-authoritative in Pumpkin (see AGENTS.md), so only the server-side item-use suppression and any speed attribute effect are ours to enforce.

## 3. Config and gating

A boolean in `PVPConfig` (`crates/pumpkin-config/src/pvp.rs:9`, wired at `crates/pumpkin-config/src/lib.rs:215-216`), default off. Read it through the `advanced_config.pvp` handle the attack path already uses (`crates/pumpkin-core/src/entity/player.rs:1664`). No game rule: 1.8 combat is a server policy, not a per-world toggle that datapacks can flip mid-fight.

## 4. Tests that must exist before this lands

1. Gametest or bot scenario: two players, iron sword, attacker hits on consecutive ticks. 1.21 mode => second hit deals reduced damage; 1.8 mode => identical damage every hit.
2. Gametest: right-click a sword in survival sets the blocking state, and an incoming melee hit from the front is reduced while one from behind is not.
3. Gametest: a blocked hit leaves sword durability unchanged in 1.8 mode, and changes it in 1.21 mode.
4. Regression: `AttackType` in 1.8 mode never yields `Sweeping`, and still yields `Critical` for a falling sprint-less hit.

## 5. Non-goals

- No 1.8 knockback/velocity retuning, no 1.8 golden-apple or potion changes, no 1.8 reach, no projectile rework.
- No support for real 1.8 clients: that needs ViaVersion-style translation, which is a plugin concern.
- No per-item "1.8 sword" variant items; the mode is global.

## 6. Open questions (resolve against decompiled 1.8 before coding)

- Exact 1.8 blocking formula and whether the reduction is applied before or after armour.
- Whether 1.8 required the attacker to be in front for melee, or only for projectiles/explosions.
- Whether 1.8 blocking should stop when the sword leaves the hand mid-block, and what the client predicts.
