# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-db7d33.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: entity_index.rs explosion.rs mod.rs

Re-checked every formerly STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `world/level/block/TntBlock.java` (Paper/CraftBukkit) and the live sources. No row remains STILL OPEN; no behaviour change was needed.
Every formerly STILL OPEN row was re-verified against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/item/PrimedTnt.java` and `.vendor/vanilla-decompiled/net/minecraft/world/level/block/TntBlock.java` (Paper/CraftBukkit, protocol V_26_3) and the live Pumpkin sources. No `.rs` change was needed: each row is already FIXED or matches vanilla in the current checkout. All cross-file symbols used by the row-3 portal fix resolve: `EntityBase::on_changed_dimension` default (`entity/mod.rs:321`) invoked from `World::transfer_entity` (`world/mod.rs:6638`), `Entity::tick_portal` (`entity/mod.rs:4110`), the `ExplosionDamageCalculator` trait (`world/explosion.rs:58-116`), `World::explode_from_with_calculator` (`world/mod.rs:5093-5105`) and the generated `Block::NETHER_PORTAL` const. `entity_index.rs:665` sits inside a `#[tokio::test]` module, not a production dimension-change path, so no hook is missing there.

## Result
VERIFIED against PrimedTnt.java and TntBlock.java (vanilla); no open row, no BLOCKED-DATA.
