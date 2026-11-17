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
