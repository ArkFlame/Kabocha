# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-122988.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

- 8. f32/i64 `to_string` formatting: VERIFIED (fix present in tree).
- 9. Pause/resume/rate return value 1: VERIFIED (fix present in tree).
- 10. `/time add` negative amount bounds: VERIFIED (matches vanilla, no change).

## Result (final pass)

Re-verified all three rows against
`.vendor/vanilla-decompiled/net/minecraft/server/commands/TimeCommand.java` (26.3,
Paper/Canvas patched). All three fixes are already present in the current tree and
match vanilla; no further source change was needed. Smallest rows first.

- 10. `/time add` negative amount bounds: VERIFIED. Vanilla `addTime` registers the
  argument as `TimeArgument.time(Integer.MIN_VALUE)` (TimeCommand.java:104).
  Pumpkin's `add_node` uses `TimeArgumentType::new(i32::MIN)` (time.rs:903). The
  `time(0)` concern in the original doc applies to `/time set`, not `/time add`,
  so it does not apply. No change.
- 9. Pause/resume/rate return value 1: VERIFIED. Vanilla `setPaused` returns
  `Command.SINGLE_SUCCESS` (TimeCommand.java:281) and `setRate` returns
  `Command.SINGLE_SUCCESS` (TimeCommand.java:295); only set/add/query return
  `wrapTime`. Pumpkin returns `Ok(1)` for `Action::Pause | Action::Resume |
  Action::Rate` in both `ActionExecutor` (time.rs:494) and `execute_other_clock`
  (time.rs:606,633,667), while set/add/query use `wrap_time`. No change.
- 8. f32/i64 `to_string` formatting: VERIFIED. Vanilla `setRate` passes the raw
  float to `Component.translatable("commands.time.rate", clock, rate)`
  (TimeCommand.java:294), which Java renders via `Float.toString`. Pumpkin renders
  every rate with `super::data::java_float_to_string(rate)` (time.rs:469,476,483,
  645,655,662), a faithful Java `Float.toString` at f32 precision (data.rs:588)
  with the pure test `java_float_to_string_matches_java_format` (data.rs:1263).
  Tick counts are i64, where Rust `to_string` already matches Java `Long.toString`.
  No change.

No pure seam exists for a command executor return literal or an argument bound
beyond the existing `wrap_time` / `java_float_to_string` tests, so no new test was
added. No source files edited this pass; no compile run (orchestrator builds all).
