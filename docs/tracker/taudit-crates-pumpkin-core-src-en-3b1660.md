# audit / crates/pumpkin-core/src/entity/passive/panda/lie_on_back.rs

Audited `panda/lie_on_back.rs` against vanilla `Panda.PandaLieOnBackGoal`. The file has no
TODO/FIXME/XXX markers: the tracker's "3 markers" are the doc-comment references to vanilla
(`lie_on_back.rs:1`, `:21`, `:27`, `:36`). No `.vendor/` Java tree and no decompiled `Panda.java`
exist in this worktree, so the vanilla facts below come from the vanilla class as known; each
uncertain item is tagged with its confidence. The sibling file `panda.rs` was audited separately
(`docs/tracker/taudit-crates-pumpkin-core-src-entity-pas.md`), which already lists "sit/lie-on-back
cooldowns and `reducedTickDelay` bounds" as parity.

## Verified at parity (no edit)

- `canUse`: `cooldown < tickCount && isLazy() && canPerformAction()`, then
  `nextInt(reducedTickDelay(400)) == 1`. `may_lie_down` (`lie_on_back.rs:46-50`) and
  `rolls_lie_on_back` (`:23-25`) do exactly that and `to_goal_ticks(400) == 200`
  (`ai/goal/mod.rs:78-80`), which is `reducedTickDelay(400)`; the selector runs
  `can_start`/`should_continue` only every second tick (`entity/mob/mod.rs:493-499`), so the
  halved bound is the right one.
- `canContinueToUse` short-circuits like vanilla: water first, then the non-lazy
  `nextInt(reducedTickDelay(600)) == 1`, then `nextInt(reducedTickDelay(2000)) == 1`.
  `keeps_lying` (`:29-34`) draws at most two numbers in the same order, so a panda that is
  already out wins no extra draw.
- `start` (`:75-80`) sets the synced `ON_BACK` flag (bit 4 of `DATA_ID_FLAGS`,
  `panda.rs:437-439`) and resets `cooldown` to 0.
- `stop` (`:82-87`) clears the flag and sets `cooldown = tickCount + 200`.
- Priority 8 with no `Controls`, no `tick` override and `should_run_every_tick` left at false;
  `tick_reach` returns an empty reach (`:89-92`), which is correct because the goal only reads
  and writes the panda itself.
- The unit test covers the 200/300/1000 bounds, the short-circuit and the draw count.

## Remaining gaps

| # | file:line | vanilla | suggested fix | size | confidence |
|---|---|---|---|---|---|
| 1 | `lie_on_back.rs:61,69` | vanilla draws from `this.panda.random`, the entity's `java.util.Random` shared with every other panda goal in tick order; `Mob::get_random()` (`entity/mob/mod.rs:1635`) returns the thread-local `ThreadRng` | give `Mob` an entity-owned `Random` and route `get_random` (plus the other vanilla-RNG call sites) through it. Distribution is identical, only seed replay differs, so this is repo-wide work rather than this goal's bug | L | high on the divergence, low value |
| 2 | `lie_on_back.rs:49` | if the vanilla revision in use does not put `canPerformAction` in `canUse`, then scared/eating/rolling/sitting pandas are blocked in Pumpkin but not vanilla. `panda.rs:644-652` claims the method is vanilla | confirm against decompiled `Panda.java` before editing; sibling audit rows 3 and 5 flag the same uncertainty | S | low |
| 3 | `lie_on_back.rs:47` | `tick_count` must be the already-incremented `Entity.tickCount` when the goal ticks | checked `mob/mod.rs:481-499`: `server_ai_step` reads the current count and runs the selectors after it. No off-by-one seen, kept only because it was never diffed against decompiled `Mob.serverAiStep` | XS | medium |

## Client-only / not blocking

Lying-on-back rendering is driven by the synced `DATA_ID_FLAGS` byte, which `set_on_back` already
updates through `PandaEntity::set_flag`.

## Gap status

- STILL OPEN: lie_on_back.rs random source and canPerformAction ordering (panda, excluded from this sweep by scope).

## Result

Re-checked the STILL OPEN rows against the decompiled vanilla now present at
`.vendor/vanilla-decompiled/net/minecraft/` (absent when this doc was first written).

- Row 2 (canPerformAction ordering) CLOSED, no edit. Decompiled
  `Panda.PandaLieOnBackGoal.canUse` (`panda/Panda.java:890-895`) is
  `cooldown < tickCount && isLazy() && canPerformAction() && nextInt(reducedTickDelay(400)) == 1`.
  `may_lie_down` (`lie_on_back.rs:46-50`, `cooldown < tick_count && is_lazy() && can_perform_action()`)
  plus `rolls_lie_on_back` in `can_start` (`lie_on_back.rs:54-63`) uses the same checks in the same
  order. The earlier uncertainty is resolved: this vanilla revision DOES include `canPerformAction`
  in `canUse`, so scared/eating/rolling/sitting pandas are correctly blocked. `canPerformAction`
  itself matches too: vanilla `panda/Panda.java:695-697`
  (`!isOnBack && !isScared && !isEating && !isRolling && !isSitting`) equals
  `panda.rs::can_perform_action` (`panda.rs:647-653`). Half-bound rolls also match:
  `Goal.reducedTickDelay = positiveCeilDiv(ticks, 2)` (`ai/goal/Goal.java:79-81`) gives 200/300/1000.
- Row 1 (random source) LEFT OPEN, reason: vanilla draws from the entity-owned `panda.random`
  (`panda/Panda.java:894,900-901`) while `Mob::get_random` (`entity/mob/mod.rs:1672-1673`) returns a
  thread-local `ThreadRng`; routing panda goals through an entity-owned `Random` is a repo-wide
  refactor touching a file outside this unit. Distribution is identical, only seed replay differs.
- Row 3 (tick_count) also checked: vanilla increments `tickCount` in `Entity.commonTick`
  (`Entity.java:966-978`) before `serverAiStep` runs the goal selectors (`Mob.java:838-845`), so the
  already-incremented count is correct. No off-by-one. No edit.

No `.rs` file changed, so no rustfmt was needed and no new unit test was added: the pure-function
tests at `lie_on_back.rs:95-115` already cover the roll bounds and short-circuit seam, and no
behaviour changed.


## Result

Independent verification against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/panda/Panda.java` (protocol 777). No `.rs` source edit was warranted in this unit's files; each STILL OPEN row is at parity or already resolved by infrastructure present in the main checkout.

- Row 1 (random source): RESOLVED by existing infrastructure, no edit. The note claimed `Mob::get_random` returned a thread-local `ThreadRng`, so panda goals did not share vanilla's entity-owned `panda.random`. The main checkout now has `EntityRng`: every `Entity` owns a `StdRng` (`entity/mod.rs:1803`, seeded at `:2171-2173`) shared across all of its goals, and `Mob::get_random` (`mob/mod.rs:1672-1674`) returns a handle to it. `lie_on_back.rs:61,69` therefore routes both rolls through that entity-owned, tick-order-shared RNG, matching vanilla (`Panda.java:894,900-901`). LEFT OPEN (out of unit): the entity RNG is seeded from runtime entropy rather than the entity UUID, so exact seed replay differs; changing the seed source requires `entity/mod.rs`, outside this unit's files.
- Row 2 (canPerformAction ordering): confirmed at parity, no edit. Decompiled `PandaLieOnBackGoal.canUse` (`Panda.java:890-895`) is `cooldown < tickCount && isLazy() && canPerformAction() && nextInt(reducedTickDelay(400)) == 1`; `may_lie_down` (`lie_on_back.rs:46-50`) plus the roll in `can_start` (`:54-63`) apply the same checks in the same order. `canPerformAction` (`Panda.java:695-697`) equals `panda.rs:654-660`.
- Row 3 (tick_count): confirmed at parity, no edit. Vanilla reads the already-incremented `tickCount` and runs the selectors after the increment, so there is no off-by-one.

No `.rs` file touched, so no rustfmt was needed and no new test added: the pure-function tests at `lie_on_back.rs:95-115` already cover the 200/300/1000 bounds and the short-circuit seam. No unwrap/expect/panic, println, or em dashes introduced.

Status: PASS fixed=0 open=1 files=none


## Result (second pass)
- Row 1 (random source): Resolved by existing EntityRng infrastructure; `lie_on_back.rs:61,69` routes through entity-owned `StdRng` via `mob.get_random()`, matching vanilla `Panda.java:894,900-901`. Seed source difference (runtime entropy vs entity UUID) is out of this unit's scope (requires `entity/mod.rs` changes).
- Row 2 (canPerformAction ordering): Confirmed at parity with decompiled vanilla; `canPerformAction` included in `canUse` with matching check order; `can_perform_action` (`panda.rs:647-660`) equals `Panda.java:695-697`.
- Row 3 (tick_count): Confirmed at parity; vanilla increments `tickCount` before goal selectors, already-incremented count is correct.
- No `.rs` files edited; existing tests at `lie_on_back.rs:95-115` cover the 200/300/1000 bounds and short-circuit seam.

## Result (batch r2_007, spot-check)
- Row 1 (random source): VERIFIED. Mob::get_random returns EntityRng (entity/mob/mod.rs:1681), vanilla Panda.java:894,900-901 uses entity-owned random. Seed-source difference remains out of scope.
- Rows 2 and 3: VERIFIED by prior pass; not re-read this pass.
