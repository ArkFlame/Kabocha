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
| 2 | mod.rs:73; ai/goal/restrict_sun.rs:11-17 | `AbstractSkeleton.registerGoals` registers `RestrictSunGoal(this)` at priority 2. Vanilla `RestrictSunGoal` only toggles `mob.getNavigation().setAvoidSun(true/false)` while it is day (`isDay()`) and `getTarget() == null`. Pumpkin's goal starts on "bright outside && !protected", returns `Controls::empty` and never touches the navigator, so it is a no-op | Add an avoid-sun flag to the pathfinder/navigator (sun-exposed nodes get a malus or are excluded unless the destination is shaded), then have `RestrictSunGoal` set it in `start`/`stop` and use vanilla `canUse` and register it at 2 | M |
| 3 | mod.rs:76, skeleton/parched.rs | Vanilla `AbstractSkeleton.getAttackInterval()/getHardAttackInterval()` (40/20) are overridden per species; Bogged is ported (70/50 in bogged.rs), the earlier TODO also claimed Parched uses 70/50, but `docs/VANILLA_REF.md` lists no interval for Parched, so it may use the 40/20 default | *verify* against decompiled `Parched.class`; if it overrides, add `get_min_attack_interval` to `ParchedSkeletonEntity` (copy bogged.rs:118-126) | S |
| 4 | mod.rs:59-101 | Vanilla `AbstractSkeleton` constructor calls `setPathfindingMalus(PathType.LAVA, 8.0F)`, so skeletons avoid lava | Needs a per-mob path-type malus API in the pathfinder; none exists today | M |

## Checked and already matching vanilla

- Goal list: `FleeSunGoal`/wolf `AvoidEntityGoal` 3 (6 blocks, 1.0/1.2), bow + melee 4 (melee gated on `!is_holding_bow`, equivalent to `reassessWeaponGoal`), water-avoiding stroll 5, look-at-player 8.0 6, random look 6. Target list: `HurtByTargetGoal` 1 (no ally alert), Player 2, Iron Golem 3, baby turtle 3.
- Sun burning itself is tag-driven (`MINECRAFT_BURN_IN_DAYLIGHT`) via `mob::sun_burn::tick`, so skeletons ignite and wear out helmets correctly.
- `populateDefaultEquipmentSlots`: generic armor population then `BOW` in the main hand; `wants_to_pick_up` rejects `MINECRAFT_SPEARS` and defers to `can_hold_item` (wither skeleton also rejects `WITHER_SKELETON_DISLIKED_WEAPONS`).
- Per-species arrows: Stray slowness 600t, Parched weakness 600t, Bogged poison (100t, pre-duration-scale), Wither flame + wither 200t on melee; wither skeleton damage 4.0 set at spawn.
