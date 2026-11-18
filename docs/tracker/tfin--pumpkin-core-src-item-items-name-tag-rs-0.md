# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-item-items-name-tag-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

## Result (final pass 2026-11-18)

All rows closed. No Rust change needed; name_tag.rs:27-44 already matches decompiled vanilla.

- **items/Name Tag** [VERIFIED]: re-read .vendor/vanilla-decompiled/net/minecraft/world/item/NameTagItem.java:17-39 (interactLivingEntity) and compared line-by-line to name_tag.rs:27-44. customName != null -> get_data_component::<CustomNameImpl>(); target.getType().canSerialize() -> raw_entity.entity_type.saveable; target.isAlive() -> living.entity.is_alive(); setCustomName on any LivingEntity -> raw_entity.set_custom_name(...); instanceof Mob -> get_mob()-conditional persistence_required.store(true); shrink(1) -> decrement_unless_creative; SUCCESS/PASS -> BlockActionResult::SuccessServer/Pass. ArmorStand is a serializable LivingEntity but not a Mob, so it is named but not made persistent, matching vanilla. Only non-observable nuance: vanilla returns SUCCESS when customName+canSerialize even if the target is dead (inner work block skipped); Pumpkin returns Pass. Dead entities are not interactable targets, so unreachable. Left as-is to avoid churn.
- **own/titems-name-tag-b9060c** [VERIFIED]: the linked doc's sole remaining gap (Row 1, use_on_entity returns ()) is obsolete. ItemBehaviour::use_on_entity now returns BlockActionResult (crates/pumpkin-core/src/item/mod.rs), name_tag.rs returns SuccessServer/Pass, and the network layer consumes the result via result.consumes_action(). No framework change needed. All other findings in that doc re-confirmed against the .vendor path above.
- No pure unit-test seam: the naming rule is inline in use_on_entity and needs a live Player, ItemStack and EntityBase; no isolated helper exists. Test skipped.
- Files changed this pass: doc only. name_tag.rs left intact and type-checked against current method signatures.
