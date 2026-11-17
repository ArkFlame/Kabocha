# Tracker: audit / crates/pumpkin-core/src/server/server_test_manager.rs

Deep audit of the GameTest server adapter. No TODO/FIXME/XXX markers exist; the two
"vanilla" markers (lines 44, 500) are accurate documentation, not stubs. Sources
reviewed: the file in full, `crates/pumpkin-gametest/src/world.rs` (trait contract),
`crates/pumpkin-world/src/tick/scheduler.rs`.

## Fixed in this pass

| Gap | Rust | Vanilla | Fix | Size |
|---|---|---|---|---|
| Scheduled fluid ticks were never cleared in the test area, so water/lava scheduled by a previous test run fired during the next run and made retries/adjacent tests flaky | `server_test_manager.rs:541-543` (before) | `TestInstanceBlockEntity.clearBlockTicks` calls `LevelChunk.getBlockTicks().clearArea(box)` **and** `getFluidTicks().clearArea(box)` | added `chunk.fluid_ticks.clear_area(min, max)` before the empty-chunk bookkeeping (which already tested `fluid_ticks.has_ticks()`) | S |

## Remaining gaps (not fixed)

| Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| Command source is `CommandSender::Console` (permission 4); vanilla runs test commands from a source at the test origin with `withSuppressedOutput()` and permission level 2, so permission-gated commands that vanilla rejects currently succeed | `server_test_manager.rs:894-925` | `GameTestHelper.runCommand` / `TestCommandHelper` build `CommandSourceStack` from `server.createCommandSourceStack().withLevel(level).withPosition(origin).withSuppressedOutput()` | add a permission-2, non-console `CommandSender` variant carrying the test position | S |
| Button detection is a heuristic (`LeverLikeProperties::handles_block_id`), not a button check | `server_test_manager.rs:845-860` | `GameTestHelper.pressButton` asserts `blockState.getBlock() instanceof ButtonBlock` | match the concrete button block family/registry like `pull_lever` matches `Block::LEVER` | S |
| `set_block_entity_nbt` removes and inserts the block entity without `update_block_entity`/chunk dirty, so the replacement may not be synced or saved | `server_test_manager.rs:462-486` | block entities from a structure NBT are loaded by the chunk placement (`BlockEntity.loadStatic` + `setChanged`) | after `add_block_entity`, call `self.world.update_block_entity(&entity)` | S |

## Verified matching vanilla (no change)

- Force-loaded test chunks: per-world refcounted lease with `was_forced` restore
  (`362-430`) matches `TestInstanceBlockEntity.forceLoadChunks` / `releaseChunks`
  without stealing pre-existing `/forceload` chunks.
- `clear_non_player_entities` (`488-522`) filters by bounding-box intersection over
  non-player entities, same as `level.getEntitiesOfClass(Entity.class, box, e -> !(e instanceof Player))`.
- `random_tick` (`876-892`) ticks the block and then the fluid state at the position,
  the same order as `ServerLevel.tickChunk`.
- `pull_lever` (`862-874`) mirrors `LeverBlock.pull`; `test_area_loaded_and_ticking`
  requires both loaded and active (ticking) chunks.
- Chunk coordinate math (`>> 4`, `(max-1) >> 4`) matches vanilla inclusive-maxible
  `BoundingBox` handling used by `StructureUtils`/`TestInstanceBlockEntity`.
