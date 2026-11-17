# blocks / Pistons (RE-REVIEW)

Reviewed 2026-11-16 against master `d28575773`. Scope:
`crates/pumpkin-core/src/block/blocks/piston/{mod.rs,piston.rs,piston_head.rs,piston_extension.rs}`,
`crates/pumpkin-core/src/block/entities/piston.rs`.
Reference: `PistonBaseBlock`, `PistonHeadBlock`, `MovingPistonBlock`, `PistonStructureResolver`,
`PistonMovingBlockEntity` (no decompiled sources in this checkout, behaviour from the vanilla class).

## Verified matching

- `isMovable` exclusions: obsidian/crying obsidian/respawn anchor/reinforced deepslate, build-height
  edges, hardness -1, piston behaviour Block/Destroy/PushOnly, extended pistons, block entities
  (`piston.rs:45-90`).
- `PistonStructureResolver` equivalent: 12-block limit, `set_moved_blocks` collision reorder,
  honey/slime stuck rules, adjacent-block pull (`mod.rs:15-236`).
- Block events/triggers extend(0)/contract(1)/drop(2), plugin events, sounds, game events
  (`piston.rs:145-310`).
- Head validity incl. moving-piston base, head/base mutual removal, creative dropless removal
  (`piston_head.rs:19-137`).
- Moving-piston entity: chunk NBT, progress tick, `finish()` block replacement, neighbour shape
  update, entity push with head slab AABB and push-out-of-body (`entities/piston.rs:25-266`).
- Issue #3833 (down-facing piston powered from the head): the head direction is skipped at
  `piston.rs:318`, so the reported case looks covered. Needs an in-game re-check, no code change.

## Gaps

1. FIXED `piston_extension.rs:15-30` - `MovingPistonBlock.playerWillDestroy` is creative-only and
   uses `level.setBlock(air, 3)`. Rust removed the base for every player (a survival break of the
   moving-piston block destroyed the piston base) and used `break_block` with `SKIP_DROPS`, which
   fires a break event. Now gated on `GameMode::Creative` and uses `set_block_state(AIR,
   NOTIFY_ALL)`. Size S.

2. FIXED `piston.rs:282-298` - `PistonBaseBlock.moveBlocks` only clears the block in the head slot
   when it `is(Blocks.PISTON_HEAD)`; both retract branches set that position to air unconditionally,
   deleting any foreign block sitting there (e.g. after the head was replaced). Now guarded by a
   `PISTON_HEAD` check. Size S.

3. `piston.rs:94-111` - `PistonBaseBlock.playerWillDestroy` only removes the head when
   `player.getAbilities().instabuild`; Rust breaks the head for every player and fires a break
   event. Vanilla survival relies on `PistonHeadBlock.canSurvive`/`updateShape` (already implemented
   at `piston_head.rs:74-107`) to turn the head into air. Suggested fix: return early unless
   `GameMode::Creative` and use `set_block_state(AIR, NOTIFY_ALL)` so no break event fires for the
   head. Left unfixed: this is the path of the recent #3900/#3144 fix and needs an in-game check
   that the neighbour shape update still clears the head in survival. Size S.

4. `piston.rs:54-73` - `isMovable` misses vanilla's world-border rejection
   (`world.getWorldBorder().isWithinBounds(pos)`), so blocks can be pushed past the world border.
   `Worldborder::contains` exists at `world/border.rs:184`; `World::worldborder` is a `Mutex`
   (`world/mod.rs:315`). Size S.

5. `entities/piston.rs:309-312` - `update_packet_nbt` returns `None`. `PistonMovingBlockEntity`
   has no update-packet override in vanilla, so a client receiving the block entity outside the
   block-event animation window gets the saved tag (`blockState`/`facing`/`progress`/`source`) from
   chunk data; with `None` such a client renders progress 0 and facing Down. Candidate root cause
   for the remaining half of #3144/#3900 (flying-machine desync). Suggested: return
   `self.chunk_data_nbt()`, verify no double animation in-game. Size M.

6. `entities/piston.rs:298` - `write_nbt` stores `last_progress` under `progress`; vanilla writes
   `progressO` (previous-tick) there, so this is likely correct. Only worth touching together with
   gap 5 and an in-game repro. Size S.
