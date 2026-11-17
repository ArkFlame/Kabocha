# Tracker: audit / crates/pumpkin-core/src/block/entities/furnace_like_block_entity.rs (AUDIT)

Audit of the shared `impl_*_for_cooking!` macros used by `furnace.rs`,
`blasting_furnace.rs` and `smoker.rs`. Compared against vanilla
`AbstractFurnaceBlockEntity` (`serverTick`, `burn`, `createExperience`,
`saveAdditional`), `BlastFurnaceBlockEntity`/`SmokerBlockEntity.getBurnDuration`
(halved) and Paper's `FurnaceBurnEvent` / `FurnaceStartSmeltEvent` /
`FurnaceSmeltEvent`.

## Fixed in this pass

| Gap | Rust | Vanilla | Fix | Size |
|---|---|---|---|---|
| Cooking state saved under Pumpkin-only snake_case keys, so vanilla worlds lose furnace progress and vanilla cannot read Pumpkin saves | `furnace_like_block_entity.rs:670-676, 750-756` (pre-fix) | `AbstractFurnaceBlockEntity.saveAdditional` writes `BurnTime`, `CookTime`, `CookTimeTotal`, `RecipesUsed` | read/write the vanilla keys, keep reading the old snake_case keys as a fallback; test `furnace_state_uses_vanilla_nbt_keys` | S |
| `FurnaceBurnEvent` fired and `lit` state written for an item in the fuel slot that has no burn duration | `furnace_like_block_entity.rs:470-480` (pre-fix) | vanilla wraps ignition in `if (burnDuration > 0)`; Paper only constructs `FurnaceBurnEvent` for a positive burn time | ignition block skipped when the fuel reports 0 ticks | S |

## Remaining gaps

| Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| Changing the input item resets cooking progress and rewrites `cooking_total_time` | `impl_inventory_for_cooking!::set_stack`, ~`furnace_like_block_entity.rs:350-360` | `setItem` only stores the stack; `cookingProgress` survives an input swap and `cookingTotalTime` is only recomputed when a smelt finishes | drop the slot-0 reset; let `serverTick` (progress reset when `canBurn` false) and the start-smelt path own the total time | S |
| On completing a smelt, `cooking_total_time` is set to the recipe just smelted | `tick`, ~`furnace_like_block_entity.rs:545-548` | `getTotalCookTime(level, furnace)` is recomputed for the *remaining* input (200-tick default when the input is empty) | look the recipe up again after `craft_recipe` and use its `cookingtime`, else 200 | S |
| `lit_total_time` (flame-bar denominator) is persisted under a custom key | `write_nbt` / `chunk_data_nbt` | vanilla persists only `BurnTime`/`CookTime`/`CookTimeTotal`; `litDuration` is rebuilt at runtime and reads 0 after a reload | keep (Pumpkin-only extra, harmless for vanilla readers) or rebuild `lit_total_time` from the fuel on load | S |
| Recipes used are only turned into XP; the recipe-book award is missing | `CookingBlockEntityBase::extract_experience_from_recipes`, `furnace_like_slot.rs:152` | `getRecipesToAwardAndPopExperience` + `Player.awardRecipes`: unlock recipes, `recipes_unlocked` criterion, stats | needs recipe-book support; award through the player recipe book once it exists | M |
| XP is granted directly as player experience instead of spawning orbs | `furnace_like_slot.rs:154-157` | `createExperience` spawns `ExperienceOrb`s at `player.position()` | optional: spawn orbs so other players/vacuum hoppers see them; same net result for the taker | S |
| `bedrock_block_actor_data` always sends `StoredXPInt = 0` | `tick` macro `bedrock_block_actor_data`, ~`furnace_like_block_entity.rs:790` | Bedrock actors store uncollected furnace XP in `StoredXPInt` | sum `recipes_used` XP into the field once a recipe-id -> xp map is available on the actor path | S |

## Verified matching vanilla (no change)

- Tick order: decrement `lit_time_remaining`, resolve recipe, fuel burn (`BlastFurnace`/`Smoker`
  halve burn ticks via `CookingFuelImpl::get_burn_ticks(is_fast)`), craft remainder
  spill, progress `+1`/reset, `cooking_time_spent` decay by 2 clamped to the total,
  then `lit` blockstate update (`tick`).
- `cooking_time_spent`/`lit_time_remaining`/... property indices 0..3 ==
  `AbstractFurnaceBlockEntity.dataAccess`.
- Slot layout, `SLOTS_FOR_UP {0}` / `SLOTS_FOR_DOWN {2,1}` / sides `{1}`, fuel-slot
  filter (fuel component or empty bucket), bucket-only downward take from slot 1,
  output slot not insertable.
- `can_accept_recipe_output` mirrors `canBurn`'s output-stack check (same item,
  count < min(max stack size, recipe stack limit)).
- Wet sponge + bucket -> water bucket and input `shrink(1)` happen in `craft_recipe`,
  same order as vanilla `burn`.
- XP split: floor + one point with probability of the fraction
  (`createExperience`), including the fractional single-smelt test.
- `RecipesUsed` compound of recipe id -> int count, cleared on extraction.
