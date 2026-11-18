# Parity rows for crates/pumpkin-core/src/entity/mob/creaking.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Creaking** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-creaking** [partial/OWN-OPEN] own audit: 12 open, 9 fixed, 9 verified, 0 blocked-data (docs/tracker/tentities-creaking.md) docs/tracker/tentities-creaking.md

## Result

Re-reviewed both rows against the decompiled vanilla now present at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/creaking/Creaking.java`, `CreakingAi.java`,
`net/minecraft/world/entity/LivingEntity.java` and `net/minecraft/world/level/block/entity/CreakingHeartBlockEntity.java`.
No source change was required: every gap in `docs/tracker/tentities-creaking.md` was already resolved in earlier
passes and the current `crates/pumpkin-core/src/entity/mob/creaking.rs` matches vanilla. Per-row results below.

- **entities/Creaking (RE-REVIEW): VERIFIED.** Full-file audit against `Creaking.java` found no divergence:
  constants (62-74) and attributes (createAttributes 126-133) match; `doHurtTarget` sets 15 ticks + entity event 4
  (140-148); `hurtServer` heart-shake gating, 8-tick invulnerability, SHAKE event 66, ENTITY_ACTION and
  `creakingHurt` on player-responsible hits (151-181); `aiStep` counter decrements and canMove-flip
  freeze/unfreeze sounds + ENTITY_ACTION (212-238); `tick` heart-protection loss -> health 0 (241-258);
  `tickDeath` 45-tick twitch before tear-down (269-278); `checkCanMove` disguised-player gaze gating
  `(!active || !is_disguised) && is_looking_at_me` (475); `isLookingAtMe` constant cone threshold
  `dot > 1.0 - 0.5` (`LivingEntity.java:2276-2299`, adjustForDistance=false); `playerIsStuckInYou` counter >4
  (363-381); push/knockback gated on canMove (189-198, 454-458); ambient/hurt/death/step/attack sounds
  (424-446); `home_pos` NBT round trip (384-393). `navigation.setCanFloat(true)` (constructor line 90) is
  covered by `SwimGoal::can_start`, which sets it each tick (`entity/ai/goal/swim.rs:39`).

- **own/tentities-creaking (OWN-OPEN): all rows VERIFIED / already resolved.**
  - row 24 `player_is_stuck_in_you`: VERIFIED (`Creaking.java:363-381`); driven from the heart despawn tick as in vanilla.
  - row 25 `is_looking_at_me` LOS: already FIXED; constant threshold `1.0 - GAZE_CONE_SIZE` (coneSize 0.5,
    adjustForDistance=false, `Creaking.java:475-478`), three gaze heights (eye, y+0.5*scale, midpoint) and
    `ClipBlock::Visual` (`LivingEntity.java:2276-2299`). Known minor approximation (not a row): Pumpkin's
    `has_line_of_sight` uses eye-to-eye endpoints rather than vanilla's per-gaze-height endpoint; no per-height
    LOS API seam exists.
  - row 26 `do_hurt_target` attack animation: VERIFIED (`Creaking.java:140-148`), 15 ticks + entity event 4 +
    CREAKING_ATTACK on a landed hit (`Creaking.java:424-426`).
  - row 27 wander/melee canMove gating + 0.3 idle speed: VERIFIED (`CreakingAi.java`, `Creaking.java:135-137`);
    `MeleeAttackGoal::new(1.0, false)` gated by `can_move`, wander at `SPEED_MULTIPLIER_WHEN_IDLING` gated by `can_move`.
  - row 28 `followingTargetEvenIfNotSeen=false`: VERIFIED as already fixed.
  - row 29 heart UUID binding: VERIFIED as already fixed in `creaking_heart.rs` (outside this file, earlier pass).
  - row 30 death-sound ordering: VERIFIED, NO change. `creakingDeathEffects` -> `die()` broadcasts EntityEvent.DEATH
    (client plays CREAKING_DEATH) then plays CREAKING_TWITCH; `tearDown` (`Creaking.java:325`) replays
    `getDeathSound()`. Vanilla intentionally plays CREAKING_DEATH twice; Pumpkin's
    `on_die`/`creaking_death_effects`/`tear_down` flow matches.
  - row 31 unused eye-glow fields: already FIXED (fields/methods absent); vanilla runs `setupAnimationStates` /
    `checkEyeBlink` only when `level().isClientSide()` (`Creaking.java:262-265`).
  - row 32 creaking loot: VERIFIED resolved by generated data (no source edit). `pumpkin-data` generated
    `ENTITIES_CREAKING` loot table has empty pools and CREAKING `experience_reward` is 0, matching vanilla
    `xpReward = 0` (`Creaking.java:91`).

Client-only note (not a row, no server-state seam, do not block per AGENTS.md): `tear_down()` still spawns one
100-count CREAKING_HEART burst at `pos`, while vanilla `tearDown` (`Creaking.java:292-323`) spawns 100
PALE_OAK_WOOD + 10 CREAKING_HEART/AWAKE `BLOCK_CRUMBLE` bursts at the bounding-box center with spread 0.3 of
the box size. Visual-only; no gameplay state affected.

Files edited: none (verification only), so no rustfmt run. No pure unit test added: every pure seam
(`heart_shake_allowed`, `creaking_can_move`, `creaking_is_active`, `mob_knockback`) already has a test;
the remaining logic needs live world/player state or generated data. UNVERIFIED (no compile in this role;
orchestrator batch-checks).
