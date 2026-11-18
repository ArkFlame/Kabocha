# Parity rows for crates/pumpkin-core/src/entity/ai/behavior/feline.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-ocelot** [partial/OWN-OPEN] own audit: 5 open, 3 fixed, 7 verified, 0 blocked-data (docs/tracker/tentities-ocelot.md) docs/tracker/tentities-ocelot.md

## Result

- **own/tentities-ocelot** — **VERIFIED** (matches vanilla; no code change needed in feline.rs).
  The referenced audit (docs/tracker/tentities-ocelot.md) is already closed (fixed=1, open=0:
  the sound fix landed in ocelot.rs, not in this unit's file). Re-checked every feline.rs helper
  against 1.21 decompiled vanilla at
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/feline/`:
  - `tempt_goal`: vanilla `Ocelot.registerGoals` builds `OcelotTemptGoal(this, 0.6,
    OCELOT_FOOD, true)` and `canScare() = super.canScare() && !isTrusting()`
    (Ocelot.java:105, 304-315). Pumpkin `TemptGoal::with_tag(TEMPT_SPEED=0.6,
    MINECRAFT_OCELOT_FOOD, true).with_scare_condition(is_wary)` with `is_wary = !is_trusting()`
    (ocelot.rs:37-41). Match.
  - `flee_players`: vanilla `reassessTrustingGoals` adds `OcelotAvoidEntityGoal(Player.class,
    16.0F, 0.8, 1.33)` at goal 4, gated `!isTrusting()` on both canUse and canContinueToUse,
    selector `EntitySelector.NO_CREATIVE_OR_SPECTATOR` (Ocelot.java:213-221, 283-302). Pumpkin
    `Flee::new(4, PLAYER, 16.0, 0.8, SPRINT_SPEED=1.33).as_long_as(is_wary)`; `as_long_as` sets
    both when/keep_when, and AvoidEntityGoal's default selector is already
    `not_creative_or_spectator` (avoid_entity.rs:26-27,70). Match.
  - `apply_tail`: vanilla Ocelot goals 7-11 = Leap 0.3, OcelotAttack, Breed 0.8,
    WaterAvoidingStroll 0.8/1.0000001E-5, LookAtPlayer 10.0 (Ocelot.java:104-115); Cat goals
    8-12 are the identical tail (Cat.java:106-123). Pumpkin `apply_tail(first=7)` for ocelot
    (ocelot.rs:76) and `apply_tail(first=8)` for cat (cat.rs:122). Match.
  - Constants `TEMPT_SPEED=0.6`, `SPRINT_SPEED=1.33`, `STROLL_PROBABILITY=1.000_000_1E-5`
    match the vanilla literals exactly.
  No pure unit-test seam exists in feline.rs: the three helpers only construct goal structs and
  need mocked `Mob`/`GoalSelector`; a constant-mirroring test would violate the MVP tests rule,
  so none was added. No code, generated-data, WIT or cross-crate changes. Not compiled by me
  (orchestrator gate).
