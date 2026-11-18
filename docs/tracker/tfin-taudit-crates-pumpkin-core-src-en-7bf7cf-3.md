# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-7bf7cf.md

Files: goals.rs sneeze.rs

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files).

## Result (batch 09 pass, 2026-11-18)

- Row 1 `genes_from_parents` (panda.rs:151-171): VERIFIED. Panda.java:583,587 rolls two independent nextInt(32) mutation checks; panda.rs:164-169 matches. Test: foal_genes_come_from_the_parents_and_mutate_independently.
- Row 2 `sit_and_eat` (panda.rs:624-638): VERIFIED. Panda.java:635-644 and AgeableMob.java:243-245; panda.rs:627 gate is the same predicate. Test: feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food.
- Row 3 `can_find_bamboo` (panda/goals.rs:146-159): VERIFIED. Panda.java:834-852 covers x,z in [-7,7], y in [0,2]; BAMBOO_SEARCH_RADIUS=7 and BAMBOO_SEARCH_HEIGHT=2 match. Test: breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown.
- Row 4 `after_sneeze` AI-island note (panda/sneeze.rs:87-121): VERIFIED against Panda.java:506-531 (sound, jump for grounded adults in inflate(10.0), gift under MOB_DROPS). Unaudited pandas are covered by the conservative reach; the sit/breed tick_reach sub-gap is closed.

No .rs file changed in this pass.
