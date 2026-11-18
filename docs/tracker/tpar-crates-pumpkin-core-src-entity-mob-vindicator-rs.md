# Parity rows for crates/pumpkin-core/src/entity/mob/vindicator.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Vindicator** [partial/COVERED-DONE] Raid movement, melee combat, revenge targeting, and creaking avoidance are implemented. Johnny-named all-mob targeting remains missing. Raid movement, melee combat, revenge targeting, and creaking avoidance are implemented.

## Result
entities/Vindicator: VERIFIED. Johnny-named all-mob targeting is implemented in vindicator.rs (VINDICATOR_JOHNNY_HUNTS, gated by is_johnny) and now matches decompiled vanilla .vendor/vanilla-decompiled/net/minecraft/world/entity/monster/illager/Vindicator.java, where VindicatorJohnnyAttackGoal (lines 207-222) uses predicate `target.attackable()` (false only for ArmorStand per LivingEntity.java:4713, already excluded by TargetMatch::Mob because ArmorStand.mob == false). Removed the incorrect creeper/ghast candidate filter. Raid movement, melee combat, revenge targeting and creaking avoidance remain implemented. No BLOCKED-DATA.
