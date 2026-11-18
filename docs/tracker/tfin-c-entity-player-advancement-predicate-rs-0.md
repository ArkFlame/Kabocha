# Final rows from docs/tracker/tpar-ates-pumpkin-core-src-entity-player-advancement-predicate-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Third pass. Re-read the last Result in the audit doc (taudit-...eaae88.md); only rows 3, 7 and 8 remained STILL OPEN. Rows 1-2, 4-6, 9-13 stay as fixed/verified in earlier sections and were not touched.

## Result
- Row 3 (EntityPredicate fields): PARTIALLY FIXED. Added `EntityState.looking_at_type` and enforced `feet_item`/`looking_at` in `matches_entity`. Remaining (nbt, slots, type_specific for fishing_hook/lightning/player/piglin/raider/sheep/frog, passenger looking_at population) are BLOCKED-DATA: requires `EntityNbtPredicate`/`SlotsPredicate`/type_specific subpredicates and target-entity state not present in pumpkin-data or this unit's callers. Vanilla reference `.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/entity/EntityPredicate.java`.
- Row 7 (PlacedBlock actual stack): FIXED. `AdvancementTrigger::PlacedBlock` now carries `enchantments: Vec<(String,u32)>`, populated from the placed stack in `block/registry.rs`, and `trigger.rs` forwards them to `PlacedBlockPredicate::matches`. Vanilla reference `.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/ItemPredicate.java`.
- Row 8 (item condition count): VERIFIED. `matches_item_condition` already takes count and all emitters (consume_item/filled_bucket/fishing_rod_hooked/used_totem/shot_crossbow) forward the real stack count. Vanilla reference `.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/ItemPredicate.java`.

Status: UNVERIFIED fixed=2 open=1 files=predicate.rs|trigger.rs|registry.rs
