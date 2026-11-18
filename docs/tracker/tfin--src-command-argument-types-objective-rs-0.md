# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-command-argument-types-objective-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files:

The single parent row `own/taudit-crates-pumpkin-core-src-co-60d58f` expands to four audit rows; all are closed below. Every row is exact vanilla parity, so no source file needed an edit and no row is BLOCKED-DATA (no missing generated pumpkin-data field or missing subsystem).

## Result

Reference read directly this pass:
`.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveArgument.java`,
`.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveCriteriaArgument.java`,
`.vendor/vanilla-decompiled/net/minecraft/world/scores/criteria/ObjectiveCriteria.java`,
`.vendor/vanilla-decompiled/net/minecraft/commands/SharedSuggestionProvider.java`, and the live Pumpkin source.

1. Prefix matching on `filter_and_suggest*` call sites (S): VERIFIED (matches vanilla).
   `objective.rs:61` calls `filter_and_suggest_one`, whose `matches_substr` (`pumpkin-command/src/suggestion/suggestions.rs:237-246`) implements `SharedSuggestionProvider.matchesSubStr` with splitters `._/` (plus `:`, which objective names cannot contain). This matches `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveArgument.java:61` -> `SharedSuggestionProvider.suggest` (`.vendor/vanilla-decompiled/net/minecraft/commands/SharedSuggestionProvider.java:245-255,290-303`). The earlier prefix variant was correctly reverted. No change.

2. Criteria as raw strings / no vanilla registry (M): VERIFIED (matches vanilla).
   `world/scoreboard.rs:1155 resolve_criterion` is the `ObjectiveCriteria.byName` equivalent: read-only health/food/air/armor/xp/level, writable dummy/trigger/deathCount/playerKillCount/totalKillCount and teamkill./killedByTeam. colours, stat names containing `:`; health renders as Hearts, all others Integer. This matches `.vendor/vanilla-decompiled/net/minecraft/world/scores/criteria/ObjectiveCriteria.java:26-38,54-62,79-95,101-107`. `commands/scoreboard.rs:72` validates it on `objectives add` and stores `resolved.render_type`; `objective.rs:24-25 criterion_is_read_only` derives read-only from the same table. Covered by `world/scoreboard/tests.rs`. No change.

3. Argument errors without cursor context (M): VERIFIED (matches vanilla).
   The audit premise is false for this argument type. `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveArgument.java:37,46` throws `ERROR_OBJECTIVE_NOT_FOUND.create(id)` and `ERROR_OBJECTIVE_READ_ONLY.create(objective.getName())` WITHOUT a reader, so `objective.rs:85,99 create_without_context` is exact parity. The only vanilla `createWithContext` case is `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveCriteriaArgument.java:51`, a distinct argument type Pumpkin does not have (criteria resolve at execution time in `commands/scoreboard.rs:72`). Parse-time argument errors elsewhere already use the context-aware `.create(reader, ...)`. Not a BLOCKED-DATA case: no missing generated field or subsystem, just no in-unit call site. No change.

4. Per-world scoreboards vs server-wide vanilla (L): VERIFIED (matches vanilla).
   `server/mod.rs:210-211` owns one shared `pub scoreboard: Arc<std::sync::Mutex<Scoreboard>>` cloned into every world at `:543` and `:933`, matching `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveArgument.java:34` `source.getServer().getScoreboard()`. Objectives are visible cross-dimension. No change.

All four rows are terminal (VERIFIED). No `.rs` file was touched, no new test is needed (rows 1/2/4 have existing coverage, row 3 is exact parity), and no rustfmt is needed. No compile/test run (worker rule); orchestrator batch-checks.
