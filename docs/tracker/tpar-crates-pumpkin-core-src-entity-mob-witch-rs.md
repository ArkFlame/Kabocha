# Parity rows for crates/pumpkin-core/src/entity/mob/witch.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Witch** [partial/COVERED-DONE] Potion combat exists. Target selection only has revenge/player targets, lacking vanilla injured-raider healing selection. Potion combat exists.
- **own/taudit-crates-pumpkin-core-src-en-449897** [partial/OWN-OPEN] own audit: 1 open, 1 fixed, 1 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-449897.md) docs/tracker/taudit-crates-pumpkin-core-src-en-449897.md

## Result

- **entities/Witch**: FIXED + VERIFIED. The healable-raider hunt (`WITCH_HEALABLE_RAIDERS`, priority 2) is present, matching vanilla `Witch.registerGoals` target goal 2 (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Witch.java:74`). This pass tightened it to the vanilla predicate: gated on an active raid (`.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/goal/target/NearestHealableRaiderTargetGoal.java:32`) and excluding witch targets (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Witch.java:65` subselector `!target.is(EntityTypes.WITCH)`). No generated data, WIT or cross-crate change needed; only witch.rs edited.
- **own/taudit-crates-pumpkin-core-src-en-449897**: VERIFIED, no further source change. All five audit items are closed in the audit doc Result: item 1 throw speed (`Witch.java:268`, pure `potion_throw_speed` + unit test), item 2 target priorities (`Witch.java:63-76`), item 3 `canBeLeader` false (`Witch.java:299-301`), item 4 tick order (no observable difference), item 5 test coverage (pure seam `potion_throw_speed` already pinned; drink math needs a `LivingEntity`, no pure seam). Re-verified this pass against the decompiled source above.
- No STILL OPEN rows remain. No BLOCKED-DATA. rustfmt --edition 2024 applied to witch.rs; cargo not run per worker rules.
