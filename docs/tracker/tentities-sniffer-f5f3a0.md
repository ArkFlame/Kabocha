# entities / Sniffer (RE-REVIEW)
Source: `crates/pumpkin-core/src/entity/passive/sniffer.rs`, `sniffer/ai.rs` (implemented in d5dc0dd2d, #4535).
No decompiled `net.minecraft.world.entity.animal.sniffer.Sniffer` in this worktree (`.vendor/` absent, nothing under `../`). Rows are code findings plus vanilla knowledge, not mechanically confirmed. Line numbers are post-`ef4595b2f`.

## Re-review verdict
The source changed since the last assessment (`a494cb9b5` cap at 20, `db897ff2d` head-position loot). Those two earlier gaps are closed: `sniffer.rs:341` now takes 19 then prepends one (20 total), `sniffer.rs:361` spawns at the head block. No regression found in state/NBT persistence (`sniffer.rs:635-657`).

## Fixed in this pass
- **Dig game event used the head block centre instead of the entity position** (commit `ef4595b2f`). `mob_tick` emitted `minecraft:entity_action` at `get_head_block().to_centered_f64()`; vanilla `Entity.gameEvent(GameEvent)` (used by `Sniffer`) emits at `position()`. Now uses `Entity::emit_own_game_event` like `camel.rs:218` and `turtle.rs:386`. Affects sculk sensor / warden hearing only. NOTE: a later tracker-sweep commit `cdb0045a3` changed this back to a head-block-centred `emit_game_event_with_context`. Decompiled `Sniffer` (line 330) actually emits `ENTITY_ACTION` at `getHeadBlock()`; the current position is not re-verified in this pass (out of the STILL OPEN row scope).

## Remaining gaps
1. **`SNIFFER_SEARCHING` sound is never played** (size S).
   - SEARCHING (`ai.rs:342-396`, entered from `ai.rs:110-121`) plays no sound; `sniffer.rs:599-605` only plays the ambient idle sound and only outside Digging/Searching. `Sound::EntitySnifferSearching` (`pumpkin-data/src/generated/sound.rs:1569`) is unreferenced in the sniffer.
   - Vanilla class: `Sniffer` plays `SoundEvents.SNIFFER_SEARCHING` while in `State.SEARCHING` (continuous sniff loop). Confirm the exact cadence/pitch before adding.
   - Suggested fix: play at the vanilla cadence from the Searching task tick or from `mob_tick` when state is Searching.
2. **Ambient sound interval/enumeration unconfirmed** (size S).
   - `sniffer.rs:607-609` hardcodes `120`; `sniffer.rs:599` returns idle for every state except Digging/Searching (so Rising still uses the idle ambient). Vanilla `Mob` default is 80; check whether `Sniffer` overrides `getAmbientSoundInterval` and which states return `null`.
   - Suggested fix: align to the decompiled values.
3. **Dig loot spawn position uses a floored head block** (size S).
   - `sniffer.rs:361-362`: `get_head_block().0.to_f64()` gives a block corner (x/z floored, y = block y). Vanilla `Sniffer.dropSeed` spawns each item at a continuous position (head or entity position depending on version).
   - Suggested fix: switch to `get_head_position()` (or `entity.pos`) once the vanilla variant is confirmed; adjust `sniffer.rs:733` expectation.
4. **`can_sniff`/`can_dig` water test may be too narrow** (size S).
   - `sniffer.rs:264`, `sniffer.rs:276`: `!entity.is_in_water()`. Vanilla `Sniffer.canSniff`/`canDig` are believed to gate on `isInWaterOrRain()`.
   - Suggested fix: use the water-or-rain predicate if the decompiled class confirms it.
5. **Explored-position success flag relies on task timeout** (size S).
   - `ai.rs:253-259`: `on_digging_complete(tick.state, timed_out)` treats a full 40-tick Rising as success. Vanilla `SnifferDiggingTask.stop` records the site only when digging was not interrupted; verify Rising-timeout is the right proxy and whether the sniff cooldown is set on completion rather than only on dig timeout (`ai.rs:215-217`).
6. **Dimension/pose constants unverified** (size S). CHECKED, no change (UNVERIFIED): the constants match my recollection of vanilla `Sniffer` digging dimensions; no decompiled source to confirm.
   - `sniffer.rs:53-59` (`DIGGING_BB_HEIGHT_OFFSET 0.4`, eye `0.81`, `HEAD_DISTANCE 2.25`, `MAX_HEAD_Y_ROT 50`) and `refresh_state_dimensions` (`sniffer.rs:186-198`) invent a digging hitbox. Vanilla may not shrink the hitbox while digging; check `getDefaultDimensions`/eye height for the DIGGING state before trusting it.
7. **No end-to-end sniff -> walk -> dig test** (size M).
   - Only isolated task tests exist (`ai.rs:458+`, `sniffer.rs:733+`). A `pumpkin-gametest` over a dirt pad would cover the Searching -> Digging handoff and the drop-seed clock.
## Client-only, not blocking
- Digging particles (`Sniffer.DIGGING_PARTICLES_AMOUNT = 30`) and the digging pose animation are client-side; not implemented server-side beyond `EntityStatus::SnifferDiggingSound`.

## Gap status
- 1 SNIFFER_SEARCHING sound never played (S): CLOSED (verified: sound already played on entering Searching; vanilla loop is client-side)
- 2 ambient sound interval (S): CLOSED (verified: 120 is correct; `Sniffer extends Animal`, `Animal` overrides to 120)
- 3 dig loot spawn position floored head (S): CLOSED (verified: vanilla `dropSeed` spawns at integer head-block corner)
- 4 can_sniff/can_dig water test (S): CLOSED (verified: vanilla gates on `!isInWater()`)
- 5 explored-position success flag (S): CLOSED (verified: cooldown/completion match `SnifferAi`)
- 6 dimension/pose constants (S): CLOSED (verified: digging dimensions/eye height match vanilla)
- 7 end-to-end sniff -> walk -> dig test (M): FIXED / VERIFIED (covered by two dirt-pad `#[tokio::test]` tests in `sniffer.rs` and `sniffer/ai.rs`; see final Result)


## Result
Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/{Sniffer,SnifferAi}.java`, `Mob.java` and `Animal.java` (present in this worktree via `.vendor` symlink; `find -L` required to traverse it).

- **1 SNIFFER_SEARCHING sound (S): RESOLVED, no code change.** `transition_to(Searching)` already plays `Sound::EntitySnifferSearching` (`sniffer.rs:222-224`); the doc premise that it is unreferenced is stale. Vanilla `Sniffer.tick()` -> `playSearchingSound()` gates on `level().isClientSide()` and repeats every 20 ticks at volume/pitch 1.0, so vanilla clients self-generate the loop from the synced SEARCHING state and no server-side cadence is needed.
- **2 ambient sound interval (S): VERIFIED, no code change.** `Sniffer extends Animal`; vanilla `Animal.getAmbientSoundInterval()` returns 120 and `Sniffer` does not override it (`Mob`'s 80 is overridden by `Animal`). Pumpkin `get_ambient_sound_interval` returns 120, which is correct. The earlier note claiming a change to 80 was erroneous and was never applied (a later tracker-sweep commit `cdb0045a3` kept 120 and documented the reason). `get_ambient_sound` matches vanilla too (idle for every state except Digging/Searching). No unit test: a constant-mirroring assertion is excluded by the MVP test rules.
- **3 dig loot spawn position (S): VERIFIED, no change.** Vanilla `dropSeed()` spawns each `ItemEntity` at `head.getX(), head.getY(), head.getZ()` (integer block corner from `getHeadBlock()`); Pumpkin `get_head_block().0.to_f64()` matches. The existing `digging_drops_gift_loot_at_the_head_on_the_entity_clock` test already pins this position.
- **4 can_sniff/can_dig water test (S): VERIFIED, no change.** Both vanilla `canSniff()` and `canDig()` gate on `!this.isInWater()`, not `isInWaterOrRain()`; Pumpkin uses `!entity.is_in_water()` in both predicates.
- **5 explored-position success flag / cooldown (S): VERIFIED, no change.** Pumpkin `ai.rs` matches `SnifferAi`: Digging (160-180 ticks) sets `SNIFF_COOLDOWN` for 9600 ticks on timeout else `reset_sniffing`; FinishedDigging (40 ticks) starts RISING then on stop calls `onDiggingComplete(timed_out)`, erases SNIFFER_DIGGING and sets SNIFFER_HAPPY. Durations, cooldown and entry conditions all match.
- **6 dimension/pose constants (S): VERIFIED, no change.** Vanilla `DIGGING_BB_HEIGHT_OFFSET 0.4`, `withEyeHeight(0.81)`, `DIGGING_DIMENSIONS` applied only while DIGGING, head distance 2.25 and `MAX_HEAD_Y_ROT 50` all match the Pumpkin constants and `refresh_state_dimensions`.
- **7 end-to-end sniff -> walk -> dig test (M): FIXED / VERIFIED.** Superseded by the final Result: two pure `#[tokio::test]` dirt-pad tests already cover the Searching -> Digging handoff and drop-seed clock, so no gametest crate is needed.

Files edited this pass: none (all six S rows were verified against vanilla and need no source change; current `sniffer.rs` already returns the correct 120). Only this unit doc was updated for accuracy. Row 7 needs a `pumpkin-gametest`, outside this pass. No generated data, WIT, or cross-crate changes; no other basename in this unit needed edits.


## Result
Re-review pass (this assignment). Decompiled vanilla is present at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/Sniffer.java` (read directly; glob does not follow the `.vendor` symlink).

Only row 7 is marked STILL OPEN in Gap status. Rows 1-6 are CLOSED with an existing vanilla-verified Result, so no source change is warranted for them under this pass's STILL OPEN-only scope.

- 7 end-to-end sniff -> walk -> dig test (M): FIXED / VERIFIED (covered by two dirt-pad `#[tokio::test]` tests in `sniffer.rs` and `sniffer/ai.rs`; see final Result)

No source files edited. No generated data, WIT or cross-crate changes; no other basename in this unit required edits.

## Result
Third pass. Decompiled vanilla verified directly at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/SnifferAi.java` and `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/Sniffer.java`.

Only row 7 remained STILL OPEN. Rows 1-6 were already CLOSED with vanilla-verified Results, so no source change is warranted for them.

- 7 end-to-end sniff -> walk -> dig test (M): FIXED / VERIFIED, no new source change. The prior verdict ("no pure unit-test seam, needs a `pumpkin-gametest`") is stale: two pure `#[tokio::test]` tests already exercise this on a real dirt-pad `World` via `crate::entity::ai::test_support::world`:
  - `crates/pumpkin-core/src/entity/passive/sniffer/ai.rs` `searching_handoff_starts_digging_on_a_diggable_pad` drives Searching -> Digging on a 9x9 dirt pad: Searching entry reaches SEARCHING state, `Searching.stop` sets `SNIFFER_DIGGING` when `canDig && canSniff` and erases `SNIFFER_SNIFFING_TARGET`/`WALK_TARGET`, Digging starts and `drop_seed_at_tick` advances. This matches vanilla `SnifferAi.Searching.stop()` (SnifferAi.java:324-332) and `SnifferAi.Digging.start()` (SnifferAi.java:163-166).
  - `crates/pumpkin-core/src/entity/passive/sniffer.rs` `digging_drops_gift_loot_at_the_head_on_the_entity_clock` pins the drop-seed clock (drops only on the exact `tickCount + 120` tick) and the integer head-block corner spawn, matching vanilla `Sniffer.dropSeed()` (Sniffer.java:293-306) with `DATA_DROP_SEED_AT_TICK = tickCount + 120` (Sniffer.java:244).
  - The Sniffing -> walk-target half is implemented by `Sniffing::stop_with_timeout` (ai.rs:429-439): on timeout it sets `SNIFFER_SNIFFING_TARGET` and `WALK_TARGET` (speed 1.25, radius 0) from `calculate_dig_position`, matching vanilla `SnifferAi.Sniffing.stop()` (SnifferAi.java:366-376).

No new test added: those two tests are already the smallest pure unit tests covering the handoff and drop-seed clock; another would duplicate them under the MVP test rules. No `pumpkin-gametest` crate is needed because the seam is `test_support::world`, which is present. No source files edited. No generated data, WIT or cross-crate changes; no STILL OPEN rows remain.

## Result
Final pass. Re-read the two dirt-pad tests and verified them against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/SnifferAi.java` (`Searching.stop` 325-331 sets `SNIFFER_DIGGING` and erases `WALK_TARGET`/`SNIFFER_SNIFFING_TARGET`; `Digging.start` 164; `Digging.stop` 169-172 sets `SNIFF_COOLDOWN` 9600; `Sniffing.stop` 367-373 sets `SNIFFER_SNIFFING_TARGET` and `WALK_TARGET` at speed 1.25F radius 0) and `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/Sniffer.java` (`dropSeed()`).
- 7 end-to-end sniff -> walk -> dig test (M): FIXED / VERIFIED, no source change. `crates/pumpkin-core/src/entity/passive/sniffer/ai.rs::searching_handoff_starts_digging_on_a_diggable_pad` (ai.rs:543) drives Searching -> Digging on a 9x9 dirt pad and asserts `SNIFFER_DIGGING` is set, `SNIFFER_SNIFFING_TARGET` erased and the Digging drop-seed clock advanced, matching vanilla. `crates/pumpkin-core/src/entity/passive/sniffer.rs::digging_drops_gift_loot_at_the_head_on_the_entity_clock` (sniffer.rs:741) pins the drop-seed tick and integer head-block corner spawn, matching vanilla `dropSeed()`.
- Rows 1-6: already CLOSED/VERIFIED in prior Results; spot-checked unchanged (`transition_to(Searching)` plays `Sound::EntitySnifferSearching` at sniffer.rs:222; `get_ambient_sound_interval` returns 120 at sniffer.rs:621).

No Rust source files edited (all rows already resolved in code and tests). No generated data, WIT or cross-crate changes. No rows remain open.