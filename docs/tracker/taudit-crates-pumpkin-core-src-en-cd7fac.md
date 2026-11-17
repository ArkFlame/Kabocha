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
| `types.rs:121-122` (`VISIBLE_ADULT_PIGLIN_COUNT`, `VISIBLE_ADULT_HOGLIN_COUNT`) | `PiglinSpecificSensor.java:107-108` writes both counts from the visible adult lists; `PiglinAi.java:780-781` and `HoglinAi.java:179-180` read them to make the outnumbered side retreat | Nothing writes or reads the counts (the `// TODO` at `types.rs:120`) | Port `PiglinSpecificSensor` and the two `retreatFromNearestTarget` call sites | M |
| `types.rs:118-122` (`NEARBY_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_PIGLINS`, `NEAREST_VISIBLE_ADULT_HOGLINS`, `NEAREST_VISIBLE_ADULT_PIGLIN`) | Written by `PiglinSpecificSensor`/`HoglinSpecificSensor` | Declared but never populated; `crates/pumpkin-core/src/entity/ai/sensing.rs` has no piglin/hoglin specific sensor | Port the two species sensors together with the counts above | L |

Checked and considered correct: all other 112 type names, their declaration order, and
codec presence (global pos, global pos set/list, block pos, uuid, int, long, bool, unit)
match vanilla; types without a codec in vanilla also have `None` here.
