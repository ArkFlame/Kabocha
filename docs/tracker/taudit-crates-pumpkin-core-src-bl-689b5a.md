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
| `furnace_like_block_entity.rs:484-496` | `getSpeedMultiplier` reads the fuel item's `CookingFuel.speedMultiplier` (`ResolvableFloat`, `cooking/speed_default` is conditional on `FAST_FURNACE`) | multiplier is derived from the block kind; a datapack fuel with an explicit `speed_multiplier` is ignored, and the value is not restored from NBT | store `speed_multiplier` on the three entities, fill it from `CookingFuelImpl::get_speed_multiplier(is_fast)` at ignition, read/write the float NBT key | M |
| `furnace_like_block_entity.rs:542-556` | Paper passes `getTotalCookTime(level, entity)` (recipe time / multiplier, default 200) to `FurnaceStartSmeltEvent` | event is fired with the stored total, which is 0 when the input arrived before `set_stack` set it | compute from `furnace_recipe` with `cooking_total_time`, default 200 | S |
| `furnace_like_block_entity.rs:233-237` | `burn` adds `burnResult.count()` to the result slot | always increments by 1 | increment by `recipe.result.count` and bound by `can_accept_recipe_output` | S |
| `furnace_like_block_entity.rs:520-530` | on relight with a different fuel multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale (only relevant once the multiplier is data-driven, gap 1) | rescale after gap 1 | S |

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
| `furnace_like_block_entity.rs:510-521,599-610` | `getSpeedMultiplier(level, fuel)` reads the fuel item's `CookingFuel.speedMultiplier` (`ResolvableFloat`, `cooking/speed_default` conditional on `FAST_FURNACE`), then `getTotalCookTime = ceil(recipeTime / speedMultiplier)` | the 2x multiplier is hardcoded per recipe kind; a datapack fuel with an explicit `speed_multiplier` is ignored, and the saved `speed_multiplier` float is written but never restored | resolve and store `speed_multiplier` per fuel ignition (`CookingFuelImpl::get_speed_multiplier`), use it for `cooking_total_time`, read the float in `from_nbt` | M |
| `furnace_like_block_entity.rs:535-560` | on (re)ignition with a different multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale (only observable once the multiplier is data-driven) | rescale after the data-driven multiplier lands | S |
| `furnace_like_block_entity.rs:252-275` | `canBurn`/`burn` use `recipe.assemble(input)`, which includes the recipe result's data components | the output stack is built from `result.id`/`result.count` only; `recipe.result.components` is dropped | build the result stack with the recipe result components | S |
| `furnace_like_block_entity.rs:521-533` | Paper `FurnaceBurnEvent#setConsumeFuel(false)` keeps the furnace lit without consuming the fuel item | the event has no `consume_fuel` field | add the field to the event (additive plugin API) and honour it | M |

## Gap status
- `furnace_like_block_entity.rs:484-496`/`510-521,599-610` data-driven speed multiplier: STILL OPEN (M; same as furnace row).
- `:542-556` `FurnaceStartSmeltEvent` total cook time: STILL OPEN (S).
- `:233-237` burn always +1 instead of `recipe.result.count`: STILL OPEN (S).
- `:520-530` / `:535-560` relight timer rescale: STILL OPEN (S).
- `:252-275` recipe result components: STILL OPEN (S).
- `:521-533` `FurnaceBurnEvent` consume-fuel flag: STILL OPEN (M; additive plugin API).
- Re-audit "audited clean" rows (progress reset, createExperience, consumeFuel, slot validity, RecipesUsed codec): verified, no change.
