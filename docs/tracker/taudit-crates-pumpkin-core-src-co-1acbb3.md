# Audit: crates/pumpkin-core/src/command/commands/execute.rs

Category: audit (coverage AUDIT). Reference used: decompiled Paper/Canvas tree at
/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/server/commands/ExecuteCommand.java
(plus SummonCommand.java, Scoreboard.java, ObjectiveArgument.java). The tracker note
"8 TODO/FIXME/vanilla markers" is a false positive: the file has no TODO/FIXME; the 8 hits
are comments that mention vanilla behaviour.

## Fixed in this pass

- execute at: keeps the source rotation and level and only moves the position, as vanilla
  ExecuteCommand.at (withLevel + withPosition + withRotation). Initial removal of the
  rotation was reverted after checking the decompiled source (2f1164271).
- execute positioned pos: resets the source anchor to FEET, like vanilla withAnchor(FEET)
  (2e6c70a77).
- execute positioned over: honours the heightmap argument (motion_blocking,
  motion_blocking_no_leaves, world_surface) and rejects unloaded chunks with
  argument.pos.unloaded, like vanilla (6ee3371e7, 8efc56c66).
- execute if blocks: compares full block states instead of block types, enforces the
  vanilla 32768-block area limit with commands.execute.blocks.toobig and requires loaded
  start/end/destination positions (dd048d127, 8702a3c95).
- execute if/unless data: reports the number of matched NBT elements as the result value
  through NumericConditionalExecutor, like vanilla createNumericConditionalHandler
  (a9d41b00b).
- execute on origin: follows the traceable owner (projectile shooter) instead of the tame
  owner, like vanilla TraceableEntity.getOwner (aa7798a95).
- execute summon: adopts the new entity name and display name, like vanilla withEntity
  (0db49fa11).
- execute facing: both `facing <pos>` and `facing entity <targets> <anchor>` now start
  from the source anchor (`CommandSourceStack#getAnchor().apply`) and wrap yaw/pitch with
  Mth.wrapDegrees like `CommandSourceStack#facing`.
- execute if|unless block and biome: resolve the position through
  BlockPosArgument.getLoadedBlockPos, which raises argument.pos.unloaded for an unloaded
  chunk (ExecuteCommand.java:498,524), instead of silently reading air.
- execute if|unless blocks: start/end/destination now resolve through getLoadedBlockPos
  before the area check, matching ExecuteCommand.checkRegions (927-935).
- execute on controller: uses EntityBase.get_controlling_passenger (first player passenger
  of a boat, saddled mob, none for a plain entity) instead of the first passenger, like
  vanilla Entity.getControllingPassenger (Entity.java:6457, ExecuteCommand.java:1069).
  execute.rs:1490 (e952b4c51).
- execute summon: now runs the same spawnable-bounds and peaceful-difficulty checks as
  `/summon`, because vanilla routes both through SummonCommand.createEntity
  (SummonCommand.java:80-87). Hostile mobs are refused on Peaceful and out-of-world
  positions raise commands.summon.invalidPosition. execute.rs:1358, summon.rs:24,32
  (ac8bbcc26).
- execute positioned over: the loaded check is chunk-only (`Level#hasChunk`,
  ExecuteCommand.java:197) instead of resolving the block at the source y, so a source
  with a y outside the build height no longer fails on a loaded chunk. execute.rs:188
  (f3c8c19bb).

## Remaining gaps (larger than one small certain fix)

1. if blocks block-entity comparison - execute.rs:1139 count_blocks_match. Vanilla
   checkRegions also compares block entity type, components and saveCustomOnly NBT.
   Pumpkin only compares block state ids, so a chest with different contents passes.
   Fix: compare BlockEntity NBT via world.get_block_entity_nbt for both positions after
   stripping id/x/y/z. Size M.
2. if predicate reference and inline shape - execute.rs:242-260 test_predicate.
   LootPredicate::Reference always returns false with a debug log; vanilla
   checkCustomPredicate evaluates the loaded datapack predicate with ORIGIN and
   THIS_ENTITY loot params. Also a bare inline NBT list is treated as `all_of`, while
   vanilla parses an inline predicate as a single LootItemCondition and rejects a list.
   Needs a loaded predicate registry. Size M.
3. if items - missing from the if/unless tree (register, literal("if") block).
   Vanilla supports items over ItemCommands.SOURCE_PROVIDERS with SlotSourceArgument and
   ItemPredicateArgument. Pumpkin has no slot source argument. Size L.
4. if function - missing; vanilla executes a datapack function as a condition. Pumpkin
   has no datapack functions in this tree. Size L (blocked on functions).
5. on origin is exact only for projectiles - execute.rs:1508 relies on
   EntityBase.get_owner_entity, which only resolves projectile owners. Vanilla
   TraceableEntity also covers primed TNT, falling blocks and evoker fangs. Size M.
6. positioned over ocean_floor - execute.rs:194 falls back to world_surface because
   pumpkin-world stores only WorldSurface, MotionBlocking and MotionBlockingNoLeaves.
   Fix: add the ocean floor heightmap to pumpkin-world and map it here. Size M.
7. store ... score orphan objectives - execute.rs:596 write_stored_score returns early
   when the objective is not active, because the scoreboard cannot keep vanilla's orphan
   scores after an objective is recreated with the same name. Vanilla stores into the
   captured Objective regardless. Size M.
8. execute summon duplicate UUID - execute.rs:1358 spawns through world.spawn_entity and
   ignores the result. Upstream vanilla throws commands.summon.failed.uuid when the UUID
   already exists (this Paper tree disables that check with `if (false)`); Pumpkin's own
   /summon still raises it. Size S, needs agreement on the Paper vs vanilla behaviour.

## Gap status
- Fixed in this pass section: FIXED per note.
- 1. if blocks block-entity comparison (`execute.rs:1139`): STILL OPEN (M).
- 2. Predicate reference and inline shape (`execute.rs:242-260`): STILL OPEN (M).
- 3. `if items` missing: STILL OPEN (M).
- 4. `if function` missing: STILL OPEN (M).
- 5. `on origin` exact only for projectiles (`execute.rs:1508`): STILL OPEN (S).
- 6. Positioned over ocean_floor fallback (`execute.rs:194`): STILL OPEN (S).


## Result

- 1. if blocks block-entity comparison: FIXED. `count_blocks_match` now compares the block
  entity's `resource_location()` and custom `write_nbt` data (vanilla `saveCustomOnly`) for both
  positions when the source has one, after the state-id check (ExecuteCommand.java:977-1005).
  Pumpkin has no components API, so the custom NBT comparison covers the data; id/x/y/z are not
  written by `write_nbt`, so no stripping is needed.
- 2. Predicate reference and inline shape: STILL OPEN. `LootPredicate::Reference` needs a loaded
  datapack predicate registry, which Pumpkin does not have; this Paper/Canvas tree also comments
  out the whole `if predicate` branch (ExecuteCommand.java:697-709). Out of unit / needs data.
- 3. `if items` missing: STILL OPEN. Requires a new slot-source argument type (a file outside
  this unit); no such argument exists in Pumpkin.
- 4. `if function` missing: STILL OPEN. Pumpkin has no datapack functions in this command tree.
- 5. `on origin` exact only for projectiles: FIXED. Added `traceable_owner` in execute.rs; beside
  `get_owner_entity` (projectiles + item entities) it resolves the `EntityReference` of primed TNT,
  area effect clouds and evoker fangs, matching vanilla `TraceableEntity` implementors
  (TraceableEntity.java; ExecuteCommand.java:1071-1075). Vex has no owner tracking in Pumpkin yet.
- 6. Positioned over ocean_floor fallback: STILL OPEN. Needs an `OceanFloor` variant added to
  `ChunkHeightmapType` in pumpkin-world and mapped in `execute_positioned_over_modifier`;
  cross-crate change outside this unit.

No pure unit-test seam exists for these command modifiers (they need a live World/context), so
no tests were added. `rustfmt --edition 2024` was run on execute.rs. No compile was run per unit
rules; orchestrator batch-checks.


## Result (re-verification pass)

The earlier "## Result" block went stale: subsequent edits in this checkout had already
resolved every row it left open. I re-checked each against ExecuteCommand.java and the
current source, and confirmed every referenced symbol exists; no further source change was
needed this pass.

- 2. Predicate reference and inline shape: FIXED (already present). `test_predicate`
  (execute.rs:299) resolves `LootPredicate::Reference` through
  `server.datapack_manager.get_predicate` and evaluates it with
  `world::loot::evaluate_predicate_value` using ORIGIN/THIS_ENTITY/world parameters,
  matching vanilla `checkCustomPredicate` (ExecuteCommand.java:869-878). A bare inline
  list returns false, matching vanilla parsing of an inline predicate as a single
  LootItemCondition (the whole `if predicate` branch is commented out in this Canvas tree,
  ExecuteCommand.java:697-709).
- 3. `if items`: FIXED (already present). `literal("items")` tree at execute.rs:575 and
  :2051; `count_items_entity`/`count_items_block` sum the counts of predicate-matching
  stacks (vanilla `countItems`, ExecuteCommand.java:801-805). Slot mapping is covered by
  `items_equipment_slot_matches_vanilla_indices`. Note this Canvas tree disables `if items`
  with `if (true) break;` (ExecuteCommand.java:742); Pumpkin implements real vanilla
  behaviour, which is correct parity.
- 4. `if function`: FIXED (already present). `literal("function")` tree at execute.rs:2061
  and :2304; `function_condition_value` runs `execute_function_with` and returns the
  function result (vanilla `ExecuteIfFunctionCustomModifier`).
- 5. `on origin`: FIXED (already present). `traceable_owner` (execute.rs:1835) resolves
  `get_owner_entity` plus the `EntityReference` owners of TNT, area effect clouds and
  evoker fangs (vanilla `TraceableEntity`). Vex still has no owner tracking in Pumpkin.
- 6. Positioned over ocean_floor: FIXED (already present). `execute_positioned_over_modifier`
  maps `ocean_floor` to `world.get_ocean_floor_height` (world/mod.rs:3845), a live
  block-state scan matching vanilla `Heightmap.Types.OCEAN_FLOOR`. The helper already exists
  in pumpkin-core, so no cross-crate change was needed.

No .rs source file needed editing this pass, so none were touched and rustfmt was not
required. The only additional unit-test seam would be `test_predicate`/`count_items`/
`traceable_owner`, all of which need a live World/context; the existing slot-index test
covers the pure mapping. No compile was run per unit rules; the orchestrator batch-checks.

## Result (re-audit)

Re-checked each STILL OPEN row against the decompiled tree; several were already stale.

- 1. if blocks block-entity comparison: FIXED (already in code, verified against
  ExecuteCommand.java:977-1005). `count_blocks_match` compares state ids, then the source
  block entity's `resource_location()` and `write_nbt` payload against the destination's.
  Vanilla additionally compares data components, which Pumpkin has no API for.
- 2. Predicate reference and inline shape: FIXED. A referenced predicate is resolved through
  `datapack_manager.get_predicate` and evaluated with ORIGIN/THIS_ENTITY loot params, and an
  unknown id now raises `argument.resource_or_id.no_such_element` (registry
  `minecraft:predicate`) instead of silently failing, which is how vanilla's
  `ResourceOrIdArgument` rejects it at parse time. An inline bare SNBT list is still rejected
  because vanilla parses an inline predicate as a single `LootItemCondition`.
- 3. `if items` missing: STALE, already implemented. `items_conditional_node` registers both
  sources (entity, block) with the `slots` and `item_predicate` arguments and the numeric
  conditional executor, and parses in the existing parse test.
- 4. `if function` missing: STALE, already implemented via `function_condition_value` and
  `execute_function_with`, reporting the function's return value.
- 5. `on origin` exact only for projectiles: FIXED (already in code). `traceable_owner` covers
  projectiles and item entities through `get_owner_entity` and the `EntityReference` of
  primed TNT, area effect clouds and evoker fangs. Vex still has no owner tracking.
- 6. Positioned over ocean_floor fallback: FIXED. `World::get_ocean_floor_height` already
  scans the column with the `blocks_motion_in_heightmap` tag (live-world map, fluids
  excluded) and `execute_positioned_over_modifier` uses it; the `WorldSurface` fallback is
  gone. Heightmap ids are now resolved by `heightmap_kind`, which accepts exactly the four
  `keepAfterWorldgen` types and rejects `*_wg` ones like `HeightmapTypeArgument`.
- 7. store score orphan objectives: STILL OPEN (M). The scoreboard cannot retain scores whose
  objective was removed, so `write_stored_score` must skip an inactive objective;
  supporting it needs `world/scoreboard.rs`, outside this unit.

Added `positioned_over_accepts_only_kept_heightmaps`, a pure unit test over `heightmap_kind`
(the only seam here that does not need a live world). `rustfmt --edition 2024` run on
execute.rs; no compile run per unit rules, the orchestrator batch-checks.

## Result (this pass)

- 1. if blocks block-entity comparison: FIXED (already in code, verified against
  ExecuteCommand.java:977-1005). count_blocks_match compares state ids, then the source block
  entity's resource_location() and write_nbt payload against the destination's. Vanilla also
  compares data components, which Pumpkin has no API for.
- 2. Predicate reference and inline shape: FIXED. A referenced predicate is resolved through
  datapack_manager.get_predicate and evaluated with ORIGIN/THIS_ENTITY loot params; an unknown
  id now raises argument.resource_or_id.no_such_element (registry minecraft:predicate) like
  ResourceOrIdArgument does at parse time, instead of silently failing. An inline bare SNBT
  list is still rejected, since vanilla parses an inline predicate as one LootItemCondition.
- 3. if items missing: STALE, already implemented. items_conditional_node registers the entity
  and block sources with the slots and item_predicate arguments plus the numeric executor, and
  the parse test covers it.
- 4. if function missing: STALE, already implemented through function_condition_value and
  execute_function_with, reporting the function's return value.
- 5. on origin exact only for projectiles: FIXED (already in code). traceable_owner covers
  projectiles and item entities through get_owner_entity and the EntityReference of primed
  TNT, area effect clouds and evoker fangs. Vex still has no owner tracking.
- 6. Positioned over ocean_floor fallback: FIXED. World::get_ocean_floor_height already scans
  the column with the blocks_motion_in_heightmap tag (live-world map, fluids excluded) and the
  modifier uses it; the WorldSurface fallback is gone. Heightmap ids now resolve through
  heightmap_kind, which accepts exactly the four keepAfterWorldgen types and rejects the *_wg
  ones like HeightmapTypeArgument.
- 7. store score orphan objectives: STILL OPEN (M). The scoreboard cannot retain scores whose
  objective was removed, so write_stored_score must skip an inactive objective; the retention
  change lives in world/scoreboard.rs, outside this unit.
- 8. execute summon duplicate UUID: FIXED. ERROR_DUPLICATE_UUID is now pub in summon.rs and
  execute_summon_modifier refuses a UUID that is already live (as /summon does) before
  spawning.

Added positioned_over_accepts_only_kept_heightmaps, a pure unit test over heightmap_kind, the
one seam here that needs no live world. rustfmt --edition 2024 was run on execute.rs and
summon.rs; no compile was run per unit rules, the orchestrator batch-checks.

## Result (native-1)

- Row 4 (`if function` gap) and row 3 are unchanged from the previous pass.
- Row 7 (store score into orphan objective): BLOCKED-M. Vanilla `Scoreboard.removeObjective` removes the objective's scores (`.vendor/vanilla-decompiled/net/minecraft/world/scores/Scoreboard.java:302-314`) and a stored score can still target the orphan `Objective` instance. Pumpkin's `Scoreboard` keys scores by holder name and objective name, so an orphan score has no identity to live under. `write_stored_score` in `commands/execute.rs:938-968` therefore keeps skipping inactive objectives. Fix needs an objective-identity model in `world/scoreboard.rs`.

## Result (native-2 / final re-audit)

Re-checked every STILL OPEN row against the decompiled tree and the current checkout. The
native-1 "BLOCKED-M" note for row 7 went stale: an objective-identity model now exists in
`world/scoreboard.rs` (`ScoreboardObjective.identity` + `is_same`, `orphan_scores`,
`set_orphan_score`/`get_orphan_score`), and `write_stored_score` (execute.rs:938) writes into
the captured objective when it is no longer live. This matches vanilla
`storeValue` -> `getOrCreatePlayerScore(name, capturedObjective)` (ExecuteCommand.java:426-436,
Scoreboard.java:121-130,156), where a removed objective's score is still created on the
captured instance. All other rows (1-6, 8) remain implemented as logged previously.

- 1. if blocks block-entity comparison: FIXED (already in code). `count_blocks_match` compares
  state ids then source/destination `resource_location()` and `write_nbt` payload
  (ExecuteCommand.java:977-1005). No data-components API in Pumpkin.
- 2. Predicate reference/inline shape: FIXED (already in code). Reference predicates resolve via
  `datapack_manager.get_predicate` with ORIGIN/THIS_ENTITY params; an unknown id raises
  `argument.resource_or_id.no_such_element`; a bare inline list matches nothing.
- 3. `if items` / 4. `if function`: STALE, already implemented and covered by the parse tests.
- 5. `on origin`: FIXED (already in code). `traceable_owner` covers projectiles/items via
  `get_owner_entity` plus the `EntityReference` of TNT, area effect clouds and evoker fangs.
  Vex still has no owner tracking in Pumpkin.
- 6. Positioned over ocean_floor: FIXED (already in code). `get_ocean_floor_height` scans the
  column with the `blocks_motion_in_heightmap` tag; `heightmap_kind` accepts only the four
  kept heightmaps.
- 7. store score orphan objectives: FIXED (identity model present). Additionally fixed a latent
  bug in `load_saved_nbt` (world/scoreboard.rs:1013): it cleared objectives/scores/display
  slots/teams but left stale `orphan_scores`, contradicting its "replaces the contents" contract.
  Added `self.orphan_scores.clear();`.
- 8. execute summon duplicate UUID: FIXED (already in code). `ERROR_DUPLICATE_UUID` refusal in
  `execute_summon_modifier`.

Added the pure unit test `loading_saved_nbt_discards_orphan_scores` in execute.rs (the only seam
that needs no live world for the scoreboard change). `unwrap` in tests is allowed by
`#![cfg_attr(test, allow(clippy::unwrap_used, ...))]` in pumpkin-core/src/lib.rs. No compile run
per unit rules; orchestrator batch-checks. No em dashes, no println, no generated/WIT/data edits.
\n## Result (verification pass)

- 1. if blocks block-entity comparison: FIXED. count_blocks_match (execute.rs:1484) compares state ids, then source/destination block entity resource_location() and write_nbt payload. Vanilla also compares data components, which Pumpkin has no API for. No further changes needed.
- 2. Predicate reference and inline shape: FIXED. test_predicate (execute.rs:328) resolves LootPredicate::Reference through datapack_manager.get_predicate with ORIGIN/THIS_ENTITY loot params. An unknown id raises argument.resource_or_id.no_such_element; a bare inline list matches nothing (vanilla parses inline predicate as one LootItemCondition).
- 3. if items missing: FIXED (already implemented). literal("items") tree registers entity and block sources with slots and item_predicate arguments plus the numeric conditional executor. Parse test covers it.
- 4. if function missing: FIXED (already implemented). function_condition_value and literal("function") trees report the function's return value. Covered by parse tests.
- 5. on origin exact only for projectiles: FIXED. traceable_owner (execute.rs:1880) covers projectiles and item entities through get_owner_entity plus the EntityReference of primed TNT, area effect clouds and evoker fangs. Vex still has no owner tracking in Pumpkin.
- 6. Positioned over ocean_floor fallback: FIXED. get_ocean_floor_height scans the column with the blocks_motion_in_heightmap tag; execute_positioned_over_modifier uses it. Heightmap ids now resolve through heightmap_kind, which accepts exactly the four keepAfterWorldgen types and rejects *_wg ones.
- 7. store score orphan objectives: FIXED. Identity model in world/scoreboard.rs (ScoreboardObjective.identity + is_same, orphan_scores, set_orphan_score/get_orphan_score) enables write_stored_score to write into the captured objective when it is no longer live, matching vanilla storeValue -> getOrCreatePlayerScore. Added self.orphan_scores.clear() fix in load_saved_nbt. Pure unit test loading_saved_nbt_discards_orphan_scores added in execute.rs.

No source files needed editing this pass. rustfmt --edition 2024 was previously run on execute.rs and summon.rs. No compile run per unit rules; orchestrator batch-checks.
## Result (coder pass 2, batch r2_003)
- 1. if blocks block-entity comparison (`execute.rs:1139`): STILL OPEN, not reached.
- 2. Predicate reference and inline shape (`execute.rs:242-260`): STILL OPEN, not reached.
- 3. `if items` missing: STILL OPEN, not reached (needs slot-source argument type).
- 4. `if function` missing: STILL OPEN, not reached (no datapack functions in command tree).
- 5. `on origin` exact only for projectiles (`execute.rs:1508`): STILL OPEN, not reached.
- 6. Positioned over ocean_floor fallback (`execute.rs:194`): STILL OPEN, not reached.
- 7. store score orphan objectives: STILL OPEN, not reached.

## Result (coder pass 3, batch r3_017)

Re-derived rows 1-8 from the Gap table and Remaining gaps list. Each row was checked by identifier grep in the Rust and by vanilla anchor lines in `.vendor/vanilla-decompiled`. No Rust edits this pass.

- 1. if blocks block-entity comparison: VERIFIED. `count_blocks_match` (crates/pumpkin-core/src/command/commands/execute.rs:1484, write_nbt at 1539-1540) vs vanilla ExecuteCommand.java:995,1000 (saveCustomOnly).
- 2. Predicate reference and inline shape: VERIFIED. `test_predicate` (execute.rs:328) uses `datapack_manager.get_predicate` (execute.rs:342) and `argument.resource_or_id.no_such_element` (execute.rs:148) vs vanilla ExecuteCommand.java:869 checkCustomPredicate.
- 3. if items: VERIFIED. `items_conditional_node` (execute.rs:595, registered at 2092 and 2335).
- 4. if function: VERIFIED. `function_condition_value` (execute.rs:381) used at 421 and 431.
- 5. on origin exact only for projectiles: VERIFIED. `traceable_owner` (execute.rs:1880) vs vanilla ExecuteCommand.java:1074 TraceableEntity.getOwner.
- 6. Positioned over ocean_floor fallback: VERIFIED. `heightmap_kind` (execute.rs:248), `HeightmapKind::OceanFloor` -> `World::get_ocean_floor_height` (execute.rs:291; crates/pumpkin-core/src/world/mod.rs:3871).
- 7. store score orphan objectives: VERIFIED. `write_stored_score` (execute.rs:938, set_orphan_score at 962), identity model in crates/pumpkin-core/src/world/scoreboard.rs:394,418,1010,1210 vs vanilla ExecuteCommand.java:432 getOrCreatePlayerScore.
- 8. execute summon duplicate UUID: VERIFIED. `ERROR_DUPLICATE_UUID` refusal (execute.rs:1742; summon.rs:35) vs vanilla SummonCommand.java:34.

Checks: grep and read only. No cargo run (forbidden by brief). Verification is by identifier presence and anchor lines, not a full semantic diff of each body. Rows 2 and 8 rest on the doc's own note about Paper vs vanilla UUID handling. I did not re-confirm that note.
