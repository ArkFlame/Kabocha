# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-passive-copper-golem-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-reviewed both STILL OPEN rows against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/golem/CopperGolem.java` (present this pass). No row left open.

## Result
FINAL PASS. Both rows closed against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/golem/CopperGolem.java`. No code changes were needed this pass; every prior fix is intact and correct.

- **entities/Copper Golem [partial/RE-REVIEW]**: FIXED / VERIFIED. `mob_on_lightning_strike` (copper_golem.rs:615) matches vanilla `thunderHit` (CopperGolem.java:488-499): super lightning effect first, dedup by bolt UUID, then on a new bolt step weathering back one stage (`weather.previous()`) and set `next_weathering_tick = UNSET (-1)` only when not already Unaffected (waxed golems at Unaffected stay waxed). Weathering progression matches vanilla `updateWeathering` (CopperGolem.java:263-282): `tick_weathering` (mob_tick) seeds the -1 timer first or defers with `game_time <= next`, and `update_weathering` (post_tick) advances at `game_time >= next`, rolls 504000..=552000 once, and runs statue conversion. Traced all timing scenarios: tick_weathering runs before update_weathering each tick, so there is no double advancement and no RNG divergence. NBT keys `next_weather_age` (long) + `weather_state` (lowercase string) match vanilla lines 166-176.
- **own/tentities-copper-golem [partial/OWN-OPEN]**: VERIFIED. All four documented gaps resolved against CopperGolem.java: (1) oxidized-to-statue conversion via `update_weathering`/`turn_to_statue` (copper_golem.rs:261/298; vanilla 263-321) with random pose/facing, EntityChangeBlockEvent, `create_statue` custom-name transfer (copper_golem_statue.rs:121-125, unit test at :178), equipment drop, leash handling, entity removal and become-statue sound; (2) item pickup/deposit via `TransportItemsBetweenContainers` (registered copper_golem_ai.rs:49) driving all CopperGolemState variants; (3) NBT `next_weather_age` (long) + `weather_state` (string); (4) `gameTime >= nextWeatheringTick` boundary (line 270). No code change needed this pass.

No pure unit-test seam exists for lightning/NBT/statue timing (each needs a live entity and world; enum-conversion tests would only mirror constants/derives). Did not compile or rustfmt (workers have no shell); the orchestrator handles that.

## Result (batch x_17, CODER pass)
- Copper golem rows: NOT RE-VERIFIED this pass. The prior VERIFIED entry above stands on its own citations; vanilla CopperGolem.java exists at .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/golem/CopperGolem.java. No edit made.
