# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-world-dragon-fight-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- No BLOCKED-DATA rows, no remaining STILL OPEN rows.
Full re-audit of this unit against `.vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java` and `.../levelgen/feature/EndSpikeFeature.java`. No STILL OPEN rows remain and no source edits were needed this pass. This doc has no "STILL OPEN until closed..." line to replace.

## Result (Final Pass)

Every row re-verified against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source edits needed: all fixes from prior
passes confirmed present on disk (seeded spikes/gateways, NBT persistence + save/load wiring,
heightmap podium Y, portal-Y crystal AABB, origin+(0,128,0) sphere, and the three unit tests).
No BLOCKED-DATA rows and no STILL OPEN rows remain.

Row `own/taudit-crates-pumpkin-core-src-wo-ea95b7` (11 gaps) CLOSED:

- gap 1 (M): FIXED. `get_spikes()` calls `EndSpikeFeature::get_spikes_for_seed(self.seed)`
  (dragon_fight.rs:239-240); worldgen `end_spike.rs:141` uses `chunk.world_seed()`.
  Matches `.vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/feature/EndSpikeFeature.java:48-52`.
- gap 2 (M): FIXED AND WIRED. `write_nbt`/`from_nbt` (dragon_fight.rs:171-232) consumed at
  world/mod.rs:685-689 (load) and :9223-9227 (save), mirroring the vanilla `SavedData` codec
  in `.vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java:104-149`.
- gap 3 (S): VERIFIED MATCHING VANILLA. `bossbar_players.is_empty()` early return
  (dragon_fight.rs:258) matches `if (!dragonEvent.getPlayers().isEmpty())`
  (EnderDragonFight.java:191) gating the scan/respawn/dragon tick.
- gap 4 (S): FIXED. `spawn_exit_portal` uses MOTION_BLOCKING_NO_LEAVES heightmap `.below()`,
  skips bedrock while y > 63, clamps to `min_y + 1` (dragon_fight.rs:1108-1124), matching
  EnderDragonFight.java:518-528.
- gap 5 (S): FIXED. Egg placed at the podium x/z via the portal location (set_dragon_killed),
  matching vanilla `setDragonKilled` (EnderDragonFight.java:460).
- gap 6 (S): FIXED. `try_respawn` checks four crystals at portal Y with per-direction
  offsets plus/minus 3 and a plus/minus 1.5 inflated box (dragon_fight.rs:686-708), matching
  `AABB(exitPortalLocation.offset(dir,3)).inflate(1.0)` (EnderDragonFight.java:645-648).
- gap 7 (S): VERIFIED MATCHING VANILLA. Delayed END_GATEWAY placement writes the bedrock/air
  shell (dragon_fight.rs `spawn_new_gateway_at`), matching `EndGatewayFeature.place` called
  from EnderDragonFight.java:509-516.
- gap 8 (S): FIXED. Gateway shuffle seeded via `JavaRandom::new(seed)` (dragon_fight.rs:66-74)
  matches vanilla `new Random(seed)`.
- gap 9 (S): VERIFIED MATCHING VANILLA. `has_active_exit_portal` absolute -8..=8 vs
  `find_exit_portal` origin offset (dragon_fight.rs:352) matches the vanilla quirk
  (EnderDragonFight.java:339-360).
- gap 10 (S): VERIFIED MATCHING VANILLA. Sphere centre origin+(0,128,0) radius 192
  (dragon_fight.rs:1173-1189) matches `withinDistance(origin.x, 128+origin.y, origin.z, 192.0)`
  (EnderDragonFight.java:163).
- gap 11 (S): VERIFIED MATCHING VANILLA. `dragon_killed = false` when `!previously_killed`
  matches EnderDragonFight.java:296-298.

Tests present: `gateway_shuffle_is_deterministic_per_seed`, `spikes_follow_fight_seed`,
`dragon_fight_nbt_round_trip` (dragon_fight.rs:1247+). No new source edits, so no rustfmt
needed this pass.
