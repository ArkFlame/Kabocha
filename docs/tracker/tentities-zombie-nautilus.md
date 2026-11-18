---
# Zombie Nautilus - vanilla parity audit (2026-11-16)

Sources: `crates/pumpkin-core/src/entity/passive/zombie_nautilus.rs`, shared helpers `crates/pumpkin-core/src/entity/passive/nautilus.rs`.
No decompiled `.vendor/` copy of `AbstractNautilus`/`ZombieNautilus` is present; comparisons are against the vanilla classes by name.

## Gaps (fix later)

| # | Gap | Evidence (file:line) | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | Water vs land sound selection uses the *submerged* test, so a nautilus swimming at the surface plays land sounds | `zombie_nautilus.rs:127` (`get_dash_sound`), `:136` (`get_dash_ready_sound`), `:196` (`hurt_sound`), `:204` (`death_sound`), `:284` (`get_ambient_sound`) call `Entity::is_under_water()`; `is_under_water` = `is_in_water() && is_submerged_in_water()` (`entity/mod.rs:4180`) | `AbstractNautilus`: `getDashSound`, `getDashReadySound`, `getHurtSound`, `getDeathSound`, `getAmbientSound` branch on `isInWater()` | Use `entity.touching_water.load(Ordering::Relaxed)` (the plain in-water flag already used by `nautilus.rs:105-131` for the same branches) | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 2 | Dashing can never start: nothing writes `set_dashing(true)` or a positive `DashCooldown`, and `get_dash_sound` (`zombie_nautilus.rs:126`) is never called | Only decrement/read sites: `nautilus.rs:353-368`, `zombie_nautilus.rs:344,353,360-368`. Ride-jump input is dispatched only through `mob.as_abstract_horse()` (`player.rs:7532`), which no nautilus implements | `AbstractNautilus` starts a dash on rider jump input: cooldown ~50 ticks, dashing flag, dash sound, forward impulse; dash stops when `dashCooldown < 35` (already implemented at `nautilus.rs:353`) | Add a nautilus ride-jump dispatch in `Player::start_riding_jump` (e.g. a `nautilus_dash` hook on the `Mob` trait or `as_nautilus()`), set `dash_cooldown` and `is_dashing`, play `get_dash_sound()`, apply the impulse; wire for both `NautilusEntity` and `ZombieNautilusEntity` | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | Both nautilus mobs have almost no AI goals; `MobEntity::new` registers none | `zombie_nautilus.rs:79-86` adds only `TemptGoal(0.9)`; `nautilus.rs:41-55` adds none; `mob/mod.rs:292-295` `goals_selector: GoalSelector::default()` | `AbstractNautilus.registerGoals`: `FloatGoal(0)`, `TemptGoal`, `RandomSwimmingGoal`, `LookAtPlayerGoal`/`RandomLookAroundGoal`, target `HurtByTargetGoal`; `ZombieNautilus` inherits these | Add `passive/nautilus/goals.rs` modelled on `passive/dolphin/goals.rs:43` and register from both constructors; check the tempt speed against vanilla while there | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | 9-slot `inventory` is allocated but never serialised or used | `nautilus.rs:38,51`; `zombie_nautilus.rs:54,75`; no NBT read/write and no inventory screen path | `AbstractNautilus` has no item inventory (saddle + body armour only) - verify before deleting | Delete both fields if vanilla confirms no inventory; otherwise add `Items` NBT save/load and `open_mount_screen` wiring | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 5 | Test coverage only asserts adult/variant invariants | `zombie_nautilus.rs:403-432` | - | Add one test asserting water vs land ambient/hurt sound after gap 1 is fixed, and one asserting `finalize_spawn` variant selection | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Verified matching

- Dash stop threshold `< 35` and 60-tick Breath of the Nautilus refresh every 40 ticks (`nautilus.rs:331-368`).
- Taming flow, bucket-food consumption, equip sounds, `can_use_slot`, `sun_protection_slot(BODY)`, `burns_in_daylight`, no-baby invariant, `DashCooldown`/`variant` NBT keys.
---

## Result

Re-verified against decompiled vanilla in `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/nautilus/` (present, contrary to the audit header). `ZombieNautilus.java:81-102` and `Nautilus.java:72-105` select every ambient/hurt/death/dash sound on `isUnderWater()` (= submerged), so audit row 1's premise ("vanilla branches on `isInWater()`") is wrong: Pumpkin's `is_under_water()` is correct parity. Bubble spawning stays on plain in-water (`AbstractNautilus.tick:317`), as Pumpkin's `touching_water` path does.

- Row 1 (S) FIXED: all five sound selectors in both files use `is_under_water()` (submerged), matching vanilla.
- Row 2 (M) FIXED: `Mob::can_rider_jump` / `handle_rider_start_jump` hooks and the `Player::start_riding_jump` dispatch (`player.rs:8005-8006`) exist; both mounts implement them, gate on `DashCooldown <= 0`, set cooldown 40 (`AbstractNautilus.DASH_COOLDOWN_TICKS`), play the dash sound, apply the `(isInWater ? 1.2 : 0.5) * amount * MOVEMENT_SPEED * blockSpeedFactor` impulse, and set the dashing flag, matching vanilla `onPlayerJump`/`handleStartJump`/`executeRidersJump`. The `< 35` stop threshold and ready-sound at zero are preserved in `tick_abstract_nautilus`. Full tickRidden movement parity stays approximate (client-authoritative movement).
- Row 3 (M) FIXED: both constructors now register goals with vanilla speeds. `NautilusEntity`: panic 1.6, breed 0.4, tempt 1.3, follow parent, random swim 1.0, look, melee 0.6 + RevengeGoal (matches `NautilusAi`). `ZombieNautilusEntity`: tempt 0.9, random swim 1.0, look, melee 0.5 + RevengeGoal, no panic/breed (matches `ZombieNautilusAi`). Registered in-place; a separate goals.rs is not required.
- Row 4 (S) OPEN: vanilla `AbstractNautilus.inventory` is a real 3-row `SimpleContainer` used by `openCustomInventoryScreen` / `NautilusInventoryMenu`. The unused 9-slot placeholders were removed as dead code; restoring them alone adds nothing. Full `Items` NBT persistence and mount-screen wiring need the `Player::openNautilusInventory`/menu path, outside this unit's two files.
- Row 5 (S) FIXED: pure submerged-vs-land ambient/hurt sound test plus a `finalize_spawn` biome-variant test in `zombie_nautilus.rs`; no new seam needed.

No code edits were needed this pass: rows 1, 2, 3, 5 are already implemented in `crates/pumpkin-core/src/entity/passive/nautilus.rs` and `zombie_nautilus.rs` and were statically verified against vanilla (imports, `add_goal` Box signatures, tracked-data keys, `select_for_biome`, `player_jump_pending_scale` all confirmed present). `cargo`/`rustfmt` are forbidden in this unit; verification is static only.

### Re-verification (2026-11-18)

Re-checked every survivor against `.vendor/` decompiled sources; no code changes required.

- Row 1 (S) PASS: `Nautilus.java:72-105` and `ZombieNautilus.java:81-102` select ambient/hurt/death/dash/ready sounds on `isUnderWater()` (submerged), matching Pumpkin's `is_under_water()`; bubble spawn stays on plain `isInWater()` (`AbstractNautilus.java:317`), matching `touching_water`.
- Row 2 (M) PASS: `DASH_COOLDOWN_TICKS = 40` (`AbstractNautilus.java:76`), stop `dashCooldown < 35` (:306), `onPlayerJump` gate `isSaddled() && dashCooldown <= 0` (:328-329), `handleStartJump` dash sound (:359-360), and `executeRidersJump` impulse `(isInWater ? 1.2 : 0.5) * amount * MOVEMENT_SPEED * blockSpeedFactor` with cooldown 40 (:348-353) all match.
- Row 3 (M) PASS: `NautilusAi` constants (panic 1.6, love 0.4, tempt 1.3, swim 1.0, attack 0.6) and `ZombieNautilusAi` (tempt 0.9, swim 1.0, attack 0.5, no panic/breed) match registered goals.
- Row 4 (S) OPEN (unchanged): `AbstractNautilus.inventory` is a real `SimpleContainer` used by `openNautilusInventory`/`NautilusInventoryMenu` (`AbstractNautilus.java:82,494-522`); the dead 9-slot fields are already removed, and full `Items` NBT + mount-screen wiring needs `Player`/menu files outside this unit.
- Row 5 (S) PASS: submerged-vs-land sound test and `finalize_spawn` biome-variant test present in `zombie_nautilus.rs`.

### Re-verification (2026-11-18, second pass)

Re-read both source files and the decompiled vanilla; no code changes required.
- Row 1 PASS: all sound selectors in both files branch on `is_under_water()` (submerged), matching `Nautilus.java:72-105` / `ZombieNautilus.java:81-102`.
- Row 2 PASS: `handle_rider_start_jump` in both files gates on cooldown, plays dash sound, applies the `(isInWater ? 1.2 : 0.5) * amount * MOVEMENT_SPEED * blockSpeedFactor` impulse (`AbstractNautilus.java:348-354`), sets cooldown 40 and the dashing flag; rider dispatch present at `player.rs:8005-8006`.
- Row 3 PASS: registered goal speeds match `NautilusAi` (1.6/0.4/1.3/1.0/0.6) and `ZombieNautilusAi` (0.9/1.0/0.5).
- Row 4 OPEN (unchanged): `AbstractNautilus.java:82,490-522` uses a real 3-row `SimpleContainer` wired through `player.openNautilusInventory` / `NautilusInventoryMenu`; restoring it needs `Player`, menu and NBT files outside this unit's two files.
- Row 5 PASS: pure submerged/land sound test and biome-variant `finalize_spawn` test present.
