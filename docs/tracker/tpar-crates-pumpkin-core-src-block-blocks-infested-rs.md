# Parity rows for crates/pumpkin-core/src/block/blocks/infested.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Infested Blocks** [partial/AUDITED] Silk Touch and block_drops game rule checks are implemented. Breaking still constructs a generic Entity with the silverfish type rather than SilverfishEntity. Silk Touch and block_drops game rule checks are implemented.

## Result

- **blocks/Infested Blocks** -> VERIFIED / FIXED.
  - Generic-Entity concern from the prior pass is already resolved: `from_type(&EntityType::SILVERFISH, ...)` now returns `SilverfishEntity` (crates/pumpkin-core/src/entity/type.rs:272), matching vanilla `InfestedBlock.spawnInfestation` which uses `EntityTypes.SILVERFISH.create(...)` + `snapTo` + `addFreshEntity` (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/InfestedBlock.java:39-46`).
  - Fixed remaining parity bug in `spawn_infestation`: the spawn animation now broadcasts `EntityStatus::SilverfishMergeAnim` (20) instead of `EntityStatus::Poof` (60). Vanilla `InfestedBlock.spawnInfestation` calls `Silverfish.spawnAnim()`, and `Mob.spawnAnim()` broadcasts `EntityEvent.SILVERFISH_MERGE_ANIM = 20` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Mob.java:403-408`, `.vendor/vanilla-decompiled/net/minecraft/world/entity/EntityEvent.java:28`). Byte 20 makes the client show the poof particles; `Poof` (60) is a distinct event used for deaths/creaking spawn.
  - Silk Touch / `BLOCK_DROPS` gating and the Peaceful guard were already correct and left unchanged.
  - No generated `pumpkin-data` fields or WIT changes required. No unit test added (behaviour is client-visible particle spawn; no pure seam).
  - Note for orchestrator: `crates/pumpkin-core/src/entity/mob/silverfish.rs:142` (merge goal) also sends `EntityStatus::Poof` for its `spawnAnim()`; that file is a different unit and was not edited here.
