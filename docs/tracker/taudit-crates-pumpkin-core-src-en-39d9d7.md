# audit / crates/pumpkin-core/src/entity/combat.rs (AUDIT)

Target: JavaMinecraftVersion::V_26_3. Reference: `.vendor` absent, compared against the
decompiled Paper/Folia tree at
`/run/media/linsaftw/SDD/GitHub/PaperMC/Folia/folia-server/src/minecraft/java/net/minecraft`
(`world/damagesource/CombatRules.java`, `CombatTracker.java`, `FallLocation.java`,
`world/item/MaceItem.java`, `world/entity/player/Player.java`,
`world/entity/LivingEntity.java`, `world/entity/Mob.java`).

Fixed in this pass:

- `combat.rs:300-317` - `CombatRules::get_damage_after_absorb` used `f32::clamp(min, max)`,
  which panics when `armor * 0.2 > 20` (datapack armor above 100). Vanilla `Mth.clamp`
  returns the max there.
- `combat.rs:245-260` - sweep particle sent a zero spread; vanilla
  `Player.doSweepAttack` sends the view direction as the particle spread (the client reads
  it as velocity for a single particle).
- `combat.rs:366-402` - `FallLocation` now mirrors `FallLocation.blockToFallLocation` /
  `getCurrentFallLocation`: a location exists only after a climb (`lastClimbablePos`) or in
  water; plain hits record `None`. Previously every hit recorded `Some(Generic)`, which made
  `get_most_significant_fall` pick an "assisted fall" entry where vanilla shows
  `death.fell.accident.generic`.
- `combat.rs:49-77` - `AttackType::new` now matches `Player.canCriticalAttack` (victim must
  be living, not climbing, not in water, not blind, not a passenger) and
  `Player.isSweepAttack` (grounded, `movement <= speed * 2.5`).
- `combat.rs:214-223` - `handle_knockback` now interrupts the attacker's sprint like
  `LivingEntity.causeExtraKnockback`.
- `combat.rs:129-147` - `mace_smash_knockback` used the horizontal distance for the 3.5
  radius test, the power falloff and the push direction. Vanilla `MaceItem.knockback` uses
  the full 3D offset (`distanceToSqr <= 12.25`, `(3.5 - offset.length()) * 0.7 * heavy *
  (1 - resistance)`, `vec3.normalize()`), so entities above or below the victim were pushed
  as if level with it.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `combat.rs:102-146` (`mace_smash_knockback`) | `MaceItem.knockback` pushes each victim with `Entity.push(x, 0.7, z, attacker)` | Uses `set_velocity`/`add_velocity`, which skip `push`'s `hurt_marked`/`has_impulse` bookkeeping, and only nearby players get an explicit motion packet | Route through the same push path as `Entity::push` once it takes an attacker, keep the manual player packet | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `combat.rs:84-88` (`can_smash_attack`) | `MaceItem.canSmashAttack` is `fallDistance > 1.5 && !isFallFlying()` | Adds `!on_ground` | Intentional: Pumpkin's fall distance is client-driven, so a ground-only movement packet would otherwise keep a stale smash; document, do not change | - |
| `combat.rs:437-497` (`record_damage`) | `CombatTracker.recordDamage` uses `source.getEntity()` and fires `mob.onEnterCombat()`; `resetCombatState` fires `onLeaveCombat()` | Uses `cause.or(source)`, and the tracker has no mob handle so no combat enter/leave hooks | Pass the owning entity into `CombatTracker` (or return a transition flag) and fire the hooks at the existing call sites | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `combat.rs:548-566` (`get_killer_entry`) | No vanilla method; Paper's combat tracker API exposes entries | Custom heuristic (`player damage >= living damage / 3`) used only as kill-credit fallback | No parity requirement; leave unless a bug is reported | - |
| `combat.rs:731-745` (`get_message_for_assisted_fall`) | `CombatTracker.getMessageForAssistedFall` reads the entity's main hand at death | Uses the custom item name snapshotted when the damage was recorded, so an item swap before death shows a stale name | Read the attacker's current main hand at message time | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `crates/pumpkin-core/src/entity/player.rs:1903-1936` (sweep damage) | `Player.doSweepAttack` uses `1 + SWEEPING_DAMAGE_RATIO * damage`, multiplies by attack strength, scans `victim.getBoundingBox().inflate(1.0, 0.25, 1.0)`, skips allies/marker armor stands/anything past 3 blocks, then knocks back and runs post-attack enchantment effects per victim | Uses the pre-1.21.2 `SWEEPING_EDGE` enchantment ratio, no attack-strength factor, no ally/marker/distance checks, no knockback or post-attack effects, and a pos-built box | Port `doSweepAttack` wholesale once the sweep damage source model is confirmed for V_26_3 | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `crates/pumpkin-core/src/entity/player.rs:1891-1950` | `Player.causeExtraKnockback` also pushes non-living targets (`push(-sin * strength, 0.1, cos * strength)`) and always decays the attacker's velocity | Knockback/causeExtraKnockback runs only for living victims | Apply the non-living push branch for boats/minecarts and other non-living attackables | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `combat.rs:42` (`AttackType::new`) | Vanilla has no platform condition | `is_bedrock` skips sweeping | Kept: Bedrock clients cannot sweep; revisit only if the Bedrock attack path changes | - |

## Gap status

- STILL OPEN: all combat.rs/player.rs rows (mace push, can_smash_attack !on_ground intentional, record_damage hooks, sweep damage, causeExtraKnockback, bedrock sweep kept). Not re-verified in h3 sweep.

## Result

- `combat.rs` `mace_smash_knockback` (S) - FIXED. The impulse now goes through
  `Entity::push_by`, which marks velocity dirty and keeps Bedrock's predicted push in step,
  and a motion packet is still sent to each pushed player, mirroring vanilla
  `MaceItem.knockback` (`push(x, 0.7, z, attacker)` + `ClientboundSetEntityMotionPacket`).
  Added `non_living_knockback_impulse` helper plus a pure unit test for it.
- `combat.rs` `record_damage` combat enter/leave hooks (M) - OPEN. Firing
  `mob.onEnterCombat()`/`onLeaveCombat()` needs the owning entity passed into the tracker
  and the hooks fired at the call sites in `entity/living.rs`, which is outside this unit.
- `combat.rs` `get_message_for_assisted_fall` live main-hand item (S) - OPEN. Reading the
  attacker's current main hand at message time requires passing the attacker entity into
  `CombatTracker::get_death_message` from the `entity/living.rs` call site, outside this unit.
- `player.rs` sweep damage (M) - OPEN. Vanilla V_26_3 `doSweepAttack` needs the sweep damage
  source/model; no sweep damage type exists in `pumpkin-data` generated data, so a faithful
  port needs generated data this unit cannot add.
- `player.rs` `causeExtraKnockback` non-living push (S) - FIXED. Non-living victims now get the
  raw push with vanilla's opposite horizontal sign and 0.1 lift, the attacker stops sprinting,
  and its velocity decays by 0.6. Verified against decompiled `Player.causeExtraKnockback`.
- `player.rs` sweep ally/marker/distance checks, knockback and box - already match vanilla in
  the current checkout; no change needed.
