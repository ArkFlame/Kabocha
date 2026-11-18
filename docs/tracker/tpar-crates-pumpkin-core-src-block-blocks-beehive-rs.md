# Parity rows for crates/pumpkin-core/src/block/blocks/beehive.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Beehive & Bee Nest** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result

- **blocks/Beehive & Bee Nest** — FIXED. Re-audited the whole block against
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/BeehiveBlock.java` (and
  `.../block/entity/BeehiveBlockEntity.java`, `.../server/level/ServerPlayerGameMode.java`).
  Concrete gap closed: `use_with_item` now calls `BeehiveBlockEntity::empty_all_living_from_hive`
  (EMERGENCY) before resetting honey when there is no campfire smoke, matching vanilla
  `useItemOn` -> `releaseBeesAndResetHoneyLevel` (`BeehiveBlock.java:195-205,216-231`). The new
  primitive shares the occupant-release path with `tick_occupants` (nectar reset, flower
  inheritance, front-face spawn, EMERGENCY player targeting, `BeehiveBlockEntity.java:120-152,214-227`).
  Added a pure unit test for `Bee.dropOffNectar` timer resetting.

  VERIFIED against the same vanilla path: honey-level >= 5 gate and item branches
  (`BeehiveBlock.java:155-193`); shear loot + shear/fluid-pickup game events and player-position
  sounds (`:132-143,173-187`); stat award (`:190-192`); smoke gate and hive-contains-bees anger
  (`:195-205,212-214`); silk-touch / creative break anger gate and BEE_NEST_DESTROYED trigger with
  hive bee count (`BeehiveBlock.java:85-104` + `ServerPlayerGameMode.java:440-467`); explosion anger
  (`:106-112`); comparator = honey level (`:74-82`); placement facing = opposite horizontal
  (`:280-283`); non-silk break anger uses `TargetReason::ClosestPlayer` (`:125-126`).

  Residual parity difference (not generated-data blocked; needs a separate cross-file unit):
  vanilla also empties the hive on break via `playerDestroy`, but Pumpkin removes the block entity
  inside `break_block` before `BlockBehaviour::broken` runs (`world/mod.rs:7154-7176`), so the saved
  occupants currently travel in the `bees` item component instead of spawning. `broken` cannot
  reach the entity; fixing it requires threading the pre-removal block entity into `BrokenArgs`
  (block/mod.rs, registry.rs, player.rs), a refactor outside this unit's beehive.rs scope.

  Files edited: `crates/pumpkin-core/src/block/blocks/beehive.rs`,
  `crates/pumpkin-core/src/block/entities/beehive.rs`.
