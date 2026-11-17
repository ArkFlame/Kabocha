# tverify-mobs7: Bogged, Breeze, Creaking, Drowned, Enderman

Audit against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster`.

## FIXED
- Drowned revenge (`crates/pumpkin-core/src/entity/mob/zombie/drowned.rs`, target goal 1): vanilla
  `HurtByTargetGoal(this, Drowned.class).setAlertOthers(ZombifiedPiglin.class)` alerts only zombified
  piglins. Rust used the zombie helper, which alerted every zombie-family mob except zombified piglins.
  Now `RevengeGoal::new(true).ignoring(DROWNED).alerting_others().alerting_only(ZOMBIFIED_PIGLIN)`.
  Test: none added. Needs a world-level gametest or the existing mob test harness; no pure seam exists.
  Check: UNVERIFIED (cargo not run per brief).

## STILL OPEN
- Drowned/Enderman/Creaking/Breeze/Bogged, goal priorities and targets: match vanilla for Drowned
  (goals 1,2,5,6,7; targets 1,2,3,5) and Enderman (goals 0,1,2,7,8,10,11; targets 1,2,3,4).
  Drowned villager target at priority 3 is a Spigot `zombieAggressiveTowardsVillager` option in vanilla
  source; kept as Pumpkin's default, not changed.
- Breeze: vanilla goals come from `BreezeAi` (Brain-based), not a `goalSelector`. Rust approximates with
  goals 0/5/6/7 and targets 1/2. Not audited against the Brain. Blocked on a full Brain audit.
- Creaking: vanilla goals and targets live in `CreakingAi` (Brain). Rust uses goals 0/4/5/6/7 and target 1.
  Not audited against the Brain. Attributes (health 1, damage 3, fight speed 0.4) match vanilla.
- Creaking hurt sound: vanilla returns `CREAKING_SWAY` when heart-bound, otherwise the super hurt sound.
  Rust plays SWAY explicitly in the heart-bound damage path and has no `hurt_sound` override. Equivalence
  not verified against the Monster default. Open.
- Bogged: inherits skeleton goals. Sounds (ambient, hurt, death, step, shear) and max health 16 match
  vanilla. Skeleton goal set not re-audited in this pass.
- Enderman: goals, targets (Endermite hunt at 3, must-see), attributes and ambient/scream/hurt/death/teleport
  sounds match vanilla. No divergence found. Stare-sound interval 400 matches `playStareSound`.
- Damage values: Enderman 7, Breeze 3, Creaking 3, Drowned via Zombie attributes. Not re-verified against
  pumpkin-data defaults.
