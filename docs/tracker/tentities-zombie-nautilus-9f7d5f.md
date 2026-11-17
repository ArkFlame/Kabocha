---
# Zombie Nautilus - vanilla parity audit (re-review 2026-11-17)

Sources: crates/pumpkin-core/src/entity/passive/zombie_nautilus.rs, shared helpers crates/pumpkin-core/src/entity/passive/nautilus.rs.
No decompiled .vendor/ copy of AbstractNautilus/ZombieNautilus is present; comparisons are against the vanilla classes by name.
Previous audit: docs/tracker/tentities-zombie-nautilus.md (2026-11-16). Source changed since then (63dbc0b9e, 2237ca5cf, this branch), so gaps were re-checked against the current files.

## Fixed since the previous review

| Gap | Resolution | Evidence |
|---|---|---|
| Water/land sound branches used the submerged test | Now Entity::is_in_water() (== touching_water), which is vanilla isInWater() | zombie_nautilus.rs:127,136,196,204,284; entity/mod.rs:4446 |
| Non-vanilla DashCooldown NBT key | Removed from the zombie nautilus write/read, matching 2237ca5cf | zombie_nautilus.rs:342-357, test :426 |

## Gaps (fix later)

| # | Gap | Evidence (file:line) | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | Dashing can never start: nothing writes set_dashing(true) or a positive dash_cooldown | Only decrement/read sites: nautilus.rs:352-367; zombie_nautilus.rs:359-367. Ride-jump input is dispatched only through Mob::as_abstract_horse (player.rs:7744-7768), which no nautilus implements | AbstractNautilus starts a dash on rider jump: cooldown ~50 ticks, dashing flag, dash sound, forward impulse; dash stops when dashCooldown < 35 (already implemented at nautilus.rs:352) | Add a nautilus ride-jump branch in Player::start_riding_jump (e.g. Mob::as_nautilus_dash or a small trait object), set dash_cooldown, set_dashing(true), play get_dash_sound(), apply the impulse; wire for both NautilusEntity and ZombieNautilusEntity | M |
| 2 | Both nautilus mobs have almost no AI goals | zombie_nautilus.rs:79-86 adds only TemptGoal(0.9); nautilus.rs:42-55 adds none; MobEntity::new registers none | AbstractNautilus.registerGoals: FloatGoal(0), TemptGoal, RandomSwimmingGoal, LookAtPlayerGoal/RandomLookAroundGoal, target HurtByTargetGoal; ZombieNautilus inherits them | Add passive/nautilus/goals.rs modelled on passive/dolphin/goals.rs:43 and register from both constructors; check the tempt speed (0.9 here) against vanilla | M |
| 3 | 9-slot inventory is allocated but never serialised or used | nautilus.rs:38,51; zombie_nautilus.rs:54,75; no NBT read/write and no inventory screen path | AbstractNautilus has no item inventory (saddle + body armour only) - verify against decompiled source before deleting | Delete both fields if vanilla confirms no inventory; otherwise add Items NBT save/load and open_mount_screen wiring | S |
| 4 | Test coverage only asserts adult/variant invariants | zombie_nautilus.rs:400-429 | - | Add one test asserting the water vs land ambient/hurt sound after the (now fixed) in-water branch, and one asserting finalize_spawn variant selection | S |
| 5 | Saddle equip sound still branches on the submerged test | nautilus.rs:483-492 (nautilus_equip_sound -> is_under_water()), shared by both mobs | AbstractNautilus.getEquipSound: confirm whether it uses isUnderWater() (as implemented) or isInWater() | Verify against decompiled AbstractNautilus; switch to touching_water only if vanilla uses isInWater() | S |

## Verified matching

- Dash stop threshold < 35, 60-tick Breath of the Nautilus refresh every 40 ticks, bubble particles (nautilus.rs:329-385).
- Taming flow, bucket-food consumption, equip sounds for non-saddle slots, can_use_slot, sun_protection_slot(BODY), burns_in_daylight, no-baby invariant, variant NBT key and variant-biome selection.
- DashCooldown is now transient for both nautilus mobs; only variant/tamable state persists.
