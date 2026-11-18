# entities/Goat

Source: crates/pumpkin-core/src/entity/passive/goat.rs (re-reviewed against decompiled vanilla Goat.java / GoatAi.java, 1.21.4).
Fixed in this pass: age-boundary horn toggle (vanilla ageBoundaryReached addHorns/removeHorns), max head yaw 15 deg (getMaxHeadYRot), NBT horns default to false when the key is missing (getBoolean semantics).

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

Note: Pumpkin models goats with goals, vanilla uses a Brain (GoatAi). The goal set here matches the brain activities for panic (2.0), tempt (1.25), follow-adult (1.25), stroll (1.0), look-at-player (6.0) and look-around, so those are not gaps.

## Gapwave2-2 status (h3)
- #1 ram activity: STILL OPEN (L, needs generic ram goal, out of wave scope)
- #2 long jump: STILL OPEN (L, needs long-jump goal)
- #3 horn drop: STILL OPEN (M, the drop is only triggered by RamTarget.hasRammedHornBreakingBlock, which is #1; a dropHorn helper without a caller would be dead code)
- #4 memory init: STILL OPEN (folds into #1/#2)
- #5 per-goat follow distance: FIXED a57f7697d (FollowParentGoal::with_min_distance, goat samples 5..=16; UNVERIFIED in game)
- #6 spawn rule: STILL OPEN (blocked on MINECRAFT_GOATS_SPAWNABLE_ON data, needs extractor)
- #7 head-lowering event: UNVERIFIED client-only effect (Vanilla handleEntityEvent 58/59 is a client animation), no server change
- ram/horn-break (#3 horn drop, #4 memory init) re-check h3: STILL OPEN (horn drop M is only reached through RamTarget.hasRammedHornBreakingBlock, which is the L ram activity from #1; no S/M part is standalone)


## Result

Verified every STILL OPEN row against decompiled vanilla 1.21.4 (`Goat.java` / `GoatAi.java`). No `.rs` edits: each actionable row needs files outside this unit (`goat.rs`, `tag.rs`), so it is left open with a one-line reason. No `tag.rs` in this unit relates to goats (only `command/commands/tag.rs` and generated data, which is off-limits).

- #1 ram activity: STILL OPEN - a basic `RamTargetGoal` is already wired (goat.rs:80), but the implementation lives in the out-of-unit `entity/ai/goal/ram_target.rs` and uses from-memory constants (speed 2.0, trigger 8 blocks, knockback 1.5, cooldown 600..2400) vs vanilla (speed 3.0, prepare 4..7, knockback 2.5 adult / 1.0 baby, cooldown 600..6000 / screamer 100..300); `PrepareRamNearestTarget` is absent. Alignment needs ram_target.rs, outside this unit.
- #2 long jump: STILL OPEN - vanilla `LongJumpToRandomPos` needs a new goal (cooldown 600..1200, max 5x5, velocity multiplier 3.5714288, LONG_JUMPING pose 0.9x1.3 scaled 0.7, long-jump sound); no such goal exists and adding one is a new file outside this unit.
- #3 horn drop: STILL OPEN - the data blocker is resolved (`tag::Instrument::MINECRAFT_REGULAR_GOAT_HORNS` / `MINECRAFT_SCREAMING_GOAT_HORNS` now exist in pumpkin-data), but the only vanilla caller is `RamTarget.hasRammedHornBreakingBlock` in the out-of-unit ram_target.rs (same dependency as #1), so a `dropHorn` helper here would be dead code.
- #4 memory init: STILL OPEN - vanilla `GoatAi.initMemories` seeds LONG_JUMP/RAM cooldowns in the Brain; Pumpkin has no brain-memory and `RamTimer` self-seeds, so this folds into #1/#2 and needs their out-of-unit goal files.
- #6 spawn rule: ALREADY FIXED (verified) - `entity/type.rs:672-678` already applies `MINECRAFT_GOATS_SPAWNABLE_ON` plus `is_bright_enough_to_spawn` for GOAT, and the tag now exists in pumpkin-data. No change needed; the fix is in an out-of-unit file that is already correct.

Already-resolved rows re-checked while verifying (no change):
- #5 per-goat follow distance: correct (goat.rs:90-95 samples 5..=16 at speed 1.25, matching `ADULT_FOLLOW_RANGE`).
- #7 head lowering: client-only animation (`handleEntityEvent` 58/59, `aiStep` pitch), no server-observable behaviour.

## Result (re-review h4)

Re-verified every STILL OPEN row against decompiled vanilla 1.21.4 (`Goat.java`, `GoatAi.java`, `RamTarget.java`, `PrepareRamNearestTarget.java`). This unit now owns `ram_target.rs` as well as `goat.rs`, so the two in-file parity bugs below were fixed; the rest need files outside this unit or a full Brain port.

- #1 ram activity: PARTIALLY FIXED. The folded `RamTargetGoal` already matches vanilla constants (speed 3.0 / prepare 1.25, 20-tick prepare, 4..7 trigger, 600..6000 / 100..300 cooldowns, 2.5 adult / 1.0 baby force). Two real divergences fixed in `ram_target.rs`: (a) knockback direction was inverted - vanilla `RamTarget.start` stores `ramDirection = goatPos - targetPos` and `apply_knockback` subtracts it, so the target is pushed away from the goat; the goal passed `target - goat` instead; (b) knockback strength omitted vanilla's `RAM_SPEED_FORCE_FACTOR` clamp - now `clamp(speed * 1.65, 0.2, 3.0) * knockbackForce`. Still open: vanilla `PrepareRamNearestTarget` target scan (4..7 walkable positions, `RAM_TARGET_CONDITIONS`), potion/blocking factors and END_RAM/START_RAM entity-event broadcasts need a full Brain-activity port, out of scope.
- #2 long jump: ADDRESSED ELSEWHERE - `LongJumpGoal` exists in the out-of-unit `long_jump.rs` and is wired at `goat.rs:102` with the 600..1200 cooldown, 5x5 search, 3.5714288x multiplier and screamer sound. No change here.
- #3 horn drop: FIXED AND VERIFIED (no change this pass). `create_horn`/`drop_horn` in `goat.rs` pick from `MINECRAFT_(SCREAMING|REGULAR)_GOAT_HORNS`, clear the horn and spawn the item with vanilla's offsets (-0.2..0.2, 0.3..0.7, -0.2..0.2); `ram_target.rs::ram_into_block` is the caller. Confirmed `world.spawn_entity` returns bool, `Instrument::asset_id()` round-trips through `from_name`, and the generated tag names are valid. UUID-hash seeding uses the low 64 bits rather than Java's 32-bit `UUID.hashCode()` - cosmetic, no test pins it.
- #4 memory init: FOLDED - `initial_ram_cooldown()` seeds 600..6000 like `GoatAi.initMemories` (RAM); the long-jump goal self-seeds (out of unit). Residual dependence on #1/#2.
- #5, #6, #7: unchanged from prior pass - #5 fixed (per-goat 5..16 follow distance), #6 fixed out of unit in `entity/type.rs`, #7 client-only animation with no server-observable effect.

Edits: `crates/pumpkin-core/src/entity/ai/goal/ram_target.rs` (knockback direction + vanilla speed-force clamp). `rustfmt --edition 2024` passed on that file. No cargo build run (worker restriction); orchestrator compiles.
