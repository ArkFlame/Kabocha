# Parity rows for crates/pumpkin-core/src/entity/mob/skeleton/skeleton.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Skeleton** [partial/COVERED] The shared bow interval is fixed at 20 ticks. Vanilla reassesses the bow goal using difficulty-dependent intervals. The shared bow interval is fixed at 20 ticks. Vanilla reassesses the bow goal using difficulty-dependent intervals.

## Result
VERIFIED. The row's premise ("shared bow interval fixed at 20 ticks") is stale: commit 53da31df2 already made the bow interval per-type and per-difficulty. Vanilla `AbstractSkeleton.reassessWeaponGoal()` (.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/AbstractSkeleton.java:148-165) sets the min attack interval to `getHardAttackInterval()` on Hard and `getAttackInterval()` otherwise (20/40 at lines 167-173); those are overridden to 50/70 in Bogged.java:125-132 and Parched.java:57-64, while Skeleton.java keeps the 20/40 defaults. Pumpkin matches: `skeleton/mod.rs:32-33,134-141` returns 20 Hard / 40 otherwise; bogged.rs:118-125 and parched.rs:59-66 return 50/70; stray.rs:54-56 and skeleton.rs:48-50 delegate to the base; ai/goal/bow_attack.rs:291-293 applies `get_min_attack_interval()` after each shot instead of a fixed constant. Tests in skeleton/mod.rs:148-156 pin 40/20. No code change needed.
