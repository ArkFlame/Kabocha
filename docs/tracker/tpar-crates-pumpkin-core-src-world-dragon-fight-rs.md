# Parity rows for crates/pumpkin-core/src/world/dragon_fight.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-wo-ea95b7** [partial/OWN-OPEN] own audit: 9 open, 7 fixed, 9 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-wo-ea95b7.md) docs/tracker/taudit-crates-pumpkin-core-src-wo-ea95b7.md

## Result (Third Pass Final)

Row `own/taudit-crates-pumpkin-core-src-wo-ea95b7` CLOSED. All 11 audit gaps re-verified against
`.vendor/vanilla-decompiled/net/minecraft/world/level/dimension/end/EnderDragonFight.java` and
`.../world/level/levelgen/feature/EndSpikeFeature.java`. Every fix from prior passes is present in
`dragon_fight.rs`; no source edits were required this pass.

- gap 1 (M): FIXED. `get_spikes()` uses `self.seed` (dragon_fight.rs:239-240); worldgen `end_spike.rs` uses `chunk.world_seed()`. Matches `EndSpikeFeature.getSpikesForLevel` (EndSpikeFeature.java:48-52).
- gap 2 (M): FIXED AND WIRED. `write_nbt`/`from_nbt` (dragon_fight.rs:168-232) persisted via world/mod.rs load/save; mirrors the vanilla `EnderDragonFight` `SavedData`.
- gap 3 (S): VERIFIED MATCHING VANILLA. Early return on empty boss bar / unloaded arena (dragon_fight.rs:258,262) matches `if (!dragonEvent.getPlayers().isEmpty())` and `isArenaLoaded()` (EnderDragonFight.java:191-195).
- gap 4 (S): FIXED. `spawn_exit_portal` MOTION_BLOCKING_NO_LEAVES heightmap `.below()`, skip bedrock while y>63, clamp min_y+1 (dragon_fight.rs:1108-1124) matches EnderDragonFight.java:518-526.
- gap 5 (S): FIXED. Egg at MOTION_BLOCKING heightmap at podium x/z (dragon_fight.rs:594-608), matches vanilla `setDragonKilled`.
- gap 6 (S): FIXED. `try_respawn` crystal check uses the portal-Y block box inflated to ±1.5 around the cell centre (dragon_fight.rs:686-708), matching `AABB(exitPortalLocation.offset(dir,3)).inflate(1.0)`.
- gap 7 (S): VERIFIED MATCHING VANILLA. Delayed END_GATEWAY shell in `spawn_new_gateway_at` (dragon_fight.rs:1023-1063) is `EndGatewayFeature.place` (EnderDragonFight.java:509-516).
- gap 8 (S): FIXED. Seeded gateway shuffle via `JavaRandom` (dragon_fight.rs:66-74,1009) matches vanilla `new Random(seed)`.
- gap 9 (S): VERIFIED MATCHING VANILLA. `has_active_exit_portal` absolute -8..=8 (dragon_fight.rs:352-371) vs `find_exit_portal` origin-chunk offset matches the vanilla quirk (EnderDragonFight.java:339-360).
- gap 10 (S): VERIFIED MATCHING VANILLA. Sphere centre origin+(0,128,0) radius 192 (dragon_fight.rs:1173-1189) matches `withinDistance(origin.x, 128+origin.y, origin.z, 192.0)` (EnderDragonFight.java:163).
- gap 11 (S): VERIFIED MATCHING VANILLA. `dragon_killed = false` when `!previously_killed` matches EnderDragonFight.java:296-298.
- "Fixed in this audit": iron-bar connection directions and crystal `set_show_bottom(spike.guarded)` re-verified in `regenerate_spike` (dragon_fight.rs:909-974).
- Tests present: `gateway_shuffle_is_deterministic_per_seed`, `spikes_follow_fight_seed`, `dragon_fight_nbt_round_trip` (dragon_fight.rs:1242+).
- No BLOCKED-DATA rows, no remaining STILL OPEN rows.
