# audit/crates/pumpkin-core/src/server/server_test_manager.rs (DEEP AUDIT)

Server-side `GameTestWorld` adapter for Pumpkin's `pumpkin-gametest` runner.
No decompiled vanilla source is present in `.vendor`; comparisons below are against
`net.minecraft.gametest.framework.GameTestHelper` / `GameTestInfo` / `StructureUtils`
and `TestInstanceBlockEntity` (1.21.x).

## Markers
The tracker note ("2 TODO/FIXME/vanilla markers") points at prose comments, not TODOs.
There is no `TODO`, `FIXME`, `XXX` or `unimplemented!` in the file. The four "vanilla" notes are
design rationale and were checked:
- `server_test_manager.rs:44-47` force-load leases mirror `TestInstanceBlockEntity.forceLoadChunks`
  (chunk tickets are refcounted in `ChunkLoading::add_force_ticket`, so overlapping tests are safe;
  `/forceload` chunks are restored via `was_forced`). No gap.
- `server_test_manager.rs:313-315` per-position chunk fetch is required because `World::set_block_state`
  is sync; the async adapter provides vanilla's pre-loaded structure guarantee. No gap.
- `server_test_manager.rs:476-477` skipping uncreatable block entities matches structure placement.
  No gap.
- `server_test_manager.rs:501-502` player exclusion matches vanilla's `removeEntities` filter
  (`World::entities` never contains players). No gap.

## Verified correct (no change)
- Force-ticket release (`release_forced_game_test_chunk`, :398-430) pairs with acquire even when the
  chunk was already player/command-forced; `remove_entity` is idempotent via `removal_reason`, so
  `clear_non_player_entities` (:489-523) cannot double-remove.
- `item_count_in_box` (:720-748), `container_item_count` (:750-782), `is_container_empty` (:784-802)
  follow `GameTestHelper.assertItemEntityCountIs` / container assertions (matching item id, non-empty,
  box intersection on entity bounding boxes).
- `spawn_entity` (:644-669) sets `persistence_required` for mobs only, like `GameTestHelper.spawnEntity`.
- `set_block_entity_nbt` (:462-487) replaces silently (no `on_block_replaced` drop), matching
  `StructureTemplate::placeInWorld` block-entity replacement.
- `random_tick` (:877-893) ticks the block and then the fluid at the position, like vanilla
  `GameTestHelper.randomTick`.

## Remaining gaps
1. `use_block` is not implemented. `world.rs:161-165` supplies only the erroring trait default, so
   `GameTestHelper#useBlock` (vanilla: assert block, then a mock CREATIVE player right-clicks the block
   with the empty hand, running `BlockState#useItemOn`/`useWithoutItem`) always errors here
   (`server_test_manager.rs:433-928` implements every other trait method but this one). Vanilla tests
   that open doors/trapdoors/containers, light campfires, use composters or cauldrons depend on it.
   Fix: add a reusable mock player in `pumpkin-core` (dummy `ClientInformation`, `GameType::CREATIVE`,
   position at the test origin) and call `BlockRegistry::on_use` with a `BlockHitResult` aimed at the
   block face (fall back to `ItemRegistry::on_use` for the empty hand). Size M.
2. `spawn_item` (:671-695) passes `pickup_delay = 0` to `ItemEntity::new_with_velocity`, so spawned
   items are pickable on the first tick. Vanilla `GameTestHelper.spawnItem` builds a plain
   `ItemEntity`, whose constructor keeps `pickupDelay = 10`. Hopper/pickup tests therefore see items
   move one tick earlier than vanilla. Fix: pass `ItemEntity::DEFAULT_PICKUP_DELAY` (10) and re-time the
   built-in pickup tests if they depend on instant pickup. Size S, but touches built-in test timings,
   so it needs a full gametest run before landing.
3. `clear_scheduled_block_ticks` (:525-554) clears only `chunk.block_ticks`, yet the removal condition
   at :544 already consults `chunk.fluid_ticks`. If vanilla `GameTestInfo` clear after structure
   replacement covers scheduled fluid ticks too, fluid ticks from the replaced structure keep firing
   into the next attempt. Fix: add `chunk.fluid_ticks.clear_area(min, max);` and confirm against
   vanilla's tick-clearing call site. Size S (one line), needs the vanilla check first.
4. `press_button` / `pull_lever` (:846-875) call `ButtonBlock::press` / `LeverBlock::pull` directly
   instead of vanilla's `useBlock`, so `normal_use` overrides, statistics and per-player interaction
   side effects are skipped. Behaviourally equivalent for stock buttons/levers (both are `LeverLikeProperties`
   and the check rejects levers in `press_button`), so only custom/override blocks differ. Size S/M,
   fix arrives together with gap 1.
5. `execute_command` (:895-927) runs as a silent `CommandSender::Console` source. Vanilla
   `GameTestHelper.runCommand` uses the mock player's command source, which changes permission level,
   `@s`/`@p` selectors and feedback for tests that assert command output. Fix: reuse the mock player
   from gap 1 as the command source (keep the position override). Size M.

## Notes
- No `cargo` was run in this pass (worker constraint); every claim above is source-level.
- `spawn_entity` ignores `World::spawn_entity`'s `false` return (plugin `EntitySpawnEvent` cancellation);
  vanilla has no plugins, so this is not a parity gap but could silently hide a cancelled test entity.
