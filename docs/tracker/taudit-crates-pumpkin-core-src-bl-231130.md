# Audit: crates/pumpkin-core/src/block/entities/furnace_like_block_entity.rs

Row: audit / coverage AUDIT. Tracker note: 11 "vanilla" hits - all are doc comments,
no TODO/FIXME markers in the file.

Reference: decompiled Canvas/Paper tree
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft/world/level/block/entity/`
(`AbstractFurnaceBlockEntity.java`, `SmokerBlockEntity.java`, `BlastFurnaceBlockEntity.java`,
`FurnaceBlockEntity.java`) for `CURRENT_MC_VERSION = V_26_3`.

## Fixed in this pass

- XP is now derived from `RecipesUsed` at extraction with vanilla `createExperience`
  (per-recipe `floor(count * experience)` plus one roll of the fractional part) instead of
  accumulating one float and carrying the fraction. This also repaired the test module, which
  already referenced an undefined `split_experience` helper and therefore did not compile.
  (`furnace_like_block_entity.rs:54-60,177-198`)
- `FurnaceStartSmeltEvent` gets `getTotalCookTime` (recipe time / speed multiplier, default 200)
  like Paper instead of the stored total, which could be 0 and then blocked smelting.
  (`furnace_like_block_entity.rs:571-590`)
- Progress is only reset when the input is empty or a recipe exists but the output cannot take
  it; a non-cooking input now freezes progress instead of resetting it, matching vanilla
  `serverTick`. (`furnace_like_block_entity.rs:628-634`)
- Output fit (`canBurn`) accounts for `recipe.result.count` and stacking against the max stack,
  and crafting adds `recipe.result.count` instead of always 1.
  (`furnace_like_block_entity.rs:218-232, 265`)
- NBT state is read with `get_numeric_short` (int and short tags) and written with
  `put_int`, matching the vanilla 26.x codec; short/snake_case keys from older Pumpkin builds
  still load. (`furnace_like_block_entity.rs:62-75, 705-712, 744-749, 831-836, tests`)

Audited clean:

- `getSlotsForFace` (down 2,1 / up 0 / sides 1), `canPlaceItem` (fuel / bucket swap),
  `canTakeItemThroughFace` (`furnace_fuel_bottom_takeable` tag) match vanilla.
- Fuel consumption with crafting remainder, wet-sponge/bucket conversion, lit block-state update,
  unlit progress decay of 2/tick, `lit_total_time` clamping by `FurnaceBurnEvent`.
- NBT key set matches `loadAdditional`/`saveAdditional` (`cooking_time_spent`,
  `cooking_total_time`, `lit_time_remaining`, `lit_total_time`, `speed_multiplier`,
  `RecipesUsed` as recipe id -> int).

## Remaining gaps

| Location | Vanilla | Gap | Suggested fix | Size |
|---|---|---|---|---|
| `furnace_like_block_entity.rs:30-38,511` | `getSpeedMultiplier` reads the fuel item's `CookingFuel.speedMultiplier` (ResolvableFloat, `cooking/speed_default`); `getTotalCookTime` divides by it | multiplier is derived from the block kind; a datapack fuel with an explicit `speed_multiplier` is ignored and the value is not restored on load | store `speed_multiplier` on the three entities, set it from `CookingFuelImpl::get_speed_multiplier(is_fast)` at ignition, use it in `cooking_total_time`, read/write the float key | M |
| `furnace_like_block_entity.rs:511-535` | on relight with a different multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale | implement after the data-driven multiplier above | S |
| `furnace_like_block_entity.rs:177-198` | `getRecipesToAwardAndPopExperience` spawns one orb per recipe id | extraction returns one summed `i32`, so one orb | return per-recipe amounts (Vec) or spawn orbs in the block entity | S |
| `pumpkin-inventory/src/furnace_like/furnace_like_slot.rs` take path | `FurnaceResultSlot.checkTakeAchievements` -> `awardUsedRecipesAndPopExperience` -> `player.awardRecipes` + `triggerRecipeCrafted` | used recipes are never unlocked in the recipe book nor counted as crafted | pass `RecipesUsed` ids to the player before clearing | M |
| `furnace_like_block_entity.rs:867-876` | Bedrock furnace actor stores `StoredXPInt` | hardcoded `0` | pass the accumulated `RecipesUsed` XP | S |
| `furnace_like_block_entity.rs:235-274` | `recipe.assemble` includes result data components | result stack is built from item id only | apply `recipe.result.components` to the output stack | S |
| `furnace_like_block_entity.rs:225` | `ItemStack.isSameItemSameComponents` | only item ids are compared | compare item plus components once result components are supported | S |
| `furnace_like_block_entity.rs:856-858` | `getUpdateTag` includes items and cook/burn state | `client_update_tag` returns an empty compound | emit the same keys as `chunk_data_nbt` | S |
