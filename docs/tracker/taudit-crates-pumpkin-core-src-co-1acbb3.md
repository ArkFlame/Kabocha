# Audit: crates/pumpkin-core/src/command/commands/execute.rs

Category: audit (coverage AUDIT). Reference used: decompiled Paper/Canvas tree at
/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/server/commands/ExecuteCommand.java
(plus Entity.java getControllingPassenger). The tracker note "8 TODO/FIXME/vanilla markers"
is a false positive: the file has no TODO/FIXME; the 8 hits are comments that mention
vanilla behaviour.

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
  from the source anchor (`CommandSourceStack#getAnchor().apply`, CommandSourceStack.java:372)
  and wrap yaw/pitch with Mth.wrapDegrees like `CommandSourceStack#facing`. Before, an
  `execute anchored eyes facing ...` aimed from the feet and could return yaw -225 instead
  of 135 (execute.rs:321 anchor_position, used at 337 and 362, wrap at 340/365).
- execute if|unless block and biome: resolve the position through
  BlockPosArgument.getLoadedBlockPos, which raises argument.pos.unloaded for an unloaded
  chunk (ExecuteCommand.java:498,524), instead of silently reading air. execute.rs:394, 490, 507.
- execute if|unless blocks: start/end/destination now resolve through getLoadedBlockPos
  before the area check, matching ExecuteCommand.checkRegions (ExecuteCommand.java:930-932).

## Remaining gaps (larger than one small certain fix)

1. if blocks block-entity comparison - execute.rs:1122 count_blocks_match. Vanilla
   checkRegions also compares block entity type, components and saveCustomOnly NBT.
   Pumpkin only compares block state ids, so a chest with different contents passes.
   Fix: compare BlockEntity NBT via world.get_block_entity_nbt for both positions after
   stripping id/x/y/z. Size M.
2. if predicate reference - execute.rs:238-256 test_predicate. LootPredicate::Reference
   always returns false with a debug log; vanilla checkCustomPredicate evaluates the
   loaded datapack predicate with ORIGIN and THIS_ENTITY loot params. Needs a loaded
   predicate registry. Size M.
3. if items - missing from the if/unless tree (register, literal("if") block).
   Vanilla supports items over ItemCommands.SOURCE_PROVIDERS with SlotSourceArgument and
   ItemPredicateArgument. Pumpkin has no item predicate/slot source arguments. Size L.
4. if function - missing; vanilla executes a datapack function as a condition. Pumpkin
   has no datapack functions in this tree. Size L (blocked on functions).
5. on controller - execute.rs:1479 uses the first passenger; vanilla uses
   Entity.getControllingPassenger, which is null unless the entity can actually be
   controlled (boat, minecart, rideable mob). Size S.
6. on origin is exact only for projectiles - execute.rs:1491 relies on
   EntityBase.get_owner_entity, which only resolves projectile owners. Vanilla
   TraceableEntity also covers primed TNT, falling blocks and evoker fangs. Size M.
7. positioned over ocean_floor - execute.rs:184 falls back to world_surface because
   pumpkin-world stores only WorldSurface, MotionBlocking and MotionBlockingNoLeaves.
   Fix: add the ocean floor heightmap to pumpkin-world and map it here. Size M.
8. store ... score orphan objectives - execute.rs:579 write_stored_score returns early
   when the objective is not active, because the scoreboard cannot keep vanilla's orphan
   scores after an objective is recreated with the same name. Vanilla stores into the
   captured Objective regardless. Size M.
