# Parity rows for crates/pumpkin-core/src/entity/mob/equipment.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-entity-mob** [partial/OWN-OPEN] own audit: 1 open, 14 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-entity-mob.md) docs/tracker/taudit-crates-pumpkin-core-src-entity-mob.md

## Result
- **own/taudit-crates-pumpkin-core-src-entity-mob / Row 1 (L) VERIFIED, matches vanilla.** No code change needed: the "duplicate dispatch" is behaviour-neutral and the "dead override" half is already resolved in this checkout. Evidence:
  - (a) `EQUIPMENT_REGISTRY` holds only skeleton, stray, bogged, wither_skeleton, piglin, pillager, vindicator (equipment.rs:258-335).
  - (b) Those reach equipment only via `MobEntity::finalize_spawn_base` -> `populate_spawn_equipment` -> `equip_mob_on_spawn` (spawn.rs:79-123).
  - (c) The trait `populate_default_equipment_slots` (mob/mod.rs:2658) is invoked only by the zombie family (zombie/mod.rs:286-287), and no zombie is in the registry, so no mob type is equipped twice.
  - (d) The skeleton's extra `equip_default_bow` (skeleton/mod.rs:105-119) is idempotent and never overwrites the registry bow.
  - (e) Grep finds no `populate_default_equipment_slots` override under skeleton/ or in piglin.rs, so the dead overrides named in the original row are gone.
Vanilla reference: `.vendor/vanilla-decompiled/net/minecraft/world/entity/Mob.java:1133` (`populateDefaultEquipmentSlots`) and `:1256-1269` (`finalizeSpawn` = follow-range bonus plus left-handedness only), which Pumpkin mirrors in spawn.rs:80-103. Each fresh mob still gets exactly one equipment roll, matching vanilla's single virtual dispatch. No generated-data or missing-subsystem dependency.
