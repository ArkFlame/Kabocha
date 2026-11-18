# Parity rows for crates/pumpkin-core/src/entity/mob/piglin.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Piglin** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result

- entities/Piglin: FIXED. Re-reviewed against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/piglin/Piglin.java`, `AbstractPiglin.java`, `PiglinAi.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ConversionType.java`. Fixed three conversion-parity gaps in `crates/pumpkin-core/src/entity/mob/piglin.rs`:
  - `Piglin.finishConversion` calls `PiglinAi.cancelAdmiring` first: added `self.cancel_admiring()` so the admired off-hand item is dropped instead of being copied onto the zombified piglin.
  - `AbstractPiglin.finishConversion` adds `MobEffects.NAUSEA` (200 ticks, amplifier 0): added the matching `Effect` on the converted mob, copying the existing pattern in `hoglin.rs` and `piglin_brute.rs`.
  - `ConversionType.convertCommon` preserves the baby state: added `spawn_as_baby()` on the zombified piglin, copying `hoglin.rs`.
  - Other reviewed behaviour (goal priorities, sensor counting, anger/retreat broadcast, bartering, spawn rules, NBT, activity sounds, 10% celebration chance) already matches vanilla; no generated-data or cross-crate blockers found.
  - Worker does not compile; orchestrator runs rustfmt and the crate checks.
