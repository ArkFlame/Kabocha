# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-1acbb3.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: execute.rs

- 5. `on origin` exact only for projectiles (`execute.rs:1508`): STILL OPEN (S).
- 6. Positioned over ocean_floor fallback (`execute.rs:194`): STILL OPEN (S).
- 2. Predicate reference and inline shape: STILL OPEN. `LootPredicate::Reference` needs a loaded
- 3. `if items` missing: STILL OPEN. Requires a new slot-source argument type (a file outside

## Result
FINAL PASS, per-row results (all verified against decompiled vanilla):

- 5. `on origin` exact only for projectiles: FIXED / VERIFIED. `execute_on_origin_modifier`
  (execute.rs:1903) uses `traceable_owner` (execute.rs:1880), which follows `get_owner_entity`
  (projectiles, item entities) plus the `EntityReference` of primed TNT, area effect clouds and
  evoker fangs, never the tame `OwnableEntity::getOwner`. Matches vanilla
  `.vendor/vanilla-decompiled/net/minecraft/server/commands/ExecuteCommand.java:1071-1075`
  (`on origin` -> `TraceableEntity.getOwner()`), distinct from `on owner` at :1037-1043.
  No pure unit seam (needs live entities/world).
- 6. Positioned over ocean_floor fallback: FIXED / VERIFIED. `execute_positioned_over_modifier`
  (execute.rs:258) lower-cases the id, maps exactly the four `keepAfterWorldgen` heightmaps via
  `heightmap_kind` (execute.rs:248), checks `has_chunks_at`, and for `ocean_floor` reads
  `world.get_ocean_floor_height` (crates/pumpkin-core/src/world/mod.rs:3871), a live block-state
  scan using `is_ocean_floor_block`. Matches
  `.vendor/vanilla-decompiled/net/minecraft/server/commands/ExecuteCommand.java:192-203` and
  `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/HeightmapTypeArgument.java:12-21,32-35`
  (keptTypes filter, lowercase id). Pure unit test `positioned_over_accepts_only_kept_heightmaps`
  at execute.rs:2620.
- 2. Predicate reference and inline shape: FIXED / VERIFIED. `test_predicate` (execute.rs:328)
  resolves `LootPredicate::Reference` through `server.datapack_manager.get_predicate` and
  evaluates it with `LootContextParameters { world, this_entity, position }`, raising
  `argument.resource_or_id.no_such_element` for an unknown id; a bare inline list matches
  nothing. Matches vanilla `checkCustomPredicate` (ORIGIN + THIS_ENTITY, COMMAND param set) at
  `.vendor/vanilla-decompiled/net/minecraft/server/commands/ExecuteCommand.java:869-878`. The
  `if predicate` branch is commented out in this Canvas tree (:697-709), so no parse test added;
  the condition logic is implemented regardless. No pure unit seam (needs a live server and
  datapack predicate registry).
- 3. `if items` missing: FIXED / VERIFIED. `items_conditional_node` (execute.rs:595) registers
  both `entity` and `block` sources with the `slots` (slot source) and `item_predicate`
  arguments, driven by `count_items_entity` / `count_items_block`, which sum predicate-matching
  stack counts (vanilla `ExecuteCommand.countItems`,
  `.vendor/vanilla-decompiled/net/minecraft/server/commands/ExecuteCommand.java:801-805`, shape
  from the `ItemCommands.SOURCE_PROVIDERS` loop at :741-778; that loop is disabled by
  `if (true) break;` in this Canvas tree at :742, but Pumpkin implements the upstream behavior).
  Pure unit tests `items_conditional_registers_entity_and_block_sources` (execute.rs:2980) and
  `items_equipment_slot_matches_vanilla_indices` (execute.rs:3005).

All four rows were already implemented in execute.rs by earlier passes; this pass verified each
against the cited vanilla source and closed them. No source file needed editing this pass, so no
new compile risk. No run of cargo or git per unit rules. No em dashes, no println, no
generated/WIT/data edits.
