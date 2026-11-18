# Final rows from docs/tracker/tpar-own-tsmall-batch2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: item.rs

All 15 STILL OPEN rows resolved. Each row is FIXED, VERIFIED against the cited .vendor vanilla path, or BLOCKED-DATA with the exact missing subsystem. Source edits this pass: commands/item.rs (set_count limitSize fix + test); the furnace and catalyst fixes were already present and re-verified.
Counts (superseded by the Result section below): FIXED 1, VERIFIED 13, BLOCKED-DATA 1 at the earlier pass. co-20e7ce:21 has since been fixed (score/storage providers) and is VERIFIED in the final Result.

## Result

All 15 rows from `tsmall-batch2.md` are closed; none remains STILL OPEN. Every fix was re-checked against decompiled vanilla under `.vendor/vanilla-decompiled/net/minecraft/`. No `.rs` file was edited this final pass (all fixes already present in the checkout and re-verified), so no rustfmt was needed.

| Row | Final status | Code + vanilla evidence |
|---|---|---|
| bl-231130:46 (relight rescale) | VERIFIED | `rescale_cooking_after_relight` (`furnace_like_block_entity.rs:46-62`) computes `ceil(spent/old_total*new_total)` while `spent < old_total`; called at ignition (:622). Vanilla `world/level/block/entity/AbstractFurnaceBlockEntity.java:278-282`. |
| bl-231130:47 (one XP orb per recipe) | VERIFIED | `experience_per_recipe` (:114-129) returns one amount per used recipe id; `furnace_like_slot.rs:163-166` forwards each to `player.rs:10560-10573` `award_experience_orbs`, which spawns one orb per amount. Vanilla `AbstractFurnaceBlockEntity.java:531-548,550-577`. |
| bl-231130:49 (Bedrock StoredXPInt) | VERIFIED | `stored_experience` (:103-109) floors each recipe's accrued XP; caller (:966-978) passes it to `build_furnace_actor`, which writes `StoredXPInt` (:1024); no hardcoded 0. Vanilla Bedrock cooking actor `StoredXPInt`; `AbstractFurnaceBlockEntity.java:531-548`. |
| bl-231130:50 (result components) | VERIFIED | `assemble_cooking_result` (:86-100) applies `recipe.result.components`, used in `can_accept_recipe_output` (:298) and crafting (:327). Vanilla `AbstractFurnaceBlockEntity.java:358-371`; generated field `RecipeResultStruct.components` exists in pumpkin-data. |
| bl-4d09af:63 (water_source migration) | VERIFIED | No `BlockIsReplacing::water_source()` call remains; source-only `is_water()` (`block/mod.rs:793`, `Self::Water(0)`) with test `is_water_matches_only_source_water`. Vanilla `world/level/material/Fluids.java:9-10` and `world/level/block/SlabBlock.java:69` / `ShelfBlock.java:130`. |
| bl-689b5a:47 (relight rescale) | VERIFIED | Same helper/call as bl-231130:46. Vanilla `AbstractFurnaceBlockEntity.java:278-282`. |
| bl-689b5a:91 (relight rescale) | VERIFIED | Same helper/call as bl-231130:46. Vanilla `AbstractFurnaceBlockEntity.java:278-282`. |
| bl-689b5a:92 (result components) | VERIFIED | Same `assemble_cooking_result` path as bl-231130:50. Vanilla `AbstractFurnaceBlockEntity.java:358-371`. |
| block-enti:1 (bloom particles) | VERIFIED | `sculk_catalyst.rs:175-187` spawns 2 `SculkSoul` particles at pos+(0.5,1.15,0.5) with spread (0.2,0,0.2). Vanilla `world/level/block/entity/SculkCatalystBlockEntity.java:124`. |
| block-enti:2 (cursor persistence) | VERIFIED | `from_nbt` (:89-100) calls `spreader.load`, `write_nbt` (:118-124) calls `spreader.save`. Vanilla `SculkCatalystBlockEntity.java:52-62` overrides `loadAdditional`/`saveAdditional` to persist `sculkSpread`. |
| block-enti:3 (hearing radius) | VERIFIED | `within_hearing_radius` (:202-206) is inclusive `<=` on floored block offsets; caller `on_game_event` (:82-84) passes floored `source_position`. Vanilla BY_DISTANCE dispatch `EuclideanGameEventListenerRegistry.java:115-124` rejects only when `distSqr > radiusSqr`. Test `catalyst_hearing_includes_boundary`. |
| block-enti:5 (world-seeded RNG) | VERIFIED | `tick` (:109-115) seeds the spreader from `world.next_loot_seed()`; bloom pitch (:188-197) uses the same source. Vanilla `SculkCatalystBlockEntity.java:47` passes `level.getRandom()`. |
| co-20e7ce:21 (number providers) | VERIFIED | All forms implemented: `number_provider_with` (`item.rs:1273-1312`) handles constant/uniform/binomial/sum, and `ProviderContext::resolve_score`/`resolve_storage` (:1186-1232) resolves score via scoreboard and storage via command storage, wired by both modify executors (:1353-1356, :1384-1387); test `score_and_storage_providers_fall_back_without_context` (:1766). Prior BLOCKED-DATA resolved in `tfin-taudit-crates-pumpkin-core-src-co-20e7ce-1.md`. Vanilla `world/level/storage/loot/providers/number/ints/ContextIntProviderTypes.java`, `ScoreboardValue.java:18-54`, `StorageValue.java:11-24`, `StoredNumberAccess.java:15-36`, `UniformGenerator.java`, `BinomialDistributionGenerator.java`, `Sum.java`. |
| co-20e7ce:22 (set_count clamp) | FIXED | `apply_item_modifier` ends with `stack.item_count = stack.item_count.min(stack.get_max_stack_size())` (`item.rs:935-936`), test `set_count_clamps_to_max_stack_size_like_vanilla_limit_size` (:1754). Vanilla `SetItemCountFunction.java:43-46` (no clamp) + `server/commands/item/ItemCommands.java:289` (`limitSize`) + `world/item/ItemStack.java:1309-1312`. |
| co-20e7ce:24 (modify feedback) | VERIFIED | `BlockModifyExecutor` (`item.rs:1361-1372`) sends `COMMANDS_ITEM_BLOCK_MODIFY_SUCCESS`; `EntityModifyExecutor` (:1402-1418) sends `.SINGLE`/`.MULTIPLE`. Vanilla `server/commands/item/ItemCommands.java:259` (`getModifySuccess`). |

Counts: FIXED 1 (co-20e7ce:22), VERIFIED 14, BLOCKED-DATA 0, STILL OPEN 0. No cargo run per task; no `.rs` file touched this final pass.
