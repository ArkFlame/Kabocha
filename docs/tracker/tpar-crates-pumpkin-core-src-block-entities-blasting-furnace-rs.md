# Parity rows for crates/pumpkin-core/src/block/entities/blasting_furnace.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-f802c2** [partial/OWN-VERIFIED] own audit: 0 open, 2 fixed, 3 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-f802c2.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-f802c2.md

## Result

THIRD PASS: closed the only row (the `own/...f802c2` audit row). Its referenced audit doc
`docs/tracker/taudit-crates-pumpkin-core-src-bl-f802c2.md` already carries four prior Result sections
closing all five of its rows; independently re-verified each against decompiled vanilla and the current
shared macro. No .rs edit or new test was required: `blasting_furnace.rs` is a thin struct + six
`impl_*_for_cooking!` macros with no logic of its own; every gap lives in the shared
`furnace_like_block_entity.rs`, which already matches vanilla. rustfmt: N/A (no .rs touched).

Vanilla reference: `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java`.

- NBT keys: FIXED. Vanilla `loadAdditional`/`saveAdditional` (:205-208, :223-226) use snake_case
  `cooking_time_spent` / `cooking_total_time` / `lit_time_remaining` / `lit_total_time`. The
  row's PascalCase-only premise is false. Pumpkin reads the vanilla key first with
  `read_saved_short(nbt, "cooking_time_spent", "CookTime")` (:789, PascalCase legacy fallback) and
  writes snake_case (:841, :923).
- `:555-557` lit with blocked output: VERIFIED. Vanilla `serverTick` :319-321 sets
  `cookingTimer = 0` when `canBurn` is false (blocked output); a null recipe has no `else` (:322)
  and freezes. Pumpkin's `else if is_top_items_empty || furnace_recipe.is_some()` arm (:727-730)
  resets on blocked output and empty input, freezing on a null recipe. Match; no change.
- `:558-569` cool-down branch: VERIFIED. Vanilla :326-327 `else if (cookingTimer > 0)
  clamp(timer - 2, 0, total)` has no `canAcceptRecipeOutput`/non-empty guard and is reached on
  `!(isLit || hasFuel && hasIngredient)`. Pumpkin's `else if !is_burning && cooking_time_spent > 0`
  (:732-738) reaches on the same negated condition and does `saturating_sub(2).min(total)`. Match.
- `:200-202` result merge: FIXED. Vanilla `canBurn` :364 uses `ItemStack.isSameItemSameComponents`;
  Pumpkin uses `are_items_and_components_equal` (:299, :331). The row's "item-only compare" premise
  is false.
- `:112-145` XP float accumulation: VERIFIED. Vanilla `createExperience` :550-555 =
  `floor(amount * value)` plus one `nextFloat() < frac` roll; `getRecipesToAwardAndPopExperience`
  :535-548 iterates `RecipesUsed`; `awardUsedRecipesAndPopExperience` :528 clears the map. Pumpkin's
  `experience_per_recipe`/`split_experience` (:66, :114) and `extract_experience_from_recipes`
  (:247) match, with direct unit tests (`split_experience_adds_the_fraction_only_when_the_roll_is_under_it`
  :1034, `experience_per_recipe_returns_one_amount_for_each_used_recipe` :1042,
  `read_saved_short_prefers_the_vanilla_key_over_the_legacy_one` :1065, NBT round-trip :1144-1175).

No generated-data, WIT, or cross-crate change needed. All rows CLOSED; no STILL OPEN remains.
