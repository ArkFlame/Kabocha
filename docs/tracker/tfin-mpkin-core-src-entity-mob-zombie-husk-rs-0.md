# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-zombie-husk-rs.md

The single row of this unit is own/tentities-camel-husk (the five rows of
docs/tracker/tentities-camel-husk.md). Read the vanilla classes under
.vendor/vanilla-decompiled/net/minecraft/ and make the Rust match (no cargo, rustfmt touched
files). Files: any source file a row needs.

## Result (FINAL PASS, 2026-11-18)

All five sub-rows of own/tentities-camel-husk are terminal FIXED/VERIFIED against the
decompiled vanilla tree under .vendor/vanilla-decompiled/net/minecraft/. No row remains
STILL OPEN, no BLOCKED-DATA condition exists, and no source or test edit was needed: the
fixes from earlier passes are present in the current checkout and were independently
re-read this pass. No compile run (worker rule); no .rs file touched, so rustfmt was not run.

- Row 1 (husk love gate): FIXED/VERIFIED. Vanilla
  .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/camel/CamelHusk.java:61-62
  overrides canMate() to false and :71-73 overrides canFallInLove() to false. Pumpkin
  camel.rs:383 gates could_set_in_love on !self.is_husk(), :425-427 can_fall_in_love
  returns false for husks, :429-431 can_mate refuses husks. Matches.
- Row 2 (love/tempt goals): FIXED/VERIFIED (inert). CamelHusk.java (full file read) has no
  createBrain override, so vanilla runs CamelAi with AnimalMakeLove/FollowTemptation;
  Pumpkin camel/ai.rs:72,75 matches. AnimalMakeLove only starts while is_in_love() holds
  (animal_make_love.rs:70,98,130-131), which a husk never gets after Row 1. No observable
  extra work.
- Row 3 (is_preventing_player_rest): FIXED/VERIFIED. In vanilla only
  world/entity/monster/Monster.java returns true; CamelHusk inherits Animal's false.
  Pumpkin camel.rs:467-471 returns false. Matches.
- Row 4 (numeric constants): FIXED/VERIFIED. CamelHusk.java:93,95 play step sounds at 0.4F
  and :130-131 chargeSpeedModifier() returns 4.0F. Pumpkin camel.rs:48
  HUSK_STEP_VOLUME = 0.4 used at :457-458 and :437-443 charge_speed_modifier returns 4.0
  for husk else 1.0. Matches.
- Row 5 (jockey pairing): FIXED/VERIFIED. Vanilla
  world/entity/monster/zombie/Husk.java:103-112 gates on HuskGroupData.triedToSpawnCamelHusk
  (true for non-natural spawns, checked before the space test, set before the 10% roll),
  then :113-126 spawns camel husk + parched and mounts both. Pumpkin husk.rs:45-55
  claim_camel_jockey_attempt, :60-106 spawn logic with CAMEL_HUSK_JOCKEY_CHANCE = 0.1, and
  spawn.rs:64-70 SpawnGroupData::Zombie.tried_spawn_camel_husk (initialized false at
  zombie/mod.rs:266 and strider.rs:347, true for villager conversion at zombie/mod.rs:662)
  implement the same once-per-group rule. Pure-seam tests at husk.rs:183-219 cover the
  once-per-group and blocked/non-natural cases. Matches.

Remaining runtime-only work (a gametest for the two-rider mount layout) needs a file
outside this unit and is a test-coverage task, not a code gap. No BLOCKED-DATA condition.
