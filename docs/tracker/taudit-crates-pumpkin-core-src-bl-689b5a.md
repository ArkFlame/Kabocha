# block/entities/smoker.rs (DEEP AUDIT)

Target: `JavaMinecraftVersion::CURRENT_MC_VERSION`, source `crates/pumpkin-core/src/block/entities/smoker.rs`.
Reference: decompiled Paper/Canvas tree in `/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`world/level/block/entity/SmokerBlockEntity.java`,
`world/level/block/entity/AbstractFurnaceBlockEntity.java`,
`world/level/storage/loot/providers/number/ints/ContextIntProviders.java`,
`world/level/storage/loot/providers/number/floats/ContextFloatProviders.java`).

`smoker.rs` holds only the struct, constructor and the five cooking macros; all behaviour lives in
`furnace_like_block_entity.rs` (shared with furnace and blast furnace). The "2 markers" in the
tracker are the two `/// ... like vanilla ...` doc comments at `smoker.rs:35-40`; there are no
`TODO`/`FIXME` markers in the file.

Fixed in this pass:

- `furnace_like_block_entity.rs:41-49,379-390,561-580` - smokers and blast furnaces cooked at
  furnace speed: `cooking_total_time` was the raw recipe time (200 for smoking/blasting recipes in
  `pumpkin-data`) and progress advanced 1/tick, so a smoker took 200 ticks. Vanilla
  `AbstractFurnaceBlockEntity#getTotalCookTime` divides the recipe time by the fuel speed
  multiplier (`cooking/speed_default` = 2 for `LootPredicates.FAST_FURNACE`), so the total is 100
  ticks. The multiplier is now shared with the existing burn-time halving through
  `is_fast_cooking`/`cooking_speed_multiplier`/`cooking_total_time`.
- `furnace_like_block_entity.rs:703-711,790-797` - `write_nbt`/`chunk_data_nbt` now persist
  `speed_multiplier` (float), as vanilla `loadAdditional`/`saveAdditional` does.
- `furnace_like_block_entity.rs:891-901` - regression test for the computed total times
  (smelting 200 -> 200, smoking/blasting 200 -> 100, campfire 600 -> 600).

Audited clean (no change needed):

- `smoker.rs:35-40` - `RecipesUsed` map keyed by recipe ID and fractional `experience`, matching
  `AbstractFurnaceBlockEntity` (`RecipesUsed` codec + per-craft `experience`).
- `smoker.rs:67-71` - `CookingRecipeKind::Smoking` selects only smoking recipes, `INVENTORY_SIZE`
  3 with input/fuel/result slot order matching `AbstractFurnaceBlockEntity`.
- `blocks/smoker.rs:50-69` - `WindowType::Smoker`, 3 slots, `container.smoker` display name and
  the `interact_with_smoker` stat match `SmokerBlockEntity`/`SmokerMenu`.
- `furnace_like_block_entity.rs:108-110,458-520` - lit countdown, unlit progress decay of 2/tick,
  `FurnaceBurnEvent` clamping of `lit_total_time`, and the halved fuel burn time all match vanilla.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `furnace_like_block_entity.rs:484-496` | `getSpeedMultiplier` reads the fuel item's `CookingFuel.speedMultiplier` (`ResolvableFloat`, `cooking/speed_default` is conditional on `FAST_FURNACE`) | multiplier is derived from the block kind; a datapack fuel with an explicit `speed_multiplier` is ignored, and the value is not restored from NBT | store `speed_multiplier` on the three entities, fill it from `CookingFuelImpl::get_speed_multiplier(is_fast)` at ignition, read/write the float NBT key | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:542-556` | Paper passes `getTotalCookTime(level, entity)` (recipe time / multiplier, default 200) to `FurnaceStartSmeltEvent` | event is fired with the stored total, which is 0 when the input arrived before `set_stack` set it | compute from `furnace_recipe` with `cooking_total_time`, default 200 | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:233-237` | `burn` adds `burnResult.count()` to the result slot | always increments by 1 | increment by `recipe.result.count` and bound by `can_accept_recipe_output` | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass | S, FIXED (already in tree: furnace_like_block_entity.rs:257-263 increments by `recipe.result.count`; no commit) |
| `furnace_like_block_entity.rs:520-530` | on relight with a different fuel multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale (only relevant once the multiplier is data-driven, gap 1) | rescale after gap 1 | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Re-audit (2026-11-17, `CURRENT_MC_VERSION = V_26_3`, same reference tree)

Confirmed against the reference tree: `AbstractFurnaceBlockEntity.serverTick`, `canBurn`, `burn`,
`consumeFuel`, `setItem`, `getTotalCookTime`, `createExperience`, `loadAdditional`/`saveAdditional`.

Fixed in this pass:

- `furnace_like_block_entity.rs:218-241` - `can_accept_recipe_output` compared only the result
  item id, while vanilla `canBurn` uses `ItemStack.isSameItemSameComponents`. A same-item output
  stack with different components was accepted, then `craft_recipe`'s component-aware merge
  refused it and the smelted result was dropped while the input was consumed. The output slot is
  now compared with `are_items_and_components_equal`, and the empty-slot test uses `is_empty()`
  like vanilla instead of `are_equal(EMPTY)`.
- `furnace_like_block_entity.rs:692-706,740-746,827-833` - persisted keys were the pre-26
  `CookTime`/`CookTimeTotal`/`BurnTime` ints (with the current snake_case names treated as
  "legacy"). Vanilla 26.3 `saveAdditional`/`loadAdditional` use `cooking_time_spent`,
  `cooking_total_time`, `lit_time_remaining`, `lit_total_time` (ints) and `speed_multiplier`
  (float). Writes now use the current names; reads accept them first and still fall back to the
  pre-26 CamelCase keys and short tags, so existing Pumpkin worlds keep loading (tests updated).

Audited clean in this pass (no change needed):

- `furnace_like_block_entity.rs:630-635` - progress reset rules match the reference exactly:
  reset when the input is empty or `canBurn` fails, freeze when the input has no recipe (vanilla
  only resets inside the `recipe != null` branch), decay of 2/tick otherwise.
- `furnace_like_block_entity.rs:56-60,179-199` - `createExperience` equivalent:
  `floor(count * xp) + (frac != 0 && rand < frac)` is exactly vanilla's `Mth.floor`/`Mth.frac`
  roll, so batching is correct.
- `furnace_like_block_entity.rs:540-566` - fuel consumption and crafting-remainder handling match
  `consumeFuel` (remainder replaces an emptied fuel slot, otherwise drops as an item entity).
- `furnace_like_block_entity.rs:427-468` - slot validity (`COOKING_FUEL` component, bucket rule)
  and `canTakeItemThroughFace` `furnace_fuel_bottom_takeable` tag match vanilla.
- `furnace_like_block_entity.rs:735-746` - `RecipesUsed` map of recipe id -> count matches the
  vanilla codec layout.
- `block/entities/mod.rs:199-209` - vanilla `preRemoveSideEffects` XP pop plus item scatter is
  handled generically for every experience container, so breaking a smoker drops its XP.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `furnace_like_block_entity.rs:510-521,599-610` | `getSpeedMultiplier(level, fuel)` reads the fuel item's `CookingFuel.speedMultiplier` (`ResolvableFloat`, `cooking/speed_default` conditional on `FAST_FURNACE`), then `getTotalCookTime = ceil(recipeTime / speedMultiplier)` | the 2x multiplier is hardcoded per recipe kind; a datapack fuel with an explicit `speed_multiplier` is ignored, and the saved `speed_multiplier` float is written but never restored | resolve and store `speed_multiplier` per fuel ignition (`CookingFuelImpl::get_speed_multiplier`), use it for `cooking_total_time`, read the float in `from_nbt` | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:535-560` | on (re)ignition with a different multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale (only observable once the multiplier is data-driven) | rescale after the data-driven multiplier lands | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:252-275` | `canBurn`/`burn` use `recipe.assemble(input)`, which includes the recipe result's data components | the output stack is built from `result.id`/`result.count` only; `recipe.result.components` is dropped | build the result stack with the recipe result components | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:521-533` | Paper `FurnaceBurnEvent#setConsumeFuel(false)` keeps the furnace lit without consuming the fuel item | the event has no `consume_fuel` field | add the field to the event (additive plugin API) and honour it | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |


## Result

Verified every STILL OPEN row against `AbstractFurnaceBlockEntity.java` in the
`mc-proof/ref` tree (Paper/Canvas, MC 26.3). All in-scope rows are already fixed in the
current tree, so no source change was needed. The one row left open requires a plugin
API change outside the files named in this unit.

- First table row 44 / re-audit row 90 (data-driven speed multiplier, M): FIXED in tree.
  `tick` resolves the fuel multiplier with `CookingFuelImpl::get_speed_multiplier(is_fast)`
  (`furnace_like_block_entity.rs:574-575,607`), `cooking_total_time` divides the recipe
  time by it (`:32-41`), and `from_nbt` restores the persisted `speed_multiplier` float
  (`:773-777`) while `write_nbt`/`chunk_data_nbt` save it (`:819,901`). Matches vanilla
  `getSpeedMultiplier` (`ResolvableFloat.getFromItem(..., 1.0F)`) and
  `getTotalCookTimeVanilla` (`ceil(cookingTime / speedMultiplier)` at
  `AbstractFurnaceBlockEntity.java:420-421,428-432`). Tests:
  `cooking_time_divides_by_the_persisted_multiplier`,
  `furnace_speed_multiplier_round_trips_through_nbt`.
- First table row 45 (FurnaceStartSmeltEvent total, S): FIXED in tree.
  `furnace_like_block_entity.rs:647-665` computes the event total from the current
  recipe via `cooking_total_time` with a default of 200, matching Paper's
  `getTotalCookTime(level, entity)` (`AbstractFurnaceBlockEntity.java:295-305,434-440`).
  The computation is covered by `cooking_time_divides_by_the_persisted_multiplier`.
- First table row 46 (result count, S): FIXED in tree.
  `furnace_like_block_entity.rs:314-315` increments the output slot by
  `recipe.result.count` and `can_accept_recipe_output` (`:285-289`) checks room for the
  whole count, matching vanilla `burn`/`canBurn`.
- First table row 47 / re-audit row 91 (relight rescale, S): FIXED in tree.
  `rescale_cooking_after_relight` (`:46-62`) implements
  `cookingTimer = ceil(timer / oldTotal * newTotal)` under the same `oldTotal > 0 &&
  spent < oldTotal` guard and is called at ignition (`:600-609`), matching
  `AbstractFurnaceBlockEntity.java:278-282`. Test:
  `relight_rescales_progress_by_the_multiplier_change`.
- Re-audit row 92 (recipe result components, S): FIXED in tree.
  `assemble_cooking_result` (`:86-100`) applies `recipe.result.components` to the output
  stack and is used by both `can_accept_recipe_output` (`:281`) and `craft_recipe`
  (`:310`), matching vanilla `recipe.assemble(input)` in `canBurn`/`burn` (`:256-257`).
  Test: `assemble_cooking_result_applies_recipe_components`.
- Re-audit row 93 (FurnaceBurnEvent `consume_fuel`, M): STILL OPEN. Reason: adding the
  additive `consume_fuel` field to the plugin event and honouring it requires edits to
  `pumpkin-core/src/plugin/api/events/inventory/furnace_burn.rs`, the matching
  `pumpkin-plugin-api` event and host/SDK wiring, all outside this unit's two files.


## Result (re-audit 2026-11-18)

Re-verified every STILL OPEN row against `AbstractFurnaceBlockEntity.java` in the
`mc-proof/ref` tree (Paper/Canvas, MC 26.3) and against `CookingFuelImpl` in `pumpkin-data`.
All in-scope rows are already fixed in the current tree, so no source change was needed
this pass.

- First table row 44 / re-audit row 90 (data-driven speed multiplier, M): FIXED in tree.
  `tick` resolves the fuel multiplier with `CookingFuelImpl::get_speed_multiplier(is_fast)`
  (`:593-594`); that helper maps `cooking/speed_default` to 2.0 for smokers/blast furnaces
  and 1.0 otherwise, matching vanilla `ResolvableFloat.getFromItem(..., 1.0F)`.
  `cooking_total_time` divides the recipe time by it (`:32-41`), and `from_nbt` restores
  the persisted `speed_multiplier` float (`:798-802`) while `write_nbt`/`chunk_data_nbt`
  save it (`:844,926`). Tests: `cooking_time_divides_by_the_persisted_multiplier`,
  `furnace_speed_multiplier_round_trips_through_nbt`.
- First table row 45 (FurnaceStartSmeltEvent total, S): FIXED in tree. `:672-683` computes
  the event total from the current recipe via `cooking_total_time` with a default of 200,
  matching Paper `getTotalCookTime(level, entity)`.
- First table row 46 (result count, S): FIXED in tree. `:332` increments the output slot by
  `recipe.result.count` and `can_accept_recipe_output` (`:303-306`) checks room for the whole
  count, matching vanilla `burn`/`canBurn`.
- First table row 47 / re-audit row 91 (relight rescale, S): FIXED in tree.
  `rescale_cooking_after_relight` (`:46-62`) implements the `oldTotal > 0 && spent < oldTotal`
  guarded `ceil(timer / oldTotal * newTotal)` rescale, called at ignition (`:621-630`),
  matching vanilla `AbstractFurnaceBlockEntity.java:278-282`. Test:
  `relight_rescales_progress_by_the_multiplier_change`.
- Re-audit row 92 (recipe result components, S): FIXED in tree. `assemble_cooking_result`
  (`:86-100`) applies `recipe.result.components` and is used by `can_accept_recipe_output`
  (`:298`) and `craft_recipe` (`:327`), matching vanilla `recipe.assemble(input)` in
  `canBurn`/`burn`. Test: `assemble_cooking_result_applies_recipe_components`.
- Re-audit row 93 (FurnaceBurnEvent `consume_fuel`, M): core portion FIXED in tree. The
  `consume_fuel` field exists on the core event
  (`plugin/api/events/inventory/furnace_burn.rs:20`) and the tick honours it
  (`furnace_like_block_entity.rs:600,608,636`), matching Paper `burnEvent.willConsumeFuel()`.
  STILL OPEN only for plugin exposure: the v0.1/v0.2 WIT `furnace-burn-event` records have no
  `consume-fuel` field and both wasm-host conversions hardcode `consume_fuel: true`, so a
  plugin cannot call `setConsumeFuel(false)`. Reason left open: exposing it needs WIT record
  edits in both versions plus regenerated bindings and host/SDK wiring, all outside this
  unit's files.

No new source file was touched, so `rustfmt` was not needed. No seam for a new pure unit
test: every in-scope fix is already covered by the pure-function tests listed above, and
the remaining row's only gap is plugin API plumbing.


## Result (pass 3, 2026-11-18)

Every row verified once more against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java`
(Minecraft 26.3, Paper/Canvas). All previously open rows are now closed.

- First table row 44 / re-audit row 90 (data-driven speed multiplier, M): FIXED in tree.
  `tick` reads `CookingFuelImpl::get_speed_multiplier(is_fast)` (default 1.0 when the fuel
  has no `cooking_fuel` component) at ignition, `cooking_total_time` divides the recipe time
  by it with vanilla's `speedMultiplier > 0.0F` guard, and both NBT paths persist/restore the
  float. Matches `getSpeedMultiplier` + `getTotalCookTimeVanilla`
  (`AbstractFurnaceBlockEntity.java:420-432`).
- First table row 45 (FurnaceStartSmeltEvent total, S): FIXED in tree. The event total is
  recomputed from the current recipe and the persisted multiplier with a 200 default, matching
  `getTotalCookTime(level, entity)` (`:434-440`) used at `:295-305`.
- First table row 46 (result count, S): FIXED in tree. `craft_recipe` increments the output by
  `recipe.result.count` and `can_accept_recipe_output` requires room for the whole count,
  matching `canBurn`/`burn` (`:358-404`).
- First table row 47 / re-audit row 91 (relight rescale, S): FIXED in tree.
  `rescale_cooking_after_relight` mirrors `:278-282` (`completionRatio * newTotal`, ceil, only
  while `cookingTimer < cookingTotalTime`).
- Re-audit row 92 (recipe result components, S): FIXED in tree. `assemble_cooking_result` applies
  `recipe.result.components` to the output stack, matching `recipe.assemble(input)` (`:256-257`).
- Re-audit row 93 (FurnaceBurnEvent `consume_fuel`, M): FIXED this pass. The core event already
  carried `consume_fuel` and `tick` skips `consumeFuel` when it is false, matching
  `:284-287` (`if (burnEvent.willConsumeFuel()) consumeFuel(...)`). The plugin gap is closed:
  `consume-fuel: bool` was added to the v0.2 `furnace-burn-event` record
  (`crates/pumpkin-plugin-wit/v0.2/event.wit`) and the wasm host v0.2 conversion now maps it in
  both directions instead of hardcoding `true`
  (`crates/pumpkin-wasm-host-v0_2/src/events/inventory.rs`). v0.1 is deprecated and frozen for
  compatibility, so its record shape was left unchanged (a record-shape edit would break the
  existing v0.1 ABI).

No test was added: the only new seam is a wasm-host conversion that needs a live
`PluginHostState`, and the shared cooking helpers keep their existing pure-function tests
(`cooking_time_divides_by_the_persisted_multiplier`,
`relight_rescales_progress_by_the_multiplier_change`,
`assemble_cooking_result_applies_recipe_components`,
`furnace_speed_multiplier_round_trips_through_nbt`).

## Result (CODER r2_001, 2026-11-18)
Status PARTIAL. Checked against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/Block.java` and `state/BlockBehaviour.java`.
- Row 1 get_inside_collision_shape default: VERIFIED. BlockBehaviour.java:336-337 returns `Shapes.block()`; block/mod.rs:305 `BoundingBox::full_block()` matches. The audit's empty-shape fallback premise does not hold for this vanilla version.
- Row 2 comparator max stack divisor: STILL OPEN (vanilla min(container max, item max) not re-read this pass; prior note says no live difference).
- Row 3 pushEntitiesUp: BLOCKED-DATA. Block.java:158-166 needs `Shapes.joinUnoptimized` / `BooleanOp.ONLY_SECOND` / `Shapes.collide` on VoxelShape; pumpkin-core has no VoxelShape boolean-op subsystem.
- Rows 4, 5, 6, 7: STILL OPEN, not reached this pass (row 4 needs Option<&Player> threading through world/mod.rs; rows 5-7 per earlier pass).

## Result (CODER r3_007, 2026-11-18)
Status PASS. Checked against `.vendor/vanilla-decompiled/net/minecraft/world/inventory/AbstractContainerMenu.java`, `world/Container.java`, `util/Mth.java`, `world/level/block/Block.java`, `world/item/context/BlockPlaceContext.java`, `world/level/block/state/BlockBehaviour.java`. No Rust edits needed.
- Row 2 comparator max stack divisor: VERIFIED. Container.java:36-38 is `min(container max, item max)`; Rust `calculate_comparator_output` (`block/mod.rs:900-902`) uses `get_max_count_per_stack().min(stack.get_max_stack_size())`. Final value `1 + p*14` floored equals `Mth.lerpDiscrete(p,0,15)` (Mth.java:545-548) for p>0. Test: `comparator_output_uses_smaller_container_or_item_max_stack`.
- Row 3 pushEntitiesUp: VERIFIED, no longer BLOCKED-DATA. `push_entities_up` (`block/mod.rs:86-139`) reproduces `joinUnoptimized(ONLY_SECOND)` with box subtraction (`added_collision_boxes`), the `Shapes.collide(Y, ..., -1.0)` clamp (`collide_down_y`), and the empty early return of Block.java:158-170. Residual difference: `teleportRelative` is replaced by `set_pos`.
- Row 4 BlockDropItemEvent player: VERIFIED. `drop_loot` takes `player: Option<&Arc<Player>>` and sets `player.cloned()` (`block/mod.rs:686-717`). `World::break_block` threads `cause` (`world/mod.rs:7083,7173`). Natural, explosion and leaf callers pass `None` (`world/mod.rs:7292`, `world/explosion.rs:975`, `blocks/leaves.rs:121`).
- Row 5 `BlockPlaceContext::at` replaceClicked: VERIFIED for the live path. Vanilla BlockPlaceContext.java:30 sets `replaceClicked` from `canBeReplaced`. Rust `at` (`block/mod.rs:829-846`) gates on `can_replace_with_other_block`. Residual, no live effect: BlockBehaviour.java:269-271 also checks the item in hand and `isDestroyable`, which `at` does not take. The only caller, `item/items/scaffolding.rs:62`, has already excluded scaffolding targets.
- Row 6 placement collision duplication: VERIFIED. `has_placement_collision` (`block/mod.rs:314-316`) is defined as `!can_place_inside_entities()`, so both paths share one signal. Vanilla `isEmpty()` check matches.
- Row 7 silk touch lookup: VERIFIED. `block/mod.rs:739` uses the generated `pumpkin_data::Enchantment::SILK_TOUCH` constant (`pumpkin-data/src/generated/enchantment.rs:2295`), not a name lookup.

No Rust file touched, so rustfmt and cargo were not run.
