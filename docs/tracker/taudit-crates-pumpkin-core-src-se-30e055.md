# server/scheduler.rs — deep audit

Source: crates/pumpkin-core/src/server/scheduler.rs (306 -> 352 lines, adds persistence since #4374).
Unit under audit: the `/schedule` function queue, vanilla equivalent `ServerFunctionManager` + `TimerQueue` (tick consumer, `schedule(function, triggerTime, append)`, saved data `scheduled_events.dat` with `events[]` of `{Name, TriggerTime, Callback{Type, Name}}`).
The tracker note "2 TODO/FIXME/vanilla markers" refers to the two doc comments at scheduler.rs:112 and :164; there are no TODO/FIXME markers in the file.

## Fixed in this audit
- scheduler.rs:187: an event whose `Callback.Type` was neither `minecraft:function` nor `minecraft:function_tag` was silently restored and executed as a plain function. Vanilla's TimerQueue only decodes those two types; unknown callbacks are now dropped with a warning. Test: `saved_event_with_unknown_callback_type_is_dropped`.
- scheduler.rs:232: a scheduled function that fails (`FunctionRunError`: chain limit, macro without arguments, unknown function after a datapack reload) was discarded with `let _ =`, invisible in logs. Now logged with the function id and error.

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | crates/pumpkin-core/src/command/commands/schedule.rs:34,86 | `/schedule function` rejects macro functions at schedule time with `commands.schedule.macro` | `ERROR_MACRO` is declared but dead (`#[expect(dead_code)]`), so a macro is accepted, stored, and then fails on every run (only a log line, no feedback to the command source). Fix: reject when the resolved function has any line starting with `$`, using a datapack accessor for macro lines. | S |
| 2 | crates/pumpkin-core/src/command/commands/schedule.rs:64,86 | The function argument is a `FunctionArgument`, parsed against the datapack, so an unknown id fails with `commands.function.unknown` / `commands.function.tag.unknown` | Pumpkin parses the id as a plain string with a suggestion provider, so `/schedule function nope:missing 10t` succeeds and then does nothing at tick. Fix: validate against `DatapackManager` in `execute_schedule`. | S |
| 3 | crates/pumpkin-core/src/command/commands/schedule.rs:66,87 | The schedule id is a `ResourceLocation`, so it is lowercased/normalised and `#NS:Foo` cannot coexist with `#ns:foo` | Ids are raw strings, so `/schedule function NS:Foo 10t` and `/schedule function ns:foo 10t` create two entries and `/schedule clear` matches only the exact spelling. Fix: normalise with the existing id parser before storing. | XS |
| 4 | crates/pumpkin-core/src/command/commands/schedule.rs:69-73,118 | Feedback and the command result report the trigger in game time (`server.getTickCount()` at schedule time + delay) | Pumpkin reports `tick_count + delay` and returns `tick_time % i32::MAX`, so the "(at N)" number differs from vanilla and from the `TriggerTime` written to disk. Fix: use `overworld_game_time()` for display/result while keeping `tick_count` for the queue. | XS |
| 5 | crates/pumpkin-core/src/server/scheduler.rs:115 + server/mod.rs:1905-1920 | Vanilla writes level.dat and its timer data from the same game time on the server thread | Saves run on `spawn_blocking` while ticks continue: `save_world_info()` writes `world_age` N, then `to_saved_nbt` converts with a later `world_age` M, so every restored event is pushed back by M-N ticks. Fix: read the overworld age once and pass it into both writes. | S |
| 6 | crates/pumpkin-core/src/server/scheduler.rs:124-128,218-221 | Vanilla cannot save mid-tick, so an event is either queued or already run | `running` is the whole due batch, not just the executing event, and in-flight events are saved as "due now": if a save lands after A ran but before the remaining due events finished, a crash re-runs A. Fix: keep only the currently executing event in `running`. | S |
| 7 | crates/pumpkin-core/src/server/scheduler.rs:24-28 | `TimerQueue` entries compare on trigger time only | `Ord` compares `trigger_tick` only while derived `PartialEq` compares every field, breaking the `Eq`/`Ord` contract (`a.cmp(b) == Equal` but `a != b`). Harmless in `BinaryHeap`; a hazard for future dedup/sorted use. Fix: hand-write `PartialEq` on `trigger_tick`. | XS |

## Checked, no change
- Heap order: `Ord` reverses `trigger_tick` so `BinaryHeap` peeks the earliest event; `tick` pops every event with `trigger_tick <= current_tick` exactly like `TimerQueue.tick(gameTime, consumer)`.
- `schedule(replace)` pops and re-pushes all entries with a different id, equivalent to vanilla's replace/append split; load path deliberately does not dedupe so `append` duplicates survive a restart.
- Tick/game-time conversion in `to_saved_nbt`/`load_saved_nbt` is lossless for equal ages and clamps overdue events to "next tick", matching vanilla's past-due behaviour; the `data` root wrapper and `events` list match the stub written by `write_scheduled_events_stub`.
- Lock order is consistently `queue` then `running` in every path (schedule/remove/get_event_ids lock only `queue`), so no deadlock; functions run outside the queue lock, so a scheduled function may safely call `/schedule`.
- Tags: `#`-prefixed ids are stored in `Callback.Name` without the prefix and restored with it, and `resolve_functions` resolves `#tag`; a scheduled tag therefore runs its members.
