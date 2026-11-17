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
