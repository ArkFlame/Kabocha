# Parity rows for crates/pumpkin-core/src/entity/decoration/mannequin.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Mannequin** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-mannequin** [partial/OWN-OPEN] own audit: 3 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-mannequin.md) docs/tracker/tentities-mannequin.md

## Result

Re-reviewed the full class against `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/Mannequin.java` (with `Avatar.java`, `LivingEntity.java`, `Entity.java` for inherited behaviour). All rows closed.

- **entities/Mannequin** (AUDITED re-review): VERIFIED + one FIX. Tracked data, `is_immobile`, `is_effective_ai`, `can_hit` and `is_pushable` all match vanilla (see paths below). FIX: `write_custom_nbt` now stores only `hide_description` and drops the custom text when hidden, matching `Mannequin.addAdditionalSaveData` (Mannequin.java:133-140); on reload the description resets to the default label. Updated the NBT round-trip test to assert no `description` tag is written while hidden.
- **own/tentities-mannequin / gap 1 equip via right-click**: VERIFIED. Vanilla `Mannequin.java` defines no `interact`/`mobInteract` (full class, lines 1-175); mannequins cannot be equipped by right-click. Pumpkin correctly implements no interaction.
- **own/tentities-mannequin / gap 2 can_hit**: VERIFIED. `!is_removed()` matches vanilla `LivingEntity.isPickable` / `Entity.canBeHitByProjectile` (`.vendor/.../world/entity/LivingEntity.java:4153`, `Entity.java:2776`); Mannequin adds no override.
- **own/tentities-mannequin / gap 3 is_pushable**: FIXED (prior pass, re-confirmed). `LivingEntity::can_be_pushed(on_climbable)` matches vanilla `LivingEntity.isPushable` -> `isCollidable` (`.vendor/.../world/entity/LivingEntity.java:4158-4171`); Mannequin adds no override, so `immovable` gates only its own travel (`Mannequin.java:115-123`), never external pushes.
- **own/tentities-mannequin / gap 4 pose restriction**: VERIFIED. `parse_valid_pose` matches vanilla `VALID_POSES`/`POSE_CODEC` = {standing, crouching, swimming, fall_flying, sleeping} (`.vendor/.../decoration/Mannequin.java:37-39`).
- **own/tentities-mannequin / gap 5 client-only pose/death**: VERIFIED. Vanilla `Mannequin.java` has no death/pose-specific override; Pumpkin uses generic `LivingEntity` handling. No server-side parity action.

Touched: crates/pumpkin-core/src/entity/decoration/mannequin.rs (hidden-description save + test). No generated data, WIT or cross-crate changes needed. Cargo/git intentionally not run per unit rules; rustfmt is run by the orchestrator.
