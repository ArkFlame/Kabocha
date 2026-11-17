# entities/Armor Stand - re-review (2026-11-17)

Source: `crates/pumpkin-core/src/entity/decoration/armor_stand.rs` (re-reviewed after
`c79e2866f` / `90bdf2a2b` / `3a22aff9c` / `031b1ffc9` / `3c51e7554` / `bb1f2c41a`).
Reference: vanilla `net.minecraft.world.entity.decoration.ArmorStand`; no decompiled
copy under `.vendor/` in this worktree, so items below are marked by confidence.

Fixed in this pass:
- `update_dimensions` (was :161-172) built the small dimensions as
  `EntityDimensions::new(width, height, 0.9875)`; vanilla
  `getDefaultDimensions` returns `super.getDefaultDimensions(pose).scale(small ? 0.5 : 1)`,
  so the eye height must scale too (1.7775 * 0.5 = 0.88875). Now `default_dimensions()`
  plus a unit test. (S)
- `cause_damage` (was :284-293) called `break_and_drop_items`, which drops an
  ARMOR_STAND item. Vanilla `ArmorStand.causeDamage` (ignite / burns armour stands)
  calls `brokenByAnything` only: the item is destroyed, only equipment drops.
  Now calls `broken_by_anything()`. (S)

Remaining gaps:
1. `armor_stand.rs:581-590` - explosion/mob-damage gating. The `mobGriefing` early
   return sits before the `BYPASSES_INVULNERABILITY` check. Vanilla `ArmorStand.damage`
   only consults `BYPASSES_INVULNERABILITY` first and has no mobGriefing gate (mob
   griefing for explosions is applied in `Explosion.finalizeExplosion`). Verified only
   from memory; confirm against decompiled `ArmorStand.damage` before changing.
   Fix: drop the gate, or move it inside the `IS_EXPLOSION` branch.
   Vanilla class: `ArmorStand.damage`. Size M.

2. `armor_stand.rs:510-529` - `read_custom_nbt` only applies `Invisible` when the NBT
   value is `true`. Vanilla reads the key whenever it is present
   (`if (compound.contains("Invisible", 1)) setInvisible(getBoolean(...))`). Only
   observable if custom NBT is read into an already invisible stand, e.g. entity
   reload in place. Fix: `if let Some(invisible) = nbt.get_bool("Invisible")`. Size S.

3. `armor_stand.rs:481-493` - no vanilla `hasVisuals()/noPhysics` equivalent
   (`noPhysics = !hasVisuals()` is set by `readAdditionalSaveData`, `setMarker` and
   `setInvisible`). Pumpkin only zeroes marker dimensions and ignores block triggers;
   marker/invisible stands are still subject to collision and fluid pushing. Needs a
   `is_pushed_by_fluids`/collision hook plus the `noPhysics` flag. Vanilla class:
   `ArmorStand.hasVisuals`. Size M.

4. `armor_stand.rs:232-263` - `broken_by_anything` drops every `EquipmentSlot`
   (main/off hand, body, saddle). 1.20+ vanilla `brokenByAnything` iterates slots but
   only pops `EquipmentSlot.Type.HUMANOID_ARMOR` (head/chest/legs/feet) in the loop
   after `dropAllDeathLoot`. Confirm the exact slot filter against the decompiled
   source before narrowing, otherwise held items would be deleted. Vanilla class:
   `ArmorStand.brokenByAnything`. Size S/M.

5. `armor_stand.rs:546-660` - pushability not modelled: vanilla armour stands cannot
   be shoved by entities (`isPushable`). Pumpkin inherits `LivingEntity::is_pushable`.
   Verify in decompiled `ArmorStand` (override may live in `LivingEntity`), then add an
   `is_pushable(&self) -> false` override. Size S.
