# Final rows from docs/tracker/tpar-ates-pumpkin-core-src-entity-player-advancement-predicate-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

Third pass. Rows 3, 7 and 8 were open in the prior pass. Rows 1-2, 4-6, 9-13 stay as fixed/verified in earlier sections and were not touched.

## Result
- Row 3 (EntityPredicate fields): BLOCKED-DATA. Fixed in code: `EntityState.looking_at_type` added and `feet_item`/`looking_at` enforced in `matches_entity` (`crates/pumpkin-core/src/entity/player/advancement/predicate.rs`, `crates/pumpkin-core/src/entity/player/advancement/trigger.rs`). Remaining requires missing subsystem/runtime state: `nbt` (`EntityNbtPredicate`/entity NBT snapshot), `slots` (`SlotsPredicate`/slot provider access), type_specific subpredicates `fishing_hook`, `lightning`, `player`, `piglin`, `raider`, `sheep`, `frog` and other entity subpredicates, plus nested vehicle/passenger/targeted entity predicate evaluation and population of passenger/looking-at target entity state in advancement contexts. Vanilla reference: `.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/entity/EntityPredicate.java`.
- Row 7 (PlacedBlock actual stack): FIXED. `AdvancementTrigger::PlacedBlock` carries `enchantments: Vec<(String,u32)>`, populated from the placed stack in `crates/pumpkin-core/src/block/registry.rs` and forwarded in `crates/pumpkin-core/src/entity/player/advancement/trigger.rs` to `PlacedBlockPredicate::matches`/`ItemPredicate`. Vanilla reference: `.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/ItemPredicate.java`.
- Row 8 (item condition count): VERIFIED. `matches_item_condition` takes `count` and all emitters (`consume_item`, `filled_bucket`, `fishing_rod_hooked`, `used_totem`, `shot_crossbow`) forward the real stack count in `crates/pumpkin-core/src/entity/player/advancement/trigger.rs` and `predicate.rs`. Vanilla reference: `.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/ItemPredicate.java`.

Status: UNVERIFIED fixed=1 verified=1 blocked=1 files=predicate.rs|trigger.rs|registry.rs
