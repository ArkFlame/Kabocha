# Parity rows for crates/pumpkin-core/src/entity/mob/skeleton/bogged.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Bogged** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result

- **entities/Bogged** - VERIFIED. No code change required; the current `bogged.rs` matches decompiled vanilla. Reference: `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/Bogged.java`.
  - MAX_HEALTH 16.0 (vanilla `createAttributes`) is present in generated `entity_type.rs` BOGGED attributes (16f64).
  - `DATA_SHEARED` default false (`defineSynchedData`) -> `mob_init_data_tracker`; save/load (`addAdditionalSaveData` / `readAdditionalSaveData`) -> `mob_write_nbt` / `mob_read_nbt`, key "sheared".
  - `mobInteract`: SHEARS + `readyForShearing` -> `shear_by_player`, which fires PlayerShearEntityEvent, emits the SHEAR game event, degrades the shears once, and calls `shear`.
  - `shear`: plays `entity.bogged.shear` at 1.0/1.0, drops from `minecraft:shearing/bogged` (vanilla `BuiltInLootTables.BOGGED_SHEAR`) at `getBbHeight()` via `entity.height()`, then sets sheared.
  - Sounds: ambient, hurt, death, step and shear all mapped to `EntityBogged*` (`SoundEvents.BOGGED_*`).
  - `getArrow` (poison 100 ticks, amplifier 0, visible, showIcon true for `new MobEffectInstance(POISON, 100)`) -> plain-arrow `PotionContentsImpl` custom effect; no `PotionDurationScale` component so the duration stays 100. Synced effect colour is set from the stack in arrow spawn metadata.
  - Attack intervals 50 hard / 70 normal (`getHardAttackInterval` / `getAttackInterval`) -> `get_min_attack_interval`.
  - `readyForShearing` -> `ready_for_shearing`; atomic swap claims the shear so concurrent shears drop loot only once.
No generated-data field or subsystem is missing; no BLOCKED-DATA rows.
