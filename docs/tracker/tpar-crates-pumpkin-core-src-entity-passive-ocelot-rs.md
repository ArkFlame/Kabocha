# Parity rows for crates/pumpkin-core/src/entity/passive/ocelot.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Ocelot** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result
Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field.
- **entities/Ocelot [partial/RE-REVIEW]** FIXED/VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/feline/Ocelot.java`. Fixes: (1) `mobInteract` now gates feeding on the running tempt goal (vanilla line 169: temptGoal == null OR temptGoal.isRunning), via `goals_selector.is_running::<TemptGoal>()`; (2) `removeWhenFarAway` now honours `!hasCustomName() && !isLeashed()` (vanilla line 139, Paper patch), via pure helper `should_despawn_ocelot` with a boundary test. All other reviewed behaviour (goal priorities, `customServerAiStep` crouch/sprint at 0.6/1.33, `isSteppingCarefully` = crouch OR sneaking matching `Entity.isSteppingCarefully`, `isFood` OCELOT_FOOD tag, `Trusting` NBT + synced data, ambient interval 900, chicken/baby-turtle hunt targets) already matched vanilla. `spawnTrustingParticles` is client-only and intentionally omitted; taming is server-authoritative via `send_entity_status`.
