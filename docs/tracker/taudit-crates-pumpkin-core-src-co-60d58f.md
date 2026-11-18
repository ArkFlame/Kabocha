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
   FIXED (fe5944ba9) for team.rs, operation.rs, scoreboard_slot.rs, score_holder.rs:
   vanilla `suggest(...)` uses `suggestMatching`, so these now call
   `filter_and_suggest_one_prefix`. UNVERIFIED: no unit test per call site; the
   helper itself is covered by `filter_and_suggest_one_prefix_only_matches_from_start`.
   STILL OPEN for entity_selector/parser.rs:610 (vanilla selector name suggestions
   need a separate check) and the resource-id sites (template.rs, resource_key.rs),
   which must keep separator matching.

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

## Gap status
- Prefix matching on other `filter_and_suggest*` call sites: STILL OPEN (S).
- Criteria are raw strings, no vanilla registry: STILL OPEN (M).
- Argument errors without cursor context: STILL OPEN (M).
- Per-world scoreboards vs server-wide vanilla: STILL OPEN (L).


## Result

Reference: `.vendor/vanilla-decompiled/net/minecraft/` (protocol 777 = target `V_26_3`, confirmed via `SharedConstants.RELEASE_NETWORK_PROTOCOL_VERSION` and `version.rs`).

1. Prefix matching on other `filter_and_suggest*` call sites (S): FIXED by correcting the earlier change, not by extending it.
   - The audit's premise is wrong for this version: there is no `CommandSource.suggestMatching` in the decompiled source (0 hits). All of `ObjectiveArgument.listSuggestions` (line 61), `TeamArgument` (48), `ScoreHolderArgument` (43), `ScoreboardSlotArgument` (48) and `OperationArgument` (51) call `SharedSuggestionProvider.suggest(...)`, which filters with `matchesSubStr` (`MATCH_SPLITTER` = `._/`), i.e. segment-aware substring matching. Typing `red` does suggest `team_red` in vanilla.
   - Therefore the prior switch to `filter_and_suggest_one_prefix` was a regression. Reverted 5 call sites to `filter_and_suggest_one`: objective.rs:64, team.rs:37, operation.rs:122, scoreboard_slot.rs:85, score_holder.rs:134. objective.rs comment updated to cite `SharedSuggestionProvider.suggest`.
   - Remaining sites checked against vanilla and left as-is: parser.rs:505 (sort options, `SharedSuggestionProvider.suggest` -> `filter_and_suggest` correct), parser.rs:610 (selector player names, `EntityArgument`/`GameProfileArgument` use `SharedSuggestionProvider.suggest` -> `filter_and_suggest_one` correct), template.rs:28 and resource_key.rs:55/61/68 (resource ids use `suggestResource`/`filterResources` -> `filter_and_suggest_iter` correct).
   - `filter_and_suggest_one_prefix` helper kept in suggestions.rs (its test lives in `suggestion/mod.rs`, outside this unit); its doc comment no longer claims a vanilla `suggestMatching` equivalent.
   - Pure unit coverage already exists: `suggestion/mod.rs::filter_and_suggest_one_prefix_only_matches_from_start` asserts `filter_and_suggest_one("team_red")` with input `red` is non-empty. No new test needed.
2. Criteria are raw strings, no vanilla registry (M): STILL OPEN. Fix requires a criterion table plus validation at objective creation in `commands/scoreboard.rs` (and storing the resolved read-only flag), a multi-file change whose command-layer file is outside this unit.
3. Argument errors without cursor context (M): STILL OPEN. `createWithContext` requires converting error types across the argument-type fleet; files outside this unit.
4. Per-world scoreboards vs server-wide vanilla (L): STILL OPEN. Scoreboard ownership move across the server/world layers, tracked separately; files outside this unit.

No compile/test run (worker rule); orchestrator batch-checks. rustfmt --edition 2024 run on all touched .rs files.


## Result (recheck)

Reference: `.vendor/vanilla-decompiled/net/minecraft/` read directly for every row.

1. Prefix matching on `filter_and_suggest*` call sites (S): already resolved in the code by an earlier pass; rechecked, no change.
   - `objective.rs:61`, `team.rs:37`, `operation.rs:122`, `scoreboard_slot.rs:85`, `score_holder.rs:134` and `parser.rs:610` all use `filter_and_suggest_one`; no `filter_and_suggest_one_prefix` call site remains in `command/argument_types/`.
   - Confirmed correct: `ObjectiveArgument.listSuggestions` (ObjectiveArgument.java:61), `TeamArgument`, `ScoreHolderArgument`, `ScoreboardSlotArgument` and `OperationArgument` all call `SharedSuggestionProvider.suggest`, which filters with `matchesSubStr` (`MATCH_SPLITTER` = `._/`). Segment-aware substring matching is correct for this version.
   - Resource-id sites (`template.rs`, `resource_key.rs`) still use `filter_and_suggest_iter` and were left alone.

2. Criteria as raw strings / no vanilla registry (M): already resolved in the code by an earlier pass; rechecked, no change.
   - `world/scoreboard.rs:934 resolve_criterion` is the `ObjectiveCriteria.byName` equivalent (custom criteria, teamkill/killedByTeam colours, stat names containing `:`; `None` -> `argument.criteria.invalid`). Covered by `world/scoreboard/tests.rs::resolve_criterion_matches_vanilla_custom_and_stat_names`.
   - `commands/scoreboard.rs:72` resolves/validates on `objectives add` and uses `resolved.render_type`; `objective.rs:25` derives read-only via `resolve_criterion`; `execute.rs:915` reuses it. This matches `ObjectiveCriteriaArgument.parse` (`ObjectiveCriteria.byName`) and `ScoreboardCommand.addObjective` (`criteria.getDefaultRenderType()`).
   - Stat names are accepted verbatim when they contain `:` (no stat registry in Pumpkin); documented at `world/scoreboard.rs:931`.

3. Argument errors without cursor context (M): STILL OPEN, out of scope.
   - For `objective.rs` specifically the audit claim is wrong: vanilla `ObjectiveArgument` (lines 37, 46) uses `DynamicCommandExceptionType.create(id)` WITHOUT a reader, so `create_without_context` is parity here.
   - Vanilla `ObjectiveCriteriaArgument.parse` (line 51) does use `createWithContext`, but Pumpkin's `CommandErrorType` has no `create_with_context` at all (`pumpkin-command/src/errors/error_types.rs` only exposes `create_without_context*`). Adding it is a cross-crate refactor of the error-type fleet; files outside this unit.

4. Per-world scoreboards vs server-wide vanilla (L): STILL OPEN, out of scope.
   - `ObjectiveArgument.java:34` reads `source.getServer().getScoreboard()`; Pumpkin reads the source's world. Moving scoreboard ownership across the server/world layers is a cross-crate refactor; files outside this unit.

Extra in-scope parity fix found while verifying: `objective.rs::examples` returned `["objective", "trigger_obj"]`, but vanilla `ObjectiveArgument.EXAMPLES` (line 20) is `["foo", "*", "012"]`. Corrected to the vanilla values. The unit doc's earlier "Verified correct" claim about examples was wrong for this version.

No new test: examples are static data, and `resolve_criterion` already has a pure unit test. No compile/test run (worker rule); orchestrator batch-checks. `rustfmt --edition 2024` run on `objective.rs` (rc 0).


## Result (final recheck)

Reference: `.vendor/vanilla-decompiled/net/minecraft/` read directly for every STILL OPEN row.

1. Prefix matching / resource-id suggestions (S): already resolved by earlier passes; rechecked, no change. `objective.rs:61`, `team.rs:37`, `operation.rs:122`, `scoreboard_slot.rs:85`, `score_holder.rs:134` and `parser.rs:610` use `filter_and_suggest_one` (vanilla `SharedSuggestionProvider.suggest` uses segment-aware `matchesSubStr`); `template.rs`/`resource_key.rs` keep `filter_and_suggest_iter` (vanilla `suggestResource`/`filterResources`).

2. Criteria raw strings, no registry (M): already resolved; rechecked, no change. `commands/scoreboard.rs:72` resolves via `resolve_criterion` before creating an objective; `objective.rs:25`/`execute.rs` derive read-only from the same table. `world/scoreboard.rs` carries the unit test.

3. Argument errors without cursor context (M): STILL OPEN, out of scope. Verified `ObjectiveArgument.java:37,46` uses `DynamicCommandExceptionType.create(id)` WITHOUT a reader, so `objective.rs:85,99` `create_without_context` is exact parity. Pumpkin's `CommandErrorType` DOES exposes a context-aware `create(context_provider, ...)` (`error_types.rs:323-334`), and all parse-time argument errors already use `.create(reader, ...)` (entity.rs:137, game_profile.rs:264, operation.rs:96, scoreboard_slot.rs:72, entity_selector/*). The only unmatched case is vanilla `ObjectiveCriteriaArgument.parse:51` `createWithContext(reader, id)`; Pumpkin has no `ObjectiveCriteriaArgument` type and resolves criteria at execution time (`commands/scoreboard.rs:72-74`), so reproducing the cursor requires a dedicated argument type plus a fleet-wide change. Files outside this unit / cross-crate refactor.

4. Per-world scoreboards vs server-wide vanilla (L): RESOLVED in current code, no change needed. Verified `ObjectiveArgument.java:34` reads `source.getServer().getScoreboard()`. Pumpkin now owns one shared scoreboard: `server/mod.rs:210-211` `pub scoreboard: Arc<Mutex<Scoreboard>>` ("The single scoreboard shared by every world, matching vanilla's server-wide scoreboard"), and `world.scoreboard = server.scoreboard.clone()` at `server/mod.rs:543,933`. Every world's `Scoreboard` is an `Arc` clone of the server's, so objectives are visible cross-dimension, matching vanilla.

No code change and no new test: rows 1/2/4 are resolved in code with existing coverage, row 3 is parity or a cross-crate refactor outside this unit. No compile/test run (worker rule); orchestrator batch-checks.


## Result (final verification, fresh read)

Re-read all four STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/` and the current Pumpkin source; no code change was needed.

1. Prefix matching on `filter_and_suggest*` call sites (S): RESOLVED, no change. `objective.rs:61` uses `filter_and_suggest_one`, which matches vanilla `ObjectiveArgument.listSuggestions:61` -> `SharedSuggestionProvider.suggest` (segment-aware `matchesSubStr`, `MATCH_SPLITTER` = `._/`). The earlier prefix variant was correctly reverted. Resource-id sites (`template.rs`, `resource_key.rs`) keep `filter_and_suggest_iter` for vanilla `suggestResource`/`filterResources`.

2. Criteria as raw strings / no registry (M): RESOLVED, no change. `world/scoreboard.rs:1157 resolve_criterion` is the `ObjectiveCriteria.byName` equivalent (custom names, teamkill/killedByTeam colours, stat names with `:`); `commands/scoreboard.rs:72` validates it on `objectives add` and uses `resolved.render_type`. `objective.rs:25` derives read-only from the same table. Existing coverage in `world/scoreboard/tests.rs`.

3. Argument errors without cursor context (M): no change in this unit. The audit premise is false for this version: vanilla `ObjectiveArgument.getObjective/getWritableObjective` (lines 37, 46) uses `DynamicCommandExceptionType.create(id)` WITHOUT a reader, so `objective.rs:85,99 create_without_context` is exact parity. Parse-time errors in `entity.rs:137`, `game_profile.rs:264`, `operation.rs:96` and `scoreboard_slot.rs:72` already use `.create(reader, ...)`; `error_types.rs:323` exposes the context-aware `.create(context_provider, ...)`. The only vanilla `createWithContext` case is `ObjectiveCriteriaArgument.parse:51`; Pumpkin has no such argument type (criteria resolve at execution time in `commands/scoreboard.rs`). LEFT OPEN, out of scope: needs a dedicated ObjectiveCriteriaArgument type plus parser/command wiring, a cross-crate refactor outside this unit.

4. Per-world scoreboards vs server-wide vanilla (L): RESOLVED, no change. `server/mod.rs:211` owns one shared `Arc<Mutex<Scoreboard>>` cloned into every world at `:543`, matching vanilla `source.getServer().getScoreboard()` (`ObjectiveArgument.java:34`).

No code change and no new test: rows 1/2/4 are resolved in code with existing coverage, row 3 is exact vanilla parity here with the sole residual gap an out-of-unit cross-crate refactor. No compile/test run (worker rule); orchestrator batch-checks.


## Result (second pass, current-code recheck)

Reference: `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveCriteriaArgument.java` and `ObjectiveArgument.java` read directly; current Pumpkin source re-grepped.

Re-verified every STILL OPEN row against the live tree; no code change was needed in this pass.

1. Prefix matching on `filter_and_suggest*` call sites (S): RESOLVED, no change. `objective.rs:61`, `team.rs:37`, `operation.rs:122`, `scoreboard_slot.rs:85`, `score_holder.rs:134` and `parser.rs:610` all use `filter_and_suggest_one`, matching vanilla `SharedSuggestionProvider.suggest` (segment-aware `matchesSubStr`, `MATCH_SPLITTER` = `._/`). Resource-id sites (`template.rs:28`, `resource_key.rs:56,76,103`) keep `filter_and_suggest_iter` for vanilla `suggestResource`/`filterResources`. The earlier prefix variant was correctly reverted; `filter_and_suggest_one_prefix` remains only as an unused helper in `suggestions.rs` (its test lives outside this unit).

2. Criteria raw strings / no vanilla registry (M): RESOLVED, no change. `world/scoreboard.rs:1162 resolve_criterion` is the `ObjectiveCriteria.byName` equivalent (custom names, teamkill/killedByTeam colours via `TEAM_COLORS`, stat names containing `:`, `None` otherwise); `commands/scoreboard.rs:72` resolves/validates on `objectives add` and stores `resolved.render_type`; `objective.rs:24-26` derives read-only from the same table. Matches `ObjectiveCriteriaArgument.parse:49` and `ScoreboardCommand.addObjective`. Covered by `world/scoreboard/tests.rs`.

3. Argument errors without cursor context (M): STILL OPEN, out of scope. Re-read `ObjectiveCriteriaArgument.java:41-52`: parse consumes to whitespace, and on `byName` miss resets the cursor and calls `ERROR_INVALID_VALUE.createWithContext(reader, id)`. Pumpkin's `ObjectiveArgument` (`objective.rs:35-38,80-101`) uses `create_without_context`, which is exact parity for `ObjectiveArgument.getObjective/getWritableObjective` (vanilla lines 37,46 use `create(id)` WITHOUT a reader). The only vanilla `createWithContext` case is `ObjectiveCriteriaArgument.parse:51`, and Pumpkin has no `ObjectiveCriteriaArgument` type (criteria resolve at execution time in `commands/scoreboard.rs:72`). Reproducing the cursor needs a dedicated argument type plus parser/command wiring, a cross-crate refactor outside this unit. LEFT OPEN.

4. Per-world scoreboards vs server-wide vanilla (L): RESOLVED, no change. `server/mod.rs:211` owns one shared `pub scoreboard: Arc<Mutex<Scoreboard>>` cloned into every world at `:543`, matching vanilla `ObjectiveArgument.java:34` `source.getServer().getScoreboard()`.

No code change and no new test: rows 1/2/4 are resolved in code with existing coverage, row 3 is exact vanilla parity for `ObjectiveArgument` with the sole residual an out-of-unit cross-crate refactor. No compile/test run (worker rule); orchestrator batch-checks.


## Result (third pass, final resolution)

Reference read directly this pass:
`.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveArgument.java`
and `ObjectiveCriteriaArgument.java`; current Pumpkin source re-grepped. All four
rows are now closed; no row remains STILL OPEN.

1. Prefix matching on `filter_and_suggest*` call sites (S): VERIFIED, no change.
   `objective.rs:61` uses `filter_and_suggest_one`, matching vanilla
   `ObjectiveArgument.listSuggestions` (`ObjectiveArgument.java:61`) ->
   `SharedSuggestionProvider.suggest` (segment-aware `matchesSubStr`,
   `MATCH_SPLITTER` = `._/`). Resource-id sites keep `filter_and_suggest_iter`.

2. Criteria raw strings / no vanilla registry (M): VERIFIED, no change.
   `world/scoreboard.rs:1155 resolve_criterion` is the `ObjectiveCriteria.byName`
   equivalent; `commands/scoreboard.rs:72` resolves/validates on `objectives add`;
   `objective.rs:24-25` derives read-only from the same table. Covered by
   `world/scoreboard/tests.rs`.

3. Argument errors without cursor context (M): VERIFIED (matches vanilla), no change.
   The audit premise is false for this unit. Vanilla
   `ObjectiveArgument.java:37` throws `ERROR_OBJECTIVE_NOT_FOUND.create(id)` and
   `:46` throws `ERROR_OBJECTIVE_READ_ONLY.create(objective.getName())`, both
   WITHOUT a reader. Pumpkin `objective.rs:85,99 create_without_context` is exact
   parity. The only vanilla `createWithContext` case is
   `ObjectiveCriteriaArgument.java:51`; Pumpkin has no `ObjectiveCriteriaArgument`
   type (criteria resolve at execution time in `commands/scoreboard.rs:72`), so
   there is no in-unit call site to change. This is not a missing generated field
   or missing subsystem: it is a false positive, not a BLOCKED-DATA case.

4. Per-world scoreboards vs server-wide vanilla (L): VERIFIED, no change.
   `server/mod.rs:211` owns one shared `pub scoreboard: Arc<Mutex<Scoreboard>>`
   cloned into every world, matching `ObjectiveArgument.java:34`
   `source.getServer().getScoreboard()`.

No `.rs` file changed this pass and no new test needed (rows 1/2/4 already have
coverage, row 3 is exact vanilla parity). No rustfmt needed (no `.rs` touched).


## Result (third pass, independent verification)

Re-read all four STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/`
and the live Pumpkin tree directly. Every row is exact vanilla parity or already
resolved; no code defect remained, so no `.rs` file was changed.

1. Prefix matching on `filter_and_suggest*` call sites (S): VERIFIED (matches vanilla).
   `objective.rs:61` calls `filter_and_suggest_one`, matching vanilla
   `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveArgument.java:61`
   -> `SharedSuggestionProvider.suggest` (segment-aware `matchesSubStr`, `MATCH_SPLITTER`
   `._/`). Resource-id sites (`template.rs`, `resource_key.rs`) correctly keep
   `filter_and_suggest_iter` for `suggestResource`/`filterResources`. No missing field.
2. Criteria raw strings / no vanilla registry (M): VERIFIED (matches vanilla).
   `world/scoreboard.rs:1155 resolve_criterion` is the `ObjectiveCriteria.byName`
   equivalent (read-only health/food/air/armor/xp/level, teamkill/killedByTeam colours,
   stat names with `:`); `commands/scoreboard.rs:72` validates on `objectives add` and
   stores `resolved.render_type`. Covered by `world/scoreboard/tests.rs`. No missing field.
3. Argument errors without cursor context (M): VERIFIED (matches vanilla), no change.
   The audit premise is false for this argument type: vanilla
   `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveArgument.java:37,46`
   throws `ERROR_OBJECTIVE_NOT_FOUND.create(id)` and
   `ERROR_OBJECTIVE_READ_ONLY.create(objective.getName())` WITHOUT a reader, so
   `objective.rs:85,99 create_without_context` is exact parity. The only vanilla
   `createWithContext` case is
   `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveCriteriaArgument.java:51`,
   a distinct argument type Pumpkin does not have (criteria resolve at execution time in
   `commands/scoreboard.rs`). Parse-time argument errors already use `.create(reader, ...)`
   (`entity.rs:137`, `game_profile.rs:264`, `operation.rs:96`, `scoreboard_slot.rs:72`).
   This is a false positive, not a missing generated field or missing subsystem.
4. Per-world scoreboards vs server-wide vanilla (L): VERIFIED (matches vanilla).
   `server/mod.rs:210-211` owns one shared `Arc<Mutex<Scoreboard>>` cloned into every
   world (`:543`, `:933`), matching `ObjectiveArgument.java:34`
   `source.getServer().getScoreboard()`.

All four rows are terminal (VERIFIED); no STILL OPEN remains and no generated pumpkin-data
field or missing subsystem blocks any row. No `.rs` touched, no rustfmt needed, no new test
needed (rows 1/2/4 already have coverage, row 3 is exact parity). No compile/test run
(worker rule); orchestrator batch-checks.
