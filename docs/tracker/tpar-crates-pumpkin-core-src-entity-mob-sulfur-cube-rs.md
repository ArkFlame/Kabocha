# Parity rows for crates/pumpkin-core/src/entity/mob/sulfur_cube.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Sulfur Cube** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result
- entities/Sulfur Cube: FIXED. Audited the whole port against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java` (with `AbstractCubeMob.java` and `SulfurCubeArchetype.java`). The port matches vanilla on goals, size/health/speed, knockback math, contact damage, player push, shear/equip/pickup, NBT, sounds, split and controls. Two gaps were fixed in `crates/pumpkin-core/src/entity/mob/sulfur_cube.rs`:
  1. `save_to_bucket_tag` now copies the swallowed body item into the bucket's `sulfur_cube_content` component, matching `saveToBucketTag` `bucket.copyFrom(DataComponents.SULFUR_CUBE_CONTENT, this)` (SulfurCube.java lines 168-176). The player bucket path (`bucket_mob_pickup` -> `save_to_bucket_tag`) previously dropped the item; the dispenser path (`fill_mob_bucket_stack`) already handled it.
  2. `tick_fuse` now clears a stale fuse when the archetype loses its explosion data, matching the Paper fuse reset (SulfurCube.java lines 349-355): `fuse` and synced `MAX_FUSE` reset to `NO_FUSE` (-1) and invulnerability is cleared. Without this a dispenser-sheared primed cube stayed primed and invulnerable forever.
  No generated-data or subsystem gaps. No pure test seam exists without a formula-mirroring test, so none added. Unverified by compile (worker cannot run cargo); orchestrator compile gate applies.
