# Final rows from docs/tracker/tentities-ocelot.md

Close each item below completely: read the vanilla class under
`.vendor/vanilla-decompiled/net/minecraft/`, change the Rust to match, and record
the per-row result below. All rows are now closed.

Vanilla reference for every row below:
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/feline/Ocelot.java`

## Result

1. `mob_interact` distance gate — **VERIFIED, no change**. Vanilla
   `Ocelot.mobInteract` (`Ocelot.java:169`) explicitly requires
   `player.distanceToSqr(this) < 9.0` alongside `!isTrusting()` and `isFood`.
   Pumpkin `ocelot.rs:207-211` keeps `dist_sqr < 9.0` and the running-TemptGoal
   gate. The tracker premise that the range check was 1.12-only was wrong.

2. `get_breed_offspring` trust inheritance — **VERIFIED, no change**. Vanilla
   `Ocelot.getBreedOffspring` (`Ocelot.java:230-232`) returns
   `EntityTypes.OCELOT.create(level, EntitySpawnReason.BREEDING)` with no copy of
   the parent's trusting state. The inherited `Animal::get_breed_offspring`
   spawns a plain ocelot (`is_trusting = false`), which matches. No override
   needed.

3. Mob sounds — **FIXED (already present in source)**. Vanilla `Ocelot`
   (`Ocelot.java:147-164`) defines `OCELOT_AMBIENT` (interval `900` at line 153),
   `OCELOT_HURT` and `OCELOT_DEATH`. `OcelotEntity` implements
   `get_ambient_sound()` returning `entity_type.ambient_sound`
   (`Sound::EntityOcelotAmbient`, generated `entity_type.rs:7745`) and
   `get_ambient_sound_interval()` returning
   `OCELOT_AMBIENT_SOUND_INTERVAL = 900`, with the pure unit test
   `ambient_sound_interval_matches_vanilla` (`ocelot.rs:261-264`). Hurt/death
   resolve through `living.rs:3877-3895` falling back to
   `entity_type.hurt_sound`/`death_sound` (`EntityOcelotHurt`/`EntityOcelotDeath`,
   `entity_type.rs:7743-7744`), so no `CustomSound` impl is needed. Vanilla has
   no step-sound override, so the default block step sound stands.

4. `mob_is_stepping_carefully` — **VERIFIED, no change**. Vanilla
   `Ocelot.isSteppingCarefully` (`Ocelot.java:279-280`) is
   `isCrouching() || super.isSteppingCarefully()` (super = shift key). Pumpkin
   `ocelot.rs:236-239` returns `pose == Crouching || is_sneaking()`, an exact
   match; cat uses the same override pair.

Fixed=1 (row 3, already in source), verified=3, blocked=0, open=0. No source
edits were needed this pass. No generated pumpkin-data, WIT or cross-crate
changes required. Not compiled by this worker (orchestrator gate).
