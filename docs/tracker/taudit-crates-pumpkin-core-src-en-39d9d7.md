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

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `combat.rs:102-146` (`mace_smash_knockback`) | `MaceItem.knockback` pushes each victim with `Entity.push(x, 0.7, z, attacker)` | Uses `set_velocity`/`add_velocity`, which skip `push`'s `hurt_marked`/`has_impulse` bookkeeping, and only nearby players get an explicit motion packet | Route through the same push path as `Entity::push` once it takes an attacker, keep the manual player packet | S |
| `combat.rs:84-88` (`can_smash_attack`) | `MaceItem.canSmashAttack` is `fallDistance > 1.5 && !isFallFlying()` | Adds `!on_ground` | Intentional: Pumpkin's fall distance is client-driven, so a ground-only movement packet would otherwise keep a stale smash; document, do not change | - |
| `combat.rs:437-497` (`record_damage`) | `CombatTracker.recordDamage` uses `source.getEntity()` and fires `mob.onEnterCombat()`; `resetCombatState` fires `onLeaveCombat()` | Uses `cause.or(source)`, and the tracker has no mob handle so no combat enter/leave hooks | Pass the owning entity into `CombatTracker` (or return a transition flag) and fire the hooks at the existing call sites | M |
| `combat.rs:548-566` (`get_killer_entry`) | No vanilla method; Paper's combat tracker API exposes entries | Custom heuristic (`player damage >= living damage / 3`) used only as kill-credit fallback | No parity requirement; leave unless a bug is reported | - |
| `combat.rs:731-745` (`get_message_for_assisted_fall`) | `CombatTracker.getMessageForAssistedFall` reads the entity's main hand at death | Uses the custom item name snapshotted when the damage was recorded, so an item swap before death shows a stale name | Read the attacker's current main hand at message time | S |
| `crates/pumpkin-core/src/entity/player.rs:1903-1936` (sweep damage) | `Player.doSweepAttack` uses `1 + SWEEPING_DAMAGE_RATIO * damage`, multiplies by attack strength, scans `victim.getBoundingBox().inflate(1.0, 0.25, 1.0)`, skips allies/marker armor stands/anything past 3 blocks, then knocks back and runs post-attack enchantment effects per victim | Uses the pre-1.21.2 `SWEEPING_EDGE` enchantment ratio, no attack-strength factor, no ally/marker/distance checks, no knockback or post-attack effects, and a pos-built box | Port `doSweepAttack` wholesale once the sweep damage source model is confirmed for V_26_3 | M |
| `crates/pumpkin-core/src/entity/player.rs:1891-1950` | `Player.causeExtraKnockback` also pushes non-living targets (`push(-sin * strength, 0.1, cos * strength)`) and always decays the attacker's velocity | Knockback/causeExtraKnockback runs only for living victims | Apply the non-living push branch for boats/minecarts and other non-living attackables | S |
| `combat.rs:42` (`AttackType::new`) | Vanilla has no platform condition | `is_bedrock` skips sweeping | Kept: Bedrock clients cannot sweep; revisit only if the Bedrock attack path changes | - |

## Audit round 2 (2026-11-17)

Vanilla compared against `world/damagesource/CombatTracker.java`,
`world/damagesource/DamageSource.java` and
`world/entity/player/Player.causeExtraKnockback`.

Fixed in this pass:

- `combat.rs:221-224` - `handle_knockback` decayed the attacker's velocity by
  `multiply(0.6, 1.0, 0.6)` even when the knockback strength was zero. Vanilla
  `Player.causeExtraKnockback` keeps both the decay and the sprint interruption inside
  `if (strength > 0.0F)`.
- `combat.rs:597-602` - a `FallVariants` death with no significant fall used
  `death.fell.accident.<loc>` (`generic` by default). Vanilla only takes the fall-variant
  branch when `getMostSignificantFall() != null` and otherwise calls
  `DamageSource.getLocalizedDeathMessage`, so a short fall is `death.attack.fall`
  (`...fall.player` with kill credit). Now falls through to the default death message.

Remaining gaps (round 2):

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `combat.rs:231-243` (`sync_hit_player_velocity`) | `Player.causeExtraKnockback` builds `PlayerVelocityEvent` from `currentMovement`, the victim's movement before the hit, and only sends the plugin-modified vector (else the post-hit one) | The event payload is the post-hit velocity | Pass `old_movement` to the event, send the modified vector when it changed, else the post-hit velocity | S |
| `combat.rs:148-192` (`mace_smash_can_knockback`) | `MaceItem.knockbackPredicate` needs only `getEntitiesOfClass(LivingEntity, ...)` | Reads entities from the index plus players; behaviour matches today, keep only if the index keeps living entities only | none while the spatial index is authoritative | - |
