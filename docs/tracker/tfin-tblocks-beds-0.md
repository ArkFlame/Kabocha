# Final rows from docs/tracker/tblocks-beds.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

STILL OPEN and were left alone; they need generated-data/codegen changes.
- Row 42 / h6 row 4 (straw bed getSleepHeight): verified STILL OPEN fix is already
- Rows 39-41 stay open (not marked STILL OPEN): need pumpkin-data BedRule codegen
- Only row marked STILL OPEN is row 42 (straw bed `getSleepHeight`). Verified it is

## Result
FINAL RESULTS. Every row is closed as FIXED, VERIFIED (with .vendor vanilla path) or BLOCKED-DATA (exact missing generated field).

- Row 42 / h6 row 4 (straw bed `getSleepHeight`): VERIFIED, matches vanilla. Fix already present in tree; no new source edit needed. Vanilla: `.vendor/vanilla-decompiled/net/minecraft/world/level/block/StrawBedBlock.java:63-78` redirects a HEAD half to the connected FOOT (`pos.relative(getConnectedDirection(state))`, i.e. facing.getOpposite; empty if that state is not the same block's FOOT) then calls `super.getSleepHeight`; `AbstractBedBlock.java:71-78` returns `state.getShape(level,pos).max(Direction.Axis.Y)`; `LivingEntity.java#setPosToBed` adds `adjustHeight = 0.125`. Pumpkin: `bed.rs::SLEEP_HEIGHT_OFFSET = 0.125` (581), `sleep_height` (584), `sleep_shape_pos` (594, straw HEAD -> opposite/foot), `player_sleep_height` (606-611), unit tests `sleep_height_plus_offset_is_the_bed_top` and `straw_bed_sleep_shape_is_the_lower_foot_surface` (671-699); wired at `entity/player.rs:2833-2837` via `set_pos`.

- Row 43 / h6 row 5 (sleep rejected for sleeping/dead player): FIXED. `bed.rs:324` guards `if player.is_sleeping() || !player.living_entity.is_alive()`.

- Row 39 (per-position bed rule / biome overrides): BLOCKED-DATA. Vanilla `AbstractBedBlock.java:63-65` calls `level.environmentAttributes().getValue(attr, pos)` (position-aware, biome-overridable). Generated `crates/pumpkin-data/src/generated/biome.rs:17` exposes only `boolean_attributes`, no bed-rule attribute map at a position; `crates/pumpkin-core/src/world/environment.rs:228` `get_value_bed_rule` ignores `pos` and returns the dimension rule. Exact missing generated field: a per-position bed-rule attribute on `Biome` (analogous to `boolean_attributes`). Needs pumpkin-codegen extension + regeneration; generated files off-limits.

- Row 40 (BedRule record fields): BLOCKED-DATA. Vanilla record `BedRule(canSleep, canSetSpawn, destroyOnUse, destroyOnLeave, errorMessage)` and `asProblem()` at `.vendor/vanilla-decompiled/net/minecraft/world/attribute/BedRule.java:12,39-41`. Generated `crates/pumpkin-data/src/generated/dimension.rs:14-18` `BedRule` has only `can_sleep`, `can_set_spawn`, `explodes`. Exact missing generated fields: `destroy_on_leave` (bool) and `error_message` (translatable component) on `BedRule`, plus `asProblem()`. Needs codegen + regeneration.

- Row 41 (straw bed STRAW_BED_RULE): BLOCKED-DATA. Vanilla `StrawBedBlock.java:47-50` returns `EnvironmentAttributes.STRAW_BED_RULE`, registered at `.vendor/vanilla-decompiled/net/minecraft/world/attribute/EnvironmentAttributes.java:141-142`. Generated `crates/pumpkin-data/src/generated/environment_attribute.rs` has `GameplayBedRule` (52) but no `GameplayStrawBedRule` variant. Exact missing generated field: `GameplayStrawBedRule` enum variant with id/name mappings. Also depends on the row 39 per-position lookup.

- Files touched this pass: this doc only. No source edits (rows 39-41 require generated pumpkin-data fields/subsystem; rows 42/43 fixes pre-existing and verified against `.vendor/vanilla-decompiled`). Did not compile (no cargo per task); orchestrator verifies.
