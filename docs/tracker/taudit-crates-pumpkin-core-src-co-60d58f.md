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

## Fixed in this pass

1. Objective suggestions now use vanilla prefix matching.
   - Before: objective.rs:58 used `Suggestions::filter_and_suggest_one`, whose
     `matches_substr` (pumpkin-command/src/suggestion/suggestions.rs:218-227) also
     matches after `.`/`_`/`/`/`:`, so typing `red` suggested `team_red`.
   - Vanilla: `ObjectiveArgument.listSuggestions` delegates to
     `CommandSource.suggestMatching`, which is `candidate.toLowerCase().startsWith(input)`.
   - Fix: new `SuggestionsBuilder::filter_and_suggest_one_prefix` (suggestions.rs:157-174),
     used by objective.rs:57-61. Test: suggestion/mod.rs
     `filter_and_suggest_one_prefix_only_matches_from_start`.
   - Commit 64a0de8ab.
2. `list_suggestions` no longer panics for a source without a world.
   - Before: objective.rs:52-56 called `CommandContext::world()` ->
     `CommandSource::world()` (`expect("Expected world to exist")`,
     context/command_source.rs:324).
   - Vanilla returns empty suggestions when the source is not a
     `ServerCommandSource`; the fix returns `builder.build()` when
     `context.source.world` is `None`.
   - Commit 78dd2191c.

## Remaining gaps (not fixed here)

1. Prefix matching is still missing on the other `filter_and_suggest*` call sites
   that mirror vanilla `suggestMatching` (team.rs:37, score_holder.rs:133,
   scoreboard_slot.rs, operation.rs, ...). The new
   `filter_and_suggest_one_prefix` helper is ready for them; each call site needs a
   per-type check first (resource-id suggestions must keep the separator matching).
   Size: S per type.

2. Criteria are raw strings, so vanilla's criterion registry is not enforced.
   - File:line: objective.rs:93 reads `ScoreboardObjective.criterion` (String,
     world/scoreboard.rs:899-923); commands/scoreboard.rs:66-99 stores the argument
     verbatim without resolving it.
   - Effect: a criterion spelled `Health` bypasses `criterion_is_read_only`, and
     unknown criteria are accepted where vanilla resolves through
     `ObjectiveCriteria.byName` and raises `ERROR_CRITERION_NOT_FOUND`.
   - Fix: add a criterion table (`ObjectiveCriteria` equivalent) to scoreboard.rs,
     resolve/validate on `objectives add`, and store the resolved read-only flag.
   - Size: M; verify the unknown-criterion branch against decompiled
     `ScoreboardCommand.addObjective` before implementing.

3. Argument errors carry no cursor context.
   - File:line: objective.rs:81 and :94-95 use `create_without_context`, while vanilla
     uses `DynamicCommandExceptionType.createWithContext(reader, name)`.
   - Effect: the client cannot highlight the failing objective name.
   - Size: S, but it is a fleet-wide convention across argument types; change
     together, not per file.

4. Scoreboards are per-world in Pumpkin, server-wide in vanilla.
   - File:line: objective.rs:52-56 and the helpers take `&Scoreboard` from the
     source's world; vanilla `ObjectiveArgument` reads
     `source.getServer().getScoreboard()`.
   - Effect: objectives created in one dimension are invisible in another; a
     cross-dimensional `/execute as ... run scoreboard ...` sees the wrong board.
   - Size: L (scoreboard ownership move), tracked separately.

## Notes

- No compile/test was run in the worker (worker rule); the orchestrator batch-checks.
