# Parity rows for crates/pumpkin-core/src/entity/passive/bee.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Bee** [partial/AUDITED] Combat exists. Hive/pollination/crop behaviour is not wired; the absence of a Brain framework is not the issue. Combat exists.

## Result

- **entities/Bee** — FIXED (verified against vanilla). Hive/pollination/crop behaviour is wired and matches the decompiled source:
  - `wantsToEnterHive`: gate + data-driven `BEES_STAY_IN_HIVE` + `!isHiveNearFire()` now match `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/bee/Bee.java:360-369`. Added pure `hive_near_fire` (3x3x3 fire scan, matching `BeehiveBlockEntity.isFireNearby` at `.../block/entity/BeehiveBlockEntity.java:104-110`) and a unit test; replaced the hardcoded night/rain approximation with `World::bees_stay_in_hive`.
  - Combat/sting, tick timers, NBT and honey-delivery nectar/crop reset match `Bee.java:251-276, 389-413, 476-524` and `BeehiveBlockEntity.java:255-256` (wired in earlier passes; verified here).
