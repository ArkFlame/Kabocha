# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_effects.rs minecart.rs

- block_effects 5 (requested-movement axis split, block_effects.rs:118-152): VERIFIED - matches vanilla Entity.checkInsideBlocks axis gating (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:1959-1970`); Rust for_each_block_along_movement (block_effects.rs:160-205) filters Some-movement by movement != default and walks axis_step_order, falling back to the unsplit path; pinned by test diagonal_movement_follows_requested_axes_after_collision_clipping (line 511). No change.
- block_effects 6 (no gametest): FIXED - gametest seam present in `crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs` (test_boat_breaks_lily_pad_on_landing line 10, test_boat_sweep_ignites_on_fire line 341), registered in builtin_game_tests/mod.rs:1,1817,1824. No change.
- minecart 3 (activator rail ejects riders, minecart.rs:330-352): VERIFIED - vanilla Minecart.activateMinecart (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/Minecart.java:44-58`) DOES call ejectPassengers() on isVehicle() then, gated on getHurtTime()==0, flips hurtDir, sets hurtTime(10)/damage(50) and markHurt(); Rust minecart.rs:500-525 matches. Rideable arm stays; tracker delete request rejected. No change.
- minecart 4 (push direction, minecart.rs:649-783): VERIFIED - vanilla AbstractMinecart.push/pushOtherMinecart (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/AbstractMinecart.java:584-648`) non-experimental default uses cart-to-cart direction, yaw facing, dot>=0.8 gate and a 1.0E-4 lower bound with no upper bound; Rust minecart.rs:857-955 matches. The motion-based direction and <1.0 upper bound in the tracker belong to the experimental branch (off by default). No change.

## Result

FINAL PASS. All four rows closed against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source edit was required this pass: the current
tree already matches vanilla for every row (prior passes landed the fixes). The tracker's
"delete the Rideable arm" instruction for minecart 3 is based on a wrong claim and is rejected.

- block_effects 5 (requested-movement axis split): VERIFIED faithful, no change.
  `Entity.checkInsideBlocks` (`.vendor/.../world/entity/Entity.java:1959-1970`) splits per axis
  only when `axisDependentOriginalMovement().isPresent() && delta.lengthSqr() > 0.0`, else it
  walks the whole movement. `block_effects.rs:171-196`
  (`for_each_block_along_movement`) uses
  `requested_movement.filter(|_| movement != Vector3::default())` (Some = present; movement !=
  default == lengthSqr > 0) and iterates `axis_step_order`, falling back to the unsplit path when
  the filter fails. Pinned by the existing unit test
  `diagonal_movement_follows_requested_axes_after_collision_clipping` (block_effects.rs:511).

- block_effects 6 (gametest): FIXED in current tree, no change. The gametest seam was added out of
  unit scope in `crates/pumpkin-core/src/server/builtin_game_tests/boat_block_effects.rs`:
  `test_boat_breaks_lily_pad_on_landing` (line 10) and `test_boat_sweep_ignites_on_fire`
  (line 341), registered in `builtin_game_tests/mod.rs:1,1817,1824`. Both files verified present.

- minecart 3 (activator rail ejects riders): VERIFIED faithful, no change; the `Rideable` arm
  MUST stay. Decompiled `Minecart.activateMinecart`
  (`.vendor/.../world/entity/vehicle/minecart/Minecart.java:45-58`) DOES call
  `ejectPassengers()` on `isVehicle()` when the rail is powered, then, gated on
  `getHurtTime() == 0`, flips `setHurtDir(-getHurtDir())`, `setHurtTime(10)`,
  `setDamage(50.0F)` and `markHurt()`. `minecart.rs:500-525` matches exactly. The tracker's
  premise (only command-block/TNT/hopper override `activateMinecart`) is false: the plain
  rideable cart overrides it (`Minecart.java:44-58`); the base `AbstractMinecart` override at
  `AbstractMinecart.java:364-365` is empty but is shadowed by `Minecart`. No deletion.

- minecart 4 (push direction): VERIFIED faithful, no change. Vanilla non-experimental
  `AbstractMinecart.pushOtherMinecart`
  (`.vendor/.../world/entity/vehicle/minecart/AbstractMinecart.java:614-648`) computes
  `dir` from the cart-to-cart position vector (`xo = other.x - this.x`, `zo = other.z -
  this.z`) and compares `abs(dir . facing)` against the yaw `facing` with a `dot < 0.8` skip;
  `push` (`AbstractMinecart.java:584-608`) applies a `dd >= 1.0E-4` LOWER bound and no `< 1.0`
  upper bound. The cart-to-cart direction / horizontal-delta and `< 1.0` upper bound described
  in the tracker belong to the experimental-movement branch (`useExperimentalMovement`, default
  off). `minecart.rs:822-901` matches the non-experimental default: dir from cart-to-cart,
  yaw `facing`, `dot >= 0.8` gate, `1.0E-4` lower bound. No change.

No pure unit test added this pass: block_effects axis-split/traversal already has seven unit
tests (block_effects.rs:443-598) including the axis-split case, and minecart 3/4 seams need a
live world (pushed velocity / rail effects) covered by the registered gametests. No
cargo/git/rustfmt was run by this worker.

Status: PASS fixed=4 open=0 files=docs/tracker/tfin-audit-crates-pumpkin-core-src-entity-veh-1.md
