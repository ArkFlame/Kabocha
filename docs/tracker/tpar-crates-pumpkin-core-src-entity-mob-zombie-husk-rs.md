# Parity rows for crates/pumpkin-core/src/entity/mob/zombie/husk.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-camel-husk** [partial/OWN-OPEN -> FIXED/VERIFIED] own audit: 5 open, 3 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-camel-husk.md) docs/tracker/tentities-camel-husk.md

## Result (third pass, closed 2026-11-18)

own/tentities-camel-husk: FIXED/VERIFIED. Independently re-read the decompiled vanilla
classes under .vendor/vanilla-decompiled/net/minecraft/ and the current Pumpkin source.
No row remains STILL OPEN; no BLOCKED-DATA condition exists; no code or test edit was
needed (the fixes from earlier passes are present and correct). All five sub-rows:

- Row 1 (husk love gate): FIXED/VERIFIED. Vanilla
  world/entity/animal/camel/CamelHusk.java:70-73 overrides canFallInLove() to false and
  :60-63 canMate() to false. camel.rs:383 gates could_set_in_love on !self.is_husk(),
  camel.rs:425-427 can_fall_in_love returns false for husks, camel.rs:429-431 can_mate
  refuses husks. Matches.
- Row 2 (love/tempt goals): FIXED/VERIFIED (inert). CamelHusk.java has no createBrain
  override, so vanilla runs CamelAi with AnimalMakeLove/FollowTemptation; Pumpkin
  camel/ai.rs:72,75 matches. ai/brain/behavior/animal_make_love.rs:97-99,130 only runs
  on is_in_love(), which a husk never gets after Row 1. No observable extra work.
- Row 3 (is_preventing_player_rest): FIXED/VERIFIED. Only
  world/entity/monster/Monster.java:140-141 returns true; CamelHusk inherits the
  Animal/Mob default false. camel.rs:467-471 returns false. Matches.
- Row 4 (numeric constants): FIXED/VERIFIED. CamelHusk.java:90-97 plays step sounds at
  0.4F, :129-132 chargeSpeedModifier() returns 4.0F. camel.rs:48 HUSK_STEP_VOLUME = 0.4
  used at :457-458, camel.rs:437-443 returns 4.0 for husk else 1.0. Matches.
- Row 5 (jockey pairing): FIXED/VERIFIED. world/entity/monster/zombie/Husk.java:93-133
  wraps group data in HuskGroupData with triedToSpawnCamelHusk = (reason != NATURAL),
  sets the flag before the 10% roll (:103-112) and spawns camel husk + parched and mounts
  both (:113-126). Pumpkin husk.rs:43-55 claim_camel_jockey_attempt, :60-108 spawn logic,
  spawn.rs:64-70 SpawnGroupData::Zombie.tried_spawn_camel_husk, initialized at
  zombie/mod.rs:262-268 (false for fresh/conversion group), zombie/mod.rs:659-663 (true
  for villager conversion), strider.rs:340-344 (false for piglin jockey). world/mod.rs:6492-
  6526 add_pending_riders mounts the husk, then the parched, onto the camel. Pure-seam
  tests at husk.rs:178-220 cover the once-per-group and blocked/non-natural cases.
  Matches.

Remaining runtime-only work (a gametest for the two-rider mount layout) needs a file
outside this unit and is not a code gap. No compile run (worker rule); no .rs file
touched, so rustfmt was not run.


## Result (FINAL PASS, 2026-11-18, direct re-verification)

Re-read the current Pumpkin source and the decompiled vanilla tree under
`.vendor/vanilla-decompiled/net/minecraft/` for every row. All five sub-rows of
own/tentities-camel-husk are terminal FIXED/VERIFIED; no row remains STILL OPEN, no
BLOCKED-DATA condition exists, and no source or test edit was needed. No compile run
(worker rule); no .rs file touched, so rustfmt was not run.

- Row 1 (husk love gate): FIXED/VERIFIED. .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/camel/CamelHusk.java:60-63 overrides canMate() to false and :70-73 overrides canFallInLove() to false. Pumpkin camel.rs:383 gates could_set_in_love on !self.is_husk(), :425-426 can_fall_in_love() returns false for husks, :429-430 can_mate() returns false for husks. Matches.
- Row 2 (love/tempt goals): FIXED/VERIFIED (inert). CamelHusk.java has no createBrain override, so vanilla runs CamelAi with AnimalMakeLove/FollowTemptation (Pumpkin camel/ai.rs:72,75). animal_make_love.rs:70,98,130-135 only starts/continues while is_in_love() and can_mate() hold, both false for a husk after Row 1. No observable extra work.
- Row 3 (is_preventing_player_rest): FIXED/VERIFIED. Only vanilla world/entity/monster/Monster.java returns true; CamelHusk inherits Animal's false. camel.rs:467-471 returns false. Matches.
- Row 4 (numeric constants): FIXED/VERIFIED. CamelHusk.java:90-97 plays step sounds at 0.4F and :129-132 chargeSpeedModifier() returns 4.0F. camel.rs:48 HUSK_STEP_VOLUME = 0.4 used at :457-458, and :437-443 returns 4.0 for husk else 1.0. Matches.
- Row 5 (jockey pairing): FIXED/VERIFIED. world/entity/monster/zombie/Husk.java:93-133 wraps group data in HuskGroupData with triedToSpawnCamelHusk = (reason != NATURAL) at :105, gates a noCollision space check plus !tried at :108-111, sets the flag before the 10% roll at :112, then spawns and mounts camel husk + parched at :113-126. Pumpkin husk.rs:43-55 claim_camel_jockey_attempt (skip if tried / non-natural / blocked, set tried, return true), :60-108 spawn logic with CAMEL_HUSK_JOCKEY_CHANCE = 0.1, and spawn.rs:64-70 SpawnGroupData::Zombie.tried_spawn_camel_husk (initialized false at zombie/mod.rs:266 and strider.rs:340, true for villager conversion at zombie/mod.rs:662) implement the same once-per-group rule. Pure-seam tests at husk.rs:178-219 cover once-per-group and non-natural/blocked cases. Matches.

Remaining runtime-only work (a gametest for the two-rider mount layout) needs a file outside this unit and is a test-coverage task, not a code gap. No new test was added because the pure seam is already covered.
