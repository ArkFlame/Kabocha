# Parity rows for crates/pumpkin-core/src/entity/mob/breeze.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Breeze** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tverify-mobs9** [partial/OWN-OPEN] own audit: 5 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/tverify-mobs9.md) docs/tracker/tverify-mobs9.md

## Result

- **entities/Breeze** [partial/RE-REVIEW] -> FIXED / VERIFIED. Re-reviewed every overworld behaviour in `crates/pumpkin-core/src/entity/mob/breeze.rs` against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/Breeze.java`: ctor pathfinding malus (`-1` for `ON_TOP_OF_TRAPDOOR`/`FIRE`, lines 73-78), `tick` whirl sound (1..80, volume 0.8+0.2r, pitch 0.7+0.4r, lines 143-146/188-192), `playAmbientSound` (183-186), `getAmbientSound` (213-216), fall-land sound above 3 blocks (273-279), `deflection` wind-charge passthrough + reverse + deflect sound (195-201), `isInvulnerableTo` breeze damage (262-265), `canAttack` players/iron golems (238-246), `getMaxHeadYRot` 30 (248-251) all match. New fix: `getHeadRotSpeed` 25 implemented as `get_max_look_yaw_change` override (vanilla lines 253-256) with unit test `head_rot_speed_matches_vanilla_breeze`. Brain AI and particles remain as classified in `own/tverify-mobs9` below.
- **own/tverify-mobs9** [partial/OWN-OPEN] -> closed. Head rot speed 25 FIXED (hook + test); pathfinding malus VERIFIED (`Breeze.java:73-78`); slide/jump particles, pose and animation VERIFIED client-only, non-blocking (`Breeze.java:120-179`); Brain AI behaviours and Shoot/charge cooldowns BLOCKED-DATA: whole missing subsystem, the Breeze AI behaviour tree (`Shoot`, `LongJump`, `Slide`, `ShootWhenStuck`) and Breeze memory types (`BREEZE_JUMP_TARGET`, `BREEZE_SHOOT_*`) are absent from `crates/pumpkin-core/src/entity/ai/brain/` (vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/BreezeAi.java`). All `STILL OPEN` rows in `docs/tracker/tverify-mobs9.md` were updated accordingly.

Files edited: `crates/pumpkin-core/src/entity/mob/breeze.rs` (override + test), `docs/tracker/tverify-mobs9.md`, this unit doc. No cargo/git run (worker restriction); compile/format by orchestrator.

## Result (x_14 re-check)
- Head rot speed 25: present, `crates/pumpkin-core/src/entity/mob/breeze.rs:140` with test at `:224-232`; vanilla `Breeze.java:253-256`.
- Brain AI and shoot/charge cooldowns: BLOCKED-DATA confirmed. `grep` finds no Breeze behaviours (`Shoot`, `LongJump`, `ShootWhenStuck`, `Slide`) in `crates/pumpkin-core/src/entity/ai/`; only the frog's own `LongJump` exists. Vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/breeze/BreezeAi.java`, `Shoot.java`, `LongJump.java`, `Slide.java`, `ShootWhenStuck.java`, `Breeze.java:258-260`.
No source edited in this pass.
