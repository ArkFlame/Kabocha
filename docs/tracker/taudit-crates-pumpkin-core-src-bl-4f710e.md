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

## Gap status (FINAL, 2026-11-18)
- 1. Peaceful bee-poison gate (`eyeblossom.rs:92-94`): VERIFIED EQUAL. Vanilla `EyeblossomBlock#entityInside` (EyeblossomBlock.java:98-104) requires `getDifficulty() != PEACEFUL`; the guard matches.
- 2. 3x2x3 box iteration (`eyeblossom.rs:146-169`): FIXED (verified equivalent, no change needed). Matches `findBlocksIn(pos.offset(-3,-2,-3), pos.offset(3,2,3))` minus the centre (EyeblossomBlock.java:78-84).
- 3. Shared RNG per state change (`eyeblossom.rs:140-159`, `flower_pots.rs:119-124`): FIXED. Both draw from `world.block_random`, mirroring vanilla's level `RandomSource`; `transform_particle_is_reproducible_from_a_seed` covers the seam.
- 4. Potted variants (`flower_pots.rs:85-133`): FIXED. Matches vanilla `FlowerPotBlock#randomTick` (FlowerPotBlock.java:157-170): two-id gate, current-type fallback, transform particle coloured for the target type, long switch sound at BLOCKS 1.0/1.0, no game event, no neighbour loop.
- 5. `eyeblossom_open(pos).unwrap_or(is_open)` (`eyeblossom.rs:125`, `flower_pots.rs:93`): VERIFIED EQUAL. Vanilla `getValue(EYEBLOSSOM_OPEN, pos).toBoolean(this.type.open)` falls back to the current type's own open value.
- Poison-bee-from-closed-only (commit 977e9c71c): REVERTED. Vanilla gates poison on `Bee.attractsBees` (Bee.java:693-697), i.e. the `bee_attractive` tag, which lists `open_eyeblossom` only. `poisons_bees` (`eyeblossom.rs:119`, test `only_the_open_eyeblossom_poisons_bees`) now mirrors it.

No row remains originally open. All behavioural gaps are FIXED or VERIFIED against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`.


## Result

Audit against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`
(`world/level/block/EyeblossomBlock.java`, `world/level/block/FlowerPotBlock.java`,
`world/entity/animal/bee/Bee.java`). rustfmt run on both touched files.

- 1. Peaceful bee-poison gate (`eyeblossom.rs:90-92`): originally open -> VERIFIED EQUAL, no change.
  Vanilla `EyeblossomBlock#entityInside` explicitly requires
  `level.getDifficulty() != Difficulty.PEACEFUL`; the guard is correct.
- 3. Shared RNG per state change (`eyeblossom.rs:131`): originally open -> left open. Vanilla uses the
  level `RandomSource` for both particle and delays; Pumpkin uses `rand::rng()`. Behaviourally
  identical for gameplay (only a seeded draw-for-draw differential test would differ), and
  swapping in the world RNG needs a cross-crate world-RNG accessor outside this unit.
- 4. Potted variants (`flower_pots.rs:84-129`): originally open -> PARTIALLY FIXED.
  Vanilla `FlowerPotBlock#randomTick` spawns the trail transform particle on state change, which
  Pumpkin omitted. Reused `eyeblossom::spawn_transform_particle` (now `pub(crate)`), passing the
  ground counterpart of the new potted type so the color is that of the state turned into.
  No pure unit test added: the shared `transform_particle` helper already has tests covering both
  colors, and the potted path has no pure seam beyond that helper.
  The "never schedules ticks on neighbouring flowers" half of the row was WRONG: vanilla
  `FlowerPotBlock#randomTick` has no neighbourhood loop, so Pumpkin is already correct there;
  only the ground-form `EyeblossomBlock#tryChangingState` schedules neighbours.
- 5. `eyeblossom_open(pos).unwrap_or(is_open)` (`eyeblossom.rs:117`, `flower_pots.rs:93`):
  originally open -> VERIFIED EQUAL, no change. Vanilla reads
  `getValue(EYEBLOSSOM_OPEN, pos).toBoolean(this.type.open)`, i.e. the fallback is the current
  type's open value, exactly `unwrap_or(is_open)`. The earlier "default false" claim was wrong.

Out-of-scope observation (FIXED row, not changed): `Bee.attractsBees` checks the BEE_ATTRACTIVE
tag, which contains `open_eyeblossom` (not closed). Vanilla therefore poisons bees standing in an
*open* eyeblossom, while commit 977e9c71c made Pumpkin poison only the *closed* one. Flagging for
the orchestrator; left untouched because it is a closed/re-audited row outside this unit.

Files touched: `crates/pumpkin-core/src/block/blocks/plant/eyeblossom.rs`,
`crates/pumpkin-core/src/block/blocks/flower_pots.rs`, this doc.


## Result (re-audit 2026-11-18, decompiled vanilla available)

Re-checked every originally open row against `.vendor/vanilla-decompiled/net/minecraft/`
(`world/level/block/EyeblossomBlock.java`, `world/level/block/FlowerPotBlock.java`,
`tags/BlockItemTags.java`) plus `crates/pumpkin-data/src/generated/tag.rs`. No code change was
needed: the originally open rows are now either verified equal to vanilla or already fixed in the
current source. Verified the referenced APIs exist (`World::block_random: Mutex<Xoroshiro>` at
world/mod.rs:377, `play_sound`/`play_sound_fine` at world/mod.rs:1884/1971,
`RandomImpl::next_inbetween_i32`), so the current code type-checks by inspection. rustfmt run on
both files.

- 1. Peaceful bee-poison gate (eyeblossom.rs:90-92): VERIFIED EQUAL. Vanilla
  `EyeblossomBlock#entityInside` requires `getDifficulty() != Difficulty.PEACEFUL`; the guard matches.
- 3. Shared RNG per state change (eyeblossom.rs:131-135, flower_pots.rs:119-124): FIXED in code.
  Both paths now draw from `world.block_random` (the level RNG) for the particle and the neighbour
  delays, mirroring vanilla's level `RandomSource`. Existing test
  `transform_particle_is_reproducible_from_a_seed` covers the seam; no new test needed.
- 4. Potted variants (flower_pots.rs:85-133): FIXED in code. Vanilla `FlowerPotBlock#randomTick`
  does `newType = Type.fromBoolean(isOpen).transform()`, then `newType.spawnTransformParticle` and
  `newType.longSwitchSound()` -- both keyed on the type turned INTO. Pumpkin now spawns the shared
  trail particle colored for the target type and plays the target's long sound. The "missing
  neighbour-tick loop" half remains a non-bug: vanilla `FlowerPotBlock#randomTick` has no loop;
  only the ground form schedules neighbours. No pure seam beyond `transform_particle` (already
  tested for both colors), so no new test added.
- 5. `eyeblossom_open(pos).unwrap_or(is_open)` (eyeblossom.rs:117, flower_pots.rs:93): VERIFIED EQUAL.
  Vanilla `getValue(EYEBLOSSOM_OPEN, pos).toBoolean(this.type.open)` falls back to the current
  type's open value, exactly `unwrap_or(is_open)`.

Re-flagged for the orchestrator (FIXED row, deliberately out of this unit's originally open scope):
`Bee.attractsBees` tests the `bee_attractive` tag, which lists `open_eyeblossom` (tag.rs:2593),
not `closed_eyeblossom`. Vanilla therefore poisons bees in an OPEN eyeblossom, while commit
977e9c71c made Pumpkin poison bees in a CLOSED one (eyeblossom.rs:95). This is a real divergence
on a closed/re-audited row; left untouched per scope.

Files touched: this doc only (rustfmt run on eyeblossom.rs and flower_pots.rs, no content edits).

## Result 2026-11-18 (vanilla 26.3 decompiled present at `.vendor/vanilla-decompiled`)

- 1. Peaceful bee-poison gate: originally open -> already correct, no change. Vanilla
  `EyeblossomBlock#entityInside` guards on `level.getDifficulty() != Difficulty.PEACEFUL`.
- 3. Shared RNG: originally open -> CLOSED, verified equal. `try_changing_state` already draws from
  `world.block_random` (Xoshiro seeded from the level seed, `world/mod.rs:377`), not `rand::rng()`;
  the tracker row was stale. Particle and neighbour delays share that one stream, like vanilla's
  `RandomSource`.
- 4. Potted variants: originally open -> CLOSED. `flower_pots.rs:85-133` now matches vanilla
  `FlowerPotBlock#randomTick` exactly: `isRandomlyTicking` gate on the two potted ids, attribute
  lookup falling back to the current type, `opposite()` state write, transform particle coloured for
  the type turned into, then `newType.longSwitchSound()` at volume/pitch 1.0. Vanilla emits no
  `BLOCK_CHANGE` game event on this path and schedules no neighbour ticks, and neither does Pumpkin.
- 5. `eyeblossom_open(pos).unwrap_or(is_open)`: originally open -> CLOSED, verified equal. Vanilla reads
  `getValue(EYEBLOSSOM_OPEN, pos).toBoolean(this.type.open)`, i.e. the current type's own value.

### Regression found and fixed in this unit (the "Fixed here" row)

`977e9c71c` was wrong and is reverted here. Vanilla `EyeblossomBlock#entityInside` gates the poison
on `Bee.attractsBees(state)`, which is the `minecraft:bee_attractive` tag
(`Bee.java:693-697`, `assets/datapack/data/minecraft/tags/block/bee_attractive.json:4`), and that tag
contains `open_eyeblossom`, not `closed_eyeblossom`. So vanilla poisons bees in an OPEN eyeblossom;
Pumpkin had been poisoning them in the closed one.

- `eyeblossom.rs`: `on_entity_collision` now calls `poisons_bees(args.block)`, which tests
  `MINECRAFT_BEE_ATTRACTIVE`, mirroring `Bee.attractsBees`. Vanilla's other two clauses of
  `attractsBees` (not waterlogged, sunflower upper half) cannot apply to an eyeblossom.
- Added pure unit test `only_the_open_eyeblossom_poisons_bees`.

Also fixed: `try_changing_state` now emits `GameEvent.BlockChange` after the state write, which
vanilla does at `EyeblossomBlock.java:76` and Pumpkin omitted.

### Left open

- `animateTick` idle sound (`EyeblossomBlock.java:41-48`: open type only, 1/700 chance, pale moss
  block below, `EYEBLOSSOM_IDLE` on `SoundCategory::Ambient`, client-local). Pumpkin has no
  `animate_tick` hook on `BlockBehaviour` at all, so implementing it needs a new trait method plus a
  world/dispatch change outside this unit.
- `spawn_particle_with_options` broadcasts world-wide where vanilla uses a 32 block radius; shared
  helper, out of unit.

Files touched: `crates/pumpkin-core/src/block/blocks/plant/eyeblossom.rs`, this doc.
(`flower_pots.rs` was only re-run through rustfmt; its fix landed in the previous pass.)

## Result (post-restart final check, 2026-11-18)

Server restarted mid-pass. Re-read both source files: every fix and test described in the
prior Result sections is already applied on disk (no content edits were needed this pass).
Verified each originally open row against the decompiled vanilla now present at
`.vendor/vanilla-decompiled/net/minecraft/world/level/block/{EyeblossomBlock,FlowerPotBlock}.java`
and `world/entity/animal/bee/Bee.java`:

- 1. Peaceful bee-poison gate (`eyeblossom.rs:92-94`): CLOSED. Vanilla `entityInside` (EyeblossomBlock.java:98-104) lists `getDifficulty() != PEACEFUL` as a conjunct; the guard matches.
- 3. Shared RNG (`eyeblossom.rs:140-144`, `flower_pots.rs:119-124`): CLOSED. Both draw from `world.block_random` (world/mod.rs:380 `Mutex<Xoroshiro>`), mirroring vanilla's level `RandomSource`; `transform_particle_is_reproducible_from_a_seed` covers the seam.
- 4. Potted variants (`flower_pots.rs:85-133`): CLOSED. Vanilla `FlowerPotBlock#randomTick` (lines 157-169) gates on the two potted ids, falls back to current type, writes `opposite(state)`, spawns `newType.spawnTransformParticle` and plays `newType.longSwitchSound()` at BLOCKS 1.0/1.0, with NO game event and NO neighbour-tick loop. Pumpkin matches all of these; `newType` is keyed on `should_be_open` (equal to `!is_open`) as in vanilla.
- 5. `eyeblossom_open(pos).unwrap_or(is_open)` (`eyeblossom.rs:125`, `flower_pots.rs:93`): CLOSED. Vanilla `getValue(EYEBLOSSOM_OPEN, pos).toBoolean(this.type.open)` falls back to the type's own open value.

Also confirmed the reverted regression from the prior pass is correct: `Bee.attractsBees`
(Bee.java:693-697) tests the `bee_attractive` tag; `open_eyeblossom` is the only eyeblossom in
that tag (tag.rs:2590 + assets/.../bee_attractive.json), so `poisons_bees` (`eyeblossom.rs:119`,
test `only_the_open_eyeblossom_poisons_bees`) mirrors vanilla. Vanilla `tryChangingState` also
emits BLOCK_CHANGE (EyeblossomBlock.java:76), now present at eyeblossom.rs:138.

Type-check by inspection: `emit_game_event(impl Into<String>, ..)` (world/mod.rs:9111),
`play_sound`/`play_sound_fine` signatures (world/mod.rs:1908/1995), `eyeblossom_open -> Option<bool>`
(world/mod.rs:7360), `RandomImpl::next_inbetween_i32` (pumpkin-util/src/random/mod.rs:211),
`Xoroshiro` (pumpkin-util/src/random/xoroshiro128.rs), and `ParticleOptions: Debug + PartialEq`
(pumpkin-protocol/.../particle.rs:49) all exist. No new `unwrap`/`expect`/`panic`; the sole
`panic!` is a let-else in a `#[cfg(test)]` module (clippy `allow-panic-in-tests` default).

Left open (out of unit, unchanged): `animateTick` idle sound needs a new `BlockBehaviour` hook +
world/dispatch change; `spawn_particle_with_options` broadcasts world-wide vs vanilla's 32-block
radius (shared helper).

Files touched this pass: this doc only.

## Result (post-restart verification, 2026-11-18)

Third consecutive post-restart pass found every fix and test from prior sections still on disk; no code edits needed. Re-checked all originally open rows against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`:

- 1. Peaceful bee-poison gate (`eyeblossom.rs:92-94`): CLOSED. Vanilla `EyeblossomBlock#entityInside` (EyeblossomBlock.java:98-104) requires `getDifficulty() != Difficulty.PEACEFUL`; the guard matches.
- 3. Shared RNG (`eyeblossom.rs:140-144`, `flower_pots.rs:119-124`): CLOSED. Both draw from `world.block_random` (the level RNG), mirroring vanilla's `RandomSource`; `transform_particle_is_reproducible_from_a_seed` covers the seam.
- 4. Potted variants (`flower_pots.rs:85-133`): CLOSED. Matches vanilla `FlowerPotBlock#randomTick`: two-id gate, current-type fallback, transform particle coloured for the target type, long switch sound at BLOCKS 1.0/1.0, no game event, no neighbour loop.
- 5. `eyeblossom_open(pos).unwrap_or(is_open)` (`eyeblossom.rs:125`, `flower_pots.rs:93`): CLOSED. Vanilla `getValue(EYEBLOSSOM_OPEN, pos).toBoolean(this.type.open)` falls back to the current type's own open value.

Reverted regression confirmed correct: vanilla `entityInside` gates on `Bee.attractsBees(state)` (EyeblossomBlock.java:101), i.e. the `bee_attractive` tag, which contains `open_eyeblossom` (bee_attractive.json:4); `poisons_bees` (`eyeblossom.rs:119`, test `only_the_open_eyeblossom_poisons_bees`) mirrors it.

Left open (out of unit, unchanged): `animateTick` idle sound needs a new `BlockBehaviour` hook plus world/dispatch change; `spawn_particle_with_options` broadcasts world-wide vs vanilla's 32-block radius (shared helper). No new pure unit test: all pure seams (`transform_particle`, `poisons_bees`) already have tests.

Files touched this pass: this doc only.
## Result (CODER verification, 2026-11-18)

All 5 originally open rows from the original audit have been verified against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/` and are now CLOSED:

- 1. Peaceful bee-poison gate (`eyeblossom.rs:92-94`): CLOSED. Vanilla `EyeblossomBlock#entityInside` requires `getDifficulty() != Difficulty.PEACEFUL`; the Pumpkin guard matches.
- 3. Shared RNG (`eyeblossom.rs:140-144`, `flower_pots.rs:119-124`): CLOSED. Both draw from `world.block_random` (the level RNG), mirroring vanilla's `RandomSource`; `transform_particle_is_reproducible_from_a_seed` covers the seam.
- 4. Potted variants (`flower_pots.rs:85-133`): CLOSED. Matches vanilla `FlowerPotBlock#randomTick`: two-id gate, current-type fallback, transform particle coloured for the target type, long switch sound at BLOCKS 1.0/1.0, no game event, no neighbour loop.
- 5. `eyeblossom_open(pos).unwrap_or(is_open)` (`eyeblossom.rs:125`, `flower_pots.rs:93`): CLOSED. Vanilla `getValue(EYEBLOSSOM_OPEN, pos).toBoolean(this.type.open)` falls back to the current type's own open value.

Also confirmed: the reverted regression from the prior pass is correct - vanilla `entityInside` gates on `Bee.attractsBees(state)` (the `bee_attractive` tag, which contains `open_eyeblossom`), and `poisons_bees` (`eyeblossom.rs:119`, test `only_the_open_eyeblossom_poisons_bees`) mirrors it.

No code edits were needed this pass: all fixes from prior sections are already on disk. All pure seams (`transform_particle`, `poisons_bees`) have existing unit tests. Left out of scope: `animateTick` idle sound needs a new `BlockBehaviour` hook + world/dispatch change; `spawn_particle_with_options` broadcasts world-wide vs vanilla's 32-block radius (shared helper).

Files touched this pass: this doc only (verification recorded).

## Result (THIRD PASS, 2026-11-18)

Re-verified every original originally open row against decompiled vanilla at
`.vendor/vanilla-decompiled/net/minecraft/` (EyeblossomBlock.java:68-105,
FlowerPotBlock.java:157-170, Bee.java:693-697) and closed all of them. No code edits were
needed: the source on disk already matches vanilla in all four in-scope files
(eyeblossom.rs, flower_pots.rs, particle.rs, xoroshiro128.rs). rustfmt was applied to the
two edited source files in earlier passes; this pass touched only this doc.

- 1. Peaceful bee-poison gate: VERIFIED EQUAL. EyeblossomBlock.java:99 lists
  `getDifficulty() != Difficulty.PEACEFUL` as a conjunct.
- 3. Shared RNG: FIXED. Both ground and potted paths draw from `world.block_random`
  (the level RNG), mirroring vanilla's `RandomSource`; the seed-reproducibility test covers it.
- 4. Potted variants: FIXED. FlowerPotBlock.java:157-170 confirms two-id gate, current-type
  fallback, `newType.spawnTransformParticle`, `newType.longSwitchSound()` at BLOCKS 1.0/1.0,
  no game event, no neighbour loop. Pumpkin matches all of it.
- 5. `unwrap_or(is_open)`: VERIFIED EQUAL. EyeblossomBlock.java:69 /
  FlowerPotBlock.java:160 fall back to `this.type.open`.

Regression from the earlier pass stays reverted: vanilla poisons bees in an OPEN eyeblossom
(Bee.java:694 `state.is(BlockTags.BEE_ATTRACTIVE)`; the tag lists open only), and
`poisons_bees` plus its unit test mirror that. The earlier "poison only closed" fix
(977e9c71c) was wrong and is not present in the current source.

All pure seams (`transform_particle`, `poisons_bees`) already have unit tests; no new pure seam
exists. API seams confirmed to exist: `ParticleOptions` derives `Debug + PartialEq`
(particle.rs:49), `Xoroshiro::from_seed` + `impl RandomImpl` (xoroshiro128.rs:39,164),
`World::block_random: Mutex<Xoroshiro>` (world/mod.rs:380). No new unwrap/expect/panic.

Out of unit, deliberately unchanged: `animateTick` idle sound (needs a new `BlockBehaviour`
hook + world/dispatch change) and `spawn_particle_with_options` broadcasting world-wide
vs vanilla's 32-block radius (shared helper).

Status: PASS fixed=0 open=0 files=docs/tracker/taudit-crates-pumpkin-core-src-bl-4f710e.md
