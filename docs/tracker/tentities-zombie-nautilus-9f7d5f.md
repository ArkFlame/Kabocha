# Zombie Nautilus - vanilla parity re-audit (2026-11-16)

Sources: `crates/pumpkin-core/src/entity/passive/zombie_nautilus.rs`, shared `crates/pumpkin-core/src/entity/passive/nautilus.rs`.
No decompiled `.vendor/` tree is present in this worktree; comparisons use the vanilla `AbstractNautilus` / `ZombieNautilus` classes by name and the already-audited plain `NautilusEntity` as the reference implementation.

Fixed in this pass: the five water/land sound getters used `Entity::is_under_water()` (submerged) instead of vanilla `isInWater()`; a nautilus swimming at the surface played the land sounds. Now uses `is_in_water()` (`zombie_nautilus.rs:127,136,196,204,284`), matching `nautilus.rs:105-131,168-200`.

## Remaining gaps

| # | Gap | Evidence (file:line) | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | Dashing can never start: nothing writes `set_dashing(true)` or a positive `DashCooldown`, `get_dash_sound` is dead | only reads/decrements at `nautilus.rs:353-368`, `zombie_nautilus.rs:344,353,360-368`; `Player::start_riding_jump` exits on `as_abstract_horse()` being `None` (`player.rs:7608`), no nautilus implements it | `AbstractNautilus` starts a dash from rider jump input: cooldown ~50 ticks, dashing flag, dash sound, stop when `dashCooldown < 35` (already implemented) | Add a nautilus branch to `start_riding_jump` (new `Mob` hook or `as_nautilus()`), set `dash_cooldown`, `set_dashing(true)`, play `get_dash_sound()`, apply forward impulse; wire both nautilus types | M |
| 2 | Both nautilus mobs have almost no AI goals | `zombie_nautilus.rs:79-86` adds only `TemptGoal(0.9)`; `nautilus.rs:41-55` adds none | `AbstractNautilus.registerGoals`: `FloatGoal(0)`, `TemptGoal`, `RandomSwimmingGoal`, `LookAtPlayerGoal`/`RandomLookAroundGoal`, target `HurtByTargetGoal`; `ZombieNautilus` inherits | Add `passive/nautilus/goals.rs` modelled on `passive/dolphin/goals.rs`, register from both constructors, check tempt speed | M |
| 3 | 9-slot `inventory` allocated but never serialised or used | `nautilus.rs:38,51`; `zombie_nautilus.rs:54,75`; no NBT and no screen path | `AbstractNautilus` has a saddle + body-armour equipment slot, not an item inventory (verify) | Delete both fields if confirmed; otherwise add `Items` NBT save/load and `open_mount_screen` wiring | S |
| 4 | Every hand interaction marks the zombie nautilus persistent, even when unhandled | `zombie_nautilus.rs:370-380` sets `persistence_required` then returns `None` | `AbstractNautilus`/`TamableAnimal` only persist on a successful interaction/taming | Drop the override (taming already handles persistence) or set the flag only after the interact result is `Some(true)` | S |
| 5 | `remove_when_far_away` returns `true` unconditionally, so tamed mounts still despawn | `zombie_nautilus.rs:253-255`; `nautilus.rs` has no override (default `true`) | Tameable mounts persist while tamed (`TamableAnimal` / per-mob override) | Return `!self.is_tame()` in both nautilus types once confirmed against vanilla | S |
| 6 | Saddle equip sound still branches on submerged rather than in-water | `nautilus.rs:488` `entity.is_under_water()` for `ItemNautilusSaddleUnderwaterEquip` | `AbstractNautilus.getEquipSound` uses `isInWater()` | Use `entity.is_in_water()` | S |
| 7 | Tests only assert adult/variant invariants | `zombie_nautilus.rs:403-432` | - | Add a water-vs-land ambient/hurt sound test and a `finalize_spawn` variant-selection test | S |

## Verified matching

- Dash stop threshold `< 35` and 60-tick Breath of the Nautilus refresh every 40 ticks (`nautilus.rs:331-368`).
- Taming flow, bucket-food consumption, eat sounds, `can_use_slot`, `sun_protection_slot(BODY)`, `burns_in_daylight`, no-baby invariant, `DashCooldown`/`variant` NBT keys, biome variant selection in `finalize_spawn`.
- Sound getters now agree with the plain nautilus on the water test.
