# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-block-entities-blasting-furnace-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No generated-data, WIT, or cross-crate change needed. All rows CLOSED; no STILL OPEN remains.

## Result (FINAL PASS, independently re-verified against .vendor decompiled vanilla)

Re-read every row directly against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java`
and the current shared macro in
`crates/pumpkin-core/src/block/entities/furnace_like_block_entity.rs`. Every row premise predicting
a Pumpkin divergence is false for this decompile; the already-applied fixes match vanilla. No
generated-data field, WIT change, or cross-crate refactor is required. `blasting_furnace.rs` is a
thin struct plus six `impl_*_for_cooking!` macros with no logic of its own, so all behaviour lives
in the shared macro, which is verified below. No `.rs` file touched; rustfmt N/A.

- NBT keys: FIXED. Vanilla `loadAdditional` / `saveAdditional` (:205-208, :223-226) use snake_case
  `cooking_time_spent` / `cooking_total_time` / `lit_time_remaining` / `lit_total_time`. Pumpkin
  reads these vanilla keys first with `read_saved_short` (:783-795, PascalCase legacy fallback) and
  writes snake_case (:840-843). The row's PascalCase-only premise is false.
- `:555-557` lit with blocked output: VERIFIED. Vanilla `serverTick` :319-321 sets
  `cookingTimer = 0` when `canBurn` is false (blocked output) and has no `else` for a null recipe
  (:322), freezing it. Pumpkin's `else if is_top_items_empty || furnace_recipe.is_some()` arm
  (:727-731) resets on blocked output and empty input, freezing only on a null recipe. Match.
- `:558-569` cool-down branch: VERIFIED. Vanilla :326-327 `else if (cookingTimer > 0)
  clamp(timer - 2, 0, total)` has no `canAcceptRecipeOutput` / non-empty guard and is reached on
  `!(isLit || hasFuel && hasIngredient)`. Pumpkin's `else if !is_burning && cooking_time_spent > 0`
  (:732-743) does `saturating_sub(2).min(total)` on the same negated condition. Match.
- `:200-202` result merge: FIXED. Vanilla `canBurn` :364 uses
  `ItemStack.isSameItemSameComponents`; Pumpkin uses `are_items_and_components_equal` (:299, :331).
  The row's "item-only compare" premise is false.
- `:112-145` XP float accumulation: VERIFIED. Vanilla `createExperience` :550-555 = `floor(count *
  xp)` plus one `nextFloat() < frac` roll; `getRecipesToAwardAndPopExperience` :535-548 iterates
  `RecipesUsed`; `awardUsedRecipesAndPopExperience` :528 clears the map. Pumpkin's
  `experience_per_recipe` / `split_experience` (:66, :114) and `extract_experience_from_recipes`
  (:247-268) match, with direct unit tests
  (`split_experience_adds_the_fraction_only_when_the_roll_is_under_it` :1034,
  `experience_per_recipe_returns_one_amount_for_each_used_recipe` :1042,
  `read_saved_short_prefers_the_vanilla_key_over_the_legacy_one` :1065, NBT round-trip).

All rows CLOSED: 2 FIXED, 3 VERIFIED, 0 STILL OPEN, 0 BLOCKED-DATA. No new test seam exists: the
tick reset/cool-down decisions live in the macro-generated `tick` requiring `Arc<World>` plus the
plugin manager; the pure arithmetic seams are already directly unit-tested.
