# entities/Mannequin (RE-REVIEW-UNTRACKED)

Source: `crates/pumpkin-core/src/entity/decoration/mannequin.rs` (399 lines, landed in 65251f765, identical to upstream/master).
Vanilla class: `net.minecraft.world.entity.decoration.Mannequin` (extends `LivingEntity`). No decompiled source under `.vendor/` in this checkout, so items marked "verify" need the vanilla class or a client repro before changing.

## Verified against generated data / call sites (no action)
- Tracked-data ids/types match vanilla serialisers: `mannequin.rs:176-192` sets profile, customisation byte, main hand varint, immovable bool, description optional (`tracked_data::mannequin`).
- Immovable gating: `mannequin.rs:194-200` (`is_immobile`/`is_effective_ai`) is honoured by `living.rs:1995` (no movement input) and `living.rs:2047-2058` (no travel), matching vanilla "immovable mannequin does not move or fall".
- NBT schema (`mannequin.rs:215-312`): profile, hidden_layers, main_hand, pose, immovable, description/hide_description; absence resets via `remove_custom_nbt`.
- Layers bitmask `0x7F` and part order (cape..hat) match the vanilla player-model mask; already tested at `mannequin.rs:337-398`.
- Registration: `entity/type.rs:251`, poses/dimensions/attributes from `pumpkin-data/src/generated/entity_type.rs:6996`; `fetchprofile.rs:135-350` round-trips the profile through summon NBT.

## Gaps
1. **Equipping via right-click is missing (M).**
   `mannequin.rs:175` (`impl EntityBase`) implements no `interact`/`interact_with_hand`/`interact_at`, and `EntityBase::interact` defaults to `false` (`entity/mod.rs:972`), so a mannequin can never be given armour or held items even though the entity exists to display them.
   Vanilla: `Mannequin` handles `interact`/`mobInteract` (equip via `getEquipmentSlotForItem`, hand items, pose cycling on empty hand) — confirm the exact vanilla branches.
   Suggested fix: mirror `armor_stand.rs:341` (`interact_at_position`), reusing `get_equipment_slot_for_item` (`mob/equipment.rs`), `LivingEntity::entity_equipment` and `send_equipment_changes` (`living.rs:724`); swap the player's stack with the slot item and return true.
2. **`can_hit` inherited as false (S, verify).**
   `EntityBase::can_hit` defaults to `false` (`entity/mod.rs:614`); only `ArmorStandEntity` (`armor_stand.rs:449`) and `Mob` (`mob/mod.rs:3062`) override it. Consequence: mannequins cannot be hit by projectiles (`entity/mod.rs:649`), cannot be selected for spectating (`net/java/play/spectate_entity.rs:43`) and are skipped by predicates (`predicate/mod.rs:38`).
   Vanilla: mannequin is a normal `LivingEntity` and `entity_type.rs:7049` marks it `attackable: Some(true)`.
   Suggested fix: `fn can_hit(&self) -> bool { !self.get_entity().is_removed() }` (same shape as armour stand). Verify against vanilla `canBeHitByProjectile`/pick behaviour first.
3. **`is_pushable` inherited as false (S, verify).**
   `entity/mod.rs:1064` defaults to `false` and `MannequinEntity` does not override it, so a movable (`immovable=false`) mannequin can never be pushed, while the vanilla `immovable` flag exists precisely to stop it moving. If vanilla `Mannequin#isPushable` is `!isImmovable()`, add that override (it pairs with the travel gate above); if vanilla mannequins are never pushable, no change.
4. **Pose restriction needs vanilla confirmation (S, verify).**
   `parse_valid_pose` (`mannequin.rs:37-46`) accepts only standing/crouching/swimming/fall_flying/sleeping; `entity_type.rs:7132-7145` also lists Dying/SpinAttack dimensions. If vanilla `Mannequin` reads NBT pose with `Pose` lookup restricted to its valid-pose list, current code is correct; if it accepts any pose name, extend the match. Unknown names fall back to Standing either way.
5. **Client-facing pose/death**: death via generic `LivingEntity` sets the Dying pose; no mannequin-specific handling observed. Client-only, not blocking.

## Next
Gap 1 is the only behaviour that is certainly absent from vanilla parity and needs a real implementation; it requires the vanilla interaction semantics (armour slot pick by hit height, hand preference, sneak behaviour) to avoid guessing. Gaps 2-3 are two-line overrides pending vanilla confirmation.
