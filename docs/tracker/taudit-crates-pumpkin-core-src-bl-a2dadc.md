# Tracker: audit / crates/pumpkin-core/src/block/entities/sculk_catalyst.rs

Sources reviewed: `crates/pumpkin-core/src/block/entities/sculk_catalyst.rs`,
`crates/pumpkin-core/src/block/blocks/sculk/sculk_catalyst.rs`,
`crates/pumpkin-core/src/entity/living.rs:3009-3019`,
`crates/pumpkin-world/src/generation/feature/features/sculk/spreader.rs`.
There are no TODO/FIXME markers in the file; the five `vanilla` markers are
documentation cross-references (lines 18, 21, 95, 108, 128).

## Fixed in this pass

| Gap | Rust | Vanilla | Fix | Size |
|---|---|---|---|---|
| Bloom sound fired at bloom start (state set) at default volume/pitch | `block/entities/sculk_catalyst.rs:125` (old) | `SculkCatalystBlock.tick`: the sound plays when the bloom animation ends, `SoundEvents.SCULK_CATALYST_BLOOM`, volume 2.0, pitch `0.6 + random * 0.4` | sound moved to `SculkCatalystBlock::on_scheduled_tick` with volume 2.0 and a random pitch, removed from `bloom()` | S |
| Soul charge used the killer's weapon: Looting inflated it | `entity/living.rs:3013` (old) | `CatalystListener.handleGameEvent` calls `getExperienceReward(level, null)`, so the charge is the base reward | pass `None` instead of `killer` | XS |

## Remaining gaps (larger)

| Gap | Rust | Vanilla | Suggested fix | Size |
|---|---|---|---|---|
| Charge cursors are never persisted: saving/reloading a chunk loses queued sculk charge | `block/entities/sculk_catalyst.rs:67-77`; `SculkSpreader` (`spreader.rs`) has no NBT API and `ChargeCursor` fields are private | `SculkCatalystBlockEntity.saveAdditional` writes `cursors` (ListTag of pos/charge/update_delay/decay_delay/facings) and `load` reads it through `SculkSpreader.save/load` | add `ChargeCursor::to_nbt/from_nbt` plus `SculkSpreader::save/load`, wire into this entity write_nbt/from_nbt/chunk_data_nbt | M |
| Phantom `decay_delay` block-entity field and NBT key: written, never read; vanilla keeps `decay_delay` per cursor entry, not on the block entity | `block/entities/sculk_catalyst.rs:28, 46-53, 67-77` | `SculkCatalystBlockEntity` stores only `cursors` | remove the field and key together with the cursor persistence work (`write_nbt` is a required trait method, so it must keep writing `cursors` afterwards) | S |
| Hearing test uses integer block distance instead of the event position distance | `block/entities/sculk_catalyst.rs:156-159` | game-event dispatch compares the event `Vec3` with the listener pos; a death at fractional coordinates 8.5 blocks away is heard in vanilla but missed here, and diagonal offsets differ | compare squared distance from the entity position to the block centre, or pass the entity position through | M |
| No `wasExperienceConsumed()` guard: a living entity whose XP was already consumed still feeds the catalyst | `entity/living.rs:3012` | `CatalystListener.handleGameEvent` requires `!livingEntity.wasExperienceConsumed()` before adding cursors | track the flag on the living entity and gate the call | S |
| Bloom reset delay may not match vanilla | `block/entities/sculk_catalyst.rs:23, 124` | `SculkCatalystBlock` schedules the reset tick with its own delay constant | check the decompiled constant and sample it if it is an IntProvider instead of the fixed `BLOOM_DURATION = 20` | S |
| No game-event listener integration: the catalyst is fed from `LivingEntity::on_death`, so entity removals that skip that path never bloom it | `block/entities/sculk_catalyst.rs:130-170`, caller `entity/living.rs:3012-3018` | `SculkCatalystBlockEntity` implements `GameEventListener` for `GameEvent.ENTITY_DIE` | implement `game_event_radius`/`on_game_event` once `emit_game_event` carries an entity context (charge needs the source entity's XP) | M |

## Verified matching vanilla (no change)

- `tick` runs `spreader.update_cursors(world, pos, random, true)` every tick,
  matching `SculkCatalystBlockEntity.serverTick` (spread veins enabled).
- Hearing radius 8 and the block-entity downcast before `add_soul`
  (`HEARING_RADIUS`, lines 20, 134-169) match the listener radius.
- Bloom sets the state once and schedules a block tick; the state is cleared in
  `SculkCatalystBlock::on_scheduled_tick` (lines 23-48).
- `add_soul` drops non-positive charges, mirroring the `i > 0` guard.
- `WorldSculkLevel` air/water/waterlogged/side-solid/full-cube queries match
  `SculkLevel`.
- `SculkSpreader::new_level_spreader` uses `(10, 4, 10, 5)`, identical to
  `SculkSpreader.createLevelSpreader`, and `add_cursors` splits charge at 1000
  per cursor like `SculkSpreader.addCursors`.

