# Parity rows for crates/pumpkin-core/src/entity/passive/copper_golem.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Copper Golem** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-copper-golem** [partial/OWN-OPEN] own audit: 7 open, 10 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-copper-golem.md) docs/tracker/tentities-copper-golem.md

## Result (FINAL PASS 2026-11-18)

Both rows closed against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/golem/CopperGolem.java`. No row remains open.

- **entities/Copper Golem [partial/RE-REVIEW]**: FIXED. `mob_on_lightning_strike` (copper_golem.rs:615) previously reset the state to `Unaffected` and re-rolled the timer before the bolt-UUID dedup, making vanilla's `thunderHit` step-back dead code. It now calls the super lightning effect first, dedups by bolt UUID, and on a new bolt steps weathering back one stage (`weather.previous()` at copper_golem.rs:99) and re-rolls the timer (`UNSET_WEATHERING_TICK`) only when the golem is not already `Unaffected`. Matches vanilla `thunderHit` exactly (CopperGolem.java:488-499: `super.thunderHit`, UUID dedup, `nextWeatheringTick = -1L`, `weatherState.previous()` when not UNAFFECTED). Waxed (-2) golems at Unaffected stay waxed, as in vanilla.
- **own/tentities-copper-golem [partial/OWN-OPEN]**: VERIFIED. All four documented gaps in `docs/tracker/tentities-copper-golem.md` remain resolved against `CopperGolem.java`: oxidized-to-statue conversion (`update_weathering`/`turn_to_statue`, lines 261-321), item pickup/deposit via `TransportItemsBetweenContainers` (copper_golem_ai.rs:49), NBT `next_weather_age` (long) + `weather_state` (string) (`mob_write_nbt`/`mob_read_nbt`, lines 166-177), and the `gameTime >= nextWeatheringTick` boundary (line 270). No code change needed this pass.

No pure unit-test seam exists (lightning/NBT/statue behaviour each needs a live entity and world; enum-conversion tests would only mirror constants/derives, excluded by test policy). Did not compile or rustfmt per the worker contract (no shell); the orchestrator handles that.


## Result (third pass 2026-11-18)

Re-reviewed both STILL OPEN rows against decompiled vanilla `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/golem/CopperGolem.java` (present this pass). No row left open.

- **entities/Copper Golem [partial/RE-REVIEW]**: FIXED. Found one real parity bug in `mob_on_lightning_strike` (copper_golem.rs:615). It unconditionally set the state to `Unaffected` and re-rolled the timer before the bolt-UUID dedup, which made vanilla's `thunderHit` step-back (`weatherState.previous()` + `nextWeatheringTick = -1`) dead code: `get_weather_state()` after line 620 always returned Unaffected, so the `weather != Unaffected` branch never ran. Rewrote the handler to call the super lightning effect first, dedup by bolt UUID, then on a new bolt step weathering back one stage and re-roll the timer only when the golem is not already Unaffected. Matches vanilla `thunderHit` (CopperGolem.java:488-499). Waxed (-2) golems at Unaffected stay waxed, as in vanilla.
- **own/tentities-copper-golem [partial/OWN-OPEN]**: VERIFIED. All four documented gaps in `docs/tracker/tentities-copper-golem.md` remain resolved against CopperGolem.java: oxidized-to-statue conversion (lines 284-321), item pickup/deposit via `TransportItemsBetweenContainers`, NBT keys `next_weather_age` (long) + `weather_state` (string) (lines 166-177), and the `gameTime >= nextWeatheringTick` boundary (line 270). The lightning behaviour above is the only new discrepancy found this pass.

No pure unit-test seam exists for lightning/NBT/statue behaviour (each needs a live entity and world; enum conversion tests would only mirror constants/derives). This worker did not run rustfmt or compile per the task contract; orchestrator handles that.
