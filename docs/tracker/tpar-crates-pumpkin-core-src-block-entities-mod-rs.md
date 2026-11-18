# Parity rows for crates/pumpkin-core/src/block/entities/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-bl-eb0fa6** [VERIFIED] own audit: all rows fixed/verified against .vendor/vanilla-decompiled (docs/tracker/taudit-crates-pumpkin-core-src-bl-eb0fa6.md)

## Result (final pass 3)

Verified every audit sub-row against the real `.vendor/vanilla-decompiled` tree (it is present).
No source file needed editing: all fixes were already in the tree from earlier passes.

- Row `own/taudit-crates-pumpkin-core-src-bl-eb0fa6` (own audit of `block/entities/mod.rs`): VERIFIED.
  - `mod.rs:571-590` (`create_block_entity_for_state`, moving-piston facing): FIXED, VERIFIED.
    The `"piston" if block == &Block::MOVING_PISTON` arm (`mod.rs:598-614`) reads
    `MovingPistonLikeProperties::from_state_id(state_id).facing.to_block_direction()` and
    builds a fresh `PistonBlockEntity` (progress 0, `extending: false`, `source: false`,
    pushed state `Block::AIR.default_state`). Matches vanilla
    `.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/MovingPistonBlock.java:46-60`
    (`newBlockEntity` returns null, `newMovingBlockEntity` passes facing explicitly) and
    `PistonMovingBlockEntity.java:38-75` (2-arg ctor leaves progress 0, extending/source false,
    movedState air, direction DOWN; 6-arg ctor overwrites direction). State-derived facing is
    the correct load/restore fallback. Unit test `moving_piston_state_supplies_the_facing`
    (`mod.rs:867`) covers it.
  - `v1_18.rs:166-175` (chunk block entity registry id): FIXED, VERIFIED.
    `block_entity_type_id` (`v1_18.rs:309-315`) resolves the unqualified NBT `id` against
    `BLOCK_ENTITY_TYPES` (generated `pumpkin-data/src/generated/block.rs:6469` uses bare names:
    `"furnace"`, `"piston"`, no `"bed"`); the writer filters unknown ids before writing the
    count (`v1_18.rs:154-171`) and `unknown_block_entity_id_is_dropped` (`v1_18.rs:328-334`)
    covers the Pumpkin-only `bed`/`map` case.
  - Pass-3 `piston.rs` parity fixes re-checked, no edit: `facing_from_data_value` (`piston.rs:57-66`)
    matches vanilla `Direction.from3DDataValue = BY_3D_DATA[abs(data % 6)]`
    (`.vendor/vanilla-decompiled/net/minecraft/core/Direction.java:341-342`); `get_numeric_float`/
    `get_numeric_short` (`piston.rs:365-367`) match vanilla `getFloatOr`/`getBooleanOr`;
    `drag_stuck_entities` (`piston.rs:70`, called at :351) and slime-block bounce in `push_entities`
    are present. Tests `facing_id_wraps_like_vanilla`, `pushed_block_state_survives_nbt_round_trip`
    are present (`piston.rs:436,417`).
  - `chest_like_block_entity.rs`: no formal row. `is_empty` cannot unpack a pending loot table
    without a world handle (vanilla `RandomizableContainerBlockEntity.isEmpty` calls
    `unpackLootTable(null)`, which no-ops without a level); hoppers already call `take_loot_table`.
    VERIFIED not a gap; no edit.
- All rows closed. No `.rs` file modified, so rustfmt was not run.
