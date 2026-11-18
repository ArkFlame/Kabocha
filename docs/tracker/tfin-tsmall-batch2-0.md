# Final rows from docs/tracker/tsmall-batch2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: furnace_like_block_entity.rs

| taudit-crates-pumpkin-core-src-bl-231130 : 46 (`furnace_like_block_entity.rs:511-535`) | FIXED (already at HEAD) | `rescale_cooking_after_relight` (`furnace_like_block_entity.rs:46-62`) rescales to `ceil(spent / oldTotal * newTotal)` while `spent < oldTotal`; called on relight (:622). VERIFIED vs `.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/AbstractFurnaceBlockEntity.java:278-281`. |
| taudit-crates-pumpkin-core-src-bl-231130 : 47 (`furnace_like_block_entity.rs:177-198`) | VERIFIED | One orb per used recipe id (`experience_per_recipe`, :114-129), cross-checked against the closed result in `docs/tracker/tfin-tpar-own-tsmall-batch2-0.md`. Vanilla `AbstractFurnaceBlockEntity.java:531-577`. |
| taudit-crates-pumpkin-core-src-bl-231130 : 49 (`furnace_like_block_entity.rs:867-876`) | VERIFIED | Caller (:966-978) passes accrued XP from `stored_experience` (:103-109) into `build_furnace_actor`, which writes `StoredXPInt` (:1024). No hardcoded 0. |
| taudit-crates-pumpkin-core-src-bl-231130 : 50 (`furnace_like_block_entity.rs:235-274`) | FIXED (already at HEAD) | `assemble_cooking_result` (:86-100) applies `recipe.result.components`; used by the output check (:298) and craft output (:327). Vanilla `AbstractFurnaceBlockEntity.java:358-371`. |

## Result
Rows 46 and 50: FIXED at HEAD, re-read this pass. Rows 47 and 49: VERIFIED. No row remains open. No `.rs` file was edited this pass.
