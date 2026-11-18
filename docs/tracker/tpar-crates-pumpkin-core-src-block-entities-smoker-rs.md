# Parity rows for crates/pumpkin-core/src/block/entities/smoker.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-689b5a** [partial/OWN-OPEN] own audit: 12 open, 12 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-689b5a.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-689b5a.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (pass 3, 2026-11-18)

Re-verified the single OWN-OPEN row (`own/taudit-crates-pumpkin-core-src-bl-689b5a`,
`docs/tracker/taudit-crates-pumpkin-core-src-bl-689b5a.md`) against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java`
(MC 26.3, Paper/Canvas) and against `SmokerBlockEntity.java` in the same tree. The row's
stale "12 open" tally is resolved: every one of its 12 rows is FIXED in the current tree,
so no source change was needed and rustfmt was not applicable.

Smoker behaviour is driven by the five macros in `smoker.rs` (struct at lines 23-67 and
`impl_*_for_cooking!` at lines 69-74); the cooking logic lives in the shared
`furnace_like_block_entity.rs`. Each row verified against the cited vanilla file:

- Data-driven speed multiplier (first table row 44 / re-audit row 90, M): VERIFIED/FIXED.
  `tick` resolves the fuel multiplier via `CookingFuelImpl::get_speed_multiplier(is_fast)`
  (`furnace_like_block_entity.rs:593-594`), `cooking_total_time` divides the recipe time by it
  with vanilla's `speedMultiplier > 0.0F` guard (`:32-41`), `getSpeedMultiplier` +
  `getTotalCookTimeVanilla` at `AbstractFurnaceBlockEntity.java:420-432`, and NBT persists/restores
  the float (`:844`, `:798-802`) per vanilla `saveAdditional`/`loadAdditional`
  (`AbstractFurnaceBlockEntity.java:209,227`). Tests: `cooking_time_divides_by_the_persisted_multiplier`,
  `furnace_speed_multiplier_round_trips_through_nbt`.
- FurnaceStartSmeltEvent total (row 45, S): VERIFIED/FIXED. Event total recomputed from the current
  recipe via `cooking_total_time` with a 200 default (`:672-683`), matching Paper
  `getTotalCookTime(level, entity)` (`AbstractFurnaceBlockEntity.java:295-305,434-440`).
- Result count (row 46, S): VERIFIED/FIXED. `craft_recipe` grows output by `recipe.result.count`
  and `can_accept_recipe_output` requires room for the whole count, matching vanilla `canBurn`/
  `burn` (`AbstractFurnaceBlockEntity.java:358-370,390-404`).
- Relight rescale (row 47 / re-audit row 91, S): VERIFIED/FIXED.
  `rescale_cooking_after_relight` (`:46-62`) implements the `oldTotal > 0 && spent < oldTotal`
  guarded `ceil(completionRatio * newTotal)`, matching `AbstractFurnaceBlockEntity.java:278-281`.
  Test: `relight_rescales_progress_by_the_multiplier_change`.
- Recipe result components (re-audit row 92, S): VERIFIED/FIXED. `assemble_cooking_result`
  (`:86-100`) applies `recipe.result.components`, matching `recipe.assemble(input)` used in
  `canBurn`/`burn` (`AbstractFurnaceBlockEntity.java:364,373-398`). Test:
  `assemble_cooking_result_applies_recipe_components`.
- FurnaceBurnEvent `consume_fuel` (re-audit row 93, M): VERIFIED/FIXED. Core event field
  (`plugin/api/events/inventory/furnace_burn.rs:20`) and tick honouring it
  (`furnace_like_block_entity.rs:600,608,636`) match Paper `burnEvent.willConsumeFuel()`
  (`AbstractFurnaceBlockEntity.java:284-287`); plugin exposure added to the v0.2
  `furnace-burn-event` record (`pumpkin-plugin-wit/v0.2/event.wit:1851`) and both conversion
  directions in `pumpkin-wasm-host-v0_2/src/events/inventory.rs:178,188`. v0.1 stays frozen for
  ABI compatibility (hardcoded `consume_fuel: true`, deprecated version).

Also confirmed clean against `SmokerBlockEntity.java`: `RecipesUsed` XP map, 3 slots in
input/fuel/result order, and window/stat wiring. No BLOCKED-DATA rows: every missing field
(`speed_multiplier` float, current snake_case NBT keys) is already present in generated data
and the shared helpers.
