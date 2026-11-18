**FIXED** `5abaeaa7e` (fix(entity): feline goal parity for cats and ocelots (#3984)). VERIFIED against code: entity/passive/ocelot.rs:29,71 (flee and tempt gated by is_wary).

# entities / Ocelot — re-review (source changed)

Source: `crates/pumpkin-core/src/entity/passive/ocelot.rs`,
`crates/pumpkin-core/src/entity/ai/goal/ocelot_attack.rs`,
`crates/pumpkin-core/src/entity/ai/behavior/feline.rs`.
Last change reviewed: `c6f7b548c` "ocelot despawn, crouch pose and spawn obstruction (#4209)".

## Fixed in this pass

- **Fabricated spawn restriction** (`ocelot.rs:219-233` before the fix): the
  override required `y >= sea_level` and the block below to be grass or leaves.
  Vanilla `Ocelot` does not override `Mob.checkSpawnObstruction` (default:
  no liquid in the bounding box, unobstructed), and floor validity is already
  `SpawnPlacementTypes.ON_GROUND` in `natural_spawner.rs:1031` /
  `is_valid_spawn_floor` (`natural_spawner.rs:1177`, `SpawnFloorPredicate::OcelotOrParrot`
  for leaves). The vanilla 1-in-3 spawn gate lives in
  `crates/pumpkin-core/src/entity/type.rs:703` (`Ocelot.checkOcelotSpawnRules`).
  The override was removed; `Mob::check_spawn_obstruction` default applies.
  Size: S.

- **Target goal priority tie** (`ocelot.rs:31-34`): `OCELOT_HUNTS` gave Chicken
  and `hunting::baby_turtles(1, false)` the same priority 1. Vanilla
  `Ocelot.registerGoals` puts `NearestAttackableTargetGoal<Chicken>` at 1 and
  the `Turtle.BABY_ON_LAND_SELECTOR` goal at 2. Turtle hunts now use priority 2.
  Size: S.

## Remaining gaps

1. **`mob_interact` distance gate** — `ocelot.rs:162` (`dist_sqr < 9.0`).
   Vanilla `Ocelot.mobInteract` feeds on `isFood` + `!isTrusting()` without a
   range check (the `< 9.0` gate belongs to the 1.12 `processInteract`).
   Suggested fix: verify against the 1.21 `Ocelot.mobInteract` body and drop
   the distance condition if absent. Size: S.

2. **No `get_breed_offspring` override** — `ocelot.rs` inherits
   `Animal::get_breed_offspring` (`animal.rs:22`), which spawns a plain ocelot
   with `is_trusting = false`. Vanilla `Ocelot.getBreedOffspring` sets the
   child trusting state from the parent (`ocelot.setTrusting(this.isTrusting())`).
   Suggested fix: override in `impl Animal for OcelotEntity`, copy
   `self.is_trusting` to the new ocelot before it is returned. Vanilla class:
   `net.minecraft.world.entity.animal.Ocelot`. Size: S.

3. **Mob sounds** — `ocelot.rs` defines no ambient/hurt/death/step sound.
   Vanilla `Ocelot` uses `SoundEvents.OCELOT_AMBIENT` / `OCELOT_HURT` /
   `OCELOT_DEATH` and `SoundEvents.CAT_STEP` (all `1.0F` volume). Check the
   mob sound table (`crates/pumpkin-core/src/entity/mob` sound dispatch) and
   add the ocelot entries if missing. Size: S/M.

4. **`mob_is_stepping_carefully`** — `ocelot.rs:209-212` returns
   `pose == Crouching || is_sneaking()`. This mirrors the existing
   `cat.rs:662` implementation; vanilla `Ocelot` has no such override and
   inherits `LivingEntity.isSteppingCarefully()` (crouching only). Low impact,
   client/stepping only. Suggested fix: verify against vanilla and then share
   one helper with cat. Size: S.

## Verified as matching vanilla (no action)

- `custom_server_ai_step` pose/sprint branches on `0.6` / `1.33` move speed
  (`ocelot.rs:187-209`) = vanilla `Ocelot.customServerAiStep`.
- `remove_when_far_away` = `!isTrusting() && tickCount > 2400`.
- Taming: `1/3` success, `40`/`41` entity events, `Trusting` NBT, `OCELOT_FOOD`
  tag, baby + trusting tracked data, `OcelotAttackGoal` radius/speeds.
- Goal ordering from `feline::apply_tail` (leap, attack, breed, stroll,
  look-at-player) matches `Ocelot.registerGoals`.

## Gap status
- 1 STILL OPEN: `ocelot.rs:162` distance gate, vanilla 1.21 `mobInteract` body not in repo (UNVERIFIED).
- 2 STILL OPEN: breeding trust inheritance UNVERIFIED; vanilla breeding for ocelots not confirmed.
- 3 STILL OPEN: sound table entries, S/M, not attempted in sweep.
- 4 STILL OPEN: stepping-carefully override, UNVERIFIED against vanilla.


## Result

Re-verified all 4 STILL OPEN rows against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/feline/Ocelot.java`
(1.21, package moved to `animal/feline`).

1. `mob_interact` distance gate — **NO CHANGE, verified matching vanilla**.
   Vanilla `Ocelot.mobInteract` (line 169) explicitly requires
   `player.distanceToSqr(this) < 9.0` alongside `isFood` and `!isTrusting()`.
   The tracker premise (range check belongs only to 1.12) was wrong; Pumpkin's
   `dist_sqr < 9.0` is correct. (Vanilla also gates on
   `temptGoal == null || temptGoal.isRunning()`, not in scope of this row.)

2. `get_breed_offspring` — **NO CHANGE, verified matching vanilla**. Vanilla
   `Ocelot.getBreedOffspring` (line 230) simply creates a fresh `OCELOT` via
   `EntityTypes.OCELOT.create(..., BREEDING)`; it does NOT copy the parent's
   trusting state. Pumpkin's inherited `Animal::get_breed_offspring` spawns a
   plain ocelot with `is_trusting = false`, which matches. Tracker premise was
   wrong.

3. Mob sounds — **FIXED**. Vanilla `Ocelot` defines `getAmbientSound` =
   `OCELOT_AMBIENT` (line 147) and `getAmbientSoundInterval` = `900` (line 152).
   Added `get_ambient_sound()` (returns `entity_type.ambient_sound`, i.e.
   `EntityOcelotAmbient`) and `get_ambient_sound_interval()` (900, as
   `OCELOT_AMBIENT_SOUND_INTERVAL`) to `impl Mob for OcelotEntity`, plus a pure
   unit test for the interval constant. Hurt/death sounds already resolve from
   generated `entity_type` data (`EntityOcelotHurt`/`EntityOcelotDeath`), so no
   `CustomSound` impl is needed. Vanilla `Ocelot` has no step-sound override
   (tracker's `CAT_STEP` claim is wrong), so default block step sound stands.

4. `mob_is_stepping_carefully` — **NO CHANGE, verified matching vanilla**.
   Vanilla `Ocelot.isSteppingCarefully` (line 279) is
   `isCrouching() || super.isSteppingCarefully()` =
   `pose == CROUCHING || isShiftKeyDown()`. Pumpkin already returns
   `pose == Crouching || is_sneaking()` (sneaking = shift flag), an exact match.
   Cat uses the identical pair of overrides, so both already match vanilla.

`rustfmt --edition 2024` run on the one touched .rs file. Not compiled by me
(orchestrator gate). No generated data, WIT or cross-crate changes needed.

### Second-pass verification

Re-read source against vanilla again; all 4 rows still match, no new edits.
Row 3 sound seam re-checked: `get_ambient_sound`/`get_ambient_sound_interval`
signatures in `mob/mod.rs:2445-2451` and the
`self.get_entity().entity_type.ambient_sound` pattern match `phantom.rs:157`.
Generated `entity_type.rs:7743-7745` confirms Ocelot hurt/death/ambient sounds
are populated, so no `CustomSound` impl is needed. Row 4 re-checked against
`Entity.java:3936` (`isSteppingCarefully()` = `isShiftKeyDown()`).

## Result

Final verification pass (1.21 decompiled vanilla `Ocelot.java` + current source).

1. `mob_interact` distance gate — **VERIFIED, no change**. Vanilla requires
   `player.distanceToSqr(this) < 9.0` (`Ocelot.mobInteract`), and Pumpkin keeps
   `dist_sqr < 9.0` (`ocelot.rs:197`). Gap-status claim that the gate was 1.12-only
   was incorrect.
2. `get_breed_offspring` trust inheritance — **VERIFIED, no change**. Vanilla
   returns a plain `EntityTypes.OCELOT` on breeding with no trust copy, matching
   inherited `Animal::get_breed_offspring` behaviour.
3. Mob sounds — **ALREADY FIXED in source**. `OcelotEntity` implements
   `get_ambient_sound()` (`entity_type.ambient_sound`) and
   `get_ambient_sound_interval()` returning `OCELOT_AMBIENT_SOUND_INTERVAL = 900`,
   with unit tests for despawn and ambient interval constants (`ocelot.rs`).
   Hurt/death resolve from generated entity-type data; no `CustomSound` impl needed.
4. `mob_is_stepping_carefully` — **VERIFIED, no change**. Vanilla returns
   `isCrouching() || super.isSteppingCarefully()` (shift/sneaking), matching
   Pumpkin `pose == Crouching || is_sneaking()`; cat uses the same override pattern.

Fixed=1 (row 3, already present), open=0. Files: `crates/pumpkin-core/src/entity/passive/ocelot.rs`, `docs/tracker/tentities-ocelot.md`. No other allowed source files changed. Not compiled (orchestrator gate). No generated data/WIT/cross-crate changes.
