# tsmall-batch2 (blocks, items, world, net, commands; first 20 STILL OPEN rows)

Vanilla reference: `.vendor/vanilla-decompiled/net/minecraft/` in the main repo. No code changed in this pass; cargo not run (task rule), so no fix was applied without a compile and test gate.

| Source (tracker file : row) | Status | Reason / evidence |
|---|---|---|
| taudit-crates-pumpkin-core-src-bl-231130 : 46 (`furnace_like_block_entity.rs:511-535`) | STILL OPEN | Vanilla `AbstractFurnaceBlockEntity:278-281` rescales `cookingTimer` to `ceil(ratio * newTotal)` on relight. Rust has no rescale (no `ceil` rescale in `furnace_like_block_entity.rs`). |
| taudit-crates-pumpkin-core-src-bl-231130 : 47 (`furnace_like_block_entity.rs:177-198`) | STILL OPEN | Not re-checked in this pass; row claim not contradicted. |
| taudit-crates-pumpkin-core-src-bl-231130 : 49 (`furnace_like_block_entity.rs:867-876`) | STILL OPEN | Bedrock writer `write_bedrock_nbt` takes `stored_xp` (line ~918); caller not read, so the hardcoded `0` is not confirmed in this pass. |
| taudit-crates-pumpkin-core-src-bl-231130 : 50 (`furnace_like_block_entity.rs:235-274`) | STILL OPEN | Verified: output stack is still `ItemStack::new(recipe.result.count, output_item)` (line ~259), so `recipe.result.components` is dropped. |
| taudit-crates-pumpkin-core-src-bl-231130 : 51 (`furnace_like_block_entity.rs:225`) | FIXED | Already at HEAD: line 229 and 261 use `are_items_and_components_equal` (item plus components). Vanilla `canBurn:364` uses `isSameItemSameComponents`. |
| taudit-crates-pumpkin-core-src-bl-4d09af : 57 (`shelf.rs:325`) | FIXED | Already at HEAD: `block/blocks/shelf.rs:327` uses `matches!(args.replacing, BlockIsReplacing::Water(_))`. Vanilla `ShelfBlock:130` `replacedFluidState.is(Fluids.WATER)` also matches flowing water. |
| taudit-crates-pumpkin-core-src-bl-4d09af : 63 (`block/mod.rs:667-673`) | STILL OPEN | `BlockIsReplacing::water_source()` still has 45 call sites under `crates/pumpkin-core/src`. Migration to an `is_water()` helper is wider than this pass. |
| taudit-crates-pumpkin-core-src-bl-689b5a : 45 (`furnace_like_block_entity.rs:542-556`) | FIXED | Already at HEAD: start event total is computed from the recipe via `cooking_total_time(kind, recipe.cookingtime)`, default 200 (lines ~583-592). Matches vanilla `getTotalCookTime`. |
| taudit-crates-pumpkin-core-src-bl-689b5a : 46 (`furnace_like_block_entity.rs:233-237`) | FIXED | Already at HEAD: line ~264 `items[2].increment(recipe.result.count)`. The row's "no commit" note is stale; the code is in HEAD. |
| taudit-crates-pumpkin-core-src-bl-689b5a : 47 (`furnace_like_block_entity.rs:520-530`) | STILL OPEN | Same gap as bl-231130:46 (no relight rescale). |
| taudit-crates-pumpkin-core-src-bl-689b5a : 91 (`furnace_like_block_entity.rs:535-560`) | STILL OPEN | Same gap as bl-231130:46. |
| taudit-crates-pumpkin-core-src-bl-689b5a : 92 (`furnace_like_block_entity.rs:252-275`) | STILL OPEN | Same gap as bl-231130:50. |
| taudit-crates-pumpkin-core-src-block-enti : 1 (`entities/sculk_catalyst.rs:125`) | STILL OPEN | Client-only bloom particles need a level-event packet the server does not send. Out of scope for a server-side fix. |
| taudit-crates-pumpkin-core-src-block-enti : 2 (`entities/sculk_catalyst.rs:46,68-77`) | STILL OPEN (row corrected) | The row's premise is wrong: vanilla `SculkCatalystBlockEntity:52-62` overrides `loadAdditional`/`saveAdditional` and persists `sculkSpreader` cursors. Rust `chunk_data_nbt`/`save` write only `decay_delay`, so spreader cursors are not persisted. The real gap is cursor persistence. |
| taudit-crates-pumpkin-core-src-block-enti : 3 (`entities/sculk_catalyst.rs:158-160`) | STILL OPEN | Verified: vanilla `Position.closerThan` is `distanceToSqr < d*d` (strict, `Vec3.java:126-128`). Rust `on_entity_death` uses `<= radius_sq` on block offsets (`sculk_catalyst.rs:170`). A strict-compare fix alone is not parity because the source point must be the entity's `Vec3`; caller change needed. |
| taudit-crates-pumpkin-core-src-block-enti : 4 (`entity/living.rs:2981`) | VERIFIED (row stale) | Vanilla `SculkCatalystBlockEntity:100` passes `Optionull.map(lastDamageSource, DamageSource::getEntity)`, i.e. the killer, not `null`. Rust passes the killer, so no divergence on this call. |
| taudit-crates-pumpkin-core-src-block-enti : 5 (`entities/sculk_catalyst.rs:62`) | STILL OPEN | House-wide RNG seam (same pattern in fire, stem, jigsaw, grass). Needs a world-seeded RNG source; not catalyst-specific. |
| taudit-crates-pumpkin-core-src-co-20e7ce : 21 (`commands/item.rs:527-532`) | STILL OPEN (partial FIXED) | `uniform` is handled at HEAD (`number_provider`, item.rs). Binomial, score, storage and sum forms are still unhandled. |
| taudit-crates-pumpkin-core-src-co-20e7ce : 22 (`commands/item.rs:507-516`) | STILL OPEN | Verified: vanilla `SetItemCountFunction:43-47` does `setCount(base + value)` with no clamp. Rust `set_count` clamps to `get_max_stack_size()`. Fixing needs an over-max representation; `item_count` is `u8`. |
| taudit-crates-pumpkin-core-src-co-20e7ce : 24 (`commands/item.rs:554-586`) | STILL OPEN | Verified: `set_block_slot` returns `Ok(1)` and the entity/modify executors return a count with no `commands.item.*.modify.success` message. Vanilla `ItemCommands:259` calls `accessor.getModifySuccess`. Fix needs translation-key wiring; not applied without a compile. |

Counts: FIXED 4 (bl-231130:51, bl-4d09af:57, bl-689b5a:45, bl-689b5a:46). VERIFIED 1 (block-enti:4). STILL OPEN 15 (includes 1 partial FIXED: co-20e7ce:21).
Checks: none run (no cargo per task). Edits: none to code.
