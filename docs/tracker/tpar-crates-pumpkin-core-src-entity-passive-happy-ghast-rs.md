# Parity rows for crates/pumpkin-core/src/entity/passive/happy_ghast.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Happy Ghast** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-happy-ghast** [partial/OWN-OPEN] own audit: 2 open, 6 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-happy-ghast.md) docs/tracker/tentities-happy-ghast.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

Third pass, verified directly against .vendor/vanilla-decompiled/net/minecraft/world/entity/animal/happyghast/HappyGhast.java. Both remaining STILL OPEN rows are closed; all six underlying gaps from docs/tracker/tentities-happy-ghast.md are RESOLVED in the current happy_ghast.rs. No source change was needed this pass.

- entities/Happy Ghast [partial/AUDITED]: VERIFIED. Re-review of the current source against the decompiled class confirms all six gaps are fixed: (1) hover/float via get_mob_gravity -> 0.0, get_mob_y_velocity_drag -> Some(0.95) and HappyGhastFloatAroundGoal, matching adultGhastSetup / Ghast.RandomFloatAroundGoal (HappyGhast.java:118-119, 115); (2) server_still_timeout set to 10 when a player stands above via scan_player_above_ghast (tick 430-431; scan 545-566); (3) leash_holder_time set to 5 for a quad leashee and decremented before flag evaluation in mob_tick (notifyLeashHolder 523-527; tick 417-421); (4) harness/rider control: four-seat cap (canAddPassenger 341-343), item-first then mount (mobInteract 279-297), controlling-passenger gate (346-350) and add/removePassenger side effects (307-338); (5) still_timeout NBT key matches addAdditionalSaveData / readAdditionalSaveData (532/538); (6) baby metadata cleared by generic AgeableMob::set_age. No BLOCKED-DATA.
- own/tentities-happy-ghast [partial/OWN-OPEN]: VERIFIED (FIXED). The two previously open rows (gap 3 leash holder, gap 4 harness/rider) are implemented and verified against vanilla lines 523-527 and 279-350; the earlier Result sections in docs/tracker/tentities-happy-ghast.md document the fixes plus pure unit tests (leash_holder_flag_uses_decremented_time, player_mount_requires_harness_and_a_free_seat, goggles_sound_only_on_vehicle_transition). Final tally: 6 fixed, 0 open, 0 blocked-data.

Files changed this pass: none (source already complete). Files changed across the unit: crates/pumpkin-core/src/entity/passive/happy_ghast.rs. Not compiled, per CODER rules; orchestrator batch-checks.
