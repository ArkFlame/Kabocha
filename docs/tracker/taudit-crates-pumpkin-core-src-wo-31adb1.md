# Audit: crates/pumpkin-core/src/world/explosion.rs

Tracker row: category=audit, coverage=AUDIT. Vanilla reference: `net.minecraft.world.level.ServerExplosion`
(1.21.x) plus `Explosion#getSeenPercent`, `ExplosionDamageCalculator`, `SimpleExplosionDamageCalculator`,
`Explosion#finalizeExplosion`. Decompiled sources are not vendored in this worktree (`.vendor/` absent), so
items marked "needs vanilla source" are unconfirmed from memory only.

## Fixed in this pass

- `explode_with_fire_roll` (destroy path, ~line 820): block loot parameters omitted `this_entity`.
  Vanilla `Explosion#finalizeExplosion` calls `Block.dropResources(state, level, pos, blockEntity,
  getIndirectSourceEntity(), ItemStack.EMPTY)`, so `LootContextParams.THIS_ENTITY` is the credited entity.
  Fixed by setting `this_entity: self.indirect_source()`; loot tables/conditions reading `this_entity` now see
  the same entity as vanilla. No automated test: reproducing needs a custom block loot table (datapack fixture).

## Remaining gaps (larger or unconfirmed)

1. `damage_entities`, lines 596-598: `if exposure == 0.0 { continue; }` skips damage entirely for a fully
   occluded entity. Vanilla `ServerExplosion#hurtEntities` computes
   `damage = ((d*d + d)/2)*7*radius + 1.0` with `d = (1 - dist/radius) * exposure`, so `exposure == 0`
   still yields exactly 1.0 damage (unit of damage, i.e. the "+1.0" term is unconditional); only the
   knockback term becomes zero. Suggested fix: drop the early `continue`, keep computing/adding damage, and
   let the existing zero knockback be applied. Size S. Needs a peek at decompiled `hurtEntities` before
   landing because it changes observable "explosions leak 1 damage through full cover" behaviour.

2. `explode_with_fire_roll`, lines 781-799 and 801-838 (Paper parity): `BlockExplodeEvent` is created with only
   `block_pos`/`yield_rate`/`world`. Paper's `com.destroystokyo.paper.event.block.BlockExplodeEvent` exposes the
   mutable affected `blockList()` and applies `yield_rate` per block (`random.nextFloat() <= yield`), and the
   server destroys exactly the surviving list. Pumpkin ignores both, destroying every collected block and
   ignoring the yield. Suggested fix: add `blocks: Vec<BlockPos>` to the event (additive on the Rust side, but a
   plugin API surface change -> bump `PLUGIN_API_VERSION` / WIT host pair), filter the destroyed set by the
   (possibly plugin-modified) list, and apply the yield before destruction. Size M.

3. `explode`, lines 742-749: no `GameEvent.EXPLOSION` is emitted. Vanilla `ServerExplosion#explode` calls
   `this.level.gameEvent(source, GameEvent.EXPLOSION, this.center)`, which drives sculk sensors, wardens and
   allays. Suggested fix: fire the world game event with the credited entity and centre position. Size S-M
   (depends on the game-event plumbing available in `World`).

4. `get_blocks_to_destroy`, lines 453-470: unloaded chunks are treated as air
   (`read_chunk_sync` returning `None` -> `Block::AIR`), so blocks in unloaded chunks survive the blast.
   Vanilla `Level#getBlockState` loads/generates the chunk. Pumpkin's sync-read-only policy is deliberate
   (never block the tick on chunk gen), so this is a documented divergence, not a one-line fix. Size M
   (needs an async pre-load of the 3x3 chunks around the centre before the raycast).

5. `get_blocks_to_destroy` line 441 and `explode` line 744: randomness comes from `rand::random` / `rand::rng()`
   instead of `world`'s RNG. Vanilla uses `level.random`, which makes explosions reproducible per level seed and
   is what `/seed`-based differential tests would rely on; Paper only relaxes this for hot paths. Suggested fix:
   thread the world RNG through `Explosion` (also lets tests keep the existing `fire_roll` hook). Size S.

6. `damage_entities`, lines 618-630: vanilla wraps the whole per-entity block in `if (d5 != 0.0)` where `d5` is
   the length of the vector entity->centre, i.e. an entity standing exactly at the explosion centre gets neither
   damage nor knockback. Pumpkin normalises a zero vector to `(0,0,0)` (no NaN, verified in
   `pumpkin-util/src/math/vector3.rs:426`) and still applies damage. Degenerate case only; suggested fix is a
   `_` length check before damage. Size XS, unconfirmed from memory.

7. `should_create_fire`, line 116: uses the position-less `BlockState::is_solid_render()`. Vanilla 1.20.5+
   `ServerExplosion` calls `state.isSolidRender(level, pos)` (position-aware), so blocks whose solidity depends
   on neighbours could differ. Size XS but needs vanilla source; no known affected block.

## Verified equivalent (no change)

- Ray set: `x/y/z > 0 && < 15` skip == vanilla's `i == 0 || i == 15 || ...` filter, 1352 rays; direction
  `i / 7.5 - 1.0`; initial `h = power * (0.7 + rand * 0.6)`; resistance `h -= (r + 0.3) * 0.3`; step 0.3 and
  `h -= 0.22500001`; `is_in_build_limit` break before the state fetch.
- Fire: `affected_positions` is inserted before the resistance subtraction, which is a superset of vanilla's
  `getToBlow` fire set, but every extra position is a surviving solid block that fails the `is_air` check, so
  the fire-eligible air positions are identical. Roll `rand(0..3) == 0` and the air/solid-render support checks
  match `ServerExplosion`'s fire loop.
- `calculate_exposure` == `Explosion#getSeenPercent` (offsets, lerp grid, `COLLIDER`/`Fluid.NONE` clip, ratio).
- `get_entity_damage_amount` == `ExplosionDamageCalculator#getEntityDamageAmount`
  (`midpoint(d*d, d) * 7 * radius + 1.0`, `radius = power * 2`).
- `SimpleExplosionDamageCalculator` immune tag -> `3600000.0`, air+empty fluid -> `None`, knockback multiplier
  default 1.0; creative-flying players get no knockback; TNT uses its position rather than eye position;
  knockback resistance attribute; destroy order drop -> set air -> `block.onBlockExploded`.
