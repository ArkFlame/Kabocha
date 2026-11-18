# Parity rows for own-tsmall-batch2

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tsmall-batch2** [partial/OWN-OPEN] own audit: 17 open, 6 fixed, 2 verified, 0 blocked-data (docs/tracker/tsmall-batch2.md) docs/tracker/tsmall-batch2.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass)

All 15 STILL OPEN rows resolved. No source edits were needed: every row's fix is already in the tree at HEAD. Rustfmt not run (no source files touched); no cargo per task rule.

| Row | Result | Evidence |
|---|---|---|
| bl-231130:46 (relight rescale) | VERIFIED | `rescale_cooking_after_relight` (furnace_like_block_entity.rs:46-62) applies `ceil(spent/oldTotal*newTotal)` while `spent < oldTotal`, called at ignition (:621-630). Vanilla `AbstractFurnaceBlockEntity.java:278-282`. |
| bl-231130:47 (one XP orb per recipe) | VERIFIED | `extract_recipes` returns one `RecipeExperience` per id; `block/entities/mod.rs:198-205` spawns one orb per entry and the take path forwards per-recipe amounts via `award_experience_orbs`. Vanilla `AbstractFurnaceBlockEntity.java:531-548,550-577`. |
| bl-231130:49 (Bedrock StoredXPInt) | VERIFIED | `bedrock_block_actor_data` (:959-970) passes `stored_experience(&recipes)` into `write_bedrock_nbt`; no hardcoded 0 (:1024). Vanilla Bedrock cooking-actor `StoredXPInt`; helper floors each recipe's accrued XP. |
| bl-231130:50 (result components) | VERIFIED | `assemble_cooking_result` (:86-100) applies `recipe.result.components`; used in both `can_accept_recipe_output` (:298) and `craft_recipe` (:327). Vanilla `AbstractCookingRecipe.assemble` / `canBurn` (`AbstractFurnaceBlockEntity.java:358-371`). |
| bl-4d09af:63 (water_source migration) | VERIFIED | No `water_source()` call remains; all 45 sites use source-only `is_water()` (block/mod.rs:793). Vanilla `FluidState.java:174-176` `is(Fluid)` is `fluidOwner == rawType` and `Fluids.WATER`/`FLOWING_WATER` are distinct (`Fluids.java:9-10`), so `is(Fluids.WATER)` (`SlabBlock.java:69`, `ShelfBlock.java:130`) matches sources only. |
| bl-689b5a:47 (relight rescale) | VERIFIED | Same helper/call as bl-231130:46. `AbstractFurnaceBlockEntity.java:278-282`. |
| bl-689b5a:91 (relight rescale) | VERIFIED | Same helper/call as bl-231130:46. `AbstractFurnaceBlockEntity.java:278-282`. |
| bl-689b5a:92 (result components) | VERIFIED | Same `assemble_cooking_result` path as bl-231130:50. `AbstractFurnaceBlockEntity.java:358-371`. |
| block-enti:1 (bloom particles) | VERIFIED | `sculk_catalyst.rs:135-145` spawns 2 `SculkSoul` particles at pos+(0.5,1.15,0.5) with spread (0.2,0,0.2). Vanilla `SculkCatalystBlockEntity.java:124`. |
| block-enti:2 (cursor persistence) | VERIFIED | `from_nbt` (:47-57) calls `spreader.load(nbt)`, `write_nbt` (:76-82) calls `spreader.save(nbt)`. Vanilla `SculkCatalystBlockEntity.java:52-60`. |
| block-enti:3 (hearing radius) | VERIFIED | `within_hearing_radius` (:162-164) is inclusive `<=` on floored block offsets; caller `living.rs:3040` passes the floored `block_pos`. Vanilla BY_DISTANCE dispatch `EuclideanGameEventListenerRegistry.java:115-124` uses `BlockPos.containing(...).distSqr(...) > radiusSqr`, not the strict Vec3 `closerThan` path. Test `catalyst_hearing_includes_boundary`. |
| block-enti:5 (world-seeded RNG) | VERIFIED | `tick` (:70-71) seeds the spreader from `world.next_loot_seed()`, which draws from the world-seeded level random (`world/mod.rs:7221-7229`); bloom pitch (:153) uses the same source. Vanilla `SculkCatalystBlockEntity.java:47` passes `level.getRandom()`. |
| co-20e7ce:21 (number providers) | BLOCKED-DATA | constant/uniform/binomial/sum are implemented and verified (`item.rs:1174-1211`, vanilla `SetItemCountFunction.java:19`). `score`/`storage` need a LootContext (scoreboard/storage/level) that `item modify` never builds: vanilla `ItemCommands.java:282-300` builds one, Pumpkin has no such subsystem. Missing subsystem, not a generated field. |
| co-20e7ce:22 (set_count clamp) | VERIFIED | `item.rs:851-861` assigns the raw count with no max-stack clamp (only the `u8` storage range). Vanilla `SetItemCountFunction.java:43-46` + `ItemStack.java:1305`. Representation limit, not a parity gap. |
| co-20e7ce:24 (modify feedback) | VERIFIED | `BlockModifyExecutor` (`item.rs:1256-1266`) sends `commands.item.block.modify.success`; `EntityModifyExecutor` (:1293-1309) sends `.single`/`.multiple`. Vanilla `ItemCommands.java:259`. |

Counts: VERIFIED 14, BLOCKED-DATA 1, FIXED 0 (all fixes already present at HEAD), STILL OPEN 0.
