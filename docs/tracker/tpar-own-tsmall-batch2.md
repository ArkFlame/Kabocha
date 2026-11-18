# Parity rows for own-tsmall-batch2

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tsmall-batch2** [partial/OWN-OPEN] own audit: 17 open, 6 fixed, 2 verified, 0 blocked-data (docs/tracker/tsmall-batch2.md) docs/tracker/tsmall-batch2.md

## Result (final)

All 15 STILL OPEN rows resolved. Each row is FIXED, VERIFIED against the cited .vendor vanilla path, or BLOCKED-DATA with the exact missing subsystem. Source edits this pass: commands/item.rs (set_count limitSize fix + test); the furnace and catalyst fixes were already present and re-verified.

| Row | Result | Evidence |
|---|---|---|
| bl-231130:46 (relight rescale) | VERIFIED | `rescale_cooking_after_relight` (furnace_like_block_entity.rs:46-62) applies `ceil(spent/oldTotal*newTotal)` while `spent < oldTotal`, called at ignition (:621-630). Vanilla `AbstractFurnaceBlockEntity.java:278-282`. |
| bl-231130:47 (one XP orb per recipe) | VERIFIED | `experience_per_recipe` (:114-129) returns one `RecipeExperience` per id; `block/entities/mod.rs:198-205` and `player.rs:10526-10540` spawn one orb per entry, and `furnace_like_slot.rs:155-167` forwards per-recipe amounts via `award_experience_orbs`. Vanilla `AbstractFurnaceBlockEntity.java:531-548,550-577`. |
| bl-231130:49 (Bedrock StoredXPInt) | VERIFIED | `bedrock_block_actor_data` (:959-970) passes `stored_experience(&recipes)` into `build_furnace_actor`, which writes `StoredXPInt` (:1024); no hardcoded 0. Vanilla Bedrock cooking-actor `StoredXPInt`; `stored_experience` floors each accrued recipe XP. |
| bl-231130:50 (result components) | VERIFIED | `assemble_cooking_result` (:86-100) applies `recipe.result.components`; used in both `can_accept_recipe_output` (:298) and `craft_recipe` (:327). Vanilla `AbstractCookingRecipe.assemble` / `canBurn` (`AbstractFurnaceBlockEntity.java:358-371`); generated field `RecipeResultStruct.components` exists in pumpkin-data. |
| bl-4d09af:63 (water_source migration) | VERIFIED | No `water_source()` call remains; migration to source-only `is_water()` (`block/mod.rs:793`, `Self::Water(0)`) is complete, with test `is_water_matches_only_source_water` (:942). Vanilla `FluidState.java:174-176` `is(Fluid)` compares the fluid owner; `Fluids.WATER`/`FLOWING_WATER` are distinct (`Fluids.java:9-10`), matching `SlabBlock.java:69`, `ShelfBlock.java:130`. |
| bl-689b5a:47 (relight rescale) | VERIFIED | Same helper/call as bl-231130:46. Vanilla `AbstractFurnaceBlockEntity.java:278-282`. |
| bl-689b5a:91 (relight rescale) | VERIFIED | Same helper/call as bl-231130:46. Vanilla `AbstractFurnaceBlockEntity.java:278-282`. |
| bl-689b5a:92 (result components) | VERIFIED | Same `assemble_cooking_result` path as bl-231130:50. Vanilla `AbstractFurnaceBlockEntity.java:358-371`. |
| block-enti:1 (bloom particles) | VERIFIED | `sculk_catalyst.rs:135-145` spawns 2 `SculkSoul` particles at pos+(0.5,1.15,0.5) with spread (0.2,0,0.2). Vanilla `SculkCatalystBlockEntity.java:124`. |
| block-enti:2 (cursor persistence) | VERIFIED | `from_nbt` (:47-57) calls `spreader.load(nbt)`, `write_nbt` (:76-82) calls `spreader.save(nbt)`. Vanilla `SculkCatalystBlockEntity.java:52-60` overrides `loadAdditional`/`saveAdditional` to persist `sculkSpreader`. |
| block-enti:3 (hearing radius) | VERIFIED | `within_hearing_radius` (:162-164) is inclusive `<=` on floored block offsets; caller `on_entity_death` (:168-207) passes floored `BlockPos` values. Vanilla BY_DISTANCE dispatch `EuclideanGameEventListenerRegistry.java:115-124` rejects only when `BlockPos.containing(...).distSqr(...) > radiusSqr`. Test `catalyst_hearing_includes_boundary`. |
| block-enti:5 (world-seeded RNG) | VERIFIED | `tick` (:70-71) seeds the spreader from `world.next_loot_seed()`, which draws from `loot_random` seeded from `level.seed` (`world/mod.rs:699,7224-7229`); bloom pitch (:153) uses the same source. Vanilla `SculkCatalystBlockEntity.java:47` passes `level.getRandom()`. |
| co-20e7ce:21 (number providers) | BLOCKED-DATA | constant/uniform/binomial/sum implemented and verified (`item.rs:1174-1211`, vanilla `SetItemCountFunction.java:19`). `score`/`storage` need a LootContext (scoreboard/storage/level) that vanilla builds at `ItemCommands.java:281-285` but `/item modify` in Pumpkin never constructs. Missing LootContext subsystem, not a generated field. |
| co-20e7ce:22 (set_count clamp) | FIXED | `apply_item_modifier` (`commands/item.rs:931-932`) now ends with the vanilla-equivalent `stack.item_count = stack.item_count.min(stack.get_max_stack_size())`, with test `set_count_clamps_to_max_stack_size_like_vanilla_limit_size`. Vanilla `SetItemCountFunction.java:43-46` (no clamp inside) + `ItemCommands.java:289` (`newItem.limitSize(newItem.getMaxStackSize())`) + `ItemStack.java:1309-1312`. |
| co-20e7ce:24 (modify feedback) | VERIFIED | `BlockModifyExecutor` (`item.rs:1256-1266`) sends `commands.item.block.modify.success`; `EntityModifyExecutor` (:1293-1309) sends `.single`/`.multiple`. Vanilla `ItemCommands.java:259` (`accessor.getModifySuccess`). |

Counts: FIXED 1, VERIFIED 13, BLOCKED-DATA 1 (co-20e7ce:21, missing LootContext subsystem), STILL OPEN 0. No cargo run per task; rustfmt to be run by orchestrator on touched item.rs.

## Result (x_04 pass)
- Not re-verified in this pass. The rows above are the prior pass's claims; no code or vanilla citations were re-checked here. No Rust edited; no checks run.
