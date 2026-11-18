# Final rows from docs/tracker/tentities-camel-husk.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN and no BLOCKED-DATA condition exists. No `.rs` source or test edit was needed.
All five rows are terminal FIXED/VERIFIED; no row is STILL OPEN, no BLOCKED-DATA condition exists, and no
Pumpkin source directly for every row. All five rows are terminal FIXED/VERIFIED; no row is STILL OPEN, no

## Result

All five rows terminal FIXED/VERIFIED against decompiled vanilla; no row STILL OPEN, no BLOCKED-DATA.

- Row 1 (husk love gate): FIXED/VERIFIED. .vendor/.../world/entity/animal/camel/CamelHusk.java:70-73 canFallInLove()=false, :60-63 canMate()=false. camel.rs:383 gates could_set_in_love on !self.is_husk(); camel.rs:425-427 can_fall_in_love returns false for husks; camel.rs:429-431 can_mate refuses husks.
- Row 2 (love/tempt goals in husk brain): FIXED/VERIFIED (inert). CamelHusk.java has no createBrain override, so vanilla runs CamelAi with AnimalMakeLove/FollowTemptation; Pumpkin camel/ai.rs:72,75 matches. animal_make_love only runs on is_in_love(), which a husk never gets after Row 1.
- Row 3 (is_preventing_player_rest): FIXED/VERIFIED. Only Monster.java returns true; CamelHusk inherits false. camel.rs:467-471 returns false.
- Row 4 (numeric constants): FIXED/VERIFIED. CamelHusk.java:90-97 step sounds at 0.4F; :129-132 chargeSpeedModifier()=4.0F. camel.rs:48 HUSK_STEP_VOLUME=0.4 used at :457-458; camel.rs:437-443 returns 4.0 for husk else 1.0.
- Row 5 (jockey pairing): FIXED/VERIFIED. Husk.java:93-133 once-per-group triedToSpawnCamelHusk gate plus 10% roll and camel husk + parched mount at :113-126. Pumpkin husk.rs:43-55 claim_camel_jockey_attempt, :60-108 spawn logic, spawn.rs:64-70 SpawnGroupData::Zombie.tried_spawn_camel_husk. Pure-seam tests at husk.rs:178-220.

Remaining runtime-only work (gametest for the two-rider mount layout) needs a file outside this unit and is not a code gap. No .rs file touched this pass, so rustfmt not run.
