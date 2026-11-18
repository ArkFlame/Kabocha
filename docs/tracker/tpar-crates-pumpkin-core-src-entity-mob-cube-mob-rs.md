# Parity rows for crates/pumpkin-core/src/entity/mob/cube_mob.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tentities-sulfur-cube-b516b8** [partial/OWN-OPEN] own audit: 5 open, 1 fixed, 2 verified, 0 blocked-data (docs/tracker/tentities-sulfur-cube-b516b8.md) docs/tracker/tentities-sulfur-cube-b516b8.md

## Result (final, worker pass 2026-11-18)

Row `own/tentities-sulfur-cube-b516b8`: VERIFIED, matches decompiled vanilla. No source change needed in `cube_mob.rs`.

The linked tracker `docs/tracker/tentities-sulfur-cube-b516b8.md` is already resolved (pass 4: PASS, fixed=1, open=0): the sulfur-cube `hurtServer` fix (`Some(true)` at `sulfur_cube.rs:1006-1008`) is present in the tree, and the pickup body-state row is VERIFIED against `SulfurCube.pickUpItem`/`collectEquipmentChanges`.

Re-audited `cube_mob.rs` against vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/AbstractCubeMob.java`:
- `tick_cube_mob` (`:79-110`) squish/landing match `AbstractCubeMob.tick` (`:123-154`): `o_squish <- squish`, `squish += (target - squish) * 0.5`, landing sound plus `target_squish = -0.5` / `1.0`, `was_on_ground`, `target_squish *= 0.6`. The squish fields are plain client-side floats in vanilla (`:48-50`) with no tracked-data accessor, so keeping them server-side is a client-only visual gap with no server parity impact and no missing pumpkin-data field (confirming the prior BLOCKED-DATA call was a false premise).
- `get_jump_delay` (`:73-75`, `rand::random_range(10..30)`) matches `getJumpDelay` (`nextInt(20) + 10`, `:156-158`); `get_sound_volume`/`get_sound_pitch`/`cube_sound_category`/`is_tiny`/`do_play_jump_sound` match (`:317-334`, `:116-118`, `:327-329`).
- `CubeMobMoveControl::tick` (`:144-202`) matches `CubeMobMoveControl.tick` (`:513-543`): rotlerp with `MAX_TURN = 90.0`, yaw/head/body sync, jump delay with aggressive `/3`, jump sound, speed = modifier * movement speed (`move_control.rs:177`).
- Goals `CubeMobFloatGoal`/`CubeMobRandomDirectionGoal`/`CubeMobKeepOnJumpingGoal` (`:205-324`) match vanilla (`:441-491`, `:546-579`): float 0.8 jump chance plus 1.2 speed, randomize `40..100` ticks / `0..360` degrees plus 1.0 speed, keep-jumping 1.0 speed.

No em dashes, no unwrap/expect/panic, no imports/helpers/macro paths/struct initializers/clones introduced (doc-only edit). Worker restriction: no cargo run.

Status: PASS fixed=0 open=0 files=none
