---
# Zombie Nautilus - vanilla parity audit (2026-11-16)

Sources: `crates/pumpkin-core/src/entity/passive/zombie_nautilus.rs`, shared helpers `crates/pumpkin-core/src/entity/passive/nautilus.rs`.
No decompiled `.vendor/` copy of `AbstractNautilus`/`ZombieNautilus` is present; comparisons are against the vanilla classes by name.

## Gaps (fix later)

| # | Gap | Evidence (file:line) | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | Water vs land sound selection uses the *submerged* test, so a nautilus swimming at the surface plays land sounds | `zombie_nautilus.rs:127` (`get_dash_sound`), `:136` (`get_dash_ready_sound`), `:196` (`hurt_sound`), `:204` (`death_sound`), `:284` (`get_ambient_sound`) call `Entity::is_under_water()`; `is_under_water` = `is_in_water() && is_submerged_in_water()` (`entity/mod.rs:4180`) | `AbstractNautilus`: `getDashSound`, `getDashReadySound`, `getHurtSound`, `getDeathSound`, `getAmbientSound` branch on `isInWater()` | Use `entity.touching_water.load(Ordering::Relaxed)` (the plain in-water flag already used by `nautilus.rs:105-131` for the same branches) | S |
| 2 | Dashing can never start: nothing writes `set_dashing(true)` or a positive `DashCooldown`, and `get_dash_sound` (`zombie_nautilus.rs:126`) is never called | Only decrement/read sites: `nautilus.rs:353-368`, `zombie_nautilus.rs:344,353,360-368`. Ride-jump input is dispatched only through `mob.as_abstract_horse()` (`player.rs:7532`), which no nautilus implements | `AbstractNautilus` starts a dash on rider jump input: cooldown ~50 ticks, dashing flag, dash sound, forward impulse; dash stops when `dashCooldown < 35` (already implemented at `nautilus.rs:353`) | Add a nautilus ride-jump dispatch in `Player::start_riding_jump` (e.g. a `nautilus_dash` hook on the `Mob` trait or `as_nautilus()`), set `dash_cooldown` and `is_dashing`, play `get_dash_sound()`, apply the impulse; wire for both `NautilusEntity` and `ZombieNautilusEntity` | M |
| 3 | Both nautilus mobs have almost no AI goals; `MobEntity::new` registers none | `zombie_nautilus.rs:79-86` adds only `TemptGoal(0.9)`; `nautilus.rs:41-55` adds none; `mob/mod.rs:292-295` `goals_selector: GoalSelector::default()` | `AbstractNautilus.registerGoals`: `FloatGoal(0)`, `TemptGoal`, `RandomSwimmingGoal`, `LookAtPlayerGoal`/`RandomLookAroundGoal`, target `HurtByTargetGoal`; `ZombieNautilus` inherits these | Add `passive/nautilus/goals.rs` modelled on `passive/dolphin/goals.rs:43` and register from both constructors; check the tempt speed against vanilla while there | M |
| 4 | 9-slot `inventory` is allocated but never serialised or used | `nautilus.rs:38,51`; `zombie_nautilus.rs:54,75`; no NBT read/write and no inventory screen path | `AbstractNautilus` has no item inventory (saddle + body armour only) - verify before deleting | Delete both fields if vanilla confirms no inventory; otherwise add `Items` NBT save/load and `open_mount_screen` wiring | S |
| 5 | Test coverage only asserts adult/variant invariants | `zombie_nautilus.rs:403-432` | - | Add one test asserting water vs land ambient/hurt sound after gap 1 is fixed, and one asserting `finalize_spawn` variant selection | S |

## Verified matching

- Dash stop threshold `< 35` and 60-tick Breath of the Nautilus refresh every 40 ticks (`nautilus.rs:331-368`).
- Taming flow, bucket-food consumption, equip sounds, `can_use_slot`, `sun_protection_slot(BODY)`, `burns_in_daylight`, no-baby invariant, `DashCooldown`/`variant` NBT keys.
---
