# items / Name Tag — re-review

Source: crates/pumpkin-core/src/item/items/name_tag.rs (registered in item/items/mod.rs:148).
Compared against vanilla `NameTagItem.useOnEntity` (Item.useOnEntity override; 1.21-era behaviour: `stack.has(CUSTOM_NAME) && entity instanceof Mob mob && mob.getType().isSaveable()` -> set custom name, set persistent, consume one).

## Fixed in this audit
- Missing `Mob` guard (name_tag.rs:20-34): the handler only required `get_living_entity()`, so a name tag renamed any living entity, including ArmorStandEntity (decoration/armor_stand.rs:525 returns a LivingEntity, is not a Mob, and ARMOR_STAND is saveable). Vanilla only allows mobs. Now the `get_mob()` match is the outer condition and persistence/consume happen inside it.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | crates/pumpkin-core/src/item/mod.rs:71 (ItemBehaviour::use_on_entity) | useOnEntity returns PASS/SUCCESS, so a successful tag swings the arm and a failed one passes the interaction on | The trait returns (), so Pumpkin cannot signal SUCCESS/PASS; the item is still consumed correctly, but the client result is implicit. Framework-wide change, not name-tag-specific. | L |

Checked, no change: CUSTOM_NAME component presence gates the action (vanilla `has(DataComponents.CUSTOM_NAME)`); `entity_type.saveable` maps to `EntityType.isSaveable`; `is_alive` guard is harmless; `decrement_unless_creative` matches `stack.consume(1, player)`; `persistence_required = true` matches `setPersistent()` (named mobs do not despawn); rename of an already-named mob works; empty custom name allowed as in vanilla.
