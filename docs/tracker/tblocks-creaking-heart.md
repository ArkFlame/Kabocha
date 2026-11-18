# blocks/Creaking Heart - re-review

Source: crates/pumpkin-core/src/block/blocks/creaking_heart.rs (+ crates/pumpkin-core/src/block/entities/creaking_heart.rs, entity/mob/creaking.rs).
No decompiled vanilla sources in this worktree (.vendor/ absent, no vanilla repo in ../), so the rows below are
against vanilla CreakingHeartBlock / CreakingHeartBlockEntity as known from 1.21.4; re-check constants with the
Extractor before fixing the rows marked verify.

## Fixed in this pass

- block/entities/creaking_heart.rs:62-69 - the block entity had no ticker at all, so CREAKING_HEART_STATE was only
  re-evaluated on neighbour updates: a heart between two aligned pale oak logs stayed dormant all night and awake
  all day because the creaking_active environment attribute was never re-read. Vanilla CreakingHeartBlockEntity
  keeps the state in sync with the environment attribute (dusk/dawn flip). State evaluation was factored out of
  placed()/on_neighbor_update() into CreakingHeartBlock::update_state and is now also called from tick() every 20
  world-age ticks (vanilla creaking heart update delay, verify exact constant). Spawn sound still only on
  UPROOTED -> active, as before.
- block/blocks/creaking_heart.rs:59-88,102-110 - placed() carried its own copy of the state-evaluation body; it now
  shares update_state with on_neighbor_update, so placement and neighbour updates cannot diverge. No behaviour change.

## Remaining gaps

| file:line | vanilla | suggested fix | size |
|---|---|---|---|
| block/entities/creaking_heart.rs:62-69 | CreakingHeartBlockEntity tick: an AWAKE heart with a player within
  SPAWN_DISTANCE spawns a Creaking at a free position next to the heart, stores its UUID (setCreakingUUID) and plays
  block.creaking_heart.spawn. Nothing in core spawns a Creaking from a heart today | port the spawn branch: search a
  free position beside the heart, CreakingEntity::new, set_transient(heart_pos), heart_be.set_creaking_uuid(Some(id)),
  play spawn sound, broadcast entity spawn. Verify SPAWN_DISTANCE and the player-distance test | L |
| block/entities/creaking_heart.rs:83 (set_creaking_uuid) | never called, so is_protector() only ever sees None |
  call it from the spawn branch above and clear it on teardown (same gap as docs/tracker/tentities-creaking.md:27) | M |
| block/entities/creaking_heart.rs:77-81 (is_protector) | with no UUID stored, is_protector returns true, so any
  creaking whose home_pos is this heart is treated as protected | once the UUID is bound, require equality and drop
  the is_none_or() fallback | S |
| block/blocks/creaking_heart.rs:59-88 | vanilla tears down the bound creaking when the heart loses its logs or goes
  dormant/uprooted | when update_state() leaves AWAKE, despawn/tear down the stored creaking UUID and clear it | M |
| block/entities/creaking_heart.rs:62-69 (verify) | block.creaking_heart.idle heartbeat for an AWAKE heart;
  Sound::BlockCreakingHeartIdle is still unused in core | check whether vanilla plays it from animateTick (client
  only, then note it and skip) or from the server tick within 32 blocks, and wire the matching path | S |
| block/blocks/creaking_heart.rs:112-118 (broken) | vanilla CreakingHeartBlock has no break-sound override, the
  sound comes from SoundType.CREAKING_HEART.CREAKING_HEART_BREAK | keep for now (Pumpkin has no generic sound-type
  break playback); delete once break sounds come from the sound type or they double-play | S |
| block/blocks/creaking_heart.rs:91-100 (on_place) | vanilla getStateForPlacement sets AXIS from the clicked face
  only | matches; no change. Axis rotate/mirror: vanilla extends BaseEntityBlock and does not override them | - |

Note: the creaking heart state machine (logs tag, axis alignment, dormant/awake/uprooted, spawn sound on uproot)
matches vanilla apart from the rows above. No test added: this pass is not allowed to run cargo.

## Status (code-verified sweep)
FIXED: block entity tick re-syncs state every 20 ticks (block/entities/creaking_heart.rs tick); awake heart with a player in 32 blocks spawns a bound creaking (try_spawn_creaking); is_protector requires a bound UUID (no fallback). UNVERIFIED: spawn distance 32 and free-spot search are from memory. STILL OPEN: idle heartbeat, teardown of bound creaking when heart goes uprooted, break sound routing.

## Result

- idle heartbeat (Sound::BlockCreakingHeartIdle): VERIFIED, no code change. Decompiled
  net/minecraft/world/level/block/CreakingHeartBlock.java:64-73 plays CREAKING_HEART_IDLE only from
  animateTick (client-side), with 1-in-16 chance, CREAKING_ACTIVE true, state != UPROOTED and
  isSurroundedByLogs. No server tick path exists. Per Kabocha rule client-only sounds are noted and
  skipped, so this row is resolved by verification, not a code fix.
- teardown of bound creaking when heart goes uprooted: FIXED. Decompiled
  CreakingHeartBlockEntity.updateCreakingState (lines 140-149) uproots only when there are no
  required logs AND creakingInfo == null; a bound heart stays awake/dormant so its protector is
  never orphaned. CreakingHeartBlock::update_state now computes should_uproot(has_logs, bound)
  from the bound creaking UUID on the block entity, so a bound heart no longer reverts to UPROOTED
  when its logs are removed. Added pure unit test bound_heart_never_uproots_without_logs.
- break sound routing: FIXED. Decompiled CreakingHeartBlock.java has no break-sound override
  (playerWillDestroy only awards XP). Pumpkin already plays the break sound client-side from the
  block SoundType (generated break_sound = BlockCreakingHeartBreak, broadcast via the
  ParticlesDestroyBlock world event where Java predicts its own break effect), so the old
  block/blocks/creaking_heart.rs broken() play_sound(BlockCreakingHeartBreak) double-played. Removed
  it; the XP-drop logic in broken() is unchanged.

Verification: rustfmt --edition 2024 run on the touched .rs file (passed). Cargo/git deliberately not
run per worker contract, so compile/test result is UNVERIFIED. No generated data, WIT, cross-crate or
out-of-unit files were touched. All prior STILL OPEN rows are resolved; open = 0.


## Result - decompiled vanilla re-verification

Sources: .vendor/vanilla-decompiled/net/minecraft/world/level/block/CreakingHeartBlock.java and
.vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/CreakingHeartBlockEntity.java.

- Previously UNVERIFIED constants now confirmed exact against the decompiled source:
  PLAYER_DETECTION_RANGE = 32 (CREAKING_SPAWN_DISTANCE 32.0), ATTEMPTS_PER_SPAWN = 5,
  SPAWN_RANGE_XZ = 16, SPAWN_RANGE_Y = 8, ON_TOP_OF_COLLIDER_NO_LEAVES with check_collisions true
  (CreakingHeartBlockEntity.java:46-52, 202-215), DISTANCE_CREAKING_TOO_FAR = 34 (line 48).
  No change.
- idle heartbeat: confirmed client-only. CreakingHeartBlock.animateTick (lines 64-73) is the
  only CREAKING_HEART_IDLE path: 1-in-16, CREAKING_ACTIVE true, state != UPROOTED, and all six
  neighbours are logs. No server tick plays it. No change, per the client-only sound rule.
- teardown on uproot: confirmed updateCreakingState (lines 140-149) uproots only when there are
  no required logs AND creakingInfo is null; a bound heart stays awake/dormant. should_uproot
  (!has_logs && !bound) already matches. No change.
- break sound routing: confirmed CreakingHeartBlock has no break-sound override. No change.
- FIXED: spawn sounds were triggered at the wrong time. Vanilla serverTick (lines 118-120)
  plays Entity CREAKING_SPAWN on the creaking and Block CREAKING_HEART_SPAWN at the heart only
  after a protector actually spawns; the block updateState plays no sound. Removed the
  BlockCreakingHeartSpawn playback from CreakingHeartBlock::update_state (which fired on
  UPROOTED to awake on log placement) and added both sounds to try_spawn_creaking.
- FIXED: bound protector teardown on heart removal. Vanilla preRemoveSideEffects (lines 313-315)
  calls removeProtector(null), tearing down the bound creaking. Added a BlockEntity
  on_block_replaced override and a remove_protector helper on CreakingHeartBlockEntity.
- No new pure unit test: both new paths need a live world and entity, so there is no pure seam.
  The existing should_uproot, is_protector and too-far tests still cover the pure logic.

Verification: rustfmt --edition 2024 ran with exit 0 on both touched .rs files. Cargo and git
were deliberately not run under the worker contract, so compile/test result is UNVERIFIED.
No generated data, WIT, cross-crate or out-of-unit files touched. open = 0.

## Result (batch r2_018, CODER, partial)
- Spawn branch (L, entities/creaking_heart.rs:62-69): still OPEN in this table. Partial look: try_spawn_creaking at block/entities/creaking_heart.rs:155, spawn at :185-199. Not compared step by step with CreakingHeartBlockEntity.java.
- Spawn distance 32 (verify): VERIFIED premise. Vanilla CreakingHeartBlockEntity.java:46 PLAYER_DETECTION_RANGE = 32, :114 getNearestPlayerThatAffectsSpawning(..., 32.0). Rust CREAKING_SPAWN_DISTANCE = 32.0 at creaking_heart.rs:31.
- set_creaking_uuid never called (M): code now calls it at creaking_heart.rs:199 and clears at :150, :225, :229, :245. STILL OPEN until teardown path is compared.
- is_protector fallback (S): STILL OPEN. Current is_protector at creaking_heart.rs:129; not compared with vanilla this pass.
- Teardown on update_state leaving AWAKE (M, blocks/creaking_heart.rs:59-88): STILL OPEN, not compared.
- Idle heartbeat (S, verify): STILL OPEN, not checked against vanilla animateTick vs tick.
- Broken sound override (S): STILL OPEN, not checked.
- on_place axis (no change row): STILL OPEN, not re-read.
- Files edited: none. rustfmt: not run (no .rs touched).
## Result (batch r3_051, CODER, partial)
- Spawn branch (L, entities/creaking_heart.rs:62-69): STILL OPEN. Checked against vanilla CreakingHeartBlockEntity.java:113-120 and 202-215 (spawn gate, 32 range, trySpawnMob args, sounds, ENTITY_PLACE). Matches in Rust creaking_heart.rs:155-215. FIXED here: missing POOF broadcast (vanilla java:213), now send_entity_status(Poof) at creaking_heart.rs:199-204. Still open: timer. Vanilla ticker is nextInt(5)+20 (java:101-102), Rust gates on world_age % 20 (creaking_heart.rs:96).
- set_creaking_uuid call sites (M): VERIFIED. Bind at creaking_heart.rs:199, clear at :150, :225, :229, :245, matching vanilla setCreakingInfo/clearCreakingInfo (java:155-169) and removeProtector (java:317-331). New deviation (STILL OPEN): vanilla keeps a missing binding for 30 ticks (java:191-193) before clearing; Rust clears at once (creaking_heart.rs:224-226).
- is_protector fallback (S): VERIFIED. Rust creaking_heart.rs entity:129-133 requires equality, matching vanilla isProtector (java:333-335).
- Teardown when update_state leaves AWAKE (M, blocks/creaking_heart.rs:59-88): VERIFIED, no change. Vanilla updateCreakingState (java:140-149) never tears down, it only uproots when there are no logs and no creaking. Teardown is in the despawn branch (java:125-134). Rust should_uproot (blocks/creaking_heart.rs:97-100) and try_despawn_creaking (entities:220-247) match.
- Idle heartbeat (S, verify): VERIFIED earlier, client-only (CreakingHeartBlock.java:64-73). No change.
- Broken sound override (S): VERIFIED earlier. No break override in vanilla, and the double-play was removed in a prior pass.
- on_place axis (no change row): STILL OPEN, not re-read against getStateForPlacement this pass.
- Files edited: crates/pumpkin-core/src/block/entities/creaking_heart.rs (POOF broadcast, imports EntityStatus and ActorEventID).
- Checks: rustfmt --edition 2024 on the touched file, exit 0. cargo not run (forbidden by brief), so compile and clippy are UNVERIFIED.
