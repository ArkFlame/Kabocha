# Tracker: entities / Zombie (shared base, `zombie/mod.rs`)

Reviewed: `crates/pumpkin-core/src/entity/mob/zombie/mod.rs` (855 lines) plus its
`zombie.rs`/`husk.rs`/`drowned.rs`/`zombie_villager.rs` subclasses and
`entity/mob/conversion.rs`.
Reference: vanilla `net.minecraft.world.entity.monster.Zombie` (no decompiled tree in this
worktree: `.vendor/` absent, so vanilla behaviour is from the class as known for the current
protocol target). The 4 ``vanilla`` markers in the file are doc comments, not TODOs.
Line numbers are post-fix.

## Fixed in this pass

- **Chicken jockey picked the first candidate, not a random one** (`roll_chicken_jockey`,
  :319-333). Vanilla collects the nearby chickens and consumes
  `random.nextInt(list.size())`, so the choice and the random stream differ. Now collects the
  candidates and indexes with `rng.random_range(0..len)`. Commit `fbf1619a6`.
- **Reinforcement caller charge stacked** (`summon_reinforcements`, was :529-542). Vanilla adds
  a modifier with the fixed id `minecraft:reinforcement_caller_charge`, which
  `addOrReplacePermanentModifier` replaces; Pumpkin read the old value and subtracted another
  0.05, so every further summon kept lowering the chance. Now replaces with `-0.05`. Commit
  `a4b1aafd2`.
- **Failed/skipped zombification suppressed the villager's loot** (`killed_entity`, :592-598).
  Vanilla `Zombie#killedEntity` returns `super.killedEntity(...)`, i.e. true, whatever
  `convertTo` returned; Pumpkin returned `convert_villager_to_zombie_villager(..).is_none()`,
  and `living.rs` treats a false return as "skip the victim's death loot". Now returns true.
  Commit `c4da7f652`.

## Remaining gaps

1. **Fire-on-hit may be over-gated by an empty main hand** — `do_hurt_target`, :564-578.
   Pumpkin only ignites the victim when the main hand is empty. Some vanilla versions apply the
   `isOnFire() && random < effectiveDifficulty * 0.3` roll unconditionally; if the current
   target's `Zombie#doHurtTarget` has no `getMainHandItem().isEmpty()` guard, an armed zombie
   never ignites. Needs the decompiled 1.21 `Zombie` to confirm; if absent, delete the
   `main_hand_empty` check — **S**.

2. **Chicken jockey candidate filter is stricter than vanilla's selector** —
   `roll_chicken_jockey`, :324-327. Vanilla filters with
   `EntitySelector.ENTITY_NOT_BEING_RIDDEN` (alive, no passengers); Pumpkin additionally
   requires `!has_vehicle()`, so a chicken that is itself riding something is skipped. Only
   reachable with command-spawned stacked chickens — **S**.

3. **Reinforcement condition/RNG ordering differs from vanilla** —
   `summon_reinforcements`, :457-480. Vanilla evaluates
   `random.nextFloat() < SPAWN_REINFORCEMENTS && difficulty == HARD && doMobSpawning` in that
   order (RNG first); Pumpkin checks difficulty, resolves the target, then rolls and only then
   checks `should_spawn_monsters()`. Same outcome, different random stream for later rolls.
   Relevant only to deterministic replays/parity tests — **S**.

4. **Halloween head uses the host process timezone** — `is_halloween`, :666-669. Vanilla reads
   the server level's local date; Pumpkin uses `OffsetDateTime::now_local()` (UTC fallback), so
   a server whose OS timezone differs from the intended world date can dress zombies a day
   early/late. Cosmetic — **S**.

## Gap status
- Fire-on-hit main-hand guard (zombie/mod.rs:564-578): BLOCKED-DATA, needs decompiled 1.21 Zombie.doHurtTarget.
- Chicken jockey candidate filter (zombie/mod.rs:332): FIXED, dropped the extra !has_vehicle() check to match ENTITY_NOT_BEING_RIDDEN. No unit test: filter needs a live world.
- Reinforcement RNG order (summon_reinforcements): STILL OPEN, S, changes the random stream; not landed this pass.
- Halloween timezone (is_halloween): STILL OPEN, S, needs the level-local date source.

## Result

Decompiled vanilla is now present at
`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/zombie/Zombie.java`
(the earlier audit had no `.vendor/`). Verified both STILL OPEN rows against it,
smallest first. Both live in files outside this unit's editable set
(zombie/mod.rs and mob/spawn.rs, neither in the allowed basename list), so no
source was edited; both are left open with reasons below.

- **Halloween timezone (`is_halloween`, zombie/mod.rs:297; defined in
  mob/spawn.rs:26)** - LEFT OPEN. Vanilla gates the head on
  `SpecialDates.isHalloween` reading the ServerLevel's date; Pumpkin uses
  `OffsetDateTime::now_local()` with a UTC fallback, i.e. the host OS clock.
  Fix needs spawn.rs (and a level-local date source threaded through mod.rs),
  both outside this unit's editable files. Cosmetic only.
- **Reinforcement condition/RNG order (`summon_reinforcements`,
  zombie/mod.rs:463-483)** - LEFT OPEN. Vanilla chain (Zombie.java:296-300) is
  `target != null && difficulty == HARD && random.nextFloat() < chance &&
  level.isSpawningMonsters()`; Pumpkin is `difficulty == HARD && resolve target &&
  random roll && should_spawn_monsters()`. Neither target resolution nor the
  difficulty check consumes RNG, so the random stream is in fact identical; the
  only divergence is the difficulty/target short-circuit order. Fix needs
  zombie/mod.rs, outside this unit's editable files. No behaviour change.

Extra verification from the newly available source (not STILL OPEN, no edits):

- Fire-on-hit main-hand guard (`do_hurt_target`, zombie/mod.rs:561) - the
  decompiled 1.21 `Zombie#doHurtTarget` (Zombie.java:349-353) DOES contain
  `this.getMainHandItem().isEmpty() && this.isOnFire() && random < difficulty*0.3`,
  so Pumpkin's existing `main_hand_empty` guard matches vanilla. The former
  BLOCKED-DATA row resolves as correct-as-is.
- No `.rs` files were edited, so rustfmt was not run.



## Result (second pass, verification only)

Re-checked both STILL OPEN rows against the decompiled vanilla source, smallest first.
Neither is a real gap in the current code, so no source file was edited and rustfmt was
not needed. No pure unit-test seam exists for either (both depend on wall-clock date or
live world state), so no test was added.

- **Halloween timezone (`is_halloween`, spawn.rs:26)** - RESOLVED as correct-as-is.
  Decompiled `SpecialDates.java:16-22` computes the date with `ZonedDateTime.now()`, i.e.
  the host process default timezone, NOT a ServerLevel-local date. Pumpkin's
  `OffsetDateTime::now_local()` (UTC fallback) is the equivalent host-local clock, so it
  already matches vanilla. The earlier "needs the level-local date source" premise was
  wrong: switching to a level date would actually diverge from vanilla. Cosmetic only.

- **Reinforcement condition/RNG order (`summon_reinforcements`, zombie/mod.rs:465-489)** -
  RESOLVED as correct-as-is. Decompiled `Zombie.java:293-301` short-circuits
  `target != null && difficulty == HARD && random.nextFloat() < chance && level.isSpawningMonsters()`.
  Pumpkin's current code resolves the target first (469-476), then difficulty (477-479),
  then rolls the RNG and checks `should_spawn_monsters()` in the same short-circuiting
  `||` (480-489): `should_spawn_monsters()` runs only when the roll passes, matching
  vanilla's trailing `&&`. Neither target lookup nor the difficulty read consumes RNG,
  so the random stream and short-circuit order are identical. The earlier "difficulty
  first" description was stale; the code already matches vanilla.

No `.rs` files were edited in this pass.
