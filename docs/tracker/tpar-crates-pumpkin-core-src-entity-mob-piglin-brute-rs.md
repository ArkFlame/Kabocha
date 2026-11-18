# Parity rows for crates/pumpkin-core/src/entity/mob/piglin_brute.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Piglin Brute** [partial/COVERED] The implementation has targeting and conversion but no stored home or return-to-home behaviour corresponding to vanilla PiglinBruteAi. 

## Result
- **entities/Piglin Brute** — FIXED. Added a stored `home_pos` (set on spawn per `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/piglin/PiglinBruteAi.java:49-52` `initMemories`, and round-tripped through NBT). Added two idle goals reproducing vanilla `StrollToPoi` (HOME, close-enough 2, too-far 100, speed 0.6) from `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/behavior/StrollToPoi.java` and `StrollAroundPoi` (HOME, radius 5, speed 0.6) from `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/behavior/StrollAroundPoi.java`, with the matching constants from `PiglinBruteAi.java:40-43`. Added a small pure boundary test. Only `piglin_brute.rs` touched; all changes compile-by-inspection only (no cargo run by worker).
