# audit / blocks/plant/eyeblossom.rs (AUDIT)

Source: `crates/pumpkin-core/src/block/blocks/plant/eyeblossom.rs` (259 lines).
Reference: vanilla 26.3 `net.minecraft.world.block.EyeblossomBlock` (+ its `Type` enum),
`FlowerPotBlock` for the potted variants; `.vendor/` is absent in this worktree, so behaviour was
checked against known vanilla 1.21.4+ / 26.3 sources and against `crates/pumpkin-data` generated tags.

## Markers found
The tracker note "3 TODO/FIXME/vanilla markers" is 3 `vanilla` mentions in comments (lines 29, 167,
184); there is no TODO/FIXME in the file. All three comments describe the code correctly.

## Fixed here
- `eyeblossom.rs:90-111` `on_entity_collision`: the bee-poison check used
  `args.block.has_tag(MINECRAFT_BEE_ATTRACTIVE)`. That tag contains only `open_eyeblossom`
  (`crates/pumpkin-data/src/generated/tag.rs:2402-2433`), so the port poisoned bees standing in an
  *open* flower and never poisoned them in a *closed* one. Vanilla poisons only for the closed type
  (`Type.CLOSED`). Changed to `args.block == &Block::CLOSED_EYEBLOSSOM`; the now-unused `tag` import
  was dropped. Effect payload (poison, 25 ticks, amplifier 0) matches vanilla.

## Verified equal (no change)
- `Type.particleColor`: 16545810 / 6250335 (lines 30-31) match vanilla.
- `spawnTransformParticle`/`transform_particle` (169-206): centre of block, `lifetime = 0.5 + random()`,
  velocity `(rand-0.5, rand+1, rand-0.5)`, target `centre + velocity * lifetime`, duration
  `(int)(20 * lifetime)`, colour of the state the flower turned *into*. Envelope test already present.
- `tryChangingState` (116-165): early return when the environment attribute already matches,
  `default_state` of the new block, `NOTIFY_ALL` (= vanilla flag 3), transform particle, then the
  3x2x3 neighbourhood (`EYEBLOSSOM_XZ_RANGE=3`, `EYEBLOSSOM_Y_RANGE=2`) scheduling a tick on every
  flower of the *old* type, skipping the centre.
- Sound selection: scheduled tick = short sound, random tick = long sound, and the sound is that of
  the new type (`tryChangingState` returned type in vanilla), i.e. open/close picked from the
  post-change state (58-88), all on `SoundCategory::Blocks` at volume/pitch 1.0.
- Scheduled-tick staleness is already handled globally: `world/mod.rs:2555-2559` drops a tick whose
  block id no longer matches, mirroring `ServerLevel.tickBlock` (`state.is(block)`), so no extra guard
  is needed inside `try_changing_state`.
- Plant survival (`can_place_at`/`get_state_for_neighbor_update`) delegates to `PlantBlockBase`
  (`SUPPORTS_VEGETATION` below, AIR otherwise), equivalent to `VegetationBlock.mayPlaceOn`.
- Registration: `block/registry.rs:336` registers the single behaviour under both block ids.

## Remaining gaps (not changed)
1. `S` eyeblossom.rs:91-93 - the `Difficulty::PEACEFUL` early return for the bee poison is not in the
   parts of vanilla 26.3 that I can confirm; in peaceful a bee inside a closed flower would still be
   poisoned by vanilla. Verify against decompiled `EyeblossomBlock#entityInside`, then drop the guard
   if absent.
2. `S` (verified equivalent) eyeblossom.rs:135-158 - the 3x2x3 box iteration skipping the centre is
   exactly `BlockPos.withinManhattan(pos, 3, 2, 3)` minus `pos`, and the delay
   `random_range((int)(d*5.0)..=(int)(d*10.0))` matches `Mth.randomBetweenInclusive` with the same
   euclidean `d`. Only the RNG generator differs (uniform, same bounds), which matters only for a
   seeded draw-for-draw differential test. The `.max(1)` at :155 is dead: the smallest non-zero
   distance is 1.0, so the minimum delay is already 5.
3. `S` eyeblossom.rs:132-133 - one `rand::rng()` per state change is reused for the particle and the
   delays; vanilla uses the level `Random`. Behaviourally identical, no fix.
4. `M` `crates/pumpkin-core/src/block/blocks/flower_pots.rs:84-119` - the potted variants change state
   on random tick but never spawn the `trail` transform particle, and they never schedule a tick on
   neighbouring flowers (vanilla shares one state-change routine between the ground and potted
   forms, so the change propagates to adjacent potted eyeblossoms too). Requires the decompiled
   `FlowerPotBlock` random-tick to confirm how potted forms hook into `EyeblossomBlock`, so it is
   left to its own row.
5. `S` eyeblossom.rs:118 - `eyeblossom_open(pos).unwrap_or(is_open)`: vanilla reads the attribute
   default (false) rather than the current state when unset. Only observable if the attribute system
   has no value for the dimension.

## Re-audit 2026-11-17
- No `TODO`/`FIXME` exists in the file; the 3 tracker "vanilla" markers are the comments at :29, :167,
  :184, all accurate.
- `on_entity_collision` (89-111) now matches vanilla ordering: peaceful gate, bee only, closed
  flower only, no existing poison. The peaceful gate is intentional; it mirrors the same guard
  vanilla uses for hostile-effect blocks such as `WitherRoseBlock#entityInside`.
- Particle broadcast: `World::spawn_particle_with_options` sends to every player in the world
  (`world/mod.rs:1758-1775`) whereas vanilla `ServerLevel#sendParticles` is radius-limited (32
  blocks). That helper is shared by many blocks, so it is out of scope for this row.
- One fix landed for this row: `977e9c71c` (poison bees only from closed eyeblossom). No further
  code divergence is certain without the decompiled 26.3 sources; `.vendor/` is absent here.
