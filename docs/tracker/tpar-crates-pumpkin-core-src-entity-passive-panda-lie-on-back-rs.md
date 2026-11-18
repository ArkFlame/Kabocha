# Parity rows for crates/pumpkin-core/src/entity/passive/panda/lie_on_back.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-3b1660** [partial/OWN-OPEN] own audit: 3 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-3b1660.md) docs/tracker/taudit-crates-pumpkin-core-src-en-3b1660.md

## Result

ALL THREE ROWS VERIFIED at parity against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java`.
No `.rs` edit was warranted, so no rustfmt or new test was needed; the pure-function
test at `lie_on_back.rs:100-115` already covers the roll bounds and short-circuit seam.

- Row 1 (random source) VERIFIED, no edit. Vanilla `PandaLieOnBackGoal.canUse/canContinueToUse`
  (`Panda.java:894,900-901`) draws from `this.panda.random`. In this decompiled tree that field is
  `Entity.random = SHARED_RANDOM` (`world/entity/Entity.java:175,356`), seeded at runtime from
  `RandomSupport.generateUniqueSeed()` (`Entity.java:179`); the entity UUID is derived FROM the
  random (`uuid = Mth.createInsecureUUID(this.random)`, `Entity.java:406`), not vice versa.
  Pumpkin's `Entity.rng` is `StdRng::from_rng(&mut rand::rng())` (`entity/mod.rs:2187-2189`), also
  runtime entropy, exposed per entity through `Mob::get_random` (`mob/mod.rs:1681-1683`) and used at
  `lie_on_back.rs:61,69`. The prior "vanilla seeds from entity UUID, so seed replay diverges" claim
  is false: both seed from runtime entropy and Pumpkin's per-entity RNG matches vanilla's per-entity
  `java.util.Random` behaviour. No BLOCKED-DATA: no generated field or subsystem is missing.
- Row 2 (canPerformAction ordering) VERIFIED, no edit. Vanilla `canUse` (`Panda.java:890-895`) is
  `cooldown < tickCount && isLazy() && canPerformAction() && nextInt(reducedTickDelay(400)) == 1`.
  `may_lie_down` (`lie_on_back.rs:46-50`) plus the roll in `can_start` (`:54-63`) apply the same checks
  in the same order. `canPerformAction` (`Panda.java:695-697`) equals `can_perform_action`
  (`passive/panda.rs:654-660`).
- Row 3 (tick_count) VERIFIED, no edit. Vanilla increments `tickCount` in `Entity.commonTick` before
  `serverAiStep` runs the selectors; Pumpkin reads the already-incremented count at
  `lie_on_back.rs:47,85` after `mob/mod.rs:484,494`. No off-by-one.

Status: PASS fixed=0 open=0 files=none
