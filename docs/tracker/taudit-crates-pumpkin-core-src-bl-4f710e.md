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
2. `S` eyeblossom.rs:135-158 - neighbour tick delay: euclidean `sqrt(dx^2+dy^2+dz^2)` scaled by 5..10
   inclusive via `random_range`. Vanilla 26.3 uses `BlockPos.withinManhattan` iteration and a
   `nextInt`-based delay; the RNG draw count/order differs for off-axis positions. Needs the
   decompiled method before changing (behaviour is the same shape, timing slightly different).
3. `S` eyeblossom.rs:132-133 - one `rand::rng()` is created per state change and reused for particle +
   delays; vanilla uses the level `Random`. Fine behaviourally, only matters if a seeded RNG is
   required for differential tests.
4. `M` `crates/pumpkin-core/src/block/blocks/flower_pots.rs:84-119` - the potted variants change state
   on random tick but never spawn the `trail` transform particle; vanilla potted eyeblossoms emit it
   too. Separate file/row.
5. `S` eyeblossom.rs:118 - `eyeblossom_open(pos).unwrap_or(is_open)`: vanilla reads the attribute
   default (false) rather than the current state when unset. Only observable if the attribute system
   has no value for the dimension.
