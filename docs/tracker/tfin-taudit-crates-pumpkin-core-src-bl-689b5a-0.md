# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-689b5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs

| `furnace_like_block_entity.rs:484-496` | `getSpeedMultiplier` reads the fuel item's `CookingFuel.speedMultiplier` (`ResolvableFloat`, `cooking/speed_default` is conditional on `FAST_FURNACE`) | multiplier is derived from the block kind; a datapack fuel with an explicit `speed_multiplier` is ignored, and the value is not restored from NBT | store `speed_multiplier` on the three entities, fill it from `CookingFuelImpl::get_speed_multiplier(is_fast)` at ignition, read/write the float NBT key | M, FIXED/VERIFIED (`.vendor/.../AbstractFurnaceBlockEntity.java:420-421,428-431`) |
| `furnace_like_block_entity.rs:542-556` | Paper passes `getTotalCookTime(level, entity)` (recipe time / multiplier, default 200) to `FurnaceStartSmeltEvent` | event is fired with the stored total, which is 0 when the input arrived before `set_stack` set it | compute from `furnace_recipe` with `cooking_total_time`, default 200 | S, FIXED/VERIFIED (`.vendor/.../AbstractFurnaceBlockEntity.java:295-305,434-440`) |
| `furnace_like_block_entity.rs:233-237` | `burn` adds `burnResult.count()` to the result slot | always increments by 1 | increment by `recipe.result.count` and bound by `can_accept_recipe_output` | S, FIXED/VERIFIED (`.vendor/.../AbstractFurnaceBlockEntity.java:358-404`) |
| `furnace_like_block_entity.rs:520-530` | on relight with a different fuel multiplier, `cookingTimer` is rescaled to `ceil(timer / oldTotal * newTotal)` | no rescale (only relevant once the multiplier is data-driven, gap 1) | rescale after gap 1 | S, FIXED/VERIFIED (`.vendor/.../AbstractFurnaceBlockEntity.java:278-282`) |

## Result

All four rows verified against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java` (MC 26.3).

- `furnace_like_block_entity.rs:484-496` (data-driven speed multiplier, M): FIXED/VERIFIED. Persisted `speed_multiplier_bits` on furnace.rs/smoker.rs/blasting_furnace.rs (default 1.0), set from `CookingFuelImpl::get_speed_multiplier(is_fast)` at ignition (`:593-594`), drives `cooking_total_time` (`:32-41`) with vanilla's `speedMultiplier > 0.0F` guard, persisted/restored under the `speed_multiplier` float key (`:798-802`, `:844`, `:926`). Matches `getSpeedMultiplier` + `getTotalCookTimeVanilla` (`AbstractFurnaceBlockEntity.java:420-432`).

- `furnace_like_block_entity.rs:542-556` (FurnaceStartSmeltEvent total, S): FIXED/VERIFIED. Event total recomputed from the current recipe via `cooking_total_time` with a 200 default (`:672-683`), matching `getTotalCookTime(level, entity)` (`AbstractFurnaceBlockEntity.java:434-440`) used at `:295-305`.

- `furnace_like_block_entity.rs:233-237` (result count, S): FIXED/VERIFIED. `craft_recipe` increments the output slot by `recipe.result.count` (`:332`) and `can_accept_recipe_output` requires room for the whole count (`:303-306`), matching `canBurn`/`burn` (`AbstractFurnaceBlockEntity.java:358-404`).

- `furnace_like_block_entity.rs:520-530` (relight rescale, S): FIXED/VERIFIED. `rescale_cooking_after_relight` (`:46-62`) implements `ceil(timer / oldTotal * newTotal)` under the `oldTotal > 0 && spent < oldTotal` guard, called at ignition (`:621-630`), matching `AbstractFurnaceBlockEntity.java:278-282`. Tests: `cooking_time_divides_by_the_persisted_multiplier`, `relight_rescales_progress_by_the_multiplier_change`, `furnace_speed_multiplier_round_trips_through_nbt`, `assemble_cooking_result_applies_recipe_components`.

