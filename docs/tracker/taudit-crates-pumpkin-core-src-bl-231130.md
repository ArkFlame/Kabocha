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
| `furnace_like_block_entity.rs:30-38,511` | `getSpeedMultiplier` reads the fuel item's `CookingFuel.speedMultiplier` (ResolvableFloat, `cooking/speed_default`); `getTotalCookTime` divides by it | multiplier is derived from the block kind; a datapack fuel with an explicit `speed_multiplier` is ignored and the value is not restored on load | store `speed_multiplier` on the three entities, set it from `CookingFuelImpl::get_speed_multiplier(is_fast)` at ignition, use it in `cooking_total_time`, read/write the float key | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:511-535` | on relight with a different multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale | implement after the data-driven multiplier above | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:177-198` | `getRecipesToAwardAndPopExperience` spawns one orb per recipe id | extraction returns one summed `i32`, so one orb | return per-recipe amounts (Vec) or spawn orbs in the block entity | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `pumpkin-inventory/src/furnace_like/furnace_like_slot.rs` take path | `FurnaceResultSlot.checkTakeAchievements` -> `awardUsedRecipesAndPopExperience` -> `player.awardRecipes` + `triggerRecipeCrafted` | used recipes are never unlocked in the recipe book nor counted as crafted | pass `RecipesUsed` ids to the player before clearing | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:867-876` | Bedrock furnace actor stores `StoredXPInt` | hardcoded `0` | pass the accumulated `RecipesUsed` XP | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:235-274` | `recipe.assemble` includes result data components | result stack is built from item id only | apply `recipe.result.components` to the output stack | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:225` | `ItemStack.isSameItemSameComponents` | only item ids are compared | compare item plus components once result components are supported | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `furnace_like_block_entity.rs:856-858` | `getUpdateTag` includes items and cook/burn state | `client_update_tag` returns an empty compound | emit the same keys as `chunk_data_nbt` | S, FIXED d047c4b2d |


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java` (`serverTick` 246-328, `canBurn` 358-371, `getSpeedMultiplier`/`getTotalCookTime` 416-440, `getRecipesToAwardAndPopExperience`/`createExperience` 531-577).

- **furnace_like_block_entity.rs:867-876 (Bedrock `StoredXPInt`, S)** -- FIXED. `bedrock_block_actor_data` now passes the XP accrued from `RecipesUsed` (new `stored_experience` helper) instead of hardcoded `0`.
- **furnace_like_block_entity.rs:235-274 (result data components, S)** -- FIXED. New `assemble_cooking_result` helper applies `recipe.result.components` to the output stack; used by both `can_accept_recipe_output` (canBurn comparison) and `craft_recipe` (stored stack).
- **furnace_like_block_entity.rs:225 (`isSameItemSameComponents`, S)** -- SATISFIED by existing `are_items_and_components_equal` calls; with result components now assembled, the comparison covers them too. No new change needed.
- **furnace_like_block_entity.rs:30-38,511 (data-driven fuel `speed_multiplier`, M)** -- OPEN: needs a persisted `speed_multiplier` field on the furnace/smoker/blast-furnace entity structs (furnace.rs, smoker.rs, blast_furnace.rs) outside this unit, plus load/restore.
- **furnace_like_block_entity.rs:511-535 (rescale timer on relight, S)** -- OPEN: depends on the persisted per-entity multiplier above; with only a block-kind multiplier there is no differing multiplier to rescale against.
- **furnace_like_block_entity.rs:177-198 (one XP orb per recipe, S)** -- OPEN: `ExperienceContainer::extract_experience` returns a summed `i32`; emitting one orb per recipe needs a cross-crate return-type change in pumpkin-inventory (`window_property.rs`), outside this unit.
- **furnace_like_slot.rs take path (unlock recipes in recipe book, M)** -- OPEN: the slot only receives summed XP from `ExperienceContainer`; exposing `RecipesUsed` ids to `awardRecipes`/`triggerRecipeCrafted` needs a new cross-crate method, outside this unit.

Tests: added `assemble_cooking_result_applies_recipe_components` and `stored_experience_floors_each_recipes_accrued_xp` (pure unit tests). `rustfmt --edition 2024` run on furnace_like_block_entity.rs. No compile check run (worker rule).


## Result (follow-up pass)

- **furnace_like_block_entity.rs:30-38,511 (data-driven fuel `speed_multiplier`, M)** -- FIXED. Vanilla `AbstractFurnaceBlockEntity` persists a `speedMultiplier` float (NBT key `speed_multiplier`, default 1.0), reads it from the fuel item's `CookingFuel.speedMultiplier` at ignition and divides recipe time by it via `getTotalCookTime`. Added a persisted `speed_multiplier_bits: AtomicU32` field to the furnace, smoker and blasting-furnace entities; the cooking macros now read/write the float key, derive `cooking_total_time` from the stored multiplier (no longer hardcoded 1 or 2 per block kind), and set it from `CookingFuelImpl::get_speed_multiplier(is_fast)` at ignition.
- **furnace_like_block_entity.rs:511-535 (rescale timer on relight, S)** -- FIXED. Added `rescale_cooking_after_relight`, applying vanilla's `cookingTimer = ceil(timer / oldTotal * newTotal)` while `timer < oldTotal`, called at ignition with the new fuel multiplier.
- **furnace_like_block_entity.rs:177-198 (one XP orb per recipe, S)** -- OPEN: vanilla `getRecipesToAwardAndPopExperience` spawns one orb per recipe id; `ExperienceContainer::extract_experience` returns a summed `i32` and its second consumer `block/entities/mod.rs` (outside this unit) spawns a single orb, so per-recipe orbs need a cross-file return-type change.
- **furnace_like_slot.rs take path (unlock recipes in recipe book, M)** -- OPEN: needs `awardRecipes`/`triggerRecipeCrafted` on `InventoryPlayer` (screen_handler.rs) and the player implementation, both outside this unit.

Tests: added `cooking_time_divides_by_the_persisted_multiplier`, `relight_rescales_progress_by_the_multiplier_change` and `furnace_speed_multiplier_round_trips_through_nbt` (pure unit tests); replaced the obsolete block-kind `cooking_speed_matches_vanilla`. `rustfmt --edition 2024` run on all four edited .rs files. No compile check run (worker rule).


## Result (re-audit pass)

Verified every STILL OPEN row from the "Remaining gaps" table against decompiled vanilla
(`AbstractFurnaceBlockEntity.java` 500-595, `FurnaceResultSlot.java` 28-53, `Player.java` 1536,
`ServerRecipeBook.java` 61-87) and the current working tree. All rows are resolved in code:

- **`furnace_like_block_entity.rs:30-38,511` (data-driven fuel `speed_multiplier`, M)** -- already
  FIXED in the follow-up pass: persisted `speed_multiplier_bits`, read/written under the vanilla
  `speed_multiplier` float key, applied at ignition from `CookingFuelImpl`.
- **`furnace_like_block_entity.rs:511-535` (rescale timer on relight, S)** -- already FIXED:
  `rescale_cooking_after_relight` implements `ceil(timer / oldTotal * newTotal)`.
- **`furnace_like_block_entity.rs:177-198` (one XP orb per recipe, S)** -- FIXED in tree.
  `ExperienceContainer::extract_recipes()` (`window_property.rs`) returns one `RecipeExperience`
  per id; `ExperienceContainer::extract_experience()` maps them to a `Vec<i32>`; and
  `block/entities/mod.rs::on_block_replaced` spawns one `ExperienceOrbEntity` per entry, matching
  vanilla `getRecipesToAwardAndPopExperience`. Covered by
  `experience_per_recipe_returns_one_amount_for_each_used_recipe`.
  Caveat: the `on_take_item` path (`furnace_like_slot.rs`) still sums the per-recipe amounts into a
  single `award_experience` call; spawning one world orb there would need a `World`-aware method on
  `InventoryPlayer` implemented in `entity/player.rs` (outside this unit). Not re-opened because
  the row's suggested fix (per-recipe amounts + block-entity orb spawn) is present.
- **`furnace_like_slot.rs` take path (unlock recipes, M)** -- FIXED in tree. `on_take_item` passes
  the extracted recipe ids to `player.award_recipes()`; `screen_handler.rs` declares the trait
  method and `entity/player.rs` implements it (recipe book update, `RecipeUnlocked` for newly
  learned, `RecipeCrafted` for every used recipe, and a `CRecipeBookAdd` packet), matching
  `awardUsedRecipesAndPopExperience` + `Player.awardRecipes` + `ServerRecipeBook.addRecipes`.
- **`furnace_like_block_entity.rs:867-876` (Bedrock `StoredXPInt`, S)** -- already FIXED via
  `stored_experience()`.
- **`furnace_like_block_entity.rs:235-274` (result data components, S)** -- already FIXED via
  `assemble_cooking_result()`.
- **`furnace_like_block_entity.rs:225` (`isSameItemSameComponents`, S)** -- SATISFIED by the
  existing `are_items_and_components_equal` calls.
- **`furnace_like_block_entity.rs:856-858` (`getUpdateTag`, S)** -- verified correct: vanilla
  `AbstractFurnaceBlockEntity` has no `getUpdateTag` override, so the base `BlockEntity` returns an
  empty compound. Removed the stale comment that claimed the update tag carried items and
  cook/burn state, and documented the real vanilla quirk.

No new tests added: both seams (per-recipe XP extraction, recipe-id collection) already have pure
unit coverage. `rustfmt --edition 2024` run on `furnace_like_block_entity.rs`. No compile check run
(worker rule).
