# Audit: crates/pumpkin-core/src/world/explosion.rs

Tracker row: category=audit, coverage=AUDIT. Vanilla reference: `net.minecraft.world.level.ServerExplosion`
(1.21.x) plus `Explosion#getSeenPercent`, `ExplosionDamageCalculator`, `SimpleExplosionDamageCalculator`,
`Explosion#finalizeExplosion`. Decompiled sources are not vendored in this worktree (`.vendor/` absent), so
items marked "needs vanilla source" are unconfirmed from memory only.

TODO/FIXME/XXX/HACK grep: 2 "vanilla" comment hits only (`:120`, `:789`), no actionable marker.

## Fixed in this pass

- `place_fire` (`~line 393`): the write condition compared the state id to `Block::AIR`, so fire was never
  placed into cave air or void air although `BlockState::is_air()` (and vanilla `BlockState.isAir()`) accepts
  both. Now checks `BlockState::from_id(state_id).is_air()`; the existing Keep/fire test fills the blast
  cave with `CAVE_AIR`, so it fails without the fix.

## Fixed earlier (verified still correct)

- `explode_with_fire_roll` destroy path: loot parameters set `this_entity: self.indirect_source()`,
  matching `Explosion#finalizeExplosion` -> `Block.dropResources(..., getIndirectSourceEntity(), ...)`.
  `drop_loot` also fills `block_entity` from the world (`block/mod.rs:649`), so no extra param is needed here.
- `explode` emits `GameEvent::Explode` with the source entity before blocks/entities are touched, matching
  `ServerExplosion#explode`. Commit c1e2f06fe.

## Remaining gaps (larger or unconfirmed)

1. `damage_entities`, lines 597-599: `if exposure == 0.0 { continue; }` skips damage entirely for a fully
   occluded entity. Vanilla `Explosion#hurtEntities` computes
   `damage = ((d*d + d)/2)*7*radius + 1.0` with `d = (1 - dist/radius) * exposure`, so `exposure == 0`
   still yields exactly 1.0 damage (the "+1.0" term is unconditional); only knockback becomes zero.
   Suggested fix: drop the early `continue`, keep computing damage, let the existing zero knockback apply.
   Size S. Needs decompiled `hurtEntities` before landing because it changes "explosions leak 1 damage
   through full cover" behaviour.
2. `damage_entities`, lines 583-586 and 619-631: the `distance <= 1.0` gate and the `(1 - distance)`
   knockback scale use the entity's feet. Vanilla `Explosion#hurtEntities` computes the normalized `h` from
   `(x, y + getStandingEyeHeight(), z)` for both, while `ExplosionDamageCalculator#getEntityDamageAmount`
   uses feet. Suggested fix: `eye_delta` for gate + knockback, feet only inside the damage formula.
   Size M (needs an off-centre parity fixture).
3. `explode_with_fire_roll`, lines 788-859 (Paper parity): `BlockExplodeEvent` is created with only
   `block_pos`/`yield_rate`/`world`. Paper's `com.destroystokyo.paper.event.block.BlockExplodeEvent` exposes
   the mutable affected `blockList()` and applies `yield_rate` per block (`random.nextFloat() <= yield`), and
   the server destroys exactly the surviving list. Pumpkin ignores both, destroying every collected block.
   Suggested fix: add `blocks: Vec<BlockPos>` to the event (plugin API surface change -> bump
   `PLUGIN_API_VERSION` / WIT host pair), filter destruction by the possibly-modified list, apply yield.
   Size M.
4. `get_blocks_to_destroy`, lines 453-470: unloaded chunks are treated as air
   (`read_chunk_sync` returning `None` -> `Block::AIR`), so blocks in unloaded chunks survive the blast.
   Vanilla `Level#getBlockState` loads/generates the chunk. Pumpkin's sync-read-only policy is deliberate
   (never block the tick on chunk gen), so this is a documented divergence. Size M (async pre-load of the
   3x3 chunks around the centre).
5. `get_blocks_to_destroy` line 442 and `explode` line 754: randomness comes from `rand::random` /
   `rand::rng()` instead of the world's RNG. Vanilla uses `level.random`, which makes explosions reproducible
   per level seed and would let `/seed`-based differential tests work. Suggested fix: thread the world RNG
   through `Explosion` (keep the `fire_roll` test hook). Size S.
6. `damage_entities`, lines 624-631: vanilla wraps damage/knockback in `if (d5 != 0.0)` where `d5` is the
   entity->centre vector length, so an entity exactly at the centre gets neither. Pumpkin normalizes a zero
   vector to `(0,0,0)` (no NaN, `pumpkin-util/src/math/vector3.rs:426`) but still applies damage. Degenerate
   case only; suggested fix is a length check before damage. Size XS, unconfirmed from memory.
7. `should_create_fire`, line 116: uses the position-less `BlockState::is_solid_render()`. Vanilla 1.20.5+
   `ServerExplosion` calls `state.isSolidRender(level, pos)` (position-aware), so blocks whose solidity
   depends on neighbours could differ. Size XS but needs vanilla source; no known affected block.
8. `preserves_rail`, lines 366-375 and 494-496: TNT minecart rail preservation forces `Some(0.0)` resistance,
   which prevents the rail from being destroyed but also stops it absorbing power, so the ray reaches further
   than vanilla. Check Paper's `MinecartTNT` rail behaviour (and whether vanilla has it); likely fix is to
   skip only the pipe insertion of the rail and keep the real blast resistance. Size S/M.
9. `world/mod.rs:4917-4930`: one `CExplosion` packet per explosion. Vanilla `finalizeExplosion` also spawns
   `EXPLOSION`/smoke particles at affected blocks (`world.sendParticles`); Pumpkin relies on the client
   re-creating them. Client-only effect; track with rendering, no server behaviour change. Size S.

## Verified equivalent (no change)

- Ray set: `x/y/z > 0 && < 15` skip == vanilla's `i == 0 || i == 15 || ...` filter, 1352 rays; direction
  `i / 7.5 - 1.0`; initial `h = power * (0.7 + rand * 0.6)`; resistance `h -= (r + 0.3) * 0.3`; step 0.3 and
  `h -= 0.22500001`; `is_in_build_limit` break before the state fetch.
- Fire: `affected_positions` is inserted before the resistance subtraction, a superset of vanilla's
  `getToBlow` fire set, but every extra position is a surviving solid block that fails the `is_air` check, so
  the fire-eligible air positions are identical. Roll `rand(0..3) == 0` and the air/solid-render checks match.
- `calculate_exposure` == `Explosion#getSeenPercent` (offsets, lerp grid, `COLLIDER`/`Fluid.NONE` clip, ratio).
- `get_entity_damage_amount` == `ExplosionDamageCalculator#getEntityDamageAmount`
  (`midpoint(d*d, d) * 7 * radius + 1.0`, `radius = power * 2`).
- `SimpleExplosionDamageCalculator` immune tag -> `3600000.0`, air+empty fluid -> `None`, knockback multiplier
  default 1.0; creative-flying players get no knockback; TNT uses its position rather than eye position;
  knockback resistance attribute; destroy order drop -> set air -> `block.onBlockExploded`;
  `explosion_radius` only for `DESTROY_WITH_DECAY`; small/large particle choice (`power < 2.0 || KEEP`).
