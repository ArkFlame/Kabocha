# entities / Sniffer (RE-REVIEW)
Source: `crates/pumpkin-core/src/entity/passive/sniffer.rs`, `sniffer/ai.rs` (implemented in d5dc0dd2d, #4535).
No decompiled `net.minecraft.world.entity.animal.sniffer.Sniffer` in this worktree (`.vendor/` absent, nothing under `../`). Rows are code findings plus vanilla knowledge, not mechanically confirmed. Line numbers are post-`ef4595b2f`.

## Re-review verdict
The source changed since the last assessment (`a494cb9b5` cap at 20, `db897ff2d` head-position loot). Those two earlier gaps are closed: `sniffer.rs:341` now takes 19 then prepends one (20 total), `sniffer.rs:361` spawns at the head block. No regression found in state/NBT persistence (`sniffer.rs:635-657`).

## Fixed in this pass
- **Dig game event used the head block centre instead of the entity position** (commit `ef4595b2f`). `mob_tick` emitted `minecraft:entity_action` at `get_head_block().to_centered_f64()`; vanilla `Entity.gameEvent(GameEvent)` (used by `Sniffer`) emits at `position()`. Now uses `Entity::emit_own_game_event` like `camel.rs:218` and `turtle.rs:386`. Affects sculk sensor / warden hearing only.

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
- 1 SNIFFER_SEARCHING sound never played (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 2 ambient sound interval (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 3 dig loot spawn position floored head (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 4 can_sniff/can_dig water test (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 5 explored-position success flag (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 6 dimension/pose constants (S): STILL OPEN (unverified: no .vendor decompile in h6; not changed)
- 7 end-to-end sniff-walk-dig test (M): STILL OPEN (size M)


## Result
Verified against `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/sniffer/{Sniffer,SnifferAi}.java` and `Mob.java` (now present in this worktree).

- **1 SNIFFER_SEARCHING sound (S): RESOLVED, no code change.** `transition_to(Searching)` already plays `Sound::EntitySnifferSearching` (`sniffer.rs:222-224`); the doc premise that it is unreferenced is stale. Vanilla `Sniffer.tick()` -> `playSearchingSound()` gates on `level().isClientSide()` and repeats every 20 ticks at volume/pitch 1.0, so vanilla clients self-generate the loop from the synced SEARCHING state and no server-side cadence is needed.
- **2 ambient sound interval (S): FIXED.** Vanilla `Sniffer` does not override `getAmbientSoundInterval()`; `Mob` default is 80 (Pumpkin `MobEntity::DEFAULT_AMBIENT_SOUND_INTERVAL` = 80). `get_ambient_sound` already matches vanilla (idle for every state except Digging/Searching). Changed `get_ambient_sound_interval` from 120 to 80 in `sniffer.rs`. No unit test added: a constant-mirroring assertion is excluded by the MVP test rules.
- **3 dig loot spawn position (S): VERIFIED, no change.** Vanilla `dropSeed()` spawns each `ItemEntity` at `head.getX(), head.getY(), head.getZ()` (integer block corner from `getHeadBlock()`); Pumpkin `get_head_block().0.to_f64()` matches. The existing `digging_drops_gift_loot_at_the_head_on_the_entity_clock` test already pins this position.
- **4 can_sniff/can_dig water test (S): VERIFIED, no change.** Both vanilla `canSniff()` and `canDig()` gate on `!this.isInWater()`, not `isInWaterOrRain()`; Pumpkin uses `!entity.is_in_water()` in both predicates.
- **5 explored-position success flag / cooldown (S): VERIFIED, no change.** Pumpkin `ai.rs` matches `SnifferAi`: Digging (160-180 ticks) sets `SNIFF_COOLDOWN` for 9600 ticks on timeout else `reset_sniffing`; FinishedDigging (40 ticks) starts RISING then on stop calls `onDiggingComplete(timed_out)`, erases SNIFFER_DIGGING and sets SNIFFER_HAPPY. Durations, cooldown and entry conditions all match.
- **6 dimension/pose constants (S): VERIFIED, no change.** Vanilla `DIGGING_BB_HEIGHT_OFFSET 0.4`, `withEyeHeight(0.81)`, `DIGGING_DIMENSIONS` applied only while DIGGING, head distance 2.25 and `MAX_HEAD_Y_ROT 50` all match the Pumpkin constants and `refresh_state_dimensions`.
- **7 end-to-end sniff -> walk -> dig test (M): STILL OPEN.** Reason: needs a `pumpkin-gametest` over a dirt pad with multi-tick state transitions; no minimal pure unit-test seam exists in these files, and it exceeds this pass's scope.

Files edited: `crates/pumpkin-core/src/entity/passive/sniffer.rs` (one literal + one explanatory comment). No generated data, WIT, or cross-crate changes; no other basename in this unit needed edits.
