# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-block-enti.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Row 6 (game-event system): STILL OPEN. Soul feeding is a manual call from `LivingEntity::on_death` only; a real `GameEvent.ENTITY_DIE` BY_DISTANCE listener is a cross-crate change outside this unit.
- Row 6 (game-event system): STILL OPEN. Real GameEvent.ENTITY_DIE BY_DISTANCE listener dispatch is a cross-crate change; only the manual LivingEntity::on_death path exists. reason: cross-crate game-event system outside this unit.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
