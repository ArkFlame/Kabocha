# Parity rows for crates/pumpkin-core/src/entity/passive/sniffer.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Sniffer** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-sniffer-f5f3a0** [partial/OWN-OPEN] own audit: 5 open, 0 fixed, 6 verified, 0 blocked-data (docs/tracker/tentities-sniffer-f5f3a0.md) docs/tracker/tentities-sniffer-f5f3a0.md
- **own/tentities-sniffer** [partial/OWN-OPEN] own audit: 8 open, 10 fixed, 2 verified, 0 blocked-data (docs/tracker/tentities-sniffer.md) docs/tracker/tentities-sniffer.md

## Result
Third pass. Decompiled vanilla was re-read directly at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/Sniffer.java`, `.../SnifferAi.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/Animal.java`. All three tracker rows are closed; no row remains open.

- `entities/Sniffer` (AUDITED): VERIFIED against vanilla. `getHeadBlock()`/`getHeadPosition()` (`Sniffer.java:142-149`) match `get_head_block`/`get_head_position`; `canSniff`/`canDig` gate on `!isInWater()` (`Sniffer.java:134-135, 268-276`); `dropSeed` spawns at the floored head block (`Sniffer.java:293-309`); `emitDiggingParticles` emits `ENTITY_ACTION` at the head block with source context every 10 ticks (`Sniffer.java:329-331`); `getAmbientSound` is null for Digging/Searching (`Sniffer.java:421-424`) and `Animal.getAmbientSoundInterval()` is 120 (`Animal.java:128-129`); `getMaxHeadYRot` is 50 (`Sniffer.java:437-439`). No code change.

- `own/tentities-sniffer-f5f3a0` (OWN-OPEN, 7 gaps): all closed. 1 SNIFFER_SEARCHING loop is client-only (`Sniffer.java:410-414`), no server cadence needed; 2 ambient interval 120 (`Animal.java:128-129`); 3 loot at floored head block (`Sniffer.java:293-309`); 4 water gate `!isInWater()` (`Sniffer.java:134-135, 272`); 5 AI cooldown/completion match `SnifferAi.java:163-175, 237-239, 367-375` (Pumpkin `sniffer/ai.rs:214-220, 253-260, 429-440`); 6 digging dimensions/eye height 0.4/0.81 and head distance 2.25 / MAX_HEAD_Y_ROT 50 (`Sniffer.java:73-75, 122-123, 148, 437-439`); 7 end-to-end chain covered by the two dirt-pad `#[tokio::test]` tests `sniffer.rs::digging_drops_gift_loot_at_the_head_on_the_entity_clock` and `sniffer/ai.rs::searching_handoff_starts_digging_on_a_diggable_pad`. No code change.

- `own/tentities-sniffer` (OWN-OPEN, 5 gaps): all closed. 1 `prepend_capped` keeps 20 previous + 1 new = 21 entries, matching `storeExploredPosition` `limit(20)` then `add(0, ...)` (`Sniffer.java:336-341`); 2 loot at floored head block (`Sniffer.java:293-309`); 3 ambient interval 120 (`Animal.java:128-129`); 4 game event at head block with source (`Sniffer.java:329-331`); 5 end-to-end covered by the builtin gametest `test_sniffer_sniffs_walks_and_digs` in `crates/pumpkin-core/src/server/builtin_game_tests/entities_and_ai.rs:94-124`. No code change.

No Rust source files edited this pass: every row was already fixed or verified in prior passes and re-confirmed against vanilla here, so no edit is warranted. No generated data, WIT or cross-crate changes. No test added (the existing pure unit tests and the builtin gametest already cover those seams; another would duplicate coverage under the MVP rules). UNVERIFIED: no build or test run (worker cannot compile).
