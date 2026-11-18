# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mob-breeze-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- **own/tverify-mobs9** [partial/OWN-OPEN] -> closed. Head rot speed 25 FIXED (hook + test); pathfinding malus VERIFIED (`Breeze.java:73-78`); slide/jump particles, pose and animation VERIFIED client-only, non-blocking (`Breeze.java:120-179`); Brain AI behaviours and Shoot/charge cooldowns BLOCKED-DATA: whole missing subsystem, the Breeze AI behaviour tree (`Shoot`, `LongJump`, `Slide`, `ShootWhenStuck`) and Breeze memory types (`BREEZE_JUMP_TARGET`, `BREEZE_SHOOT_*`) are absent from `crates/pumpkin-core/src/entity/ai/brain/` (vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/BreezeAi.java`). All `STILL OPEN` rows in `docs/tracker/tverify-mobs9.md` were updated accordingly.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
