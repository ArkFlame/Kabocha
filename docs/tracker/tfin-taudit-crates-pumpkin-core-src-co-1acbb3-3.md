# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-1acbb3.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- 7. store score orphan objectives: STILL OPEN (M). The scoreboard cannot retain scores whose
Re-checked every STILL OPEN row against the decompiled tree and the current checkout. The

## Result

- 7. store score orphan objectives: FIXED and VERIFIED (no source edit needed this pass; the
  implementation was already present in this checkout and re-verified against vanilla).

  Vanilla evidence: `.vendor/vanilla-decompiled/net/minecraft/world/scores/Scoreboard.java:121-129`
  `getOrCreatePlayerScore(holder, objective)` calls `playerScore.getOrCreate(objective, ...)`,
  keying the score by the `Objective` instance with no check that the objective is still registered.
  `removeObjective` (same file, :302-318) removes scores from the live `PlayerScores` maps by that
  objective instance and unregisters the name, but does not invalidate a captured `Objective`
  reference. So an execute-store handler that captured an objective and runs after its removal
  still recreates a score under the orphan objective instance via `getOrCreatePlayerScore`.

  Pumpkin match: `ScoreboardObjective` carries an identity token (`Arc<()>`, clone-preserving,
  fresh token on a same-name recreate) with `is_same`
  (`crates/pumpkin-core/src/world/scoreboard.rs:1180-1213`); an `orphan_scores` store keyed by that
  identity with `set_orphan_score`/`get_orphan_score` (`:394-427`); `remove_objective` migrates the
  removed live scores into `orphan_scores` keyed by the removed objective (`:376-388`); and
  `load_saved_nbt` clears `orphan_scores` (`:1010`). `write_stored_score`
  (`command/commands/execute.rs:938-971`) gates on `objective.is_same(active)` and, when the captured
  objective is no longer the live one, writes via `set_orphan_score` instead of touching the
  same-name replacement; read-only criteria still return the vanilla error without affecting the
  replacement (`:948-958`).

  Pure unit tests already cover the seam without a live world:
  `score_store_ignores_removed_and_recreated_objective`,
  `loading_saved_nbt_discards_orphan_scores`,
  `objective_update_preserves_identity_for_pending_score_store`, and
  `removed_readonly_objective_still_fails_without_touching_replacement`
  (execute.rs:2816-2897). No new test was needed (coverage complete). No em dashes, no println,
  no unwrap/expect in production code, no generated/WIT/data edits. No compile run per unit rules;
  the orchestrator batch-checks.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-taudit-crates-pumpkin-core-src-co-1acbb3-3.md
