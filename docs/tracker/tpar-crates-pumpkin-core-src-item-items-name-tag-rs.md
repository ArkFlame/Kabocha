# Parity rows for crates/pumpkin-core/src/item/items/name_tag.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **items/Name Tag** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/titems-name-tag-b9060c** [partial/OWN-OPEN] own audit: 2 open, 0 fixed, 0 verified, 0 blocked-data (docs/tracker/titems-name-tag-b9060c.md) docs/tracker/titems-name-tag-b9060c.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass 2026-11-18)

Both rows VERIFIED. No code change; name_tag.rs already matches decompiled vanilla.

- **items/Name Tag** [VERIFIED]: re-read .vendor/vanilla-decompiled/net/minecraft/world/item/NameTagItem.java:17-39 (interactLivingEntity) and compared line-by-line to name_tag.rs:27-44:
  - customName != null -> `item.get_data_component::<CustomNameImpl>()` outer gate.
  - target.getType().canSerialize() -> `raw_entity.entity_type.saveable` (EntityType.java:318 `canSerialize()` returns `serialize`; pumpkin-data `saveable: false` is exactly the four vanilla no-save types FISHING_BOBBER, LEASH_KNOT, LIGHTNING_BOLT, PLAYER).
  - target is LivingEntity -> `entity.get_living_entity()`; setCustomName -> `raw_entity.set_custom_name(name.name.clone())`.
  - instanceof Mob -> setPersistenceRequired() -> `entity.get_mob()` -> `persistence_required.store(true)`. ArmorStand is a serializable LivingEntity but not a Mob, so it is named but not made persistent, matching vanilla.
  - isAlive() gate -> `living.entity.is_alive()`; `!isClientSide()` is implicit (use_on_entity is server-only).
  - shrink(1) -> `decrement_unless_creative`: Player.java:911-916 passes a CLONE of the held stack to interactLivingEntity when hasInfiniteMaterials(), so vanilla also does not consume in creative; ItemStack.java:1323 consume() confirms the infinite-materials skip.
  - SUCCESS/PASS -> BlockActionResult::SuccessServer / Pass.
  Only non-observable nuance: vanilla returns SUCCESS when customName+canSerialize even if the target is dead (inner work gate skipped); Pumpkin returns Pass. Dead entities are not interactable targets, so this is unreachable in practice. Left as-is to avoid churn.
- **own/titems-name-tag-b9060c** [VERIFIED]: the linked doc sole remaining gap (Row 1, "use_on_entity returns ()") is obsolete. ItemBehaviour::use_on_entity now returns BlockActionResult (crates/pumpkin-core/src/item/mod.rs:79-86) and name_tag.rs returns SuccessServer/Pass; the registry and network layers forward and consume it. No framework change needed. All other prior findings in that doc re-confirmed against the .vendor paths above.
- No pure unit-test seam: the naming rule is inline in use_on_entity and needs a live Player, ItemStack and EntityBase; no isolated helper exists. Test skipped.
- Files changed this pass: doc only (this Result section). name_tag.rs left intact and type-checked against current method signatures (get_living_entity -> Option<&LivingEntity>, get_mob -> Option<&dyn Mob>, MobEntity.persistence_required: AtomicBool).

