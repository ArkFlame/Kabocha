# entities / mob / neutral.rs - audit

Audited `crates/pumpkin-core/src/entity/mob/neutral.rs` (442 lines) plus its callers
(`entity/mob/mod.rs:3136,3325,3346`, `entity/ai/goal/active_target.rs`,
`entity/ai/goal/reset_universal_anger.rs`, `entity/player.rs:6229`) and the
implementors (bee, wolf, iron_golem, polar_bear, zombified_piglin, enderman).
No decompiled vanilla tree in the worktree (`.vendor/` absent); vanilla behaviour
is from knowledge of 1.21.x `NeutralMob` / `AngerManagement` / `Mob`, not 26.3 sources.

## Fixed in this pass
- neutral.rs:128 (former TODO) - `set_anger_end_time` never synced
  `DATA_ANGER_END_TIME`, so bee/wolf never showed the angry pose on clients.
  Added `NeutralMob::anger_end_time_data()` (default `None`) and a
  `set_synced_data` call in `set_anger_end_time`; implemented for bee
  (`bee::DATA_ANGER_END_TIME`, id 19, LONG) and wolf (id 22, LONG). Those are the
  only two entities in `pumpkin-data`'s generated `tracked_data.rs` that define the
  accessor. Client-side visual only; `write_anger_nbt`/`read_anger_nbt` already
  went through `set_anger_end_time`, so a restored grudge is synced too.

## Matches vanilla (no action)
- `DEFAULT_ANGER_DURATION` 400..=780 = `TimeUtil.rangeOfSeconds(20, 39)`,
  fresh roll per grudge (`startPersistentAngerTimer`).
- `NO_ANGER_END_TIME = -1`, `isAngry()` end-time comparison (end tick already calm),
  legacy `AngerTime` countdown -> absolute end time on load.
- `stopBeingAngry`: clears last-hurt-by, grudge uuid, target (ForgotTarget reason)
  and end time, in vanilla order.
- `isAngryAt`: `canAttack` first, then universal anger for unfocused grudges, then
  uuid equality - matches `NeutralMob.isAngryAt` including the
  `universal_anger && target == null && isAngry()` guard.
- `playerDied`: gamerule gate plus uuid equality, called from a 32/10/32 search in
  `tellNeutralMobsThatIDied` (player.rs:6229).
- NBT keys `anger_end_time` (long) / `angry_at` (uuid) and the post-load target
  restore (`restore_target`), which vanilla does lazily once the entity ticks.
- Dead-target handling keeps the `target instanceof Mob` guard, i.e. dying players
  are forgiven only through the gamerule path.

## Remaining gaps
1. neutral.rs:228-290 - timer refresh semantics. Vanilla `updatePersistentAnger`
   decrements the remaining time when the target uuid is unchanged and only calls
   `startPersistentAngerTimer` for a *new* target; here `stays_angry_with_target()`
   is true by default and re-rolls the timer every tick while the target lives, so
   non-bee neutral mobs effectively never calm down while the target stays alive
   (bee overrides it to false). Needs the decompiled 1.21.x `AngerManagement` to
   settle, then a per-mob flag or a countdown. Size M.
2. neutral.rs:314-333 - the death notice box starts from the player's block pos
   (1x1x1) instead of the player bounding box, so the searched box is 0.6 blocks
   narrower in x/z and 0.9 lower at the top than vanilla
   `getBoundingBox().inflate(32, 10, 32)`. Cosmetic boundary difference. Size S.
3. neutral.rs:140-143 - `start_persistent_anger_timer_at` uses `rand::random_range`
   (thread rng) rather than the world/entity random; vanilla samples
   `PERSISTENT_ANGER_TIME.sample(level.getRandom())`. Only matters for seeded
   determinism / tests. Size S.
4. Bee/wolf still do not `SynchedEntityData::define` the anger accessor, so a mob
   spawned calm carries no metadata entry until its anger changes (client default
   -1 makes this invisible in practice). Add a `define` next to the other
   `mob_init_data_tracker` values if spawn-packet parity is wanted. Size S.
5. Iron golem, polar bear and zombified piglin implement `NeutralMob` but have no
   `DATA_ANGER_END_TIME` in the generated `tracked_data.rs`. Verify with the
   extractor whether vanilla defines that accessor for them; if it does, the
   generated data needs regenerating (codegen/extractor), not a hand edit. Size M.
6. `write_anger_nbt` always writes `anger_end_time` (even -1) and only writes
   `angry_at` when present; unverified against 1.21.x `addPersistentAngerSaveData`
   without the decompiled source. Harmless but not byte-identical. Size S.
