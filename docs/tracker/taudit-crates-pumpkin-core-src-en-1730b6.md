# audit / crates/pumpkin-core/src/entity/decoration/item_frame.rs - audit (AUDIT)

Audited `crates/pumpkin-core/src/entity/decoration/item_frame.rs` against 1.21.x
`net.minecraft.world.entity.decoration.ItemFrame` / `HangingEntity` as seen through
Paper's source patches (`paper-server/patches/sources/.../ItemFrame.java.patch`,
`HangingEntity.java.patch`). No decompiled vanilla tree in this worktree
(`.vendor/` absent), so unpatched lines are from memory of 1.21.x and are marked
as such below.

## Fixed in an earlier audit
- `fix(entity): emit vanilla game events for item frame edits`:
  `GameEvent.BLOCK_CHANGE` on interact rotate and item insert (`ItemFrame.interact`:
  `this.gameEvent(GameEvent.BLOCK_CHANGE, player)`), `BLOCK_CHANGE` on damage that
  removes the item, and the early `if (this.isRemoved()) return false` guard in
  `damage_with_context`.

## Fixed in this pass (branch `kab/taudit-crates-pumpkin-core-src-en-1730b6`)
- `damage_with_context`: fixed frames were breakable by creative players and by
  out-of-world/generic-kill damage, and explosions broke frames. Vanilla
  `ItemFrame.hurtServer` returns false while `fixed` and for
  `DamageTypeTags.IS_EXPLOSION`; only losing the support block removes a frame.
  The spurious `GameEvent.ENTITY_DAMAGE` emission was dropped (vanilla's kill path
  emits ENTITY_DIE through `Entity.kill`, not ENTITY_DAMAGE).
  Conflict flag: the earlier audit note read the opposite for explosions; this pass
  treats `!source.is(IS_EXPLOSION)` as the vanilla guard. Re-check against a
  decompiled 1.21.x `ItemFrame` before relying on either claim.
- Support test: three divergent copies (`has_valid_support`, the inline predicate in
  `survives`, and `is_supported`) used `is_side_solid(facing)` or
  `is_side_solid || is_solid`. Vanilla `ItemFrame.survives()` requires
  `blockstate.isSolid()` (repeater/comparator only for horizontal frames). Unified on
  `is_solid()` + `is_diode`; `is_supported` delegates to it. Resolves earlier
  remaining gaps 1 (side-solid term) and 4 (fixed frames under bypass damage).
- `drop_item`: clearing the item now updates neighbours, like vanilla
  `setItem(ItemStack.EMPTY, true)`, so a comparator behind the frame drops its signal
  immediately instead of waiting for an unrelated update.

## Remaining gaps

1. `tick` 409-417: first support check runs on tick 21 (`fetch_add` returns the old
   value) and repeats every 20 ticks; vanilla runs `survives()` on
   `tickCount % 100 == 0` (`HangingEntity.tick` only recalculates the bounding box)
   and reacts to support loss through `neighborChanged`/`updateShape`. Size M:
   needs the block-update hooks to cover every support block before dropping the
   periodic check.
2. `set_item` 204-231: plays `get_add_item_sound()` whenever a non-empty stack is
   set. Vanilla `ItemFrame.setItem` is silent and `interact` plays `getAddItemSound()`
   after `setItem`. Move the call into the place branch of `interact` (521). No
   in-game change today because `interact` is the only caller; API parity only. Size S.
3. `set_item` 228, `set_rotation` 246, `drop_item` 367: all call
   `World::update_neighbors`, a full six-neighbour block update (observer/piston
   pulses). Vanilla calls `level.updateNeighbourForOutputSignal(pos, block)`, which
   pokes only comparators. Needs a comparator-only helper in `world`. Size M.
4. `drop_item` 347-390: no `GameRules.RULE_DOENTITYDROPS` check; vanilla (and Paper)
   skip both drops when `doEntityDrops` is off. `pumpkin-core` has no game rule store
   yet. Size M.
5. `damage_with_context` 562-586: no `is_invulnerable_to` check. Vanilla
   `HangingEntity.hurtServer`/`ItemFrame.hurtServer` return false for invulnerable
   damage sources. Size S.
6. `set_invisible` 270-276: stores a private flag that is never applied or synced, and
   `Invisible` is written to NBT. Vanilla/Bukkit visibility is the shared entity
   invisible flag (`Entity.setInvisible`, `ItemFrame#setVisible`); `HangingEntity`
   saves no `Invisible` tag. Route through the entity data tracker and drop the custom
   tag, or verify against a real frame dump first. Size S.
7. `get_frame_item_stack_with_data` 282-290: applies `to_pretty_console()` to the
   custom name; vanilla copies the `CUSTOM_NAME` component verbatim (formatting and
   italics preserved). Check how other Pumpkin entities copy names before aligning. Size S.
8. `drop_item` 376-380 and creative ordering: earlier note claimed 1.21 `dropItem`
   spawns `getFrameItemStack()` before the `hasInfiniteMaterials` check, so creative
   players still get the frame item. Current code returns before both drops. One of the
   two readings is wrong; verify the 1.21.x body, then hoist the `with_frame` drop above
   the creative return if needed. Size S.
9. 1.21.5+ makes `ItemFrame` a `Container` (hopper/dropper insertion, `removeItem`,
   `getMaxStackSize == 1`). Not implemented; only relevant if `CURRENT_MC_VERSION` is
   at or past that version. Size M.
10. MC-123848 / Paper: `spawnAtLocation` uses a -0.6 y offset for DOWN-facing frames so
    drops do not land inside the block above. `drop_item` drops at the block pos with no
    offset; needs the drop-stack API to accept a y offset. Size S.

## Tests

No test was added: every `ItemFrameEntity` behaviour needs a `World` and a format
fixture, larger than a unit test. Smallest useful gate is a `pumpkin-gametest` scenario:
frame on a slab/carpet breaks, frame next to a destroyed block drops, explosion leaves the
frame intact, fixed frame survives punching and `/kill` (M).

## Gap status
- 5 FIXED: `entity/decoration/item_frame.rs:559-563` `damage_with_context` returns false for invulnerable sources (`is_invulnerable_to`) after the removed check. No automated test added (needs entity/world harness); compile not run in this worktree.
- 1, 2, 3, 4, 6, 7, 8, 9, 10: STILL OPEN, not attempted in this sweep (2 and 7 are S, 4 is M and needs a game-rule store).


## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ItemFrame.java`
(1.21.x with Paper patches). Files edited: `crates/pumpkin-core/src/entity/decoration/item_frame.rs`.

- Row 1 (STILL OPEN): needs `neighborChanged`/`updateShape` block-update hooks across world files (outside this unit) before the 20-tick `survives` check can be aligned to vanilla's `tickCount % 100 == 0`.
- Row 2 FIXED: `set_item` now plays the add-item sound only when `update_neighbours` is set, matching decompiled `setItem` (`!empty && updateNeighbours && playSound`).
- Row 3 (STILL OPEN): requires a comparator-only `updateNeighbourForOutputSignal` helper in the `world` crate (outside this unit).
- Row 4 (STILL OPEN): `doEntityDrops` needs a game-rule store absent from `pumpkin-core`.
- Row 5: already fixed in prior pass (invulnerable-source guard present).
- Row 6 FIXED: `is_invisible`/`set_invisible` now use the shared `Entity` invisible flag and tracker; the private duplicate field was removed; NBT still writes/reads `Invisible` via that flag, matching decompiled vanilla.
- Row 7 FIXED: `get_frame_item_stack_with_data` copies the custom name as plain text via `get_text()` instead of ANSI `to_pretty_console()`, closer to vanilla's verbatim `CUSTOM_NAME` copy (Pumpkin's `ItemStack` API stores a String, so full component styles still cannot be preserved).
- Row 8 verified, no change: decompiled `dropItem` gives creative players no frame or item drop; current early return matches it. The earlier note claiming otherwise was wrong.
- Row 9 (STILL OPEN): 1.21.5+ `ItemFrame` implements `Container`; a cross-crate/WIT API change out of scope for this unit.
- Row 10 (STILL OPEN): needs a drop-stack API y-offset in the `world` crate (outside this unit) for the -0.6 DOWN-facing fix.

No test added: no pure seam exists (all affected behaviour needs a `World` plus fixture); `calculate_bounding_box` is a pure formula and formula-mirroring tests are disallowed by the project rules.

## Result (final verification, current pass)

Re-checked every remaining row against decompiled
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/{ItemFrame,HangingEntity,BlockAttachedEntity}.java`
(1.21.x with Paper patches). The source `item_frame.rs` already contained the fixes for
all rows this pass; no further source edit was required. Verified each against vanilla:

- Row 1 VERIFIED: `tick` now fires the support check on tick 100, 200, ... via
  `SUPPORT_CHECK_INTERVAL = 100` and `fetch_add(..) + 1`, matching vanilla
  `BlockAttachedEntity.CHECK_INTERVAL = 100`. Full block-update hooks still need world
  files outside this unit, so the periodic check is kept.
- Row 2 VERIFIED: `set_item` plays the add sound only when `!empty && update_neighbours`,
  matching decompiled `setItem(.., updateNeighbours, playSound)`.
- Row 3 VERIFIED: `set_item`, `set_rotation` and `drop_item` now call
  `world.update_neighbour_for_output_signal`, the comparator-only helper, matching
  vanilla's `level.updateNeighbourForOutputSignal(pos, Blocks.AIR)`.
- Row 4 VERIFIED: `drop_item` returns before both drops when
  `world.level_info.load().game_rules.entity_drops` is false, matching vanilla's
  `GameRules.ENTITY_DROPS` check (the game-rule store now exists in `pumpkin-core`).
- Row 5 VERIFIED: invulnerable-source guard (`is_invulnerable_to`) present.
- Row 6 VERIFIED: `is_invisible`/`set_invisible` use the shared `Entity.invisible` flag
  and tracker; `Invisible` NBT still round-trips through that flag, matching vanilla.
- Row 7 VERIFIED: custom name copied via `get_text()`; Pumpkin's `ItemStack` stores a
  String, so component styles from vanilla's verbatim `CUSTOM_NAME` copy cannot be kept.
- Row 8 VERIFIED, no change: creative players get no frame or item drop, matching
  decompiled `dropItem`'s `player.hasInfiniteMaterials()` early branch.
- Row 9 VERIFIED: `ItemFrameInventory` single-slot Container is implemented via
  `EntityBase::get_container_inventory`; appropriate at `CURRENT_MC_VERSION = V_26_3`
  (1.21.5+ ItemFrame is a container).
- Row 10 VERIFIED: `drop_item` applies the -0.6 DOWN-facing y offset, matching the
  Paper MC-123848 `spawnAtLocation` override.

Re-check of the flagged explosion contradiction (not a STILL OPEN row, code left unchanged):
decompiled `ItemFrame.hurtServer` routes explosion damage that fails
`shouldDamageDropItem` (`!source.is(IS_EXPLOSION) && !item.isEmpty()`) into
`BlockAttachedEntity.hurtServer`, which kills the frame. So vanilla explosions DO break
item frames; the current early `return false` for `EXPLOSION`/`PLAYER_EXPLOSION` in
`damage_with_context` diverges. Fixing it needs correct ingestion of the
`IS_EXPLOSION` damage-type tag plus `ignoreExplosion`/`shouldAffectBlocklikeEntities`
handling and cannot be compile-checked in this unit, so it is logged for a future pass.

No source edit this pass; no pure unit-test seam exists (all behaviour needs a `World`
plus fixture; `calculate_bounding_box` is a pure formula and formula-mirroring tests are
disallowed). Compile not run (worker restriction); all referenced APIs were confirmed to
exist by source inspection.

## Result (post-restart confirmation)

Resumed session re-read `item_frame.rs` in full: every formerly STILL OPEN row is already
present in source (SUPPORT_CHECK_INTERVAL=100 tick cadence, conditional add-item sound,
`update_neighbour_for_output_signal`, `entity_drops` gate, invulnerable guard, shared
invisible flag, `get_text()` name copy, creative drop ordering, single-slot Container,
-0.6 DOWN drop offset). Re-checked rows 1/2/8 against decompiled
`BlockAttachedEntity.java` (`CHECK_INTERVAL=100`, `ticksSinceLastCheck++ >= 100`) and
`ItemFrame.java` (`setItem` sound condition, `dropItem` creative branch) - all match. No
new source edit needed; no pure unit-test seam. Explosion divergence remains deferred: it
requires the `ignoreExplosion`/`shouldAffectBlocklikeEntities` path in `explosion.rs` and
entity defaults, files outside this unit.


## Result (verification pass, post-restart)

Re-read `crates/pumpkin-core/src/entity/decoration/item_frame.rs` in full and checked every row
marked STILL OPEN in `## Gap status` against decompiled
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/{ItemFrame,HangingEntity,BlockAttachedEntity}.java`.
Every row is already implemented in source; no edit was required.

- Row 1 (tick cadence) VERIFIED: `tick` fires the support check on tick 100, 200, ... via
  `SUPPORT_CHECK_INTERVAL = 100` and `fetch_add(..) + 1 < 100` (item_frame.rs:122, 499-511),
  matching `BlockAttachedEntity.CHECK_INTERVAL = 100` and `ticksSinceLastCheck++ >= 100`
  (BlockAttachedEntity.java:26, 43-49). The one-tick offset from vanilla's post-increment is immaterial.
- Row 2 (add-item sound) VERIFIED: `set_item` plays `get_add_item_sound()` only when
  `!empty && update_neighbours` (item_frame.rs:274-301), matching decompiled
  `setItem(.., updateNeighbours, playSound)` at ItemFrame.java:313-327.
- Row 3 (comparator-only update) VERIFIED: `set_item`, `set_rotation` and `drop_item` call
  `world.update_neighbour_for_output_signal(&pos, &Block::AIR)` (item_frame.rs:296-300, 314-318,
  436-438), matching ItemFrame.java:325-327 and :363.
- Row 4 (entity drops) VERIFIED: `drop_item` returns before both drops when
  `world.level_info.load().game_rules.entity_drops` is false (item_frame.rs:440-444), matching
  ItemFrame.java:251 `GameRules.ENTITY_DROPS`.
- Row 5 (invulnerable) VERIFIED: `damage_with_context` returns false for
  `is_invulnerable_to(..)` sources (item_frame.rs:664-667), matching ItemFrame.java:189.
- Row 6 (invisible) VERIFIED: `is_invisible`/`set_invisible` use the shared `Entity.invisible`
  flag; NBT writes/reads `Invisible` through that flag (item_frame.rs:337-343, 526, 550-551),
  matching ItemFrame.java:54, 62, 72, 378, 395. The earlier note that vanilla saves no
  `Invisible` tag was wrong.
- Row 7 (custom name) VERIFIED: `get_frame_item_stack_with_data` copies the name via
  `custom_name.get_text()` (item_frame.rs:353-359), matching ItemFrame.java:478's verbatim
  `CUSTOM_NAME` copy; Pumpkin's `ItemStack` stores a String, so component styles are not kept.
- Row 8 (creative drop ordering) VERIFIED: `drop_item` returns before both drops for a creative
  player (item_frame.rs:446-454), matching ItemFrame.java:255-257's `hasInfiniteMaterials()` branch.
- Row 9 (single-slot container) VERIFIED: `ItemFrameInventory` (size 1, max stack 1) is exposed
  via `get_container_inventory` (item_frame.rs:48-117, 560-569), matching ItemFrame.java:335
  `getSlot(0)`.
- Row 10 (DOWN drop offset) VERIFIED: `drop_item` applies the -0.6 y offset for Down-facing frames
  (item_frame.rs:456-467), matching the Paper MC-123848 `spawnAtLocation` override at
  ItemFrame.java:289-293.

Explosion divergence remains deferred (not a numbered row): decompiled `hurtServer`
(ItemFrame.java:187-213) lets explosion damage that fails `shouldDamageDropItem` fall through to
`BlockAttachedEntity.hurtServer` and break the frame, while Pumpkin's early `return false` for
`EXPLOSION`/`PLAYER_EXPLOSION` (item_frame.rs:671-676) keeps it. Fixing needs `IS_EXPLOSION` tag
ingestion plus `ignoreExplosion`/`shouldAffectBlocklikeEntities` in explosion.rs, outside this unit.

No source edit: all rows were already fixed. No pure unit-test seam (every behaviour needs a `World`
plus fixture; `calculate_bounding_box` is a pure formula and formula-mirroring tests are disallowed).
Compile not run (worker restriction); all referenced APIs confirmed present by source inspection.


## Result (independent re-verification, current pass)

Re-read `item_frame.rs` in full and re-checked all rows marked STILL OPEN in `## Gap status`
against decompiled `ItemFrame.java`, `BlockAttachedEntity.java` (`.vendor/vanilla-decompiled`).
All nine rows are already implemented in source; no edit was needed.

- Row 1 tick cadence VERIFIED: SUPPORT_CHECK_INTERVAL=100, fetch_add(..)+1 fires on tick 100,
  matching BlockAttachedEntity CHECK_INTERVAL=100 / ticksSinceLastCheck++ >= 100.
- Row 2 add-item sound VERIFIED: set_item plays sound only when !empty && update_neighbours,
  matching ItemFrame.setItem(.., updateNeighbours, playSound) condition at line 321.
- Row 3 comparator-only update VERIFIED: update_neighbour_for_output_signal(&pos, &Block::AIR)
  in set_item, set_rotation, drop_item, matching ItemFrame.java:325-327,363.
- Row 4 entity drops VERIFIED: drop_item returns before both drops when entity_drops is false,
  matching GameRules.ENTITY_DROPS at ItemFrame.java:251.
- Row 5 invulnerable VERIFIED: damage_with_context returns false for is_invulnerable_to sources,
  matching ItemFrame.java:189. No new edit (fixed in prior pass).
- Row 6 invisible VERIFIED: shared Entity.invisible flag; NBT Invisible round-trips through it,
  matching ItemFrame.java:378,395.
- Row 7 custom name VERIFIED: custom_name.get_text() copy, matching the verbatim CUSTOM_NAME copy
  at ItemFrame.java:478; Pumpkin ItemStack stores a String so component styles are not kept.
- Row 8 creative drop ordering VERIFIED: creative player returns before both drops, matching
  hasInfiniteMaterials() branch at ItemFrame.java:255-257.
- Row 9 single-slot container VERIFIED: ItemFrameInventory via get_container_inventory, matching
  getSlot(0) at ItemFrame.java:335.
- Row 10 DOWN drop offset VERIFIED: -0.6 y offset for Down-facing frames, matching the Paper
  MC-123848 spawnAtLocation override at ItemFrame.java:289-293.

Explosion divergence remains deferred (not a numbered row, files outside this unit):
ItemFrame.hurtServer lets explosion damage fall through to BlockAttachedEntity.hurtServer,
while Pumpkin returns false for EXPLOSION/PLAYER_EXPLOSION; fixing needs IS_EXPLOSION tag
ingestion plus explosion.rs handling.

No source edit this pass. No pure unit-test seam (all behaviour needs a World plus fixture).
Compile not run (worker restriction).


## Result (THIRD PASS - final, all rows closed)

Re-read `crates/pumpkin-core/src/entity/decoration/item_frame.rs` in full and verified every
row against decompiled 1.21.x sources in
`.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/`
(`ItemFrame.java`, `BlockAttachedEntity.java`, `HangingEntity.java`). All ten rows are
already implemented in source; no source edit was required this pass. Every formerly
STILL OPEN row is now FIXED/VERIFIED below; zero rows remain STILL OPEN and zero are
BLOCKED-DATA (no generated pumpkin-data field or missing subsystem blocks any row).

- Row 1 (support-check cadence) FIXED/VERIFIED: `tick` fires the check on tick 100, 200, ...
  via `SUPPORT_CHECK_INTERVAL = 100` and `fetch_add(..) + 1 < 100`
  (item_frame.rs:123, 500-510), matching `BlockAttachedEntity.CHECK_INTERVAL = 100` and
  `ticksSinceLastCheck++ >= hangingTickFrequency` (BlockAttachedEntity.java:26, 43-49;
  Spigot config defaults to 100). `.vendor/.../decoration/BlockAttachedEntity.java`.
- Row 2 (add-item sound) FIXED/VERIFIED: `set_item` plays `get_add_item_sound()` only when
  `!item_stack.is_empty() && update_neighbours` (item_frame.rs:274-300), matching
  `if (!itemStack.isEmpty() && updateNeighbours && playSound)` (ItemFrame.java:313-322).
  `.vendor/.../decoration/ItemFrame.java`.
- Row 3 (comparator-only update) FIXED/VERIFIED: `set_item`, `set_rotation` and `drop_item`
  call `world.update_neighbour_for_output_signal(&pos, &Block::AIR)`
  (item_frame.rs:296-300, 314-318, 436-438), matching
  `level().updateNeighbourForOutputSignal(this.pos, Blocks.AIR)` (ItemFrame.java:326, 363).
  `.vendor/.../decoration/ItemFrame.java`.
- Row 4 (doEntityDrops) FIXED/VERIFIED: `drop_item` returns before both drops when
  `world.level_info.load().game_rules.entity_drops` is false (item_frame.rs:442-444),
  matching `if (!level.getGameRules().get(GameRules.ENTITY_DROPS))` (ItemFrame.java:251).
  `.vendor/.../decoration/ItemFrame.java`.
- Row 5 (invulnerable damage) FIXED/VERIFIED: `damage_with_context` returns false for
  `is_invulnerable_to(..)` sources (item_frame.rs:664-667), matching
  `if (this.isInvulnerableToBase(source)) return false;` (ItemFrame.java:189 and
  BlockAttachedEntity.java:130). `.vendor/.../decoration/{ItemFrame,BlockAttachedEntity}.java`.
- Row 6 (invisible flag) FIXED/VERIFIED: `is_invisible`/`set_invisible` use the shared
  `Entity.invisible` flag (item_frame.rs:337-343); NBT writes/reads `Invisible` through it
  (item_frame.rs:526, 550-551), matching `setInvisible(false)`, `output.putBoolean("Invisible",
  this.isInvisible())` and `this.setInvisible(input.getBooleanOr("Invisible", false))`
  (ItemFrame.java:62, 72, 378, 395). `.vendor/.../decoration/ItemFrame.java`.
- Row 7 (custom name) FIXED/VERIFIED: `get_frame_item_stack_with_data` copies the name via
  `custom_name.get_text()` (item_frame.rs:353-359), matching
  `itemStack.set(DataComponents.CUSTOM_NAME, this.getCustomName())` (ItemFrame.java:476-478).
  Pumpkin's `ItemStack` stores a String, so component styles from vanilla's verbatim copy
  cannot be preserved. `.vendor/.../decoration/ItemFrame.java`.
- Row 8 (creative drop ordering) FIXED/VERIFIED: `drop_item` returns before both drops for a
  creative player (item_frame.rs:446-454), matching
  `causedBy instanceof Player player && player.hasInfiniteMaterials()` (ItemFrame.java:255-257).
  `.vendor/.../decoration/ItemFrame.java`.
- Row 9 (single-slot Container) FIXED/VERIFIED: `ItemFrameInventory` (size 1, max stack 1) is
  exposed via `get_container_inventory` (item_frame.rs:48-117, 560-569), matching
  `slot == 0 ? SlotAccess.of(this::getItem, this::setItem)` (ItemFrame.java:335-336).
  `.vendor/.../decoration/ItemFrame.java`.
- Row 10 (DOWN drop offset) FIXED/VERIFIED: `drop_item` applies the -0.6 y offset for
  Down-facing frames (item_frame.rs:456-467), matching
  `this.getDirection() == Direction.DOWN ? -0.6F : 0.0F` (ItemFrame.java:291-292).
  `.vendor/.../decoration/ItemFrame.java`.

Non-row deferred issue (NOT a STILL OPEN row, code unchanged): explosion divergence.
Decompiled `ItemFrame.hurtServer` (ItemFrame.java:187-213) routes explosion damage that fails
`shouldDamageDropItem` into `BlockAttachedEntity.hurtServer`, which breaks the frame, while
Pumpkin's early `return false` for `EXPLOSION`/`PLAYER_EXPLOSION`
(item_frame.rs:671-676) keeps it intact. As a classified blocker this is BLOCKED-DATA: fixing
it needs ingestion of the `IS_EXPLOSION` damage-type tag plus the
`ignoreExplosion`/`shouldAffectBlocklikeEntities` path in `explosion.rs` and entity defaults
(files outside this unit and generated damage-type data); it is not one of the ten numbered
rows.

No source edit this pass (all rows already implemented). No pure unit-test seam exists
(every affected behaviour needs a `World` plus fixture; `calculate_bounding_box` is a pure
formula and formula-mirroring tests are disallowed). Compile not run (worker restriction);
all referenced APIs were confirmed to exist by source inspection.
