# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-block-enti.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: living.rs mod.rs

- Row 5 (random reseeding): STILL OPEN. `Xoroshiro::from_seed(rand::rng().random())` reseeds from OS entropy each tick instead of `level.getRandom()`; a house-wide pattern (fire/stem/jigsaw/grass) and there is no level-random accessor in `world/mod.rs`, so not catalyst-specific.
- Row 6 (game-event system): STILL OPEN. Dispatch is manual from `LivingEntity::on_death` only; a real `GameEvent.ENTITY_DIE` BY_DISTANCE listener system is a cross-crate change outside this unit.
- Row 6 (game-event system): STILL OPEN. Only the manual LivingEntity::on_death dispatch path exists; a real GameEvent.ENTITY_DIE BY_DISTANCE listener system is a cross-crate change outside this unit.
- Row 4 (getExperienceReward killer): STILL OPEN, outside this unit. `entity/living.rs` passes `None` for catalyst soul charge and killer for XP orbs (not a file this unit may edit); diverges from decompiles that pass `lastDamageSource` entity for Looting on soul charge if/when implementations use it.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
