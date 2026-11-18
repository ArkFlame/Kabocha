# audit / crates/pumpkin-core/src/entity/ai/brain/memory/types.rs

Reference: `.vendor` absent; compared against the decompiled tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/entity/ai/memory/MemoryModuleType.java`
(115 registered types). Every entry in `types.rs` was diffed by name, declaration order
and codec presence against that file.

Fixed in this pass:

- `types.rs:86` - `IS_TEMPTED` (`minecraft:is_tempted`) does not exist in vanilla
  `MemoryModuleType`; temptation uses `TEMPTING_PLAYER`/`TEMPTATION_COOLDOWN_TICKS`
  (`TemptGoal` writes `TEMPTING_PLAYER`). The extra entry shifted every later id by one.
  Removed, registry renumbered to 115 entries in vanilla order, persistent codec count
  53 -> 52. After the change all 115 remaining names, ids and codec presence match vanilla.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `types.rs:121-122` (`VISIBLE_ADULT_PIGLIN_COUNT`, `VISIBLE_ADULT_HOGLIN_COUNT`) | `PiglinSpecificSensor.java:107-108` writes both counts from the visible adult lists; `PiglinAi.java:780-781` and `HoglinAi.java:179-180` read them to make the outnumbered side retreat | Nothing writes or reads the counts (the `// TODO` at `types.rs:120`) | Port `PiglinSpecificSensor` and the two `retreatFromNearestTarget` call sites | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `types.rs:118-122` (`NEARBY_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_HOGLINS`, `NEAREST_VISIBLE_ADULT_PIGLIN`) | Written by `PiglinSpecificSensor`/`HoglinSpecificSensor` | Declared but never populated; `crates/pumpkin-core/src/entity/ai/sensing.rs` has no piglin/hoglin specific sensor | Port the two species sensors together with the counts above | L, STILL OPEN: size L, skipped per wave brief |

Checked and considered correct: all other 112 type names, their declaration order, and
codec presence (global pos, global pos set/list, block pos, uuid, int, long, bool, unit)
match vanilla; types without a codec in vanilla also have `None` here.

## Gap status
- VISIBLE_ADULT_PIGLIN/HOGLIN_COUNT never written or read (types.rs:120-122): STILL OPEN, M, needs PiglinSpecificSensor and HoglinAi/PiglinAi call sites.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_* never populated (types.rs:118-122): STILL OPEN, L, no piglin/hoglin specific sensor in sensing.rs.

## Result

Both remaining rows were verified against decompiled vanilla
(`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/sensing/PiglinSpecificSensor.java`
and `HoglinSpecificSensor.java`). Both need files outside this unit
(`entity/ai/brain/sensing/mod.rs`, new species sensor files there, the piglin/hoglin
brain sensor registration, and their `retreatFromNearestTarget` AI call sites), so
neither could be fixed within `ai/sensing.rs` (only a line-of-sight cache) or
`brain/memory/types.rs` (declarations already match all 115 vanilla entries).

- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN. Needs
  PiglinSpecificSensor/HoglinSpecificSensor writing both brain memories plus the
  PiglinAi/HoglinAi retreat call sites; all outside this unit's two files.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS /
  NEAREST_VISIBLE_ADULT_HOGLINS / NEAREST_VISIBLE_ADULT_PIGLIN (L): STILL OPEN. Needs
  the two species sensors wired into the piglin/hoglin brain in `brain/sensing/`;
  outside this unit. Piglins/hoglins currently use goal-based AI with no brain sensors.

No source files changed; no rustfmt needed.

## Result (2026-11-18, re-verified against .vendor/vanilla-decompiled, source fixed)

Re-read `HoglinSpecificSensor.java` and `MemoryModuleType.java` and compared with the
current `hoglin_specific.rs`, `hoglin.rs`, `sensing.rs` and `types.rs`.

Verified the hoglin half of both STILL OPEN rows is already wired: the counts
(`VISIBLE_ADULT_PIGLIN_COUNT`, `VISIBLE_ADULT_HOGLIN_COUNT`) and the
`NEAREST_VISIBLE_ADULT_PIGLIN`/`NEAREST_VISIBLE_ADULT_HOGLINS` lists are written by
`hoglin_specific.rs` and read by `hoglin.rs:43` (`piglins_outnumber_hoglins`). The stale
`// TODO` at `types.rs:120` is already gone. No change needed in `types.rs` (all 115
entries match vanilla) or `sensing.rs` (line-of-sight cache only).

Fixed one real parity gap inside this unit's own file: vanilla
`HoglinSpecificSensor.requires()` lists `NEAREST_REPELLENT` (6 memories) and `doTick`
sets it from `findBlocksInBoxByManhattanDistance(pos, 8, 4)` filtered by
`BlockTags.HOGLIN_REPELLENTS`. `hoglin_specific.rs` omitted it. Added
`types::NEAREST_REPELLENT` to `REQUIRES`, added a `find_nearest_repellent` helper with
the same 8/4 box and Manhattan tie-break, and set the memory each scan using
`Block::MINECRAFT_HOGLIN_REPELLENTS`. Added one pure unit test
`nearest_repellent_prefers_smaller_manhattan_distance`.

Remaining open (both still need files outside this unit, so left open):
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN only for the
  piglin side; needs `PiglinSpecificSensor` writes and the `PiglinAi` retreat call site,
  both outside this unit.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): STILL OPEN; needs the
  `PiglinSpecificSensor` brain sensor (outside this unit). The hoglin memories are done.

## Result (re-verified 2026-11-18 against .vendor/vanilla-decompiled)

The decompiled tree is now present at `.vendor/vanilla-decompiled/net/minecraft/`.
Verified `world/entity/ai/memory/MemoryModuleType.java`: 115 registrations in the
same order as `types.rs` (`DUMMY`=0 through `BREEZE_LEAVING_WATER`=114); no
`IS_TEMPTED`; `NEARBY_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_*`,
`VISIBLE_ADULT_PIGLIN_COUNT`/`VISIBLE_ADULT_HOGLIN_COUNT` and
`NEAREST_VISIBLE_ZOMBIFIED` are all registered without a codec, matching the
`None` entries in `types.rs`. Both declaration files in this unit are already
correct; the two STILL OPEN rows need write/read sites that live in other files.

- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN.
  Vanilla writes both in `PiglinSpecificSensor.java:107-108` and
  `HoglinSpecificSensor.java:56-57`, and reads them in the `PiglinAi`/`HoglinAi`
  retreat logic. The hoglin write now exists in `brain/sensing/hoglin_specific.rs`
  (outside this unit); the piglin write and both retreat call sites are also
  outside this unit's two files. No seam exists in `types.rs` or `ai/sensing.rs`.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): STILL OPEN.
  Written only by `PiglinSpecificSensor.java:105-106` in `brain/sensing/` (outside
  this unit); `NEAREST_VISIBLE_ADULT_HOGLINS`/`NEAREST_VISIBLE_ADULT_PIGLIN` are
  now written by `brain/sensing/hoglin_specific.rs`. The piglin species sensor is
  absent and lives outside this unit; `ai/sensing.rs` is only a line-of-sight cache.

No source files changed; no rustfmt needed.


## Result (2026-11-18, re-verified against .vendor/vanilla-decompiled, no in-scope fix)

Re-read `PiglinSpecificSensor.java`, `HoglinSpecificSensor.java`, `HoglinAi.java` and the
current `hoglin.rs`, `hoglin_specific.rs`, `sensing.rs` and `types.rs`.

Verified both STILL OPEN rows are the piglin half only; the hoglin half is complete:
- `hoglin_specific.rs` writes NEAREST_REPELLENT, NEAREST_VISIBLE_ADULT_PIGLIN,
  NEAREST_VISIBLE_ADULT_HOGLINS, VISIBLE_ADULT_PIGLIN_COUNT and
  VISIBLE_ADULT_HOGLIN_COUNT with the same filters/8-4 box as vanilla; it is wired into
  the hoglin brain (`hoglin.rs make_brain` registers SensorType::HoglinSpecific).
- `hoglin.rs` reads both counts in `piglins_outnumber_hoglins` and
  `outnumbered_by_piglins(p, h) = p > h + 1` matches vanilla `HoglinAi.java:179-181`
  (memory count + 1 for self). No change needed.
- `types.rs` declares all 115 vanilla memory types with matching ids/codecs; the
  count/list/zombified/nemesis entries this unit owns are all present. `sensing.rs`
  is only a line-of-sight cache. No seam for a fix here.

Remaining open (both need files outside this unit, so left open):
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN for the
  piglin side only. Needs a `PiglinSpecificSensor` (new file in `brain/sensing/`) writing
  both memories and the `PiglinAi` retreat call site, both outside this unit.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): STILL OPEN. Written only by
  `PiglinSpecificSensor.java:105-106` in `brain/sensing/` (outside this unit); the
  hoglin equivalents are done. No piglin brain sensor exists.

No source files changed; no rustfmt needed.


## Result (2026-11-18, final re-verify; decompiled tree present)

Re-checked both STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/`
and the four files in this unit (`hoglin.rs`, `hoglin_specific.rs`, `ai/sensing.rs`,
`brain/memory/types.rs`). The hoglin half of both rows is complete and correct:
`HoglinSpecificSensor.doTick` writes NEAREST_REPELLENT plus the four adult/list/count
memories; `hoglin.rs` reads both counts in `piglins_outnumber_hoglins`
(`piglin_count > hoglin_count + 1`, matching `HoglinAi.java:174-182`) and registers
`SensorType::HoglinSpecific`; `types.rs` declares all entries with vanilla ids and
`None` codecs; `ai/sensing.rs` is a line-of-sight cache. No change needed in any of
the four files.

Both rows remain STILL OPEN only on the piglin side and need files outside this unit:
- VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT (M): STILL OPEN. Needs
  `PiglinSpecificSensor` to write both counts (`PiglinSpecificSensor.java:107-108`)
  and the `PiglinAi` retreat call site, in `brain/sensing/` and `mob/piglin.rs`,
  outside this unit.
- NEARBY_ADULT_PIGLINS / NEAREST_VISIBLE_ADULT_PIGLINS (L): STILL OPEN. Needs the
  `PiglinSpecificSensor` brain sensor (`PiglinSpecificSensor.java:105-106`), a new
  `SensorType` variant and piglin brain wiring, all outside this unit; `ai/sensing.rs`
  offers no seam.

No source files changed; no rustfmt needed.


## Result (2026-11-18, second pass with cross-file edits; vanilla .vendor re-verified)

Re-read `PiglinSpecificSensor.java`, `PiglinAi.java` (retreat/anger broadcast and helper
section) and Pumpkin's `piglin.rs`, `piglin_ai.rs`, `hoglin.rs` and `sensing/`. Both
STILL OPEN rows are the piglin half; Pumpkin piglins use goal AI, so the sensor's effect
is reproduced with entity fields rather than brain memories (the hoglin half already uses
brain memories via `hoglin_specific.rs`).

- Row 1 (VISIBLE_ADULT_PIGLIN_COUNT / VISIBLE_ADULT_HOGLIN_COUNT, M): FIXED (already in
  tree, verified). `piglin.rs::refresh_visible_adult_counts` computes both counts from
  visible, non-baby PIGLIN/PIGLIN_BRUTE and HOGLIN in follow range with a line-of-sight
  clip, matching `PiglinSpecificSensor.doTick`; `piglin_ai.rs::hoglins_outnumber_piglins`
  reads them (`hoglin_count > piglin_count + 1`, matching `PiglinAi.java:779-783`).
  The stale `types.rs` TODO cited in the row is long gone; `types.rs` needs no change.
- Row 2 (NEAREST_VISIBLE_ADULT_PIGLINS / NEARBY_ADULT_PIGLINS, L): partially fixed.
  Added `PiglinEntity.visible_adult_piglins: Mutex<Vec<Weak<dyn EntityBase>>>`, populated
  in `refresh_visible_adult_counts` from the same visible-adult scan (the functional
  equivalent of writing `NEAREST_VISIBLE_ADULT_PIGLINS`), and made `was_hurt_by` consume
  it by broadcasting the hoglin retreat to every visible adult piglin, matching
  `PiglinAi.broadcastRetreat`/`retreatFromNearestTarget` (`PiglinAi.java:747-761,605`).
  Extracted the shared effect into `PiglinEntity::start_retreat_from_hoglin`. Added pure
  behaviour test `retreat_broadcasts_to_visible_adult_piglins`.
  NEARBY_ADULT_PIGLINS remains open: its only vanilla consumer is
  `PiglinAi.broadcastAngerTarget` (`PiglinAi.java:659-660,688-697`), which needs a whole
  anger/retaliation memory subsystem Pumpkin's goal AI does not have. Not adding an
  unpopulated/unread field.

Files changed: `crates/pumpkin-core/src/entity/mob/piglin.rs` only (plus this doc).
No generated pumpkin-data, WIT or cross-crate changes. Not compiled by worker (orchestrator
batch-check). Skipped: the anger-broadcast subsystem (missing subsystem, out of scope).
