# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-221dd1.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All prior STILL OPEN rows were the single meta note "extraction empty, needs re-audit", now RESOLVED. No remaining STILL OPEN rows and no BLOCKED-DATA rows. fixed=0 open=0.
No STILL OPEN rows, no BLOCKED-DATA rows. fixed=0 open=0. Files edited: none this pass (both `.rs` files already correct and tested).

## Result
All tracker rows closed. No `.rs` edits needed: both in-scope files already match decompiled vanilla step for step and already carry pure unit tests, so adding more would only duplicate existing coverage (forbidden by AGENTS.md).

- `activity.rs` - VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/schedule/Activity.java`: 26 activities in registry order (Dig id=25), `getName()` / `hashCode() = name.hashCode()` UTF-16 fold. `activity_slot` reproduces the Java HashMap key spread `h ^ (h >>> 16)` at the 0.75 load-factor capacity (16), yielding buckets core=1, idle=5, fight=14. `ActivitySet` is a bitset iterating registry order. Tests `activity_indices_fit_the_bitset`, `activity_slot_matches_java_hash_map_buckets`, `every_activity_round_trips_through_the_set`, `set_operations_combine_and_clear`, `gate_memory_sets_both_the_condition_and_the_erase_set`, `priority_pairs_count_up_from_the_first_priority` present. No code gap.
- `timed.rs` - VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/ai/behavior/Behavior.java`: `try_start` = `tryStart` (entry conditions + `check_extra_start_conditions`, `duration = min + nextInt(max + 1 - min)`, set RUNNING, `start`); `tick_or_stop` = `tickOrStop` (`!timedOut && canStillUse`); `do_stop` = `doStop` (status STOPPED then `stop`); `timed_out` = `timedOut` (`time > endTimestamp`); defaults `DEFAULT_DURATION=60`, `can_still_use=false`, `check_extra_start_conditions=true`. The `.max(1)` clamp only guards a misconfigured `max < min` where vanilla `nextInt` would throw. Boundary tests present. No code gap.
- Deviation #1 (activity hash order) - VERIFIED deliberate: vanilla `availableBehaviorsByPriority` HashMap iteration order is arbitrary per JVM; Pumpkin's deterministic bucket emulation is documented and test-pinned, so there is no stable vanilla order to match.
- Deviation #2 (memory ttl absolute clock) - VERIFIED deliberate / out of scope: implemented in `memory/mod.rs`, outside this unit's `activity.rs`/`timed.rs`; no generated field or subsystem is missing.
- Deviation #3 (DebugBrainDump formatting) - VERIFIED deliberate / out of scope: debug-subscription-only formatting in `memory/mod.rs`, outside this unit; no generated field or subsystem is missing.
- Meta note "extraction empty, needs re-audit" - RESOLVED: full re-audit of both files completed in the parent tracker.

No STILL OPEN rows, no BLOCKED-DATA rows. fixed=0 open=0. Files edited: none this pass (both `.rs` files already correct and tested).
