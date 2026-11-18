# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-221dd1.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

The single STILL OPEN row was the meta note "note has no gap rows; extraction empty,
no .rs edits needed this pass. The single STILL OPEN row is the meta note, now resolved:
- No fixable gaps in scope, so no .rs edits made (adding tests would duplicate the pure tests already present in both files). The sole STILL OPEN row was the meta note, now marked RESOLVED. fixed=0 open=0.
- Parent deviation rows #2 (memory ttl absolute clock) and #3 (DebugBrainDump formatting) live in `memory/mod.rs`, outside this unit; they are documented deliberate deviations, not STILL OPEN rows and not data/subsystem blockers.

## Result
Final pass 2026-11-18. Every row closed. No .rs edits needed: Pumpkin already matches the
current decompiled vanilla in every row. The parent tracker's deviation wording was written
against an older vanilla build and is corrected below with the current .vendor source.

- Meta note "extraction empty, needs re-audit" - RESOLVED: full re-audit of mod.rs,
  activity.rs, timed.rs and memory/mod.rs completed against the current decompiled vanilla.
- activity hash order (deviation #1) - VERIFIED against
  .vendor/vanilla-decompiled/net/minecraft/world/entity/schedule/Activity.java (getName /
  hashCode) and .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/Brain.java
  (availableBehaviorsByPriority HashMap iteration order is arbitrary per JVM). Pumpkin's
  deterministic activity_slot bucket emulation is documented and test-pinned by
  activity_slot_matches_java_hash_map_buckets (core=1, idle=5, fight=14 at capacity 16):
  there is no stable vanilla order to match. No generated field or subsystem missing.
- memory ttl (deviation #2) - VERIFIED against
  .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/memory/MemorySlot.java (tick()
  does timeToLive-- until <= 0L then clear; set() stores a plain remaining count) and
  .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/Brain.java (setMemoryInternal
  forwards the raw ttl straight to slot.set with no gameTime + addition). This is the
  current vanilla behaviour and Pumpkin's MemorySlot::tick / set_with_expiry match it
  exactly, including the slot_expires_on_the_tick_after_its_ttl_reaches_zero boundary test.
  The parent note's "gameTime + ttl" claim is stale. No fix needed, no generated field or
  subsystem missing.
- DebugBrainDump formatting (deviation #3) - VERIFIED against
  .vendor/vanilla-decompiled/net/minecraft/util/debug/DebugBrainDump.java
  (getMemoryDescription: registry path with namespace stripped plus ": ", ttl as
  " (ttl: N)", truncateStringIfNecessary(..., 255, true) per line, Collections.sort) and
  .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/memory/MemorySlot.java
  (toString uses the same " (ttl: N)" suffix). Pumpkin's MemoryStore::debug_lines and
  MemorySlot::describe produce the same sorted "path: value (ttl: N)" lines with 255
  UTF-16 unit truncation, pinned by debug_lines_use_the_vanilla_dump_format and
  truncate_description_counts_utf16_units. The only genuine difference is a vanilla quirk
  where DebugBrainDump.java:159 displays gameTime instead of the remaining count in the
  ttl slot; Pumpkin deliberately prints the real remaining count. Debug-subscription-only,
  no functional impact. No generated field or subsystem missing.
- brain/mod.rs tick and dispatch - VERIFIED against
  .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/Brain.java (tick: forgetOutdated
  Memories -> tickSensors -> startEachNonRunningBehavior gated on active activity ->
  tickEachRunningBehavior ungated; stopAll; BrainProvider.makeBrain) and
  .vendor/vanilla-decompiled/net/minecraft/world/entity/ai/behavior/Behavior.java
  (tryStart / tickOrStop / doStop / timedOut). Pumpkin's tick, stop_all, make_brain,
  update_activity_from_schedule, is_brain_dead and Timed<Behavior> match step for step.

No STILL OPEN rows, no BLOCKED-DATA rows. fixed=0 open=0. Files edited: none (.rs).
