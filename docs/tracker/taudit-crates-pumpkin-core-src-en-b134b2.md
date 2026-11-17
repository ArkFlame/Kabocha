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
