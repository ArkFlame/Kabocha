# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-wo-f070f4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: tests.rs

- Former STILL OPEN "BedrockScoreboard wrapping_add": VERIFIED FIXED. Both `Scoreboard::add_score` (`:494`) and `BedrockScoreboard::add_score` (`:1575`) use `wrapping_add`. Vanilla `Score.value(int)` is a plain Java int assignment (`.vendor/.../world/scores/Score.java:38-40`), so wrapping is the correct overflow emulation; a Rust debug `+` would panic.
- Former STILL OPEN "score map orientation": VERIFIED FIXED. `ScoreboardBuilder::score` inserts with objective as the outer key (`:1715`), matching every reader (`get_score` indexes `self.scores.get(objective)?.get(entity)`); pinned by `builder_scores_are_keyed_by_objective` (scoreboard/tests.rs:352).

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
