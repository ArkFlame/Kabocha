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
