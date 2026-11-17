# Tracker: blocks / Piston Head (RE-REVIEW-UNTRACKED)
Re-audit after `945f7538e` (pass the breaking player when removing the piston
base) and `7a9b21932` (blocks in front of a headless piston on retract).
Source reviewed: `crates/pumpkin-core/src/block/blocks/piston/piston_head.rs`
(plus its call chain: `entity/player.rs::break_block`, `world/mod.rs::on_block_state_set`).

## Fixed in this pass
| Gap | Rust | Vanilla | Fix | Size |
|---|---|---|---|---|
| `broken()` re-removed the piston base after the head was already gone, so the player attribution added in `945f7538e` never ran and a second, never-fired `BlockBreakEvent` path existed | `piston_head.rs:39-53` (old) | `PistonHeadBlock.playerWillDestroy` is creative-only and runs before the head is removed; the base is dropped/removed once in `PistonHeadBlock.onRemove` | deleted `broken()`; `on_state_replaced` (vanilla `onRemove`) is now the only base-removal path. Probe: `Player::break_block` calls `World::break_block` (`entity/player.rs:6424,6444`) whose `set_block_state` fires `on_state_replaced` for the head (`world/mod.rs:6223-6231`) before `block_registry.broken` (`entity/player.rs:6436,6456`), so the base was always air by then | S |

## Remaining gaps
| Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| Base-removal guard is stricter than vanilla: the head TYPE must match the base kind | `piston_head.rs:24-34` (`is_fitting_base`) | `PistonHeadBlock.isFittingBase` accepts either piston block and checks `EXTENDED` + the same `FACING` | Relax to `base_block == PISTON || base_block == STICKY_PISTON` + extended + facing. Only reachable via `/setblock` or plugin-written inconsistent states (piston code always writes the matching TYPE), so normal play is unchanged | S |
| Base removal uses `World::break_block`, which fires a plugin `BlockBreakEvent` (cause `None`) and sends destroy particles | `piston_head.rs:53-57` | `Level.removeBlock(basePos, false)` / `Block.dropResources`: flag 3, no block event, no particles | Use a plain `set_block_state(AIR, NOTIFY_ALL)` plus explicit loot drop when drops are enabled, or accept the extra event/particles | S/M |

## Verified matching vanilla (no change)
- `on_state_replaced` == `PistonHeadBlock.onRemove`: skips when moved, only when the
  block actually changed (`world/mod.rs:6223` gates on a different block), removes the
  base, and propagates `SKIP_DROPS` so a creative break is dropless
  (`piston_head.rs:38-60`).
- `get_state_for_neighbor_update` == `updateShape`: only the neighbour opposite the
  facing can invalidate the head, via `can_place_at` (`piston_head.rs:62-84`).
- `can_place_at` == `canSurvive`: a fitting extended base, or a moving piston with the
  same facing (`piston_head.rs:86-95`).
- `on_neighbor_update` == `neighborChanged`: survives, then forwards the update to the
  base position (`piston_head.rs:97-120`).
- `is_pathfindable` returns false (`piston_head.rs:122-124`).
- The `short` property is never written or read; vanilla only uses it for the
  `piston_head_short` model (client-side, reachable via commands/datapacks).
- Collision/outline boxes are not implemented here: Pumpkin takes them from the
  generated `pumpkin-data` block collision shapes, like every other block.
