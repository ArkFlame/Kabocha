# Parity rows for crates/pumpkin-core/src/entity/ai/goal/ram_target.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-goat** [partial/OWN-OPEN] own audit: 14 open, 6 fixed, 8 verified, 0 blocked-data (docs/tracker/tentities-goat.md) docs/tracker/tentities-goat.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, final)

Re-verified every row of `docs/tracker/tentities-goat.md` against decompiled vanilla 1.21.4.
This unit owns `ram_target.rs`; the substantive ram gaps were closed in the previous pass and no
new parity bug was found. One defensive cleanup was applied: the `match self.candidate` with an
`#[expect(clippy::single_match_else)]` was rewritten as `if let ... else` (behaviour identical),
removing a possible unfulfilled-expectation error under `-D warnings`. Braces re-balanced, file
re-read, no cargo run (worker restriction); orchestrator compiles.

- #1 ram activity: FIXED / VERIFIED. `ram_target.rs` matches
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/behavior/PrepareRamNearestTarget.java`
  (`calculateRammingStartPosition` 139-168, `isWalkableBlock` 170-172, `tick` 105-130) and
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/behavior/RamTarget.java`
  (`start` 72-78, `tick` 81-122, `finishRam` 131-135). Confirmed: 4..7 four-direction walkable
  scan, min distance 4, squared-distance sort, reachable-path filter, 20-tick prepare, 3.0 ram /
  1.25 prepare speed, 2.5 adult / 1.0 baby knockback, 1.65 speed-force factor clamped 0.2..3.0,
  0.25*(speed-slowness) potion boost, 0.5 shield blocking factor, ramDirection = goat - target
  (push away), START_RAM/END_RAM broadcasts, and 600/100 fail cooldown. Constants match
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/goat/GoatAi.java` 36-59.
- #2 long jump: VERIFIED, no change. `long_jump.rs` matches
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/behavior/LongJumpToRandomPos.java`.
- #3 horn drop: VERIFIED, no change. `goat.rs::create_horn`/`drop_horn` and
  `ram_target.rs::ram_into_block` match `RamTarget.hasRammedHornBreakingBlock` (124-129).
- #4 memory init: VERIFIED folded, no change. `initial_ram_cooldown()` = 600..6000, per
  `GoatAi.initMemories` 61-64.
- #5 per-goat follow distance: VERIFIED, no change (goat.rs samples 5..=16 at 1.25, matching
  `GoatAi.ADULT_FOLLOW_RANGE` line 38).
- #6 spawn rule: VERIFIED already fixed out of unit in `crates/pumpkin-core/src/entity/type.rs`
  (GOATS_SPAWNABLE_ON tag + brightness).
- #7 head lowering: VERIFIED client-only; START_RAM/END_RAM broadcasts drive the client animation.

No BLOCKED rows and no missing generated data.

Status: UNVERIFIED fixed=1 open=0 files=ram_target.rs
