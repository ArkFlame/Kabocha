# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-c13d0e.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 


## Result
- VERIFIED, parse_result count 1..=99 (row 5 and row 6 / row 44, same issue): `parse_result` in `crates/pumpkin-core/src/data/datapack/recipe_loader.rs` defaults an absent count to 1 and rejects a present count that is not an integer in 1..=99 (non-u64, 0, or >99 returns `None`) instead of clamping or defaulting. Matches vanilla `.vendor/vanilla-decompiled/net/minecraft/world/item/ItemStackTemplate.java` (`ExtraCodecs.intRange(1, 99).optionalFieldOf("count", 1)` in `MAP_CODEC`). Covered by existing tests `result_count_must_be_within_one_to_ninety_nine` and `result_count_must_be_an_integer`; no source change needed.
- VERIFIED, parse_cooking checked int/float conversion (row 7 / row 45): `parse_cooking` in `crates/pumpkin-core/src/data/datapack/recipe_loader.rs` parses `cookingtime` with `i32::try_from(t.as_i64()?).ok()?` (rejecting non-integer or out-of-i32-range values instead of wrapping) and parses `experience` from `as_f64()? as f32` with a finite check (rejecting overflow to infinity). Matches vanilla `.vendor/vanilla-decompiled/net/minecraft/world/item/crafting/AbstractCookingRecipe.java` (`cookingMapCodec`: `Codec.INT.fieldOf("cookingtime")`, `Codec.FLOAT.optionalFieldOf("experience", 0.0F)`). Covered by existing test `cooking_time_and_experience_use_checked_conversion`; no source change needed.

Status: VERIFIED (all rows already fixed in source with tests; no `.rs` edits, no rustfmt run, not compiled by worker).
