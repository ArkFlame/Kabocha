# Parity rows for crates/pumpkin-core/src/command/argument_types/objective.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-co-60d58f** [partial/OWN-OPEN] own audit: 15 open, 2 fixed, 1 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-co-60d58f.md) docs/tracker/taudit-crates-pumpkin-core-src-co-60d58f.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, final closure)

Reference re-read this pass directly:
`.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveArgument.java`
(lines 19-70), `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ObjectiveCriteriaArgument.java`,
and `.vendor/vanilla-decompiled/net/minecraft/commands/SharedSuggestionProvider.java`
(lines 245-255 and 290-301). Live `objective.rs` compared below. No generated
pumpkin-data field or missing subsystem is involved, so no row is BLOCKED-DATA.

Single unit row `own/taudit-crates-pumpkin-core-src-co-60d58f` (own audit of
objective.rs, four audit rows): VERIFIED (matches vanilla), no source change
needed.

- Audit row 1, prefix matching on `filter_and_suggest*` call sites (S): VERIFIED.
  `objective.rs:61` calls `filter_and_suggest_one`, whose `matches_substr`
  (pumpkin-command/src/suggestion/suggestions.rs:237-246) implements
  `SharedSuggestionProvider.matchesSubStr` with splitters `._/` (plus `:`,
  irrelevant to objective names). This matches `ObjectiveArgument.java:61`
  -> `SharedSuggestionProvider.suggest` (`SharedSuggestionProvider.java:245-255`).
  The earlier prefix variant was a regression and is not present.
- Audit row 2, criteria as raw strings / no registry (M): VERIFIED.
  `world/scoreboard.rs:1155 resolve_criterion` is `ObjectiveCriteria.byName`;
  `commands/scoreboard.rs:72` resolves on `objectives add`;
  `objective.rs:24-25 criterion_is_read_only` reads the same table. Matches
  `ObjectiveCriteriaArgument.java:49` and `ScoreboardCommand.addObjective`.
  Covered by `world/scoreboard/tests.rs`.
- Audit row 3, argument errors without cursor context (M): VERIFIED.
  `ObjectiveArgument.java:37,46` use `DynamicCommandExceptionType.create(id)`
  WITHOUT a reader, so `objective.rs:85,99 create_without_context` is exact
  parity. The only vanilla `createWithContext` is
  `ObjectiveCriteriaArgument.java:51`; Pumpkin has no such argument type
  (criteria resolve at execution time). False positive, not BLOCKED-DATA.
- Audit row 4, per-world vs server-wide scoreboards (L): VERIFIED.
  `server/mod.rs:211` owns one shared `Arc<Mutex<Scoreboard>>` cloned into every
  world, matching `ObjectiveArgument.java:34 source.getServer().getScoreboard()`.

`objective.rs` is exact vanilla parity for EXAMPLES (`["foo", "*", "012"]`,
line 68), `parse` (`read_unquoted_string`, line 36), and the no-world empty
suggestion fallback (lines 51-53). No `.rs` file edited, no new test (every row
is parity or already covered), no rustfmt needed. No compile/test run
(worker rule); orchestrator batch-checks.
