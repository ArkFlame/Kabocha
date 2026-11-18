# Parity rows for crates/pumpkin-core/src/entity/decoration/item_frame.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-1730b6** [partial/OWN-OPEN] own audit: 10 open, 4 fixed, 30 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-1730b6.md) docs/tracker/taudit-crates-pumpkin-core-src-en-1730b6.md

## Result
All ten numbered rows are FIXED/VERIFIED in `crates/pumpkin-core/src/entity/decoration/item_frame.rs`; no row remains STILL OPEN and none is BLOCKED-DATA. No source edit was required this final pass: every fix is present in source and was re-verified against the per-row 1.21.x citations in `docs/tracker/taudit-crates-pumpkin-core-src-en-1730b6.md`. The decompiled vanilla tree is absent from this worktree (`.vendor/` empty), so citations use the canonical paths the audit recorded: `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/{ItemFrame,HangingEntity,BlockAttachedEntity}.java`.

- Row 1 (support-check cadence): VERIFIED. `SUPPORT_CHECK_INTERVAL = 100` (item_frame.rs:123) and `tick` fires on ticks 100, 200, ... via `fetch_add(..) + 1 < 100` (item_frame.rs:499-511), matching `BlockAttachedEntity.CHECK_INTERVAL = 100` and `hangingTickFrequency`. `.vendor/.../decoration/BlockAttachedEntity.java`.
- Row 2 (add-item sound): VERIFIED. `set_item` plays `get_add_item_sound()` only when `!item_stack.is_empty() && update_neighbours` (item_frame.rs:274-300), matching `ItemFrame.setItem(stack, updateNeighbours, playSound)`. `.vendor/.../decoration/ItemFrame.java`.
- Row 3 (comparator-only update): VERIFIED. `set_item`, `set_rotation` and `drop_item` call `world.update_neighbour_for_output_signal(&pos, &Block::AIR)` (item_frame.rs:296-300, 314-318, 436-438), matching `level.updateNeighbourForOutputSignal(pos, Blocks.AIR)`. `.vendor/.../decoration/ItemFrame.java`.
- Row 4 (doEntityDrops): VERIFIED. `drop_item` returns before both drops when `world.level_info.load().game_rules.entity_drops` is false (item_frame.rs:440-444), matching `GameRules.ENTITY_DROPS`. `.vendor/.../decoration/ItemFrame.java`.
- Row 5 (invulnerable damage): VERIFIED. `damage_with_context` returns false for `is_invulnerable_to(..)` sources (item_frame.rs:664-667), matching `isInvulnerableToBase(source)`. `.vendor/.../decoration/{ItemFrame,HangingEntity}.java`.
- Row 6 (invisible flag): VERIFIED. `is_invisible`/`set_invisible` use the shared `Entity.invisible` flag (item_frame.rs:337-343) and NBT writes/reads `Invisible` through it (item_frame.rs:526, 550-551), matching `setInvisible(false)` and `output.putBoolean("Invisible", this.isInvisible())`. `.vendor/.../decoration/ItemFrame.java`.
- Row 7 (custom name): VERIFIED. `get_frame_item_stack_with_data` copies the name via `custom_name.get_text()` (item_frame.rs:353-359), matching the verbatim `CUSTOM_NAME` copy. Pumpkin's `ItemStack` stores a String so component styles are not kept. `.vendor/.../decoration/ItemFrame.java`.
- Row 8 (creative drop ordering): VERIFIED. `drop_item` returns before both drops for a creative player (item_frame.rs:446-454), matching the `hasInfiniteMaterials()` branch. `.vendor/.../decoration/ItemFrame.java`.
- Row 9 (single-slot Container): VERIFIED. `ItemFrameInventory` (size 1, max stack 1) is exposed via `get_container_inventory` (item_frame.rs:48-117, 560-569), matching `getSlot(0)`. `.vendor/.../decoration/ItemFrame.java`.
- Row 10 (DOWN drop offset): VERIFIED. `drop_item` applies the -0.6 y offset for Down-facing frames (item_frame.rs:456-467), matching the Paper MC-123848 `spawnAtLocation` override. `.vendor/.../decoration/ItemFrame.java`.

Non-row deferred issue (NOT one of the ten numbered rows): explosion divergence. Pumpkin's early `return false` for `EXPLOSION`/`PLAYER_EXPLOSION` (item_frame.rs:671-676) keeps the frame intact, while decompiled vanilla routes explosion damage that fails `shouldDamageDropItem` into `BlockAttachedEntity.hurtServer` and breaks the frame. BLOCKED-DATA: fixing it needs the generated `IS_EXPLOSION` damage-type tag plus the `ignoreExplosion`/`shouldAffectBlocklikeEntities` path in `explosion.rs` (a file outside this unit, whose only editable source is `item_frame.rs`). It blocks no numbered row.


## Result (third pass - final, unit closed)

Re-read `crates/pumpkin-core/src/entity/decoration/item_frame.rs` in full. No decompiled vanilla
tree is present in this worktree (`.vendor/` is empty), so verification rests on the per-row
vanilla citations recorded in `docs/tracker/taudit-crates-pumpkin-core-src-en-1730b6.md` against
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/{ItemFrame,HangingEntity,BlockAttachedEntity}.java`
plus fresh source inspection. Every formerly STILL OPEN item is already implemented in source;
no source edit was required this pass.

Row `own/taudit-crates-pumpkin-core-src-en-1730b6`: FIXED/VERIFIED. All ten audited gaps are
closed in `item_frame.rs`:
- Row 1 (support-check cadence) VERIFIED: `tick` fires the check on tick 100, 200, ... via
  `SUPPORT_CHECK_INTERVAL = 100` and `fetch_add(..) + 1 < 100` (item_frame.rs:123, 499-510),
  matching `BlockAttachedEntity.CHECK_INTERVAL = 100` / `ticksSinceLastCheck++ >= hangingTickFrequency`.
  `.vendor/.../decoration/BlockAttachedEntity.java`.
- Row 2 (add-item sound) VERIFIED: `set_item` plays `get_add_item_sound()` only when
  `!item_stack.is_empty() && update_neighbours` (item_frame.rs:274-294), matching
  `ItemFrame.setItem(.., updateNeighbours, playSound)`. `.vendor/.../decoration/ItemFrame.java`.
- Row 3 (comparator-only update) VERIFIED: `set_item`, `set_rotation` and `drop_item` call
  `world.update_neighbour_for_output_signal(&pos, &Block::AIR)` (item_frame.rs:296-300, 314-318,
  436-438), matching `level().updateNeighbourForOutputSignal(this.pos, Blocks.AIR)`.
  `.vendor/.../decoration/ItemFrame.java`.
- Row 4 (doEntityDrops) VERIFIED: `drop_item` returns before both drops when
  `world.level_info.load().game_rules.entity_drops` is false (item_frame.rs:442-444), matching
  `GameRules.ENTITY_DROPS`. `.vendor/.../decoration/ItemFrame.java`.
- Row 5 (invulnerable damage) VERIFIED: `damage_with_context` returns false for
  `is_invulnerable_to(..)` sources (item_frame.rs:664-667), matching `isInvulnerableToBase(source)`.
  `.vendor/.../decoration/{ItemFrame,BlockAttachedEntity}.java`.
- Row 6 (invisible flag) VERIFIED: `is_invisible`/`set_invisible` use the shared `Entity.invisible`
  flag (item_frame.rs:337-343); NBT writes/reads `Invisible` through it (item_frame.rs:526, 550-551),
  matching `output.putBoolean("Invisible", this.isInvisible())` / `this.setInvisible(input.getBooleanOr("Invisible", false))`.
  `.vendor/.../decoration/ItemFrame.java`.
- Row 7 (custom name) VERIFIED: `get_frame_item_stack_with_data` copies the name via
  `custom_name.get_text()` (item_frame.rs:353-359), matching the verbatim `CUSTOM_NAME` copy;
  Pumpkin's `ItemStack` stores a String so component styles are not kept.
  `.vendor/.../decoration/ItemFrame.java`.
- Row 8 (creative drop ordering) VERIFIED: `drop_item` returns before both drops for a creative
  player (item_frame.rs:446-454), matching `hasInfiniteMaterials()` branch.
  `.vendor/.../decoration/ItemFrame.java`.
- Row 9 (single-slot Container) VERIFIED: `ItemFrameInventory` (size 1, max stack 1) is exposed via
  `get_container_inventory` (item_frame.rs:48-117, 560-569), matching `getSlot(0)`.
  `.vendor/.../decoration/ItemFrame.java`.
- Row 10 (DOWN drop offset) VERIFIED: `drop_item` applies the -0.6 y offset for Down-facing frames
  (item_frame.rs:456-467), matching `this.getDirection() == Direction.DOWN ? -0.6F : 0.0F`.
  `.vendor/.../decoration/ItemFrame.java`.

Non-row deferred issue (NOT one of the ten rows): explosion divergence. Decompiled
`ItemFrame.hurtServer` routes explosion damage that fails `shouldDamageDropItem` into
`BlockAttachedEntity.hurtServer` and breaks the frame, while Pumpkin's early `return false` for
`EXPLOSION`/`PLAYER_EXPLOSION` (item_frame.rs:671-676) keeps it intact. Classified BLOCKED-DATA:
fixing needs ingestion of the generated `IS_EXPLOSION` damage-type tag plus the
`ignoreExplosion`/`shouldAffectBlocklikeEntities` path in `explosion.rs` and entity defaults
(files outside this unit and generated pumpkin-data fields). It blocks no row in this unit.

No pure unit-test seam exists (every affected behaviour needs a `World` plus fixture;
`calculate_bounding_box` is a pure formula and formula-mirroring tests are disallowed). Compile
not run (worker restriction); every referenced API confirmed present by source inspection.
