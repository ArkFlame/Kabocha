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
