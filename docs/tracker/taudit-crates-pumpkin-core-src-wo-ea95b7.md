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
| 5 | crates/pumpkin-core/src/world/dragon_fight.rs:504-511 (`set_dragon_killed`) | Places the egg at `level.getHeightmapPos(MOTION_BLOCKING, EndPodiumFeature.getLocation(origin))` | Uses `exit_portal_location.y + 4` at the exact portal x/z. Equivalent on a pristine podium, wrong if the portal Y was mis-detected (gap 4) or the podium is buried: the egg can end up inside blocks. | S |
| 6 | crates/pumpkin-core/src/world/dragon_fight.rs:589-617 (`try_respawn`) | Four crystals, one per `Direction.Plane.HORIZONTAL`, each found in the `AABB(exitPortalLocation.offset(dir, 3)).inflate(1.0)` (i.e. the block box around portal Y, not above it) | Checks a 1.5-block cube centred on `portal.y + 1`, so a crystal sitting on the portal ring exactly at portal Y can fall outside while one floating 1 block higher matches. Positions inside the actual vanilla box are rejected/accepted on a different Y range. | S |
| 7 | crates/pumpkin-core/src/world/dragon_fight.rs:909-928 (`spawn_new_gateway`), 930-970 (`spawn_new_gateway_at`) | 1.21 `spawnNewGateway` only sets the single `END_GATEWAY` block at (96cos, 75, 96sin) | `spawn_new_gateway_at` additionally writes a bedrock/air shell (-1..1 x -2..2 x -1..1), which is the worldgen `EndGatewayFeature` pattern, not the dragon-gateway code. If decompiled vanilla confirms the single-block form, every dragon gateway gets 20+ extra bedrock/air writes (air writes also erase player blocks at the site). Verify before changing. | S |
| 8 | crates/pumpkin-core/src/world/dragon_fight.rs:912-916 (`spawn_new_gateway`) | Refill shuffle is deterministic (`new Random(level.getSeed())`) | Uses `rand::rng()` (entropy), so the gateway order differs from vanilla for the same world seed, even though `new_with_seed`/ DragonFight::`init` already produce a seeded order. | S |
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
