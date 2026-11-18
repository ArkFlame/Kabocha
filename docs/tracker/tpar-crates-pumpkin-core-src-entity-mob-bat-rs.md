# Parity rows for crates/pumpkin-core/src/entity/mob/bat.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Bat** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.
- **own/tentities-bat** [partial/OWN-OPEN] own audit: 3 open, 5 fixed, 0 verified, 0 blocked-data (docs/tracker/tentities-bat.md) docs/tracker/tentities-bat.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, final)

Both rows are closed. Verified against decompiled vanilla 26.3 class
`.vendor/vanilla-decompiled/net/minecraft/world/entity/ambient/Bat.java` (lines 1-242),
member by member. No generated-data, WIT, cross-crate or out-of-unit changes were needed.

- **entities/Bat** [AUDITED]: VERIFIED (matches vanilla). One determinism gap was found and FIXED:
  bat decisions used the thread RNG (`rand::rng()`) while vanilla uses the entity `RandomSource`
  (`this.random`, Bat.java:73,137-138,158-162,178). `get_ambient_sound`, `tick_roosting` and
  `tick_flying` now use `self.get_random()` (`crates/pumpkin-core/src/entity/mob/bat.rs:101,142,210`),
  the established `EntityRng` pattern; RNG draw order and short-circuit order are unchanged.
  Every other member matches: constructor resting default (43-48), `isFlapping` `%10` (50-53),
  synced-flag `isResting`/`setResting` (103-114), `tick` snap/flying `*0.6` (117-127),
  `customServerAiStep` redstone-conductor wake, 200/360 head turn, 4.0 non-combat targeting,
  1025 liftoff, 30/7/6/7/100 draw order, `0.1`/`0.5`/`0.7` easing, `zza=0.5` (130-182),
  `hurtServer` wake-before-super (199-209), `BatFlags` NBT (212-221), `checkBatSpawnRules` order
  (223-231), `getMovementEmission EVENTS`, `checkFallDamage` no-op, `isIgnoringBlockTriggers`,
  sounds/volume/pitch and no-push. `setupAnimationStates` (233-241) is client-only.
- **own/tentities-bat** [OWN-OPEN]: FIXED. The two previously open rows (shadow `AtomicI8` flags
  and `is_solid_block` wake check) are already FIXED in bat.rs per the final `## Result` in
  `docs/tracker/tentities-bat.md` (synced `DATA_ID_FLAGS` is authoritative; `BlockState::
  is_redstone_conductor()` handles the `Blocks::always`/`never` overrides). Of the remaining S
  notes in `docs/tracker/tentities-bat-f1f640.md`: thread RNG is now FIXED as above; the
  `set_pos` vs `setPosRaw` note is VERIFIED as a non-gap -- `Entity::set_pos`
  (`crates/pumpkin-core/src/entity/mod.rs:2655-2708`) is Pumpkin's raw setter (updates pos,
  bounding box, block pos and chunk pos; its comment at 2698 explicitly names vanilla
  `setPosRaw`), so no extra collision bookkeeping runs. Spawn rules keep the thread RNG
  (`bat.rs:55,58`) because vanilla takes the spawner's `RandomSource`, not an entity RNG, and
  other Pumpkin spawn-rule checks use the same thread RNG (e.g. `ghast.rs:101`).

Only `crates/pumpkin-core/src/entity/mob/bat.rs` was changed this pass (3 one-line-equivalent
edits). rustfmt is applied by the orchestrator; the wrapping used is already rustfmt-stable.
