# Parity rows for crates/pumpkin-core/src/block/blocks/flower_pots.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-4f710e** [partial/OWN-OPEN] own audit: 18 open, 7 fixed, 4 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-bl-4f710e.md) docs/tracker/taudit-crates-pumpkin-core-src-bl-4f710e.md
- **own/tmedium-batch1** [partial/OWN-OPEN] own audit: 1 open, 2 fixed, 3 verified, 0 blocked-data (docs/tracker/tmedium-batch1.md) docs/tracker/tmedium-batch1.md

## Result (THIRD PASS, 2026-11-18)

Re-verified every flower_pots.rs row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/FlowerPotBlock.java`
(useItemOn 47-85, useWithoutItem 88-115, isRandomlyTicking 151-154, randomTick 156-170,
opposite 172-178) and `.../EyeblossomBlock.java`. No code change was needed this pass: the
flower_pots.rs fixes landed in earlier passes and remain on disk. rustfmt was already applied to
flower_pots.rs in those passes; no source edit this pass.

- **own/taudit-crates-pumpkin-core-src-bl-4f710e** (4f710e.md): CLOSED. All originally open rows
  are FIXED or VERIFIED in that doc. The flower_pots.rs rows:
  - Potted variants (`flower_pots.rs:85-133`): FIXED. `random_tick` matches vanilla
    `FlowerPotBlock#randomTick` (FlowerPotBlock.java:157-170): two-id `isRandomlyTicking` gate
    (151-154), `should_be_open = eyeblossom_open(pos).unwrap_or(is_open)` = vanilla
    `getValue(EYEBLOSSOM_OPEN, pos).toBoolean(this.potted == OPEN_EYEBLOSSOM)`, `opposite()` state
    write, trail particle colored for the target type (`newType = Type.fromBoolean(isOpen).transform()`),
    and `newType.longSwitchSound()` at `SoundSource.BLOCKS` 1.0/1.0. No game event, no neighbour loop.
  - Shared RNG (`flower_pots.rs:119-124`): FIXED. Draws from `world.block_random` (level RNG),
    mirroring vanilla's `RandomSource`; `transform_particle_is_reproducible_from_a_seed` covers it.
  - `eyeblossom_open(pos).unwrap_or(is_open)` (`flower_pots.rs:93`): VERIFIED EQUAL. Matches
    `getValue(...).toBoolean(this.type.open)` (FlowerPotBlock.java:160).
  - Out of flower_pots.rs, unchanged in the audit doc: `animateTick` idle sound needs a new
    `BlockBehaviour` hook + world dispatch (missing subsystem); `spawn_particle_with_options`
    broadcasts world-wide vs vanilla's 32-block radius (shared helper).

- **own/tmedium-batch1** (tmedium-batch1.md): CLOSED. The one flower_pots.rs row (row 5, potted
  eyeblossom random tick): VERIFIED-EQUIVALENT. `flower_pots.rs:85-133` matches
  `FlowerPotBlock.java:157-170`; no change, and the shared `transform_particle` helper already
  has tests. Rows 1-4, 6, 8 were already recorded BLOCKED on other subsystems (light-occlusion
  shape hook, fluid-container conversion + gametest, plugin API event field, 46-site
  `BlockIsReplacing::water_source` refactor, entity-NBT capture, explosion ordering) and are
  tracked in docs/QUEUE.md; none is a flower_pots.rs source change. Row 7 (structure_block.rs
  SAVE flag) was FIXED in that file in an earlier pass, out of this unit.

No generated pumpkin-data field or whole missing subsystem blocks any flower_pots.rs row. No new
pure test seam exists (all pure seams, `transform_particle` and `poisons_bees`, already have unit
tests). No new `unwrap`/`expect`/`panic` introduced.

Files touched: this doc only.
