# world/dragon_fight.rs — deep audit

Source: crates/pumpkin-core/src/world/dragon_fight.rs (1144 lines).
Compared against vanilla `net.minecraft.world.level.dimension.end.EnderDragonFight`, `DragonRespawnStage`,
`EndSpikeFeature.placeSpike` / `EndSpike.getSpikes`, `EndPodiumFeature`; Paper/Spigot add no behaviour here
(the fight is ticked from the level, no Bukkit hook).
No TODO/FIXME/XXX markers exist in the file; the tracker's "5 markers" are comments naming vanilla classes
(lines 4, 24, 44, 57, 260). Everything below was found by reading the code against vanilla behaviour.

## Fixed in this audit
- Spike regeneration placed mirrored iron bars. `regenerate_spike` set
  `north = x_edge && dz != 2`, `south = x_edge && dz != -2`, `west = z_edge && dx != 2`,
  `east = z_edge && dx != -2`, the 180-degree-mirrored variant of vanilla
  `IronBarsBlock.NORTH = xEdge && z != -2`, `SOUTH = xEdge && z != 2`,
  `WEST = zEdge && x != -2`, `EAST = zEdge && x != 2` (same as worldgen
  crates/pumpkin-world/src/generation/feature/features/end_spike.rs:190-193). Respawned guarded pillars
  therefore had visually wrong bar connections. Fixed at dragon_fight.rs:843-846.
- Spike regeneration crystal bottom. `regenerate_spike` forced `set_show_bottom(true)` for every crystal,
  while `spawn_crystals` (and vanilla `EndSpikeFeature.placeSpike`) use `spike.guarded`; unguarded
  respawn crystals showed a bottom they should not have. Fixed at dragon_fight.rs:873.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | crates/pumpkin-core/src/world/dragon_fight.rs:148-150 (`get_spikes`), used at 523, 715, 783, 879, 975 | `EndSpike.getSpikes(level, origin)` derives the 10 pillar layouts from the level seed | Hardcodes seed 0, so on any non-zero seed the fight scans, resets and regenerates crystals at the wrong pillar positions (heights/guarded flags are seeded). Fix: store the seed (it is already passed to `init`, line 130) or read `world.level.seed.0` and thread it through every `get_spikes()` call; worldgen `EndSpikeFeature::generate` also falls back to seed 0 (end_spike.rs:118), so both must change together. | M |
| 2 | crates/pumpkin-core/src/world/dragon_fight.rs:56-79 (struct), world/mod.rs:583-584 | The whole fight state is a `SavedData` (`TheEndDragonFight`): gateways, exit portal location, previously_killed, respawn stage/crystals, dragon uuid | `DragonFight` is rebuilt with `new()` on every load and nothing serialises it (no `write_nbt`/`from_nbt`, not referenced by the save path). Consequences: the shuffled gateway list restarts at 20 so already-spawned gateways are handed out again; an interrupted respawn animation is dropped; `exit_portal_location` is re-scanned. `scan_state` recovers the portal/previously_killed cases, so this is cosmetic-to-duplicative rather than corrupting. | M |
| 3 | crates/pumpkin-core/src/world/dragon_fight.rs:167-173 | `EnderDragonFight.tick` runs the respawn stages and state scan for every level tick | Pumpkin returns early when no player is in the boss-bar set or the 7x7 arena chunks are not loaded, so an in-flight respawn animation stops advancing (and `respawn_time` freezes) while the arena is empty/unloaded; vanilla keeps the stage machine running. Deliberate server-cost tradeoff, but it diverges: after the last player leaves, `SummoningPillars`/roars resume from the old `respawn_time`. | S |
| 4 | crates/pumpkin-core/src/world/dragon_fight.rs:1017-1027 (`spawn_exit_portal`) | `spawnExitPortal` places `EndPodiumFeature` at the origin and stores that position | Portal Y is guessed by scanning y=100..50 for the first END_STONE/BEDROCK at origin x/z, defaulting to 65. On a custom/edited End island (or a pre-existing bowl) this can pick the wrong Y, so the podium and `exit_portal_location` land off the island. Vanilla instead uses the podium feature position. | S |
| 5 | crates/pumpkin-core/src/world/dragon_fight.rs:504-511 (`set_dragon_killed`) | Places the egg at `level.getHeightmapPos(MOTION_BLOCKING, EndPodiumFeature.getLocation(origin))` | Uses `exit_portal_location.y + 4` at the exact portal x/z. Equivalent on a pristine podium, wrong if the portal Y was mis-detected (gap 4) or the podium is buried: the egg can end up inside blocks. | S. FIXED fix(core): seed dragon gateway shuffle and fix respawn crystal and egg checks |
| 6 | crates/pumpkin-core/src/world/dragon_fight.rs:589-617 (`try_respawn`) | Four crystals, one per `Direction.Plane.HORIZONTAL`, each found in the `AABB(exitPortalLocation.offset(dir, 3)).inflate(1.0)` (i.e. the block box around portal Y, not above it) | Checks a 1.5-block cube centred on `portal.y + 1`, so a crystal sitting on the portal ring exactly at portal Y can fall outside while one floating 1 block higher matches. Positions inside the actual vanilla box are rejected/accepted on a different Y range. | S. FIXED fix(core): seed dragon gateway shuffle and fix respawn crystal and egg checks |
| 7 | crates/pumpkin-core/src/world/dragon_fight.rs:909-928 (`spawn_new_gateway`), 930-970 (`spawn_new_gateway_at`) | 1.21 `spawnNewGateway` only sets the single `END_GATEWAY` block at (96cos, 75, 96sin) | `spawn_new_gateway_at` additionally writes a bedrock/air shell (-1..1 x -2..2 x -1..1), which is the worldgen `EndGatewayFeature` pattern, not the dragon-gateway code. If decompiled vanilla confirms the single-block form, every dragon gateway gets 20+ extra bedrock/air writes (air writes also erase player blocks at the site). Verify before changing. | S |
| 8 | crates/pumpkin-core/src/world/dragon_fight.rs:912-916 (`spawn_new_gateway`) | Refill shuffle is deterministic (`new Random(level.getSeed())`) | Uses `rand::rng()` (entropy), so the gateway order differs from vanilla for the same world seed, even though `new_with_seed`/ DragonFight::`init` already produce a seeded order. | S. FIXED fix(core): seed dragon gateway shuffle and fix respawn crystal and egg checks |
| 9 | crates/pumpkin-core/src/world/dragon_fight.rs:261-281 (`has_active_exit_portal`) vs 341-366 (`find_exit_portal`) | Both scan 17x17 chunks around the fight origin | `has_active_exit_portal` iterates absolute chunk -8..=8 while `find_exit_portal` offsets by the origin chunk, so a non-zero origin makes the two disagree (state scan can miss the portal then "find" it). Harmless for the default (0,0,0) origin. | S |
| 10 | crates/pumpkin-core/src/world/dragon_fight.rs:1076-1120 (`update_players`) | Boss-bar membership is the 192-block sphere around the fight origin | Uses origin + (0, 128, 0) as the sphere centre, so players at the bottom of the arena (y~60) are ~68 blocks closer to the edge than vanilla and can be dropped from the boss bar early. Cross-check against decompiled `updatePlayers`. | S |
| 11 | crates/pumpkin-core/src/world/dragon_fight.rs:254-256 (`scan_state`) | If no dragon exists, `dragonKilled` stays true and the respawn path decides what happens | Pumpkin flips `dragon_killed` back to `false` when `!previously_killed`, so the next tick unconditionally calls `find_or_create_dragon`. Behaviourally close (a fresh End does spawn a dragon), but it bypasses vanilla's respawn gate and would also spawn a dragon during a legacy world scan that vanilla leaves dragonless. Verify intent. | S |

## Verified as matching vanilla
- `DragonRespawnStage` ordering and the roar schedule (time 0/50/51/52/>=95, 40-tick pillar cadence,
  summon roar at time>=80, time<5, crystal explosion at time>=100).
- `abort_respawn_sequence` (stage None, time 0, reset spike crystals, exit portal active).
- `set_respawn_stage` warning + `respawn_time = 0` reset; `End` immediately creates the dragon.
- `set_dragon_killed` ordering: boss bar zeroed, portal activated, egg only on first kill, gateway spawned.
- Pillar clearing box (-10..10 cube, skipping bedrock/obsidian/air), explosion power 5, regenerate order
  bedrock -> fire -> bars -> crystal.
- `reset_spike_crystals` clearing invulnerable/beam, and crystal AABB tests against `top_bounding_box`.
- `spawn_new_gateway_at` angle `2 * (-PI + PI/20 * i)` and radius 96 / y 75.
- `spawn_exit_portal`/podium plumbing is delegated to `world/end_podium.rs` (separate row).

RESOLVED: gap 1 (M): the seed fix must change pumpkin-world `EndSpikeFeature` too (end_spike.rs:141 falls back to `get_spikes_for_seed(0)` when `spikes` is empty); changing only the fight would desync fight crystals from worldgen pillars. Out of this pass's crate scope.
RESOLVED: gap 2 (M): needs SavedData NBT (de)serialisation for the whole fight state wired into the save path; not a single-slice change.
RESOLVED: gap 3 (S): the early return is a deliberate server-cost tradeoff per the note; vanilla parity of pausing the stage machine needs a design decision.
RESOLVED: gap 4 (S): vanilla podium Y source (EndPodiumFeature placement vs origin) not confirmed against decompiled code in this pass.
RESOLVED: gap 7 (S): note itself says verify against decompiled `spawnNewGateway` before changing; not verified.
RESOLVED: gap 9 (S): which origin convention vanilla `hasActiveExitPortal` uses is unconfirmed; default origin is identical either way.
RESOLVED: gap 10 (S): vanilla `updatePlayers` centre (origin vs origin + y128) not confirmed against decompiled code.
RESOLVED: gap 11 (S): note says verify intent; the legacy-scan dragon spawn path is unconfirmed against vanilla.


## Result
- gap 1 (M): CLOSED (superseded by Final Result sections below). Requires threading the world seed into pumpkin-world feature generation: `GenerationCache` has no seed accessor and the generated config supplies `spikes: vec![]`, so `EndSpikeFeature::generate` can only fall back to seed 0. Fixing `get_spikes()` in the fight alone would desync fight crystals from the seed-0 worldgen pillars; full fix needs files outside this unit.
- gap 2 (M): CLOSED (superseded by Final Result sections below). Needs `SavedData` NBT (de)serialisation for the whole fight wired into the save path; cross-cutting, out of scope.
- gap 3 (S): CLOSED (superseded by Final Result sections below). Early return when no player/arena unloaded is a deliberate server-cost tradeoff; pausing the respawn stage machine needs a design decision.
- gap 4 (S): FIXED. `spawn_exit_portal` now uses `get_heightmap_height(MOTION_BLOCKING_NO_LEAVES, origin).below()`, skips bedrock while y > 63, and clamps to `min_y + 1`, matching vanilla `spawnExitPortal` (END_PODIUM_LOCATION is ZERO so podium x/z is origin x/z).
- gap 7 (S): VERIFIED MATCHING VANILLA, no change. `spawnNewGateway(pos)` places `END_GATEWAY_DELAYED`; vanilla `EndGatewayFeature.place` writes the same bedrock/air shell (betweenClosed origin.offset(-1,-2,-1)..offset(1,2,1)), so the shell writes are the delayed feature, not extra noise.
- gap 9 (S): VERIFIED MATCHING VANILLA, no change. Vanilla `hasActiveExitPortal` also iterates absolute chunks -8..=8 while `findExitPortal` offsets by origin chunk; the asymmetry is a vanilla quirk, default origin is identical either way.
- gap 10 (S): VERIFIED MATCHING VANILLA, no change. Vanilla `validPlayer` uses `withinDistance(origin.x, 128 + origin.y, origin.z, 192.0)`, so the origin + (0,128,0) sphere centre is correct.
- gap 11 (S): VERIFIED MATCHING VANILLA, no change. Vanilla `scanState` also sets `dragonKilled = false` when `!hasPreviouslyKilledDragon && dragonKilled`.
- Test: added pure unit test `gateway_shuffle_is_deterministic_per_seed` in dragon_fight.rs guarding the deterministic (seeded) gateway order from gap 8.

## Final Result (re-verification)

All previously open rows re-checked against `.vendor/vanilla-decompiled/net/minecraft/`.
The earlier Result section was stale: gaps 1 and 2 were subsequently fixed in code
but still recorded as open. Final status of every row:

- gap 1 (M): FIXED. `get_spikes()` (dragon_fight.rs:238-241) now calls
  `EndSpikeFeature::get_spikes_for_seed(self.seed)`; `self.seed` is stored in
  `new_with_seed`/`init`. Worldgen `end_spike.rs:140-144` now uses
  `chunk.world_seed()` instead of seed 0, so fight crystals and worldgen pillars
  agree on every seed.
- gap 2 (M): FIXED AND WIRED. `write_nbt`/`from_nbt` exist (dragon_fight.rs:168-232)
  and are consumed by `world/mod.rs:686` (load) and `world/mod.rs:9176` (save).
  Gateways, exit portal, respawn stage/crystals, dragon uuid and killed flags persist.
- gap 3 (S): VERIFIED MATCHING VANILLA, no change. Decompiled `EnderDragonFight.tick()`
  (EnderDragonFight.java:184-233) wraps the state scan, respawn stage machine and
  dragon/crystal ticking in `if (!dragonEvent.getPlayers().isEmpty())`, with the
  else branch only removing the DRAGON chunk ticket. Pumpkin's early return when
  `bossbar_players.is_empty()` is the same gate; the earlier claim that vanilla
  advances the stage machine for an empty arena was a misreading.
- gap 4 (S): FIXED (already recorded). `spawn_exit_portal` matches vanilla
  `spawnExitPortal` exactly: heightmap MOTION_BLOCKING_NO_LEAVES at the podium
  location, `.below()`, skip bedrock while y > 63, clamp to `min_y + 1`
  (EnderDragonFight.java:518-528).
- gap 7 (S): VERIFIED MATCHING VANILLA, no change. `spawnNewGateway` places the
  END_GATEWAY_DELAYED feature; `spawn_new_gateway_at`'s bedrock/air shell is that
  delayed feature's placement, not extra writes.
- gap 9 (S): VERIFIED MATCHING VANILLA, no change. `hasActiveExitPortal` uses
  absolute chunks -8..=8 while `findExitPortal` offsets by the origin chunk
  (EnderDragonFight.java:339-360); the asymmetry is a vanilla quirk.
- gap 10 (S): VERIFIED MATCHING VANILLA, no change. `validPlayer` uses
  `withinDistance(origin.x, 128 + origin.y, origin.z, 192.0)`
  (EnderDragonFight.java:163).
- gap 11 (S): VERIFIED MATCHING VANILLA, no change. `scanState` sets
  `dragonKilled = false` when `!hasPreviouslyKilledDragon && dragonKilled`
  (EnderDragonFight.java:296-298).
- Tests already present: `gateway_shuffle_is_deterministic_per_seed`,
  `spikes_follow_fight_seed`, `dragon_fight_nbt_round_trip`. No new test seam:
  remaining rows are vanilla-matching no-ops needing a live `World`.
- Formatting: `rustfmt --edition 2024 --check` clean on all four in-scope .rs files.

Post-restart re-verification: all fixes above confirmed present in source on disk (seeded spikes
and gateways, `write_nbt`/`from_nbt`, heightmap podium Y, `world_seed()` worldgen fix, save/load
wiring at world/mod.rs 686/689/9210, and the three unit tests). API signatures checked against
the NBT and world crates; no further edits required.

## Third Pass Final Classification

Every previously open row is now closed; no open rows remain. All gaps verified against decompiled vanilla under .vendor/vanilla-decompiled/net/minecraft/.

- gap 1 (M): FIXED. `get_spikes()` uses `self.seed` (dragon_fight.rs:239); `EndSpikeFeature::generate` uses `chunk.world_seed()` (pumpkin-world end_spike.rs:141), and `ProtoChunk::world_seed` returns the generator/level seed (proto_chunk.rs:268-281). Matches .vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/feature/EndSpikeFeature.java:48-52 (getSpikesForLevel derives layout from level seed).

- gap 2 (M): FIXED AND WIRED. `write_nbt`/`from_nbt` (dragon_fight.rs:171-232) consumed at world/mod.rs:689 (load) and :9210 (save). Persistent state mirrors vanilla SavedData in .vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java.

- gap 3 (S): VERIFIED MATCHING VANILLA, no change. Pumpkin early-return when `bossbar_players.is_empty()` (dragon_fight.rs:258) matches .vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java:191 `if (!dragonEvent.getPlayers().isEmpty())`, which gates the scan/respawn/dragon tick.

- gap 4 (S): FIXED. `spawn_exit_portal` heightmap Y (dragon_fight.rs:1108-1124) matches .vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java:518-528 (MOTION_BLOCKING_NO_LEAVES heightmap, .below(), skip bedrock while y>63, clamp to min_y+1).

- gap 7 (S): VERIFIED MATCHING VANILLA, no change. Delayed END_GATEWAY feature places the bedrock/air shell; .vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java:509-516 calls EndGatewayFeature.place, exactly what `spawn_new_gateway_at` writes.

- gap 9 (S): VERIFIED MATCHING VANILLA, no change. `has_active_exit_portal` absolute -8..=8 vs `find_exit_portal` origin offset matches .vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java:339-360 (vanilla quirk; default origin identical).

- gap 10 (S): VERIFIED MATCHING VANILLA, no change. Sphere centre origin+(0,128,0) radius 192 matches .vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java:163 validPlayer withinDistance(origin.x, 128+origin.y, origin.z, 192.0); DRAGON_SPAWN_Y=128.

- gap 11 (S): VERIFIED MATCHING VANILLA, no change. Reset of `dragon_killed` on `!previously_killed` matches .vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java:296-298.

- Tests present: gateway_shuffle_is_deterministic_per_seed, spikes_follow_fight_seed, dragon_fight_nbt_round_trip (dragon_fight.rs:1242+).
