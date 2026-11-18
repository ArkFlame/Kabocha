# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-689b5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Verified every STILL OPEN row against `AbstractFurnaceBlockEntity.java` in the
- Re-audit row 93 (FurnaceBurnEvent `consume_fuel`, M): STILL OPEN. Reason: adding the
Re-verified every STILL OPEN row against `AbstractFurnaceBlockEntity.java` in the
STILL OPEN only for plugin exposure: the v0.1/v0.2 WIT `furnace-burn-event` records have no

## Result (final pass, 2026-11-18)

All rows re-verified against
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java`
(Minecraft 26.3, Paper/Canvas). Every row is FIXED in tree; no source edit was needed this pass.

- First table row 44 / re-audit row 90 (data-driven speed multiplier, M): FIXED, VERIFIED.
  `tick` reads `CookingFuelImpl::get_speed_multiplier(is_fast)` (default 1.0) at ignition,
  `cooking_total_time` divides the recipe time under vanilla's `speedMultiplier > 0.0` guard, and
  both NBT paths persist/restore the float. Matches `getSpeedMultiplier` + `getTotalCookTime`
  (`:420-432`, applied at `:274-277`).
- First table row 45 (FurnaceStartSmeltEvent total, S): FIXED, VERIFIED. The event total is
  recomputed from the current recipe and the persisted multiplier with a 200 default
  (`furnace_like_block_entity.rs:672-683`), matching `getTotalCookTime(level, entity)` (`:434-440`)
  used at `:295-305`.
- First table row 46 (result count, S): FIXED, VERIFIED. `craft_recipe` increments the output by
  `recipe.result.count` (`:332`) and `can_accept_recipe_output` requires room for the whole count
  (`:303-306`), matching `canBurn`/`burn` (`:257`, `:358-404`).
- First table row 47 / re-audit row 91 (relight rescale, S): FIXED, VERIFIED.
  `rescale_cooking_after_relight` (`:46-62`) mirrors `:278-282` (`completionRatio * newTotal`, ceil,
  only while `cookingTimer < cookingTotalTime`).
- Re-audit row 92 (recipe result components, S): FIXED, VERIFIED. `assemble_cooking_result`
  (`:86-100`) applies `recipe.result.components` to the output stack, matching `recipe.assemble(input)`
  (`:256-257`).
- Re-audit row 93 (FurnaceBurnEvent `consume_fuel`, M): FIXED, VERIFIED. The core event carries
  `consume_fuel` (default true) and `tick` skips `consumeFuel` when it is false
  (`furnace_like_block_entity.rs:600-636`), matching `:284-287`
  (`if (burnEvent.willConsumeFuel()) consumeFuel(...)`). Plugin exposure is closed for v0.2:
  `consume-fuel: bool` is on the `furnace-burn-event` record
  (`crates/pumpkin-plugin-wit/v0.2/event.wit:1845-1853`) and the wasm host maps it in both
  directions (`crates/pumpkin-wasm-host-v0_2/src/events/inventory.rs:170-192`). v0.1 is deprecated
  and ABI-frozen, so its record shape was left unchanged by design per
  `crates/pumpkin-plugin-wit/AGENTS.md`.

No new source edits or tests were needed: all six rows were already FIXED in tree, and every
shared cooking helper keeps its pure-function tests
(`cooking_time_divides_by_the_persisted_multiplier`,
`relight_rescales_progress_by_the_multiplier_change`,
`assemble_cooking_result_applies_recipe_components`,
`furnace_speed_multiplier_round_trips_through_nbt`).
