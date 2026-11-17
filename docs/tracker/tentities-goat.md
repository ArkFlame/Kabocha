# entities/Goat

Source: crates/pumpkin-core/src/entity/passive/goat.rs (re-reviewed against decompiled vanilla Goat.java / GoatAi.java, 1.21.4; source re-read after 6d7bc3015).
Fixed in this pass: age-boundary horn toggle (vanilla ageBoundaryReached addHorns/removeHorns), max head yaw 15 deg (getMaxHeadYRot), NBT horns default to false when the key is missing (getBoolean semantics).
Fixed in the re-review pass: `finalize_spawn` now ends in `finalize_spawn_super`, like vanilla's `super.finalizeSpawn`, so `AgeableMobGroupData` (group size, baby chance) advances and a group-spawned baby crosses the age boundary and loses its horns (test `group_spawn_baby_clears_horns`). Previously it called `finalize_spawn_base()` and returned the untouched group data, so goats never spawned as babies in a group and the group counter never incremented.

## Remaining gaps

| # | Location | Vanilla class | Suggested fix | Size |
|---|---|---|---|---|
| 1 | goat.rs (no equivalent) | GoatAi.initRamActivity / RamTarget / PrepareRamNearestTarget | Ram attack activity: PrepareRamNearestTarget (4..7 blocks, 20 ticks, 1.25 speed) then RamTarget (3.0 speed, knockback 2.5 adult / 1.0 baby, impact sound per screaming, horn-break sound). Cooldown RAM_COOLDOWN_TICKS 600..6000 (screamer 100..300). Needs a generic ram-goal port, not just goal wiring. | L |
| 2 | goat.rs (no equivalent) | GoatAi.initLongJumpActivity / LongJumpToRandomPos | Long jump: cooldown 600..1200, max height/width 5, velocity multiplier 3.5714288, LONG_JUMPING pose dimensions (0.9x1.3 scaled 0.7), GoatLongJump / GoatScreamingLongJump sound. Requires a long-jump goal. | L |
| 3 | goat.rs (no createHorn/dropHorn) | Goat.createHorn, Goat.dropHorn, RamTarget.hasRammedHornBreakingBlock | Horn item drop: pick a random instrument from InstrumentTags.REGULAR_GOAT_HORNS / SCREAMING_GOAT_HORNS using a random seeded from the goat UUID hash, spawn an ItemEntity offset by 0.3..0.7 on Y, and clear the horn data on ram into a horn-breaking block. | M |
| 4 | goat.rs:222 finalize_spawn | Goat.finalizeSpawn, GoatAi.initMemories | Memory init (long-jump and ram cooldowns) is absent; only relevant once #1/#2 land, fold into those. | S |
| 5 | goat.rs:307 goals (FollowParentGoal) | GoatAi.initIdleActivity BabyFollowAdult(UniformInt.of(5, 16), 1.25) | Pumpkin FollowParentGoal uses a fixed 3-block minimum follow distance; vanilla samples a 5..16 follow distance per goat. Add a configurable min-follow distance to FollowParentGoal. | M |
| 6 | goat.rs (no spawn rule) | Goat.checkGoatSpawnRules | Spawn only when the block below is in BlockTags.GOATS_SPAWNABLE_ON and light is high enough. `MINECRAFT_GOATS_SPAWNABLE_ON` is not in pumpkin-data yet (no tag.rs entry), so this is blocked on the extractor/data. | S |
| 7 | goat.rs (no equivalent) | Goat.handleEntityEvent(58/59), getRammingXHeadRot, aiStep lowerHeadTick | Head-lowering animation state (isLoweringHead, lowerHeadTick clamped 0..20, 30 deg of pitch). Server has no renderer; only the entity event broadcast matters. Client-only effect, not blocking. | S |
| 8 | goat.rs:314 mob_interact (bucket) | Goat.mobInteract -> this.playSound(getMilkingSound()) | Milk sound is played as `SoundCategory::Players` at the player position; vanilla plays the mob sound (`NEUTRAL`) at the goat. Shared with cow.rs (same block), so fix both together. | S |

Note: Pumpkin models goats with goals, vanilla uses a Brain (GoatAi). The goal set here matches the brain activities for panic (2.0), tempt (1.25), follow-adult (1.25), stroll (1.0), look-at-player (6.0) and look-around, so those are not gaps.
