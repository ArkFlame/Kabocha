# entities/Mannequin (re-review)

Source: crates/pumpkin-core/src/entity/decoration/mannequin.rs
Vanilla class: net.minecraft.world.entity.decoration.Mannequin (extends LivingEntity). No decompiled source is vendored, so the behaviour below is checked against the minecraft.wiki Mannequin page (Behavior, Entity data) and the generated data.

## Fixed
1. **can_hit missing (S).** `mannequin.rs:176` (EntityBase impl) inherited `can_hit` = false (`entity/mod.rs:625`), so mannequins were skipped by attack/projectile raycasts, predicates and spectate (`entity/mod.rs:650`, `entity/projectile/owner.rs:189`, `entity/predicate/mod.rs:38`, `net/java/play/spectate_entity.rs:43`). Vanilla mannequins take damage, are hit by projectiles and are targeted by Johnny vindicators, and `entity_type.rs` marks them `attackable: Some(true)`. Added `!self.get_entity().is_removed()` (same shape as `armor_stand.rs:451`) and an assertion in the existing NBT test.

## Previous gaps that are not gaps
- **Right-click equipping (old gap 1).** Not vanilla: the wiki says items are placed into hands/armor slots by commands or by dispenser; armor `EquippableImpl` has `equip_on_interact: false` (`generated/item.rs:36199`), and `dispenser.rs:1358-1431` already equips any living entity into an empty slot when `allowed_entities` is None. No `interact` override is needed.
- **Pose whitelist (old gap 4).** Vanilla valid values are exactly standing, crouching, swimming, fall_flying, sleeping; `parse_valid_pose` (`mannequin.rs:37-46`) matches it. Correct as-is.
- **Dispenser equipping.** Covered by the generic living-entity path in `dispenser.rs:1380-1411` (mannequin exposes `LivingEntity` equipment), no mannequin branch needed.

## Remaining gaps
1. **is_pushable inherited as false (S, verify).** `mannequin.rs:175` does not override `is_pushable`; `entity/mod.rs:1075` defaults to false. A non-immovable mannequin is a normal `LivingEntity` in vanilla, so it is probably pushable; the wiki does not state it. Suggested fix if confirmed: `!self.get_immovable() && self.living_entity.can_be_pushed(EntityBase::on_climbable(self))`. Needs the vanilla class or a client repro before changing.
2. **World-border damage immunity (S, verify).** Vanilla mannequins take no damage outside the world border. A grep found no world-border damage path in `pumpkin-core` at all, so this is a general missing system, not mannequin-specific; track separately.
3. **Death pose (S, client-only).** Generic `LivingEntity` death sets the Dying pose; vanilla mannequin pose values exclude dying, but this only affects client rendering. Not blocking.

## Next
Gap 1 needs vanilla confirmation. Gaps 2-3 should be separate tasks (world border damage system; client pose rendering).
