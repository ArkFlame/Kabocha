# world/scoreboard.rs — deep audit

Sources: crates/pumpkin-core/src/world/scoreboard.rs, crates/pumpkin-core/src/world/scoreboard/tests.rs.
Compared against vanilla `Scoreboard`, `ServerScoreboard`, `ScoreboardSaveData`, `PlayerTeam`/`ClientboundSetPlayerTeamPacket`, `Objective`/`ClientboundSetObjectivePacket`, `ClientboundSetScorePacket`, and Bedrock `SetDisplayObjective`/`SetScore` semantics; Paper adds no scoreboard behaviour here.

## Fixed in this audit
- Duplicate `mod tests`: the file declared both `mod tests;` (scoreboard/tests.rs) and an inline `mod tests { ... }`, which is E0428 for any `cfg(test)` build; the three inline Bedrock tests were moved into scoreboard/tests.rs.
- `ScoreboardBuilder::score` put the entity name in the outer `scores` map and the objective name in the inner map, the inverse of every reader (`get_score` and the packet senders index by objective first), so builder-built boards never returned their scores; keys swapped and pinned by `builder_scores_are_keyed_by_objective`.
- `BedrockScoreboard::add_score` used `+` on `i32` (debug-build overflow panic); now `wrapping_add`, matching the Java `int` overflow of `ScoreboardScore.add` and the JE path.

Both audit markers in the file are comments, not TODOs: line 731 (vanilla display-slot names) and line 774 (teams not persisted).

## Remaining gaps
| # | Location | Vanilla | Gap | Size |
|---|---|---|---|---|
| 1 | crates/pumpkin-core/src/world/scoreboard.rs:773-831, 835-894 | `ScoreboardSaveData` saves and loads `data.Teams` (Name, DisplayName, Prefix, Suffix, FriendlyFire, SeeFriendlyInvisibles, NameTagVisibility, CollisionRule, Color, Players) | Teams are never written to or read from `scoreboard.dat`, so every team, prefix/suffix and colour is lost on restart while objectives/scores/display slots survive | M |
| 2 | crates/pumpkin-core/src/world/scoreboard.rs:250-266, 297-305 | Bedrock only shows an objective through `SetDisplayObjective`; adding/updating an objective must not touch an existing display | Every JE `add_objective`/`update_objective` also broadcasts a Bedrock `SetDisplayObjective(sidebar)` for that objective without recording a display slot, so a Bedrock client shows an objective the server considers undisplayed (and an objective update clobbers another sidebar without the remove+redisplay+row-restore sequence `BedrockObjectivePacket` uses) | M |
| 3 | crates/pumpkin-core/src/world/scoreboard.rs:316-320, 668-673 | Bedrock has only list/sidebar/belowname | Every `sidebar.team.*` (and any future) JE display slot is sent to Bedrock as `sidebar`, so `/scoreboard objectives setdisplay sidebar.team.red x` overwrites the Bedrock sidebar even though the server keeps both slots | S |
| 4 | crates/pumpkin-core/src/world/scoreboard.rs:776-831 | `ScoreboardSaveData` persists per-objective and per-score number formats, and the objective criterion is required to survive | `criterion` is persisted but `number_format` is dropped on save and load, so a blanked/fixed number format reverts to the default after restart | S |
| 5 | crates/pumpkin-core/src/world/scoreboard.rs:569-595 | `Scoreboard.addPlayerToTeam` only accepts a holder that is not already on a team and the old team gets a `RemovePlayers` packet | The code only early-returns when the holder is already in *this* team, so a holder passed through the plugin API can end up in two teams and clients keep the stale membership (needs a vanilla check before changing) | S |
| 6 | crates/pumpkin-core/src/world/scoreboard.rs:455-473 | `ServerScoreboard.removePlayerScore` resets only scores that existed | A reset packet is broadcast even when the score does not exist; harmless on JE clients, but it announces removals that never happened | S |
| 7 | crates/pumpkin-core/src/world/scoreboard.rs:167-169 | Objective names are immutable map keys | `get_objective_mut` lets callers change `name`, desynchronising the map key from the objective, after which `update_objective` inserts a duplicate under the new name and removal targets the wrong key; needs an invariant check or a keyed update API | S |

Checked, no change: display-slot persistence names match vanilla (`list`, `sidebar`, `below_name`, `sidebar.team.*`); `Locked` defaults to true like `dummy` (read-only criteria); `remove_objective` drops its scores and display references; team packet fields (`options` bit flags, visibility/collision names, colour id) and `TeamMethod` transitions match `ClientboundSetPlayerTeamPacket`; objective add/update/remove modes and render types match; `NoTarget`/`Player`/`World` targets gate on `scoreboard_synced` so pre-spawn changes are replayed by `send_to_player`.

## Status (code-verified sweep)
STILL OPEN: BedrockScoreboard wrapping_add row not found in world/scoreboard.rs; score map orientation not re-read.


## Result

Re-read both STILL OPEN verification items against the code and decompiled vanilla (`.vendor/vanilla-decompiled/net/minecraft/`, target V_26_3).

- STILL OPEN "BedrockScoreboard wrapping_add": VERIFIED FIXED, no change. `BedrockScoreboard::add_score` (`world/scoreboard.rs:1298`) uses `current.wrapping_add(delta)` and the JE path `Scoreboard::add_score` (`:450`) uses `current_val.wrapping_add(delta)`. Vanilla `Score.value(int)` (`world/scores/Score.java:38-40`) is a plain Java `int` assignment, so wrapping on overflow is the correct emulation; a Rust debug `+` would panic. No pure unit-test seam exists: `add_score` needs a live `Player` with a Bedrock client.
- STILL OPEN "score map orientation": VERIFIED FIXED, no change. `ScoreboardBuilder::score` (`world/scoreboard.rs:1436-1440`) inserts with `.entry(obj_s).or_default().insert(entity_s, score)`, i.e. objective as the outer key, matching every reader (`get_score` `:187-189` indexes `self.scores.get(objective_name)?.get(entity_name)`). Decompiled vanilla (`world/scores/Scoreboard.java:39,231`) keys `scoresByName` by player first then objective, the inverse of Pumpkin's layout, but Pumpkin is consistently objective-first across readers and packet senders, so the builder must follow that. `world/scoreboard/tests.rs:352` (`builder_scores_are_keyed_by_objective`) already pins the orientation.

No source file changed: both items were already applied by a prior pass and this pass only re-verified them. No rustfmt run (no `.rs` file touched). No test added (orientation already pinned; no pure seam for the Bedrock overflow path).
