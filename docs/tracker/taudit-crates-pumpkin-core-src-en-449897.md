# entity/mob/witch — deep audit

Source: crates/pumpkin-core/src/entity/mob/witch.rs.
Compared against vanilla `Witch` (goal registration, performRangedAttack, aiStep drinking branch, getDamageAfterMagicAbsorb, setDrinkingPotion), `Raider`/`PatrollingMonster` base goals, and Paper (no extra witch behaviour).

## Fixed in this audit
- Missing villager and iron golem targets: vanilla registers `NearestAttackableTargetGoal(AbstractVillager, false)` and `(IronGolem, true)` at priority 3; the file only had the player hunt. Added both via `Hunt` (witch.rs:68-72).
- Target clearing on a raider hit: vanilla `setTarget(null)` runs for every raider target (healing and regeneration), the file only cleared it in the low-health branch, so witches kept throwing regeneration potions at raiders at full health. Moved the clear outside the branch (witch.rs:219-229).
- Drinking duration: vanilla `aiStep` adds `duration / 8` for each potion effect (instant effects keep full amplifier potency). The code applied the full potion duration, making every self-drink 8x too long (Fire Resistance 3600t instead of 450t). Now builds the effects per vanilla, instant effects via `apply_instant_effect` with potency 1.0 (witch.rs:342-363).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | witch.rs:251 | `performRangedAttack` shoots with a constant 0.75 speed / 8.0 inaccuracy | Pumpkin uses 0.45 speed below 2 blocks. Cannot verify against decompiled source here; if vanilla really is constant, close-range potions undershoot. Confirm against `Witch.java` before changing. | S |
| 2 | witch.rs:75-78 | `Raider.registerGoals` (super call, runs first) registers HurtByTarget at 1 and Player at 2; `NearestHealableRaiderTargetGoal` would then also be 2, inserted after the player goal | Pumpkin registers the healable-raider hunt at 2 but the player hunt at 3, so injured raiders are searched a tick class earlier and insertion order differs. Aligning requires knowing the base-class registration order used by the Raider port; low impact (target choice only). | S |
| 3 | witch.rs:473 | `PatrollingMonster.canBeLeader()` defaults to true in vanilla | Witch reports `can_be_leader() == false`, so a witch can never receive the ominous banner as a patrol leader. Verify against vanilla `Witch`/`PatrollingMonster` (banner may simply be invisible on the witch model) before changing. | S |
| 4 | witch.rs:431-441 | `aiStep` calls `super.aiStep()` last, after the drink/hat-magic branch | Pumpkin runs `tick_drinking` before the base mob tick. Equivalent unless the base tick cancels the tick (death) mid-way; no observable difference found, listed for completeness. | XS |
| 5 | no test | Drink math (`duration / 8`) is unit-testable only through a `LivingEntity` | No automated coverage pins the witch drink scaling or the raider target clear. A gametest spawning a witch, forcing `start_drinking`, and asserting the effect duration would lock fix 3. | S |

Checked, no change: `getDamageAfterMagicAbsorb` 0.15 multiplier (85% resistance), `setDrinkingPotion` transient -0.25 `ADD_VALUE` speed modifier and `DATA_USING_ITEM`, potion choice order/chances (water breathing 0.15 eye-in-fluid, fire resistance 0.15 on fire, healing 0.05, swiftness 0.5 at >11 blocks), `SPLASH_POTION` velocity formula `d, e + g*0.2, f`, drink/throw sounds with 0.8-1.2 pitch, hat-magic entity status at 7.5e-4, hurt/death/celebrate sounds, raider NBT (no extra saved fields).
