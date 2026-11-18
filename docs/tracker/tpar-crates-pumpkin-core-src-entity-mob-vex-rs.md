# Parity rows for crates/pumpkin-core/src/entity/mob/vex.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Vex** [partial/AUDITED] Only generic ground goals are present. Flying charge, owner-target copying and summoned lifetime management are not implemented. 

## Result

- **entities/Vex** [partial/AUDITED] - FIXED. All three listed gaps are closed against
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Vex.java` and
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/illager/Evoker.java`:
  - Flying charge (`VexChargeAttackGoal`, Vex.java 222-280): swoops at the target's eyes at
    speed 1.0, hits on bounding-box contact, re-steers within 9 blocks, starts past 4 blocks on
    the `reducedTickDelay(7)` roll, and now clears the charging flag on `stop` and requests
    every-tick updates (`requiresUpdateEveryTick`).
  - Owner-target copying (`VexCopyOwnerTargetGoal`, Vex.java 282-299, registered at target
    priority 2 in `registerGoals` line 94): `VexEntity` stores an `EntityReference` owner set by
    the evoker summon (`Evoker.java` 264), and the goal adopts the owner's current target.
  - Summoned lifetime (`set_limited_life`, Vex.java 181-184 / Evoker.java 266): starvation every
    20 ticks in `mob_tick` (Vex.java 79-82) and `summoned_vex_life_ticks` rolling
    `20 * (30 + nextInt(90))`, covered by the existing pure unit test.

Non-blocking notes (outside this row, no generated-data or subsystem blocker):
- `setIsCharging`/`DATA_FLAGS_ID`, `isFlapping` and the VEX_CHARGE sound are client-only
  animation/audio; authoritative AI and damage are modelled.
- `Vex.setBoundOrigin`/`bound_pos` NBT is not wired: `VexRandomMoveGoal` uses the documented
  fallback to the vex's own block. Evoker.java 265 sets the origin to the spawn block; adding
  it needs a new `BlockPos` field plus NBT keys, not a missing generated field.

