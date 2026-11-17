**FIXED** `2077c4c85` (fix(entity): make mannequins pushable and keep hidden descriptions on save). VERIFIED against code: entity/decoration/mannequin.rs:180-185 (is_pushable).

# entities/Mannequin (re-review)

Source: crates/pumpkin-core/src/entity/decoration/mannequin.rs
Vanilla class: net.minecraft.world.entity.decoration.Mannequin (extends LivingEntity). No decompiled source is vendored; behaviour is checked against the minecraft.wiki Mannequin page (wikitext API, Behavior/Entity data) and the generated data (entity_type.rs, tracked_data.rs).

## Fixed in this re-review
1. **is_pushable missing (S).** The mannequin implements EntityBase directly (it is not a Mob), so it inherited the EntityBase default false (entity/mod.rs:1107): non-immovable mannequins ignored body pushes and never pushed other entities (entity/mod.rs:1173, 1192). Vanilla mannequin is a LivingEntity and only the immovable flag stops movement (wiki: "immovable ... cannot be moved, defaults to false"). Added "!self.get_immovable() && self.living_entity.can_be_pushed(EntityBase::on_climbable(self))" plus test assertions.
2. **Hidden custom description dropped on save (S).** write_custom_nbt used get_description(), which returns None while hide_description is set, so a custom description with hide_description=true saved only hide_description and lost the text. The two tags are independent in vanilla NBT. The writer now reads the stored description directly and writes both tags; the NBT test round-trips the hidden description.

## Previously fixed
- **can_hit missing (S)** (commit 43f78a5c0). Vanilla mannequins take damage, are hit by projectiles and targeted by Johnny vindicators; entity_type marks them attackable. Damage now reaches LivingEntity through the EntityBase defaults (damage_with_context entity/mod.rs:990, kill 1415).

## Checked and not gaps
- **Death pose.** Generated MANNEQUIN poses include Dying (0.2x0.2), so the Dying pose is vanilla on death. Only the pose NBT field is restricted to standing/crouching/swimming/fall_flying/sleeping, and pose_to_name normalisation matches that.
- **Description vs name.** The description renders where a player below_name score shows (wiki); no getName override is needed.
- **Right-click equipping.** Not vanilla; items are placed by commands or dispensers, and dispenser.rs already equips any living entity.
- **Mob-only tags.** CanPickUpLoot/NoAI/LeftHanded/DeathLootTable/drop_chances/home_pos/PersistenceRequired are written by the mob path only; the mannequin is not a MobEntity and cannot be leashed, so they never appear.
- **Description default** matches the wiki: entity.minecraft.mannequin.label ("NPC") when omitted.

## Remaining gaps
1. **World-border damage immunity (M).** Vanilla mannequins do not take world-border damage (wiki). pumpkin-core has no world-border damage path for any entity; this is a general missing system, tracked separately.
2. **is_collidable (S, verify).** EntityBase default false (entity/mod.rs:653); vanilla Mannequin may override canBeCollidedWith for boat/entity collision. Needs the vanilla class or a client repro before changing.
3. **Pose saved while dying (S, low risk).** pose_to_name maps Dying to "standing", so a mannequin autosaved during its death animation reloads standing. Only observable if the entity is saved before removal; no action proposed.
