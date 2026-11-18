# Native agent 3 summary

Status: UNVERIFIED (no code changes, no compile).

Work list gap: `/tmp/kabocha-runs/native-3.md` contains no row list; it is a pointer stub. The 28 rows were reconstructed from `LEFT OPEN` / `STILL OPEN` markers in `docs/tracker/tblocks-*.md` latest Result sections.

Reconstructed rows and decisions:
- tblocks-chests row 3 (lid event count u8): NOT A GAP. Wire is `writeByte` in vanilla `ClientboundBlockEventPacket.java:37-38`. Recorded in `docs/tracker/tblocks-chests.md`.
- tblocks-creaking-heart rows 4-6 (creaking goals, hurt particles, piston push reaction): BLOCKED. Push reaction needs generated `pumpkin-data` Block field; creaking goals need a Brain/Behaviour system (`CreakingAi.java:30-85`) Pumpkin lacks; particles are client-only.
- tblocks-beds rows 39-42 and bed.rs rule rows: BLOCKED-DATA. `BedRule` fields live in `crates/pumpkin-data/src/generated/dimension.rs`, which must not be hand-edited.
- tblocks-dispenser gap 3 (feature flags): BLOCKED-DATA. Needs item feature-set data from Extractor.
- tblocks-jukebox #1 (dynamic jukebox_song registry): BLOCKED. Needs cross-crate registry access at four call sites.
- tblocks-structure-block rows (entity capture, collision palette split, strict placement, empty palette with entities): LEFT OPEN. Multi-file structure-template port, too large for an uncompiled pass.
- tblocks-lodestone rows 1, 3, 6 (compass.rs): out of unit, left open.
- tblocks-vault gaps 3, 6 (display cycle, ominous activation): client visual or unported event; left open.
- tblocks-beacon rows 36, 38, 39: client beam rendering and world handle for sound; left open.
- tblocks-campfire row 3 (cooked result split): latent, left open.
- tblocks-fire gap 3 (TNT before priming, deliberate ordering): left open.
- tblocks-chiseled-bookshelf rows 4-6: ordering, block-entity stack read, gametest harness; left open.
- tblocks-daylight-detector-b4adbb rows 1 and 3 and tblocks-daylight-detector 6px occlusion (L): left open.
- tblocks-sculk-sensor row 1 (occlusion test): reported as STILL OPEN in its note; not re-verified this pass.
- tblocks-pistons row 4 (players excluded from push): intentional divergence, left open.
- tblocks-note-block row 1: premise false per its own note, NOT A GAP.

Not done: the rest of the 28 cross-file fixes. A single compile is needed to land any of them safely, and the list could not be recovered from disk.

## Second pass against docs/investigations/work/native-3.md (31-line list)

Status: UNVERIFIED. No new source edits in this pass. Not complete: most of the list is still open.

Row decisions:
- tblocks-chests row 3 (lid viewer count u8): NOT A GAP. Vanilla `ClientboundBlockEventPacket.java:37-38` writes `writeByte`, matching Pumpkin `CBlockEvent`. Evidence in `docs/tracker/tblocks-chests.md`.
- tblocks-campfire row 12 (isSmokeyPos): already FIXED per tracker note; not re-verified here.
- tblocks-campfire row 3 (cooked result split): NOT DONE. Latent; needs the campfire cooking path read.
- taudit-bl-4d09af `BlockIsReplacing::water_source` / 46 call sites: NOT DONE. Source is `block/registry.rs:662-683` (`BlockIsReplacing::Water`/`Other`/`Itself`); the 46-site audit was not started.
- taudit-bl-4d09af shelf collision/occlusion shape: NOT DONE. `light_occlusion` grep in `crates/pumpkin-data/src` found nothing, so this may need generated data. Needs confirmation.
- taudit-bl-4d09af shelf piston push reaction: NOT DONE. Depends on a push-reaction field that creaking row 6 says is absent from generated `pumpkin-data`.
- taudit-bl-6f9a28 row 9 (multiblock bucketing): NOT DONE. Vanilla `addToLists` ordering needs the template path.
- taudit-bl-4f710e row 5 (eyeblossom open fallback): already CLOSED per list.
- tblocks-daylight-detector shape-based occlusion (6px column): NOT DONE. Same missing occlusion-shape data as the shelf row.
- tblocks-pistons row 4 (players excluded): intentional divergence, kept.
- tblocks-beds rows 1 and 3: BLOCKED-DATA. `BedRule` lives in generated `pumpkin-data/src/generated/dimension.rs`, which must not be hand-edited.
- tblocks-beacon `BeaconBeamOwner.Section`: BLOCKED. Client-side beam rendering.
- taudit-en-f12c04 row 21: NOT DONE. Needs `OnEntityCollisionArgs` extension (block/mod.rs:419) and dedupe with `Entity::tick_block_collisions`. Row 22 already fixed per its note.
- taudit-en-06afb0 row 7 (`bypasses_armor`): BLOCKED-DATA, generated damage-type tag.
- taudit-block-regi: BLOCKED for the one open row (out-of-unit, vanilla default).
- taudit-entity-veh block_effects 1-6: NOT DONE, out-of-scope files.
- tentities-bee row 3 (pitcher crop two-block plants): NOT DONE.
- tentities-tadpole: BLOCKED-DATA gap 1.
- titems-mace row 16 (gametest): NOT DONE; fixture outside the unit's editable scope.
- tmedium-batch1 row 1 (shape light occlusion): BLOCKED, needs `useShapeForLightOcclusion` data.
- tmedium-batch1 row 2 (source water to LiquidBlockContainer): NOT DONE, in fluid spreading.
- tmedium-batch1 row 4 (BlockIsReplacing::water_source): NOT DONE (same as above).
- tmedium-batch1 row 6 (entity capture on save): NOT DONE, needs entity NBT capture in the template path.
- tmedium-batch1 row 8 (onExplosionHit ordering): NOT DONE, needs an explosion world test.

Totals: NOT A GAP 1, already FIXED/CLOSED 2, BLOCKED 6, NOT DONE 15, intentional 1.

Honest gap: this pass implemented no rows. The remaining rows are multi-file changes that need a compile and the 46-site audit. Next step: one worker per row group, with compile.

## Third pass (code)

Status: UNVERIFIED (one Rust file changed, not compiled).

- Entity collision (taudit-en-f12c04 row 21): FIXED. `crates/pumpkin-core/src/entity/mod.rs` `tick_block_collisions` now calls `on_entity_collision` for every non-air block the deflated box touches, as vanilla `Entity.checkInsideBlocks` does (no collision-shape gate). The powder snow flag stays gated on intersection. `OnEntityCollisionArgs` (block/mod.rs:419) already carries everything vanilla `entityInside` needs, so the struct is unchanged. Every implementor (35 files under `block/`) already receives the same args. Regression check: not added (needs a world with an entity box; no pure seam).
- Campfire cooked count (tblocks-campfire row 3): NOT A GAP. Vanilla `CampfireBlockEntity.java:100` uses `recipe.assemble(input)` (count = recipe result count), `orElse(itemStack)` with the raw stack. Pumpkin `campfire.rs:151-162` matches both.
- Structure block entity capture (tblocks-structure-block row 'entity capture'): ALREADY FIXED. `structure_block.rs:305-342` already fills `entity_info_list` from live entities, excluding players, as vanilla `fillEntityList`.
- Structure block collision palette split: ALREADY FIXED. `structure_block.rs:287-293` buckets block-entity, full-cube non-dynamic, and other blocks as `addToLists` does (`has_dynamic_shape` at 199-226).
- BlockIsReplacing water_source audit: ALREADY FIXED. `block/mod.rs:787-812` already splits `is_water` (source only), `is_any_water`, `is_full_water`. Call sites found by grep: signs.rs:203, ladder.rs:33, light.rs:18, leaves.rs:59, dynamic.rs:29 (is_water); small_dripleaf.rs:72 (is_any_water); conduit.rs:18, coral_plant.rs:31, coral_fan.rs:55,88,96 (is_full_water). No edit.
- Explosion ordering (taudit-block-regi / tmedium row 8): ALREADY FIXED. `world/explosion.rs:935-936` runs `explode` before removal, as vanilla `interactWithBlocks` runs `onExplosionHit` before the stack pop (`ServerExplosion.java:664-668`). Entity `onExplosionHit` is called per entity after damage (`explosion.rs:724`, vanilla 563).
- Source water into LiquidBlockContainer (tmedium row 2): covered by `block/fluid/flowing_trait.rs:465-477` for waterloggable containers. Remaining gap (non-waterloggable containers) needs the exact row text; not changed.
- Piston push reaction (shelves, creaking row 6): BLOCKED-DATA. `grep PushReaction|push_reaction` finds nothing in `crates/pumpkin-data/src` or `crates/pumpkin-core/src`.
- Shelf collision/occlusion shape: BLOCKED-DATA. Shape data is generated; not hand-editable.
- Bee pitcher crop row and mace gametest: skipped, no pure seam.
- Structure block palette, daylight/sculk occlusion: BLOCKED-DATA (shape occlusion data is generated).

Rust files changed in this pass: 1 (`crates/pumpkin-core/src/entity/mod.rs`).
