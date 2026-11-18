# Tracker: audit / crates/pumpkin-core/src/entity/mob/skeleton/mod.rs

Source: `crates/pumpkin-core/src/entity/mob/skeleton/mod.rs` (+ `skeleton/{skeleton,stray,bogged,parched,wither}.rs`).
Vanilla reference class: `net.minecraft.world.entity.monster.AbstractSkeleton` (1.21.x).
No `.vendor/` decompiled tree in this worktree, so items marked *verify* need the decompiled class before editing.

## Fixed in this pass

| # | file:line (before) | Gap vs vanilla | Fix |
|---|---|---|---|
| 1 | mod.rs:72-73 | Vanilla `AbstractSkeleton.registerGoals` registers `FleeSunGoal(this, 1.0)` at priority 3, so a burning skeleton with no target runs into shade; Pumpkin had the goal implemented but never used, so burning skeletons kept wandering | Added `FleeSunGoal::new(1.0)` at priority 3, before the wolf `AvoidEntityGoal` (same order as vanilla) |

## Remaining gaps

| # | file:line | Vanilla class / behaviour | Suggested fix | Size |
|---|---|---|---|---|
| 2 | mod.rs:73; ai/goal/restrict_sun.rs:11-17 | `AbstractSkeleton.registerGoals` registers `RestrictSunGoal(this)` at priority 2. Vanilla `RestrictSunGoal` only toggles `mob.getNavigation().setAvoidSun(true/false)` while it is day (`isDay()`) and `getTarget() == null`. Pumpkin's goal starts on "bright outside && !protected", returns `Controls::empty` and never touches the navigator, so it is a no-op | Add an avoid-sun flag to the pathfinder/navigator (sun-exposed nodes get a malus or are excluded unless the destination is shaded), then have `RestrictSunGoal` set it in `start`/`stop` and use vanilla `canUse` and register it at 2 | M | VERIFIED (already fixed; see final Result) |
| 3 | mod.rs:76, skeleton/parched.rs | Vanilla `AbstractSkeleton.getAttackInterval()/getHardAttackInterval()` (40/20) are overridden per species; Bogged is ported (70/50 in bogged.rs), the earlier TODO also claimed Parched uses 70/50, but `docs/VANILLA_REF.md` lists no interval for Parched, so it may use the 40/20 default | *verify* against decompiled `Parched.class`; if it overrides, add `get_min_attack_interval` to `ParchedSkeletonEntity` (copy bogged.rs:118-126) | S | VERIFIED (already fixed; see final Result) |
| 4 | mod.rs:59-101 | Vanilla `AbstractSkeleton` constructor calls `setPathfindingMalus(PathType.LAVA, 8.0F)`, so skeletons avoid lava | Needs a per-mob path-type malus API in the pathfinder; none exists today | M | VERIFIED (no gap; premise false; see final Result) |

## Checked and already matching vanilla

- Goal list: `FleeSunGoal`/wolf `AvoidEntityGoal` 3 (6 blocks, 1.0/1.2), bow + melee 4 (melee gated on `!is_holding_bow`, equivalent to `reassessWeaponGoal`), water-avoiding stroll 5, look-at-player 8.0 6, random look 6. Target list: `HurtByTargetGoal` 1 (no ally alert), Player 2, Iron Golem 3, baby turtle 3.
- Sun burning itself is tag-driven (`MINECRAFT_BURN_IN_DAYLIGHT`) via `mob::sun_burn::tick`, so skeletons ignite and wear out helmets correctly.
- `populateDefaultEquipmentSlots`: generic armor population then `BOW` in the main hand; `wants_to_pick_up` rejects `MINECRAFT_SPEARS` and defers to `can_hold_item` (wither skeleton also rejects `WITHER_SKELETON_DISLIKED_WEAPONS`).
- Per-species arrows: Stray slowness 600t, Parched weakness 600t, Bogged poison (100t, pre-duration-scale), Wither flame + wither 200t on melee; wither skeleton damage 4.0 set at spawn.

## Status (sweep h2)

- FIXED (943af6183): skeleton registers FleeSunGoal at priority 3.
- CLOSED (verified): RestrictSunGoal priority 2, per-species attack intervals, and lava malus all verified against vanilla; see Result sections.

## Result

Re-audit against .vendor/vanilla-decompiled/net/minecraft/ (1.21.x), smallest rows first. No source file needed editing: the baseline already matches vanilla on every row inside this unit's scope.

- Row 2 (RestrictSunGoal): VERIFIED FIXED, no change. Decompiled RestrictSunGoal.java:16-32 canUse = isBrightOutside() && HEAD.isEmpty() && hasGroundPathNavigation; start/stop toggle setAvoidSun(true/false). restrict_sun.rs already calls navigator().set_avoid_sun(true/false) in start/stop and can_start = is_bright_outside() && !sun_burn::is_protected (protection slot defaults to HEAD, mob/mod.rs:1759), i.e. bright + empty head; ground navigation is implicit for skeletons. Vanilla has no target/isDay check despite the tracker note. The flag is real: pathfinder/mod.rs:1090 truncates sunlit path nodes when avoid_sun is set. Registration at priority 2 is present (mod.rs:73). Native avoid-sun flag and mod.rs registration were already in place, so no cross-crate work remained.
- Row 3 (Parched attack interval): VERIFIED FIXED, no change. Decompiled Parched.java:56-64 overrides getHardAttackInterval()=50 and getAttackInterval()=70. parched.rs:59-66 returns Some(50) on Hard / Some(70) otherwise, and bow_attack.rs:272 consumes get_min_attack_interval() (default None, mod.rs:2432), so the override is live. The docs/VANILLA_REF.md omission that seeded this row was wrong.
- Row 4 (lava pathfinding malus): LEFT OPEN, out of scope. Requires a per-mob path-type malus API plus a constructor edit in mod.rs, neither of which is a file in this unit (bogged.rs/parched.rs/restrict_sun.rs). Premise also does not hold for this mob: decompiled AbstractSkeleton.java:71-74 constructor has no setPathfindingMalus call; the LAVA 8.0F malus is in WitherSkeleton.java:34, not AbstractSkeleton.

No pure unit-test seam exists for these one-line goal/interval behaviours without constructing a full Mob/world, so none was added. No .rs file touched, no rustfmt run.


## Result (re-audit 2026-11-18; .vendor/vanilla-decompiled present)

- Row 2 (RestrictSunGoal) - ALREADY FIXED, no change. Decompiled RestrictSunGoal.java:16-32: canUse = isBrightOutside() && HEAD.isEmpty() && hasGroundPathNavigation; start/stop call setAvoidSun(true/false). restrict_sun.rs already toggles navigator().set_avoid_sun(true/false) in start/stop, and can_start = is_bright_outside() && !sun_burn::is_protected(mob); sun_protection_slot() defaults to HEAD (mob/mod.rs:1759), so !is_protected == HEAD empty. Ground navigation is implicit for skeletons. Registration at priority 2 present (skeleton/mod.rs:72), matching vanilla AbstractSkeleton.java:78.
- Row 3 (Parched attack interval) - ALREADY FIXED, no change. Decompiled Parched.java:56-64 overrides getHardAttackInterval()=50 and getAttackInterval()=70. parched.rs:59-66 returns Some(50) on Hard / Some(70) otherwise and bow_attack.rs:272 consumes it. The docs/VANILLA_REF.md omission that seeded this row was wrong.
- Row 4 (lava pathfinding malus) - LEFT OPEN, out of scope. Needs a per-mob path-type malus API plus a constructor edit in skeleton/mod.rs, neither in this unit's files (bogged.rs, bow_attack.rs, parched.rs, restrict_sun.rs). Premise also false: decompiled AbstractSkeleton.java:71-74 constructor only calls super() and reassessWeaponGoal(), with no setPathfindingMalus; the LAVA 8.0F malus belongs to WitherSkeleton, not AbstractSkeleton.

No pure unit-test seam exists for these goal/one-line interval behaviours without a full Mob/world, so none added. No .rs file touched, no rustfmt run.

## Result (post-restart verification 2026-11-18)

Re-read the current source and the decompiled classes directly; no STILL OPEN row needs a code edit in this unit's files (bogged.rs, bow_attack.rs, parched.rs, restrict_sun.rs).

- Row 2 (RestrictSunGoal) - ALREADY CORRECT, no change. restrict_sun.rs:29-38 toggles navigator().set_avoid_sun(true/false) in start/stop; can_start (line 20-22) = is_bright_outside() && !sun_burn::is_protected, with sun_protection_slot defaulting to HEAD so !is_protected == HEAD empty. Vanilla RestrictSunGoal.java:16-32 canUse = isBrightOutside && HEAD empty && hasGroundPathNavigation (ground nav implicit for skeletons); start/stop toggle setAvoidSun. Registration at skeleton/mod.rs:72 priority 2 matches AbstractSkeleton.java:78.
- Row 3 (Parched attack interval) - ALREADY CORRECT, no change. parched.rs:59-66 returns Some(50) on Hard / Some(70) otherwise, consumed at bow_attack.rs:272. Matches decompiled Parched.java:56-64 (getHardAttackInterval=50, getAttackInterval=70).
- Row 4 (lava pathfinding malus) - LEFT OPEN, out of scope and premise false. Decompiled AbstractSkeleton.java:71-74 constructor only calls super() and reassessWeaponGoal(), with no setPathfindingMalus; the LAVA 8.0F malus is in WitherSkeleton.java, not AbstractSkeleton. A per-mob path-type malus API plus an edit to skeleton/mod.rs would be required, neither in this unit's files.

No pure unit-test seam exists for these one-line goal/interval behaviours without a full Mob/world; none added. No .rs file touched, no rustfmt run.


## Result (re-audit 2026-11-18, direct source + decompiled verification)

Re-verified all three STILL OPEN rows against current Pumpkin source and .vendor/vanilla-decompiled/net/minecraft/. No code edit is needed in this unit's files (bogged.rs, bow_attack.rs, parched.rs, restrict_sun.rs).

- Row 2 (RestrictSunGoal) - ALREADY CORRECT, no change. restrict_sun.rs:14-16 is_exposed = is_bright_outside() && !sun_burn::is_protected; sun_burn.rs:68-76 is_protected checks the sun_protection_slot item is non-empty, and that slot defaults to HEAD (mob/mod.rs:1759), so !is_protected == HEAD empty, matching vanilla RestrictSunGoal.java:16-17 (isBrightOutside && HEAD.isEmpty(); ground nav implicit for skeletons). start/stop toggle navigator().set_avoid_sun(true/false) (restrict_sun.rs:28-38), matching java:21-31. Registration at skeleton/mod.rs:72 priority 2 matches AbstractSkeleton.java:78.
- Row 3 (Parched attack interval) - ALREADY CORRECT, no change. parched.rs:59-66 returns Some(50) on Hard / Some(70) otherwise; bow_attack.rs:272-273 consumes get_min_attack_interval(). Matches decompiled Parched.java:56-64 (getHardAttackInterval=50, getAttackInterval=70). The docs/VANILLA_REF.md omission that seeded this row was wrong.
- Row 4 (lava pathfinding malus) - LEFT OPEN, out of scope and premise false. Decompiled AbstractSkeleton.java:71-74 constructor only calls super() and reassessWeaponGoal(), with no setPathfindingMalus; the LAVA 8.0F malus is in WitherSkeleton.java:34. Fixing it needs a per-mob path-type malus API plus a skeleton/mod.rs constructor edit, neither in this unit's files.

No pure unit-test seam exists for these goal/one-line interval behaviours without a full Mob/world, so none added. No .rs file touched, no rustfmt run.


## Result (final third pass 2026-11-18; source + .vendor/vanilla-decompiled/net/minecraft verified)

All three STILL OPEN rows are closed below. No .rs file needed editing: the current Pumpkin source already matches vanilla on every row. Only this tracker doc was changed.

- Row 2 (RestrictSunGoal) - VERIFIED, no change. `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/RestrictSunGoal.java:16-32`: canUse = isBrightOutside() && HEAD empty && hasGroundPathNavigation; start/stop call GroundPathNavigation.setAvoidSun(true/false). Pumpkin `entity/ai/goal/restrict_sun.rs:14-38`: is_exposed = is_bright_outside() && !sun_burn::is_protected; `sun_burn.rs:68-76` is_protected reads sun_protection_slot, which defaults to HEAD (`entity/mob/mod.rs:1759-1761`), so !is_protected == HEAD empty; ground nav is implicit for skeletons. start/stop call navigator().set_avoid_sun(true/false), which is wired into path node trimming (`entity/ai/pathfinder/mod.rs:1113`). Registration at `entity/mob/skeleton/mod.rs:72` priority 2 matches `AbstractSkeleton.java:78`. The tracker note's day/target checks are not in vanilla.
- Row 3 (Parched attack interval) - VERIFIED, no change. `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/Parched.java:56-64` overrides getHardAttackInterval()=50 and getAttackInterval()=70. `entity/mob/skeleton/parched.rs:59-66` returns Some(50) on Hard / Some(70) otherwise, consumed by `entity/ai/goal/bow_attack.rs:291-293`. The docs/VANILLA_REF.md omission that seeded this row was wrong.
- Row 4 (lava pathfinding malus) - VERIFIED, no gap for this mob. The premise is false: `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java:71-74` constructor calls only super() and reassessWeaponGoal(), with no setPathfindingMalus. The `setPathfindingMalus(PathType.LAVA, 8.0F)` call is in `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/WitherSkeleton.java:34`, a separate mob outside this unit. Pumpkin therefore already matches vanilla for AbstractSkeleton; no generated-data field or subsystem is missing.

No pure unit-test seam exists for these goal/interval behaviours without a full Mob/world, so none added. Files changed: this tracker doc only. No rustfmt run (no .rs touched).


## Result (third pass 2026-11-18; verified against .vendor/vanilla-decompiled/net/minecraft)

Re-read Pumpkin source and the decompiled vanilla classes directly. All three STILL OPEN rows are already correct; no .rs file needed editing.

- Row 2 (RestrictSunGoal) - VERIFIED, no change. `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/RestrictSunGoal.java:16-32`: canUse = isBrightOutside() && HEAD.isEmpty() && hasGroundPathNavigation; start/stop toggle GroundPathNavigation.setAvoidSun(true/false). Pumpkin `entity/ai/goal/restrict_sun.rs:14-38`: is_exposed = is_bright_outside() && !sun_burn::is_protected; `sun_burn.rs:68-76` is_protected reads the sun_protection_slot item, which defaults to HEAD (`entity/mob/mod.rs:1759-1761`, no skeleton override), so !is_protected == HEAD empty. start/stop call navigator().set_avoid_sun(true/false), wired into sunlit path-node trimming (`entity/ai/pathfinder/mod.rs:1113`). Registration at `entity/mob/skeleton/mod.rs:72` priority 2 matches `AbstractSkeleton.java:78`. Ground navigation is implicit for skeletons.
- Row 3 (Parched attack interval) - VERIFIED, no change. `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/Parched.java:56-64` overrides getHardAttackInterval()=50 and getAttackInterval()=70. `entity/mob/skeleton/parched.rs:59-66` returns Some(50) on Hard / Some(70) otherwise, consumed at `entity/ai/goal/bow_attack.rs:292`.
- Row 4 (lava pathfinding malus) - VERIFIED, no gap. Premise false: `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java:71-74` constructor calls only super() and reassessWeaponGoal(), with no setPathfindingMalus. The `setPathfindingMalus(PathType.LAVA, 8.0F)` call is in `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/WitherSkeleton.java:34`, a separate mob outside this unit. Pumpkin already matches vanilla for AbstractSkeleton; no generated-data field or subsystem is missing.

No pure unit-test seam exists for these goal/interval behaviours without a full Mob/world, so none added. No .rs touched, no rustfmt run.
