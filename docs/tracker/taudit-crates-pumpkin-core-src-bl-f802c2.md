# Audit: crates/pumpkin-core/src/block/entities/blasting_furnace.rs

Row: audit / coverage AUDIT. Tracker note: 2 markers.

## Markers
Only two comment lines mention vanilla; both are accurate, not TODOs:
- `blasting_furnace.rs:35` `RecipesUsed` NBT format comment (matches vanilla key).
- `blasting_furnace.rs:39` `experience` fraction comment (matches vanilla `AbstractFurnaceBlockEntity.experience`).

## File result
No logic of its own: struct + `new()` + six macros. Diffed against `furnace.rs`/`smoker.rs`:
identical apart from ID and `CookingRecipeKind` (Blasting). Registration checked and correct:
`block/entities/mod.rs:406` (`minecraft:blast_furnace` NBT id), `:656` (block id), `block/blocks/blast_furnace.rs`
(`WindowType::BlastFurnace`, `translate` display name, comparator, XP via generic `on_block_replaced`).
Fuel halving matches vanilla `BlastFurnaceBlockEntity#getBurnDuration` (`super/2`, truncating):
`pumpkin-data/.../utility.rs:142`. No local gap found.

## Gaps in the shared cooking macro that affect blast-furnace behaviour
All in `crates/pumpkin-core/src/block/entities/furnace_like_block_entity.rs`
(shared with `furnace.rs` and `smoker.rs`; not blasting-specific, so left to a macro-level task).

1. `furnace_like_block_entity.rs:609-624,661-664` — NBT keys are snake_case
   (`cooking_time_spent`, `cooking_total_time`, `lit_time_remaining`, `lit_total_time`), while vanilla
   `AbstractFurnaceBlockEntity#readAdditional/saveAdditional` uses `CookTime`, `CookTimeTotal`, `BurnTime`,
   `BurnTimeTotal`. A vanilla world/region file loads a blast furnace at 0 progress/0 burn and Pumpkin then
   rewrites the chunk, destroying the values. Fix: read vanilla keys as fallback (and write vanilla keys), or
   migrate at chunk load. Size: M (touches furnace/smoker saves too; needs vanilla key names confirmed from
   decompiled 1.21.x).

2. `furnace_like_block_entity.rs:555-557` — when lit but the output slot blocks the result
   (`can_accept_output == false`), Pumpkin resets `cooking_time_spent` to 0. Vanilla
   `AbstractFurnaceBlockEntity#tick` only increments while output is acceptable and otherwise leaves
   progress untouched, so a furnace resumes near completion once the output is freed; Pumpkin restarts.
   Fix: drop the reset, only clamp in the `!is_burning` branch. Size: S (shared macro, affects furnace/smoker).

3. `furnace_like_block_entity.rs:558-569` — the cool-down branch lacks vanilla's
   `canAcceptRecipeOutput` guard (and input/fuel non-empty guard), so progress decays with an unloadable
   input where vanilla would not. Size: S.

4. `furnace_like_block_entity.rs:200-202` — merging results requires
   `are_items_and_components_equal`; vanilla only compares the item (`ItemStack.is(Item)`), so datapack
   blast-furnace recipes that return component-differing results are rejected. Size: S.

5. `furnace_like_block_entity.rs:112-145` — XP is accumulated per recipe into a float and only the floor is
   paid out; vanilla derives XP from `RecipesUsed` at take time (`awardUsedRecipes`, with the remaining
   fraction kept). Observable difference only for large batches/datapack XP. Size: M (needs vanilla
   `awardUsedRecipes` port).

## Gap status
- Shared cooking macro gaps (snake_case NBT keys): FIXED. Vanilla `loadAdditional`/`saveAdditional` uses snake_case keys; Pumpkin reads them with `read_saved_short` and writes snake_case.
- `:555-557` lit with blocked output: VERIFIED (matches vanilla, `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java` `serverTick` :319-321 resets `cookingTimer = 0` on blocked output; no change needed).
- `:558-569` cool-down branch: VERIFIED (matches vanilla, `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java` `serverTick` :326-327 `else if (cookingTimer > 0)` has no extra guard; no change needed).
- `:200-202` result merge requires component equality: FIXED (`:299,331` component-aware compare, matching vanilla `canBurn` :364 `ItemStack.isSameItemSameComponents`).
- `:112-145` XP float accumulation: VERIFIED (matches vanilla, `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java` `createExperience` :550-555 floor + fraction roll, `getRecipesToAwardAndPopExperience` :535-548; no change needed).


## Result

Verified all three originally open rows against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java` (`serverTick`, `createExperience`). The current `furnace_like_block_entity.rs` shared macro already matches vanilla in every case, so no source change was required. The tick reset/cool-down decisions live inside the macro-generated `tick` (needs `Arc<World>` + plugin manager), so there is no pure seam to add a test for them; the XP seam (`split_experience`, `stored_experience`) is already directly unit-tested below.

- `:555-557` lit with blocked output: VERIFIED (no change). Current macro resets `cooking_time_spent` when lit and `can_accept_output == false` (the `else if is_top_items_empty || furnace_recipe.is_some()` arm of `if is_burning && can_accept_output`). Vanilla `serverTick` line 320 does the same (`cookingTimer = 0` when `canBurn` is false). The row's premise that vanilla leaves progress untouched is not true for this decompiled 1.21.x source (also true in Mojang 1.21.x: blocked output -> `canAcceptRecipeOutput` false -> reset).
- `:558-569` cool-down branch: VERIFIED (no change). Current macro decays progress only in the outer `else`, reached when `!is_burning && (fuel_empty || input_empty)`, matching vanilla `serverTick` line 326 `else if cookingTimer > 0 { clamp(cookingTimer - 2, 0, cookingTotalTime) }`. An unloadable input (ingredient present, recipe null) stays inside the outer `if` and freezes progress, matching vanilla's recipe-null path. The row's claimed extra `canAcceptRecipeOutput`/input-fuel guard does not exist in vanilla's cool-down branch.
- `:112-145` XP float accumulation: VERIFIED (no change). `extract_experience_from_recipes` now derives XP from `RecipesUsed` at take time with one `split_experience` roll per recipe (`floor(count * xp) + P(roll < frac)`), matching vanilla `createExperience` lines 550-555 and clearing the map like `awardUsedRecipesAndPopExperience` line 528. `split_experience` and `stored_experience` have direct unit tests.

Source files changed: none. rustfmt: not applicable (no .rs file touched).

Re-verified 2026-11-18 against `AbstractFurnaceBlockEntity.java`: `serverTick` line 320 resets
`cookingTimer = 0` on blocked output (row 2 premise false); lines 326-327 cool-down has no extra
`canAcceptRecipeOutput`/non-empty guards (row 3 premise false); `getRecipesToAwardAndPopExperience`
(lines 535-548) + `createExperience` (lines 550-555) award per-recipe floor + fraction roll, matched
by `experience_per_recipe`/`split_experience` with direct unit tests (row 5 already fixed). All three
rows CLOSED, no source edit, no new test seam needed.

## Result (independent re-verification)

Re-checked each originally open row directly against decompiled
`world/level/block/entity/AbstractFurnaceBlockEntity.java` and the current shared macro. No source
file needed editing; the existing `## Result` above is confirmed accurate.

- `:555-557` lit with blocked output: VERIFIED (no change). Vanilla `serverTick` line 257 gates on
  `canBurn`, and the output-blocked `else` at lines 319-321 sets `cookingTimer = 0`; a null recipe
  has no `else` and freezes. Pumpkin maps this exactly: `can_accept_recipe_output` returns false for
  output blocked/null recipe, and the `else if is_top_items_empty || furnace_recipe.is_some()` arm
  (line 727) resets only for empty input or a found recipe, freezing on null recipe. The row's
  "vanilla leaves progress untouched" premise is false for this decompile.
- `:558-569` cool-down branch: VERIFIED (no change). Vanilla's outer `else if (cookingTimer > 0)` at
  lines 326-327 has no `canAcceptRecipeOutput`/input-fuel guard and clamps `cookingTimer - 2` to
  `[0, cookingTotalTime]`. Pumpkin's `else if !is_burning && cooking_time_spent > 0` at line 732
  does `saturating_sub(2).min(total)`, identical, reached on the same `!(isLit || hasFuel &&
  hasIngredient)` condition. The row's claimed extra guard does not exist in vanilla.
- `:112-145` XP float accumulation: VERIFIED (no change). `extract_experience_from_recipes` (lines
  247-267) derives XP from `RecipesUsed` at take time via `experience_per_recipe`, rolls each
  recipe's fraction once in `split_experience` (floor + `frac != 0 && roll < frac`), and clears the
  map. Matches vanilla `getRecipesToAwardAndPopExperience`/`createExperience` lines 535-555 and
  `awardUsedRecipesAndPopExperience` line 528. `split_experience`, `experience_per_recipe`,
  `stored_experience`, and the NBT round trip already have direct unit tests.

No pure seam needed: the tick reset/cool-down decisions live in the macro-generated `tick` that
requires `Arc<World>` + plugin manager. Files changed: none. rustfmt: not applicable.


## Result (CODER re-verification, 2026-11-18)

Checked every originally open row against `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java` (1.21.x decompile, Canvas/Paper patches included) and the current shared cooking macro in `crates/pumpkin-core/src/block/entities/furnace_like_block_entity.rs`.

- `:555-557` lit with blocked output: VERIFIED, no change. Vanilla `serverTick` `canBurn(...)` false arm is `:319-321` `entity.cookingTimer = 0;`; Pumpkin's `else if is_top_items_empty || furnace_recipe.is_some()` arm (`:727-731`) resets on blocked output and on empty input, and freezes only on a null recipe (vanilla has no else there, `:251-322`). The row premise (vanilla leaves progress untouched) is false for this source.
- `:558-569` cool-down branch: VERIFIED, no change. Vanilla `:326-327` `else if (cookingTimer > 0) cookingTimer = Mth.clamp(cookingTimer - 2, 0, cookingTotalTime)` has no `canAcceptRecipeOutput`/non-empty guards; Pumpkin `:732-743` does `saturating_sub(2).min(total)` on the same `!(isLit || hasFuel && hasIngredient)` condition.
- `:112-145` XP float accumulation: VERIFIED, no change. `extract_experience_from_recipes` (`:247-268`) derives XP from `RecipesUsed` at take time via `experience_per_recipe`/`split_experience` (floor plus one fraction roll per recipe id) and clears the map, matching `getRecipesToAwardAndPopExperience` (`:535-548`), `createExperience` (`:550-555`) and `awardUsedRecipesAndPopExperience` `:528`. Direct unit tests exist (`:1033-1062`, `:1181-1187`).
- Re-confirmed the two rows already marked FIXED: `saveAdditional` (`:221-231`) uses `cooking_time_spent`/`cooking_total_time`/`lit_time_remaining`/`lit_total_time`, matching `read_saved_short`; `canBurn` `:364` uses `ItemStack.isSameItemSameComponents`, matching `are_items_and_components_equal` at `:299,331`.

No source file changed. The remaining tick decisions live inside the macro-generated `tick` that needs `Arc<World>` plus the plugin manager, so there is no pure seam for a new unit test; the arithmetic seams (`cooking_total_time`, `rescale_cooking_after_relight`, `split_experience`, `experience_per_recipe`, `read_saved_short`) are already covered.

Note (not a gap, no action): a relight whose fuel reports 0 burn ticks skips Pumpkin's rescale, while vanilla rescales `cookingTotalTime` before resetting `cookingTimer` at `:317`. Both end with 0 spent progress; only the stored total differs until the next recipe.


## Result (SECOND PASS, independent re-verification)

Re-read the three originally open rows against decompiled
.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java
(serverTick lines 250-328, createExperience lines 550-555) and the current shared macro in
furnace_like_block_entity.rs. All three row premises are false for this source; the code already
matches vanilla. No source edit or new test was required.

- :555-557 lit with blocked output: VERIFIED, no change. Vanilla canBurn false arm is
  :319-321 entity.cookingTimer = 0;, so vanilla DOES reset on blocked output. Pumpkin's
  else if is_top_items_empty || furnace_recipe.is_some() arm (:727-731) resets on blocked
  output and empty input, freezing only on a null recipe (vanilla has no else at :322). Match.
- :558-569 cool-down branch: VERIFIED, no change. Vanilla's outer else if (cookingTimer > 0)
  (:326-327) has no canAcceptRecipeOutput/non-empty guard; an unloadable input keeps the outer
  if (isLit || hasFuel && hasIngredient) true and freezes via the recipe-null path. Pumpkin's
  else if !is_burning && cooking_time_spent > 0 (:732-743) reaches only on the same negated
  outer condition and does saturating_sub(2).min(total). Match.
- :112-145 XP float accumulation: VERIFIED, no change. extract_experience_from_recipes
  (:247-268) derives XP from RecipesUsed at take time via experience_per_recipe /
  split_experience (floor plus one fraction roll per recipe id) and clears the map, matching
  getRecipesToAwardAndPopExperience (:535-548) and createExperience (:550-555). Direct
  unit tests exist: split_experience_adds_the_fraction_only_when_the_roll_is_under_it (:1034),
  experience_per_recipe_returns_one_amount_for_each_used_recipe (:1042),
  read_saved_short_prefers_the_vanilla_key_over_the_legacy_one (:1065),
  stored_experience_floors_each_recipes_accrued_xp (:1224).

Also confirmed vanilla decompile writes snake_case NBT keys (cooking_time_spent,
cooking_total_time, lit_time_remaining, lit_total_time at :205-208,223-226), matching
Pumpkin's read_saved_short/saveAdditional. No generated-data, WIT or cross-crate change needed.

Files changed: none (tracker doc status lines updated to CLOSED to stop re-dispatch). rustfmt: N/A.


## Result (THIRD PASS, verified against .vendor vanilla decompile)

Independently re-read all five rows against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java`
and the current shared macro in
`crates/pumpkin-core/src/block/entities/furnace_like_block_entity.rs`.
Every row premise that predicted a Pumpkin divergence is false for this decompile; the already-applied
fixes match vanilla. No further source edit or new test was required. Terminal labels now use
FIXED / VERIFIED as required; no open row remains.

- NBT keys (row 1): FIXED. Vanilla `loadAdditional`/`saveAdditional` (:205-208, :223-226) use
  `cooking_time_spent` / `cooking_total_time` / `lit_time_remaining` / `lit_total_time`. Pumpkin reads
  those vanilla keys first with `read_saved_short` (:783-795, PascalCase legacy fallback) and writes
  snake_case (:841, :923). The row's PascalCase-only premise is false.
- `:555-557` lit with blocked output: VERIFIED. Vanilla `serverTick` :319-321 sets `cookingTimer = 0`
  when `canBurn` is false (blocked output) and has no `else` for a null recipe (:322), freezing it.
  Pumpkin resets via `else if is_top_items_empty || furnace_recipe.is_some()` (:727-731) on blocked
  output and empty input, freezing on a null recipe. Match.
- `:558-569` cool-down branch: VERIFIED. Vanilla :326-327 `else if (cookingTimer > 0) clamp(timer - 2,
  0, total)` has no `canAcceptRecipeOutput`/non-empty guard, reached on
  `!(isLit || hasFuel && hasIngredient)`. Pumpkin :732-743 reaches on the same negated outer condition
  and does `saturating_sub(2).min(total)`. Match.
- `:200-202` result merge: FIXED. Vanilla `canBurn` :364 uses `ItemStack.isSameItemSameComponents`;
  Pumpkin uses `are_items_and_components_equal` (:299, :331). The row's "vanilla only compares the item"
  premise is false.
- `:112-145` XP float accumulation: VERIFIED. Vanilla `createExperience` :550-555 = floor(count*xp)
  plus one `nextFloat() < frac` roll; `getRecipesToAwardAndPopExperience` :535-548 iterates
  `RecipesUsed`; `awardUsedRecipesAndPopExperience` :528 clears the map. Pumpkin's
  `experience_per_recipe`/`split_experience` (:111-129) and `extract_experience_from_recipes`
  (:247-268) match, with direct unit tests. No generated-data, WIT or cross-crate change needed.

Files changed: tracker doc only (status labels + this Result). No `.rs` touched; rustfmt N/A.
