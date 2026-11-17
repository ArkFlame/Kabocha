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
