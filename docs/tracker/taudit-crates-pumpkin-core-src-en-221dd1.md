# audit / ai/brain/mod.rs — deep audit

Source: `crates/pumpkin-core/src/entity/ai/brain/mod.rs` (1046 lines). No `TODO`/`FIXME`/`todo!`
markers; the three tracker "vanilla" hits are explanatory comments (lines 50, 440, 817).
Compared against vanilla `Brain` (tick order, activity switching, schedule refresh,
`startEachNonRunningBehavior` / `tickEachRunningBehavior` / `stopAll`, `addActivity`,
`codec`/`PackedMemories`, `isBrainDead`, `DebugBrainDump`) plus the submodules it drives
(`activity.rs`, `behavior/mod.rs`, `behavior/timed.rs`, `memory/mod.rs`).

## Verified equivalent (no change needed)
- `Brain::tick` order matches `Brain.tick`: forget outdated memories (`memories.tick`,
  mod.rs:332) -> sensors (:334) -> start non-running behaviors gated on
  `activeActivities` (:344-360) -> tick running behaviors without an activity gate
  (:361-376), i.e. a behaviour whose activity just stopped keeps ticking until
  `canStillUse` fails, as in vanilla.
- `add_activity` merges per-(priority, activity) in insertion order (`LinkedHashSet` in
  vanilla), registers required memories and stores the requirement/erase slices
  (:289-322). Re-adding the same activity keeps a previously stored erase set, which is
  also vanilla's behaviour for the 4-arg overload.
- `set_active_activity` order (erase other activities' memories -> clear -> core set ->
  insert) matches `Brain.setActiveActivity` (:256-277); `set_active_activity_if_possible`
  falls back to the default activity (:239-245); `set_active_activity_to_first_valid`
  leaves the current activity when none match (:247-254).
- `update_activity_from_schedule` matches `Brain.refreshActivities` (20-tick gate, only
  switches when the scheduled activity is not already active), including the 1.21.5+
  environment-attribute activity lookup (:208-220).
- `stop_all` matches `Brain.stopAll`: priority order, only RUNNING behaviours, state
  writes kept (:379-396, test :790).
- `make_brain` mirrors `BrainProvider.makeBrain` (register provider memories + sensor
  requires + behaviour requires, then pack load, then core = {CORE},
  `useDefaultActivity`) (:466-500).
- `is_brain_dead` uses the same three emptiness checks (:398-401).

## Deliberate (non-gap) deviations, debug or ordering only
| # | Location | Vanilla | Note | Size |
|---|---|---|---|---|
| 1 | `activity.rs:7-20` | `availableBehaviorsByPriority` is a `HashMap<Integer, HashMap<Activity, Set<..>>>`; activity order is enum identity-hash order, i.e. arbitrary per JVM run | Pumpkin sorts groups by a Java-`HashMap` bucket emulation over `Activity::name().hashCode()`. Deterministic, test-pinned (:817), and vanilla has no stable order to match. | none |
| 2 | `memory/mod.rs:124-132`, `mod.rs:332` | Expiry is `gameTime + ttl` compared to the world clock, so time passes while the brain is not ticked (unloaded chunk, reload) | ttl is decremented once per brain tick and persisted as a remaining count, so a paused brain resumes with its remaining ttl instead of expiring on load. Only observable across save/load of an expiring memory; changing it needs an absolute clock plumbed through `pack`/`BrainTick`. | S |
| 3 | `memory/mod.rs:137-145`, `mod.rs:440-458` | `DebugBrainDump` prints `type=value` for every registered memory, with the absolute expiry time | Prints `type: value (ttl: N)` and strips the namespace; same lines, different formatting. Debug subscription only. | S |

## Remaining gaps
None found: every code path in this file either matches the vanilla `Brain` call chain or is
one of the deliberate deviations above. No code change was made.
