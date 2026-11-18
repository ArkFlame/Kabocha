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

## Gap status

- STILL OPEN: note has no gap rows; extraction empty, needs re-audit.


## Result

Re-audited both in-scope files against decompiled vanilla (`Activity`,
`Behavior`, `BehaviorControl`, `Brain`) on 2026-11-17.

- `activity.rs`: `activity_slot` correctly reproduces Java `HashMap` spreading
  (`h ^ (h >>> 16)`) and the load-factor resize (`activities > cap/4*3`), with the
  registry-name hash from `pumpkin_data::environment_attribute::Activity::name()`.
  `ALL_ACTIVITIES` matches the vanilla registry order exactly (26 entries, Dig=25).
  `ActivitySet` is a bitset over enum discriminants; `iter()` yields registry order,
  matching vanilla's `ActivityBitSet`. No code gap.
- `timed.rs`: `try_start`, `tick_or_stop`, `do_stop`, `timed_out` and defaults match
  vanilla `Behavior` step for step. `stop_with_timeout` passes `has_timed_out(tick.time)` = `time > end_timestamp`,
  identical to vanilla's `timedOut(timestamp)` inside `stop`; the sniffer Digging /
  FinishedDigging behaviors override it and use that flag (verified vs SnifferAi.java).
  The `.max(1)` clamp on the random range is a safe deviation (vanilla throws on a
  misconfigured max<min). No code gap.

The single STILL OPEN row was the meta note "note has no gap rows; extraction empty,
needs re-audit". Re-audit is now complete: no fixable gaps exist in these two files.

Fix applied: added a pure unit test `activity_slot_matches_java_hash_map_buckets` in
`activity.rs` pinning the Java HashMap bucket values for core=1, idle=5, fight=14 at
capacity 16 (verified against the Java algorithm and the existing mod.rs ordering
test). `rustfmt --edition 2024` run on `activity.rs`.

Remaining open: none in scope. Deviation rows #1-#3 in the parent note remain
deliberate / out-of-scope (memory ttl clock and debug formatting live in
`memory/mod.rs`; #1 is the documented hash-order deviation).

Re-verified 2026-11-18 against decompiled vanilla `Behavior.java` (tryStart/tickOrStop/doStop/timedOut match step for step) and `Activity.java` (name.hashCode + h^(h>>>16) + resize at 0.75 load factor). Bucket values core=1, idle=5, fight=14 confirmed by hand. Both tests present and correct; no new gaps, no .rs edits needed this pass.
