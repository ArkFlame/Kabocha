# Parity rows for crates/pumpkin-core/src/entity/passive/fox/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Fox** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result
- **entities/Fox** FIXED / VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/fox/Fox.java`. Changes in `crates/pumpkin-core/src/entity/passive/fox/mod.rs`:
  - `isConsumableFood` (Fox.java:262-264) requires BOTH `FOOD` and `CONSUMABLE`; added `is_consumable_food` and used it in `can_eat` (Fox.java:258-260) and `can_hold_item` (Fox.java:524-527), which previously checked only `FOOD`.
  - `addTrustedEntity` (Fox.java:430-436) keeps slot 0 and overwrites slot 1 once both are taken; `add_trusted` previously kept slot 1 and dropped slot 0.
  - `tick` wake-up and sit clear (Fox.java:569-577) use `isInWater()`; changed `is_in_water_or_rain()` to `is_in_water()`.
  - faceplant block event (Fox.java:579-583) is emitted for any block state; removed the non-air guard and its incorrect vanilla comment.
  - All goal registration/goal code, variant handling, target-goal ordering (Fox.java:173-213, 371-381), NBT (Fox.java:438-460), sounds (Fox.java:677-702) and death drops already match vanilla; no blocker.
No generated-data, WIT or cross-crate changes required. No unit test: no side-effect-free seam exists without constructing a full `FoxEntity`/`World`, and the pure `FoxVariant` helpers are trivial.
