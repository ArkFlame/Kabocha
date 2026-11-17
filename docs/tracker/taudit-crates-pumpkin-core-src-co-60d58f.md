# Audit: crates/pumpkin-core/src/command/argument_types/objective.rs

Coverage: AUDIT. Tracker note "3 TODO/FIXME/vanilla markers" = the three doc-comment
references to vanilla at lines 74, 85 and 92. All three claims were checked against
vanilla `ObjectiveArgument` / `ObjectiveCriteria` and are accurate; there is no
TODO/FIXME in the file.

## Verified correct (no change needed)

- `parse` (objective.rs:38-41) uses `StringReader::read_unquoted_string`; the allowed
  charset (pumpkin-command/src/string_reader.rs:164) matches vanilla
  `StringReader.isAllowedInUnquotedString` exactly (`0-9 A-Z a-z _ - . +`).
  Vanilla `ObjectiveArgument.parse` is a bare `readUnquotedString`.
- `get` (objective.rs:70-72) mirrors `ObjectiveArgument.getObjective(ctx, name)`
  -> `StringArgumentType.getString`.
- `objective_or_error` (objective.rs:76-83) mirrors `ObjectiveArgument.getObjective`,
  including order: lookup first, then `arguments.objective.notFound` with the name.
- `writable_objective_or_error` (objective.rs:87-98) mirrors
  `ObjectiveArgument.getWritableObjective`: not-found check first, then
  `arguments.objective.readonly`.
- `criterion_is_read_only` (objective.rs:24-29): vanilla `ObjectiveCriteria` registers
  only health, food, air, armor, xp, level as read-only. dummy, trigger, deathCount,
  playerKillCount, totalKillCount, teamkill.<color>, killedByTeam.<color> are writable,
  which the existing table test asserts (commands/scoreboard.rs:1272-1307).
- `examples` matches vanilla `EXAMPLES` ("objective", "trigger_obj").

## Remaining gaps (not fixed here)

1. Objective suggestions use segment/substring matching, not vanilla prefix matching.
   - File:line: argument_types/objective.rs:58 -> `Suggestions::filter_and_suggest_one`.
   - Cause: pumpkin-command/src/suggestion/suggestions.rs:218-227 `matches_substr`
     advances past `.`/`_`/`/`/`:`, so typing `red` suggests `team_red`.
   - Vanilla: `CommandSource.suggestMatching` is `candidate.toLowerCase().startsWith(input)`.
   - Fix: add a prefix-only filter and use it for objective names (and the equivalent
     scoreboard_slot/team/score_holder/operation call sites after per-type checks).
   - Size: M (shared helper, many argument types).

2. `list_suggestions` panics when the source has no world.
   - File:line: objective.rs:52-56 -> `CommandContext::world` -> `CommandSource::world`
     (`expect("Expected world to exist")`, context/command_source.rs:324).
   - Vanilla: `ObjectiveArgument.listSuggestions` returns empty suggestions when the
     source is not a `ServerCommandSource`.
   - Fix: return `builder.build()` when the source lacks a world.
   - Size: S, but only reachable from a hand-built source today (all production
     sources get a world in `CommandSource::into_source`).

3. Criteria are raw strings, so vanilla's criterion registry is not enforced.
   - File:line: objective.rs:93 reads `ScoreboardObjective.criterion` (String,
     world/scoreboard.rs:733); commands/scoreboard.rs:66-99 stores the argument
     verbatim without resolving it.
   - Effect: a criterion spelled `Health` bypasses `criterion_is_read_only`, and
     unknown criteria are accepted where vanilla resolves through
     `ObjectiveCriteria.byName` and raises `ERROR_CRITERION_NOT_FOUND`.
   - Fix: add a criterion table (`ObjectiveCriteria` equivalent) to scoreboard.rs,
     resolve/validate on `objectives add`, and store the resolved read-only flag.
   - Size: M; verify the unknown-criterion branch against decompiled
     `ScoreboardCommand.addObjective` before implementing.

4. Argument errors carry no cursor context.
   - File:line: objective.rs:81 and :94-95 use `create_without_context`, while vanilla
     uses `DynamicCommandExceptionType.createWithContext(reader, name)`.
   - Effect: the client cannot highlight the failing objective name.
   - Size: S, but it is a fleet-wide convention across argument types; change
     together, not per file.

## Notes

- No code change was committed for this row: the file itself is vanilla-correct.
- No compile/test was run (worker rule).
