# Parity rows for crates/pumpkin-core/src/entity/mob/skeleton/parched.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Parched** [partial/COVERED] The shared bow attack uses a plain ARROW rather than the variant-specific payload. This was previously only marked unverified. The shared bow attack uses a plain ARROW rather than the variant-specific payload.

## Result

- **entities/Parched** (plain ARROW vs variant-specific arrow payload): FIXED and VERIFIED.
  - The shared bow attack in `entity/ai/goal/bow_attack.rs::shoot` used to hardcode `&EntityType::ARROW`. It now derives the arrow entity type from the projectile item via `ArrowEntity::entity_type_for_item(projectile.item)`, matching vanilla `ProjectileUtil.getMobArrow` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/ProjectileUtil.java:288-293`), where the projectile's `ArrowItem.createArrow` decides the arrow entity (plain/spectral/tipped). The helper is already covered by `arrow.rs:958-974` (`projectile_payload_keeps_components_at_one_count`), so no duplicate test was added.
  - Parched's own variant payload is confirmed correct: `skeleton/parched.rs::get_arrow` adds WEAKNESS for 600 ticks, matching `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/skeleton/Parched.java:22-30` (`getArrow` -> `addEffect(WEAKNESS, 600)`). `bow_attack.rs::shoot` calls `mob.get_arrow(arrow)` before spawning, `ArrowEntity::add_effect` stores the effect in the item stack's potion contents (`arrow.rs:363-390`), and `do_post_hurt_effects` (`arrow.rs:844-864`) applies it to whatever the arrow hits. Parched fires a plain arrow in vanilla too, so no Parched-specific entity-type change was needed.
