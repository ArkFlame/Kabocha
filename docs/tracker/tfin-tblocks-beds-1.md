# Final rows from docs/tracker/tblocks-beds.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Source rows and their details live in docs/tracker/tblocks-beds.md.

## Result

- Row 42 (straw bed getSleepHeight): VERIFIED. Matches vanilla
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/StrawBedBlock.java:63-78`
  (HEAD redirects to the connected FOOT via getConnectedDirection, returns empty for a
  non-FOOT partner, then super.getSleepHeight) and
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/AbstractBedBlock.java:71-78`
  (shape.max(Direction.Axis.Y)). Pumpkin: `bed.rs::player_sleep_height` ->
  `sleep_shape_pos` (straw HEAD -> facing.opposite() FOOT) -> `sleep_height` (foot
  shape top 4/16 = 0.25), sent via set_pos in `entity/player.rs` with
  SLEEP_HEIGHT_OFFSET = 0.125 (`LivingEntity#setPosToBed`). Pure unit tests in bed.rs
  (`sleep_height_plus_offset_is_the_bed_top`,
  `straw_bed_sleep_shape_is_the_lower_foot_surface`) cover it. No source edit needed.
- Row 43 (sleeping/dead guard): FIXED. `crates/pumpkin-core/src/block/blocks/bed.rs:322-324`
  returns the problem when `player.is_sleeping() || !player.living_entity.is_alive()`,
  matching vanilla `ServerPlayer#getBedResult` OTHER_PROBLEM. Verified present in tree.
- Row 39 (per-position bed rule / biome overrides): BLOCKED-DATA. Vanilla
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/AbstractBedBlock.java:63-65`
  calls `level.environmentAttributes().getValue(attr, pos)` with per-position biome
  overrides. Pumpkin `crates/pumpkin-core/src/world/environment.rs::get_value_bed_rule`
  ignores `pos`, and generated `crates/pumpkin-data/src/generated/biome.rs` exposes only
  `boolean_attributes` (no bed-rule attribute value to read at a position). Missing
  generated field: a positional `EnvironmentAttribute<BedRule>` override on biome data.
- Row 40 (BedRule record fields): BLOCKED-DATA. Vanilla record
  `.vendor/vanilla-decompiled/net/minecraft/world/attribute/BedRule.java:12,17-19,39-41`
  has `destroyOnLeave`, `Optional<Component> errorMessage` and `asProblem()`. Generated
  `crates/pumpkin-data/src/generated/dimension.rs:14-18` `BedRule` has only `can_sleep`,
  `can_set_spawn`, `explodes`. Missing generated fields: `destroy_on_leave` and
  `error_message` (plus `asProblem()`). Needs codegen extension and regeneration from
  assets/.
- Row 41 (straw bed STRAW_BED_RULE): BLOCKED-DATA. Vanilla
  `.vendor/vanilla-decompiled/net/minecraft/world/level/block/StrawBedBlock.java:48-50`
  returns `EnvironmentAttributes.STRAW_BED_RULE`, registered at
  `.vendor/vanilla-decompiled/net/minecraft/world/attribute/EnvironmentAttributes.java:141-142`
  with default `BedRule.DESTROY_ON_LEAVE`. Generated
  `crates/pumpkin-data/src/generated/environment_attribute.rs` has `GameplayBedRule` but
  no `GameplayStrawBedRule` variant. Missing generated variant: `GameplayStrawBedRule`
  (`gameplay/straw_bed_rule`). Also depends on the per-position lookup from row 39.

No .rs source files edited this pass: rows 42/43 are already fixed and verified against
.vendor/vanilla-decompiled; rows 39-41 require generated pumpkin-data fields that are
off-limits plus codegen regeneration, so no in-scope source fix exists. No rustfmt needed
(no .rs touched). Did not compile (no cargo per task); orchestrator verifies.
