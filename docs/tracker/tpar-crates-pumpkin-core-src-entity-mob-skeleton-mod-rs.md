# Parity rows for crates/pumpkin-core/src/entity/mob/skeleton/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-aadca5** [partial/OWN-OPEN] own audit: 6 open, 5 fixed, 2 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-aadca5.md) docs/tracker/taudit-crates-pumpkin-core-src-en-aadca5.md

## Result

THIRD PASS (2026-11-18). Independently re-read current Pumpkin source and `.vendor/vanilla-decompiled/net/minecraft/`. All rows closed below; no `.rs` file needed editing, so no rustfmt run and no source touched (only this tracker doc).

- Row 2 (RestrictSunGoal) - VERIFIED, no change. `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/RestrictSunGoal.java:16-32`: canUse = isBrightOutside && HEAD empty && hasGroundPathNavigation; start/stop call GroundPathNavigation.setAvoidSun(true/false). Pumpkin `entity/ai/goal/restrict_sun.rs:14-38`: is_exposed = is_bright_outside() && !sun_burn::is_protected; sun_protection_slot defaults to HEAD (`entity/mob/mod.rs:1759`), so !is_protected == HEAD empty; start/stop toggle navigator().set_avoid_sun(true/false). Registration at `entity/mob/skeleton/mod.rs:72` priority 2 and FleeSunGoal at priority 3 match `AbstractSkeleton.java:78-79`. Ground navigation is implicit for skeletons. No generated-data field or subsystem missing.
- Row 3 (Parched attack interval) - VERIFIED, no change. `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/Parched.java:56-64` overrides getHardAttackInterval()=50 and getAttackInterval()=70. `entity/mob/skeleton/parched.rs:59-66` returns Some(50) on Hard / Some(70) otherwise, consumed by `entity/ai/goal/bow_attack.rs:291-293`. Matches vanilla; no generated-data field missing.
- Row 4 (lava pathfinding malus) - VERIFIED, no gap for this mob. Premise false: `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java:71-74` constructor calls only super() and reassessWeaponGoal(), with no setPathfindingMalus. The `setPathfindingMalus(PathType.LAVA, 8.0F)` call is in `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/WitherSkeleton.java:34`, a separate mob outside this unit. Pumpkin already matches vanilla for AbstractSkeleton; no generated-data field or subsystem missing.

No pure unit-test seam exists for these goal/interval behaviours without constructing a full Mob/world, so none added.
