# Parity rows for crates/pumpkin-core/src/entity/mob/zombie/husk.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-camel-husk** [partial/OWN-OPEN] own audit: 5 open, 3 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-camel-husk.md) docs/tracker/tentities-camel-husk.md

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
