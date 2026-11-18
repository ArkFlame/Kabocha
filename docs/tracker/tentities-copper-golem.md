# entities / Copper Golem

Coverage: RE-REVIEW (reviewed source changed: `95b40037c fix(entity): weather copper golems over time (#4201)`).
Source: `crates/pumpkin-core/src/entity/passive/copper_golem.rs`
Vanilla reference: `net.minecraft.world.entity.animal.golem.CopperGolem` (1.21.9). No decompiled copy in this worktree (`.vendor/` absent), gaps below are from the vanilla class + tracked-data contract.

## Fixed in this pass (commit)
- `mob_interact` honeycomb waxing did not consume the honeycomb. Vanilla `CopperGolem.mobInteract` consumes one item (`ItemStack.consume`); now `decrement_unless_creative`.
- `mob_interact` axe scraping did not damage the axe. Vanilla `WeatheringCopper` scrape path calls `hurtAndBreak(1)`; both scrape branches now `player.damage_item_stack(item_stack, 1)` (creative-safe).
- `mob_on_lightning_strike` reset the state to Unaffected but kept the pending `next_weathering_tick` (0 at oxidized). The golem re-oxidized on the very next tick. Now the timer is re-rolled to -1 unless waxed (-2 sentinel preserved, matching waxed copper block lightning behaviour).

## Remaining gaps
### 1. Oxidized golem never converts into a copper golem statue (M/L)
- Rust: `copper_golem.rs:266-273` — once `WeatherState::Oxidized` and `next_weathering_tick == 0`, `mob_tick` returns forever; the comment at :271 references statue conversion that is not implemented.
- Vanilla: `CopperGolem.updateWeathering` converts a fully oxidized golem into `Blocks.COPPER_GOLEM_STATUE` with a random `CopperGolemPose` (standing/sitting/running/star), plays `SoundEvents.COPPER_GOLEM_BECOME_STATUE`, and transfers custom name (and drops/keeps the antenna). Statues are otherwise unobtainable, so this is a real gameplay hole.
- Fix: in the oxidized branch call a `convert_to_statue` helper that reads the block state (`Block::COPPER_GOLEM_STATUE` + `CopperGolemStatueLikeProperties` from `pumpkin-data::block_properties`) with the golem's weather state and a random pose, removes the entity and spawns `CopperGolemStatueBlockEntity` (see `block/blocks/weathering_copper.rs:851-930`, `block/entities/copper_golem_statue.rs`), plus the `EntityCopperGolemBecomeStatue` sound. Size M (block-entity wiring) / L with equipment transfer and tests.

### 2. Item pickup / chest deposit behaviour is missing (L)
- Rust: `CopperGolemState` (`copper_golem.rs:75-102`) defines `GettingItem`, `GettingNoItem`, `DroppingItem`, `DroppingNoItem`, but no goal or tick path ever calls `set_state` (grep: only `mob_init_data_tracker` writes Idle). The tracked-data field changes are therefore dead.
- Vanilla: 1.21.9 `CopperGolem` walks to nearby item entities, picks one up (GettingItem/GettingNoItem), seeks a chest and inserts it (DroppingItem/DroppingNoItem), with cooldowns and a `MoveToBlockGoal`-style target selection.
- Fix: add `copper_golem_pickup` + `copper_golem_deposit` goals under `entity/ai/goal/`, register in `CopperGolemEntity::new` at the vanilla priorities, drive the animation states, and add a `pumpkin-gametest`. Size L.

### 3. Weathering NBT keys unverified against vanilla (S)
FIXED (UNVERIFIED): writes `next_weathering_tick` (long) and reads it with fallback to legacy `next_weather_age`; `weather_state` stays a string (copper_golem.rs). Recalled from vanilla, no decompiled source.
- Rust: `copper_golem.rs:288-303` writes `next_weather_age` (long) and `weather_state` (int).
- Vanilla `CopperGolem.addAdditionalSaveData` is not available in this worktree; if the vanilla keys/types differ (`next_weathering_tick`? `weather_state` as string vs int?), worlds round-tripped between vanilla and Pumpkin lose the weathering progress.
- Fix: check decompiled `CopperGolem` save/load and align the key names and types. Size S once the reference is available.

### 4. Weathering boundary is one tick early (S)
- Rust: `copper_golem.rs:266` uses `if game_time < next_weathering_tick { return; }`, so oxidation fires on `game_time == next_weathering_tick`; vanilla tests `getGameTime() > nextWeatheringTick`.
- Fix: compare with `<=`. Cosmetic one-tick drift, nothing else depends on it.

## Gap status
- 1 oxidized golem statue conversion (copper_golem.rs:266-273): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 item pickup / chest deposit (L): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 weathering NBT keys (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 4 weathering boundary one tick early (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)

## Result

Verified all four STILL OPEN rows against decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/golem/CopperGolem.java (present this pass; the doc's "no decompiled copy" note is stale).

- Row 1 (oxidized golem to statue, M/L): ALREADY FIXED, no change. post_tick -> update_weathering plus turn_to_statue in copper_golem.rs implement vanilla canTurnToStatue/turnToStatue: oxidized + air + chance <= 0.0058, EntityChangeBlockEvent, oxidized statue block with random pose and facing, create_statue (custom name transfer in copper_golem_statue.rs), drop_preserved_equipment, entity remove, become-statue sound, leash drop. Matches vanilla lines 277-321.
- Row 2 (item pickup / chest deposit, L): ALREADY FIXED, no change. TransportItemsBetweenContainers brain behavior (entity/ai/brain/behavior/transport_items_between_containers.rs), registered in copper_golem_ai.rs, drives all CopperGolemState variants through set_state. Those files are outside this unit's allowed set.
- Row 3 (weathering NBT keys, S): FIXED in copper_golem.rs. Vanilla addAdditionalSaveData writes next_weather_age (long) and weather_state via a string codec (CopperGolem.java lines 166-176). Pumpkin wrote next_weathering_tick. Now writes next_weather_age through a shared constant; next_weathering_tick kept as a read fallback for older Pumpkin saves. weather_state string names already matched vanilla. rustfmt applied.
- Row 4 (weathering boundary one tick early, S): NO BUG, no change. Vanilla uses gameTime >= nextWeatheringTick (line 270), not > as the doc recalled. Effective progression runs in update_weathering (post_tick) using >=, matching vanilla; the <= in tick_weathering is redundant because post_tick progresses first at the boundary.

Test: no pure unit-test seam (NBT read/write require a live entity and world), so none added.


## Result (re-review 2026-11-18)

Re-verified all four STILL OPEN rows against decompiled vanilla `net/minecraft/world/entity/animal/golem/CopperGolem.java` (present this pass). No code changes needed; the prior pass's fixes are intact.

- Row 1 (statue conversion): ALREADY FIXED. `turn_to_statue` in copper_golem.rs matches vanilla `turnToStatue`/`canTurnToStatue` (lines 284-321): oxidized block + random pose/facing, EntityChangeBlockEvent, `create_statue` name transfer (copper_golem_statue.rs:121, covered by its own unit test), equipment drop, entity remove, become-statue sound, leash drop; constants TURN_TO_STATUE_CHANCE 0.0058 and weathering 504000..=552000 match vanilla lines 266/285.
- Row 2 (item pickup / chest deposit): ALREADY FIXED. `TransportItemsBetweenContainers` (transport_items_between_containers.rs) drives every CopperGolemState variant via set_state and is registered in copper_golem_ai.rs:49.
- Row 3 (NBT keys): ALREADY FIXED. mob_write_nbt writes `next_weather_age` (long) + `weather_state` string via constants; matches vanilla lines 166-176. Legacy `next_weathering_tick` kept as read fallback only.
- Row 4 (boundary): NO BUG. Vanilla uses `gameTime >= nextWeatheringTick` (line 270); update_weathering uses `>=`, tick_weathering uses `<=` and is redundant because post_tick advances first at the boundary. No double advancement: tick_weathering only seeds the -1 timer.

No pure unit-test seam for the verified behaviour (NBT and statue conversion need a live entity/world); enum conversion tests would only mirror constants/derives, excluded by the test policy.


## Result (re-review 2026-11-18)

Third pass. Re-verified all four STILL OPEN rows against decompiled vanilla `net/minecraft/world/entity/animal/golem/CopperGolem.java` (present at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/golem/CopperGolem.java`). No code changes: every row is already resolved by prior passes and confirmed correct.

- Row 1 (oxidized golem to statue, M/L): ALREADY FIXED. `update_weathering` (copper_golem.rs:261) gates statue conversion on Oxidized + air + `TURN_TO_STATUE_CHANCE` 0.0058 (line 292), matching vanilla `canTurnToStatue` (line 285). `turn_to_statue` (line 298) sets OXIDIZED_COPPER_GOLEM_STATUE with random pose and horizontal facing, fires EntityChangeBlockEvent, calls `create_statue` (custom name transfer in copper_golem_statue.rs:121-125, covered by its own unit test), drops preserved equipment, handles the leash, removes the entity, and plays EntityCopperGolemBecomeStatue. Matches vanilla lines 284-321.
- Row 2 (item pickup / chest deposit, L): ALREADY FIXED. `TransportItemsBetweenContainers` (transport_items_between_containers.rs) drives every CopperGolemState variant: `interaction_state` (lines 384-397) selects GettingNoItem/GettingItem/DroppingItem/DroppingNoItem, and `on_container_interaction` (copper_golem.rs:226) calls `set_state` plus the four interaction sounds. Registered as an Idle behavior at priority 0 in copper_golem_ai.rs:49. File carries its own unit tests (lines 720, 731).
- Row 3 (weathering NBT keys, S): ALREADY FIXED. `mob_write_nbt` (copper_golem.rs:582) writes `next_weather_age` (long) and `weather_state` as a lowercase string via `WeatherState::name()` (lines 107-113), matching vanilla `addAdditionalSaveData` lines 166-170. `mob_read_nbt` (line 594) prefers `next_weather_age`, keeps `next_weathering_tick` as a legacy read fallback, and decodes string then int, matching vanilla lines 172-177.
- Row 4 (weathering boundary, S): NO BUG. Traced both paths across a full cycle: `tick_weathering` (line 464, strict `game_time > next`) and `update_weathering` (line 261, `game_time >= next`) never double-advance because whichever advances first pushes `next` to a future tick (or 0 when Oxidized, where the `state != Oxidized` guard blocks further advancement). Combined effect is advancement at `game_time >= next`, matching vanilla line 270; `tick_weathering` only seeds the -1 timer.

No pure unit-test seam (NBT/statue conversion need a live entity and world; enum tests would only mirror constants/derives). Skipped: no code changed, so no rustfmt/compile run by this worker.
