# Tracker: blocks / Campfire (RE-REVIEW)

Re-audit after the source moved past tracker commit `56622ca89`: `694cb9949`
(isSmokeyPos blocker), `eb9d9a675` (cook result spawn point), `c0d5b4c11`
(BlockCookEvent cancel resets progress) and `e1ed99c1b` (block-entity game events).
Sources: `crates/pumpkin-core/src/block/blocks/campfire.rs`,
`crates/pumpkin-core/src/block/entities/campfire.rs`.
No decompiled vanilla tree is vendored in this worktree (`.vendor/` absent), so the
remaining rows below are marked with the vanilla entry point used for the comparison
and, where the exact 26.3 body could not be read, as "verify".

## Fixed in this pass

| Gap | Rust | Vanilla / Paper | Fix | Size |
|---|---|---|---|---|
| A cancelled `CampfireStartEvent` left `cooking_total_times` at 0, so the event was refired every tick (20/s) for a permanently blocked slot | `entities/campfire.rs:114-118, 143-146` | Paper fires the start event once per sitting; no repeat | new transient `cooking_cancelled[slot]` flag: set on cancel, cleared on insertion (`blocks/campfire.rs:100`), on completion (`entities/campfire.rs:202`) and on container-component load (`entities/campfire.rs:233`) | S |

## Verified matching vanilla (no change)

- Placement: `lit = !waterlogged`, `signal_fire` from hay block below, `facing` from
  the player, `blocks/campfire.rs:204-212`.
- Extinguish via `minecraft:douses_campfires`, sound world event, `BLOCK_CHANGE`,
  1 durability, `blocks/campfire.rs:121-143`.
- Waterlogging clears `lit` and schedules the water fluid tick, `blocks/campfire.rs:214-235`.
- Inside collision 0..0.4375 and `on_entity_collision` 1.0 / soul 2.0 damage, skipped by
  Frost Walker or Fire Resistance, `blocks/campfire.rs:148-183`.
- Item insertion per slot, recipe required, 1 item unless creative, stat + game event +
  block-entity update, `CONSUME` when all four slots are full, `blocks/campfire.rs:78-119`.
- Cooked result spawns at `(x+0.5, y+1.0, z+0.5)`, `entities/campfire.rs:344-354`.
- Lit tick increments progress, unlit tick cools by 2, `entities/campfire.rs:86-202, 320-333`.
- NBT keys `Items` / `CookingTimes` / `CookingTotalTimes`, `entities/campfire.rs:37-68, 282-318`.
- `is_lit_campfire` uses the `minecraft:campfires` tag so soul campfires pacify beehives,
  `blocks/campfire.rs:38-41`.
- Result count: `recipe.result.count` is used as-is (`entities/campfire.rs:172`); every
  campfire-cooking recipe assembles exactly one item, so this already equals vanilla
  `CampfireBlockEntity.litServerTick` (`assemble` of a 1-item input). Previous tracker
  row 3 ("split 1") is dismissed.

## Remaining gaps (not changed)

| # | Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | Non-cookable stacks (hopper/command/NBT-inserted) sit forever: the recipe lookup short-circuits before the progress increment | `entities/campfire.rs:120-124` | `CampfireBlockEntity.litServerTick` increments every non-empty stack and resolves the recipe only at completion, dropping the input back when none matches (verify) | increment progress first, look the recipe up when `progress >= total`, and eject/keep the stack when there is no recipe | M |
| 2 | Smoke obstruction test uses collision-shape intersection with the 0.375..0.625 column instead of vanilla's `isSolidRender` stop test | `blocks/campfire.rs:56-63` | `CampfireBlock.isSmokeyPos` (verify: solid-render vs collision) | replace with the vanilla predicate once decompiled source is vendored | S |
| 3 | `CampfireStartEvent` fires from the cook tick, not at item insertion | `entities/campfire.rs:127-149` | Paper fires it when the item is placed on the campfire (verify patch) | move the fire point into `use_with_item` / `apply_item_components` | M |
| 4 | Outline shape / particles / crackle sound missing | `blocks/campfire.rs` (no `get_outline_shape`, no client effects) | `CampfireBlock` SHAPE `0..7/16` + `AnimateTick` | client-only: add outline shape for selection box, note particles in QUEUE | S (client) |
