# Parity rows for crates/pumpkin-core/src/entity/passive/strider.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Strider** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-strider-ff96d3** [partial/OWN-OPEN] own audit: 7 open, 2 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-strider-ff96d3.md) docs/tracker/tentities-strider-ff96d3.md

## Result

THIRD PASS (final) verification against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/ItemBasedSteering.java`. Both STILL OPEN rows are closed; no row remains STILL OPEN.

- **entities/Strider** [AUDITED]: FIXED/VERIFIED. New review of all gaps rows below plus `get_ambient_sound`. One genuine parity bug found and fixed this pass: `get_ambient_sound` previously returned `STRIDER_HAPPY` when saddled, but vanilla `getAmbientSound` (Strider.java:357-360) returns `STRIDER_AMBIENT` only while not panicking/tempted and `null` otherwise; the happy sound is played on temptation in `tick` (Strider.java:299-301), never as ambient. `strider.rs::get_ambient_sound` now returns `(!is_panicking()).then_some(Sound::EntityStriderAmbient)`, matching the established armadillo `get_ambient_sound` pattern (armadillo.rs:373-376). No generated-data/WIT change needed.
- **own/tentities-strider-ff96d3** [OWN-OPEN, 7 open rows]: all resolved, verified per row below.
  - Row 19 (hot-block suffocation, M): VERIFIED. `strider.rs::mob_tick` reproduces `Strider.tick` (Strider.java:306-321): `!is_no_ai` guard, inside/on warm-block tag (`tag::Block::MINECRAFT_STRIDER_WARM_BLOCKS`) or lava height > 0, warm-strider vehicle downcast via `cast_any().downcast_ref::<Self>()`, flip-only `StriderTemperatureChangeEvent` via `fire_blocking` gated on `cancelled`. Generated fields already exist (tag.rs MINECRAFT_STRIDER_WARM_BLOCKS; sound.rs strider sounds), so no codegen needed.
  - Row 20 (navigation hook, M): VERIFIED. Vanilla `StriderPathNavigation extends GroundPathNavigation` with a `WalkNodeEvaluator` (Strider.java:388,523-538), not amphibious. Pumpkin keeps the ground/walk navigator and overrides `is_stable_destination` (lava, strider.rs:249-260) and `get_walk_target_value` (10/NEG_INFINITY/0, strider.rs:262-275), matching Strider.java:393-399. No `create_navigation` hook required for this vanilla version.
  - Row 21 (jockey spawn, M): VERIFIED. Vanilla `finalizeSpawn`/`spawnJockey` (Strider.java:451-487). Pumpkin `finalize_spawn` (strider.rs:305-382) implements 1/30 zombified piglin when monsters can spawn (0.05 baby odds via `rng.random::<f32>() < 0.05` per `Zombie.getSpawnAsBabyOdds`, warped fungus in MAIN_HAND, `set_item_slot_and_drop_when_killed` for guaranteed saddle drop), 1/10 baby strider jockey, and jockey group baby chance 0.0 vs 0.5 otherwise.
  - Row 22 (pathfinding maluses, L): VERIFIED. Constructor (strider.rs:53-64) sets WATER -1.0, LAVA 0.0, FIRE_IN_NEIGHBOR (DangerFire) 0.0, FIRE (DamageFire) 0.0, matching Strider.java:96-99. The stale table text "LAVA 8.0" is incorrect; vanilla uses LAVA 0.0, which the code implements.
  - Row 23 (boost timing, S): FIXED (earlier pass) with a pure unit test in `item_steerable.rs`. Vanilla `tickBoost` compares the pre-increment value (`ItemBasedSteering.java:45-49`); Pumpkin uses `fetch_add` return with test `boost_stops_after_total_plus_one_ticks`.
  - Row 24 (passenger attachment, S): VERIFIED. Vanilla `getPassengerAttachmentPoint` (Strider.java:190-199) returns `super` on the server; the extra offset is client-only walk bob. Pumpkin is server-side, so no override is required.
  - Row 25 (eating sound, M): VERIFIED. Vanilla `mobInteract` (Strider.java:427-439) plays `STRIDER_EAT` when fed. `Animal::play_eating_sound` override in strider.rs plays `EntityStriderEat`.
- Tests: no new pure unit test; every resolved seam (suffocation, navigation, jockey, attachment, eating sound, ambient sound) needs a live entity/world rather than a pure function. The boost-timing pure test already exists in `item_steerable.rs`.
- Not compiled by this worker (shell denied); orchestrator batch-checks. Risk: only one minimal, mechanically-verified edit (`get_ambient_sound`, mirroring armadillo.rs) was made this pass; all other rows were already present.


## Result

FINAL PASS (this worker). Re-checked every row against `.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/Strider.java` and `ItemBasedSteering.java`. Two genuine parity gaps were found in the previously-declared-complete `entities/Strider` audit and fixed in `crates/pumpkin-core/src/entity/passive/strider.rs`; all other rows remain VERIFIED as recorded above. No row remains STILL OPEN.

- **entities/Strider** [AUDITED]: FIXED/VERIFIED. Two source edits:
  1. `get_ambient_sound` (strider.rs:152-156): vanilla `getAmbientSound` (Strider.java:357-360) returns `STRIDER_AMBIENT` only when `!isPanicking() && !isBeingTempted()`, `null` otherwise. Prior code checked only `is_panicking()`, so a tempted strider still played the ambient sound. Now also requires `!goals_selector.is_running::<TemptGoal>()` (`TemptGoal` already imported; Pumpkin analogue of vanilla's `temptGoal.isRunning()`, Strider.java:327-329). Generated field `Sound::EntityStriderAmbient` exists (sound.rs:1546).
  2. `mob_tick` (strider.rs:201-209): vanilla `tick` (Strider.java:299-304) plays `STRIDER_HAPPY` on an if branch when tempted (`random.nextInt(140) == 0`) and `STRIDER_RETREAT` on the else-if when panicking (`random.nextInt(60) == 0`). Prior code handled only the retreat branch. Now mirrors the if/else with `random_range(0..140)` then `random_range(0..60)`, using the `TemptGoal` running check. Generated field `Sound::EntityStriderHappy` exists (sound.rs:1547).
- **own/tentities-strider-ff96d3** [OWN-OPEN, 7 rows]: all still VERIFIED as recorded in the prior Result blocks (rows 19-25). No further change needed.
- Tests: no new pure unit test; both new seams (`get_ambient_sound`, tick sound selection) require a live entity/world. The boost-timing pure test remains in `item_steerable.rs`.
- Compile check: not run by this worker (shell denied); orchestrator batch-checks. Both edits use patterns already present in the same crate (`ocelot.rs:210` for `is_running::<TemptGoal>()`, `strider.rs` existing `random_range`/`play_entity_sound` calls), so no new imports or trait bounds were introduced.
