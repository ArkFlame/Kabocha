# native-1 summary

Work list `/tmp/kabocha-runs/native-1.md` only held a context-search pointer, not the 24 rows. Rows were taken from the OPEN entries in the unit docs `taudit-crates-pumpkin-core-src-co-20e7ce`, `taudit-crates-pumpkin-core-src-co-1acbb3` and `taudit-crates-pumpkin-core-src-command-mo`.

| Unit | Row | Status |
|---|---|---|
| command-mo | 11 CustomName in sender name | FIXED (`command/mod.rs`) |
| co-20e7ce | 1 replace slot-source | FIXED (stale) |
| co-20e7ce | 2 item set | FIXED (stale) |
| co-20e7ce | 3 entity slot mapping | PARTIAL: mob/horse FIXED; cursor and crafting grid BLOCKED-L |
| co-20e7ce | 4 loot functions | BLOCKED-L |
| co-20e7ce | 5 number providers | FIXED (stale) |
| co-20e7ce | 6 set_count clamp | NOT A GAP (vanilla does not clamp) |
| co-20e7ce | 7 mark_dirty | FIXED (stale) |
| co-20e7ce | 8 item modify feedback | FIXED (stale) |
| co-20e7ce | 9 dedup | SKIPPED (no behaviour change) |
| co-1acbb3 | 7 orphan score store | BLOCKED-M (scoreboard keyed by name) |

Counts: 1 fixed in code this pass, 5 stale rows confirmed fixed, 1 not a gap, 1 partial, 3 blocked, 1 skipped. Not compiled (per instructions); no tests added, since the only new seam needs a live CommandBlockEntity.

## Pass 2 (real list docs/investigations/work/native-1.md)

- co-1acbb3 row 7 (orphan score store): FIXED. `world/scoreboard.rs`: `remove_objective` moves the removed objective's scores into `orphan_scores`, keyed by objective identity (`ScoreboardObjective::is_same`, Arc ptr). New `set_orphan_score` / `get_orphan_score`. `command/commands/execute.rs` `write_stored_score` writes to the orphan store for inactive objectives instead of dropping them, matching vanilla `PlayerScores` orphan Score objects. Not sent to client (vanilla does not either). Not persisted to saved NBT (vanilla persistence of orphans not modelled).
- co-20e7ce row 3 (entity slots 300-307 / 499 / 500-514): FIXED, already in code. Player cursor 499 and crafting 500-503 in `read_player_menu_slot` / `write_player_menu_slot` (`command/commands/item.rs` ~L394-443). Mob 300-307 in `read_mob_slot` / `write_mob_slot`. Horse 499-514 in `read_horse_slot` / `write_horse_slot`.
- co-20e7ce row 1 (replace one-to-many, target.failed*): FIXED, already in code. `BlockReplaceExecutor` and `EntityReplaceExecutor` iterate slot sources and emit `COMMANDS_ITEM_TARGET_FAILED_*`; `response_counts` mirrors vanilla `CommandResponseTracker`.
- co-20e7ce row 2 (`item set`, SuccessFeedback::Set): FIXED, already in code (`item.rs:69-72`, `:175-184`, `:224-227`).
- co-20e7ce row 4 (loot functions): PARTIAL. Added `sequence` and `limit_count` arms to `apply_item_modifier` (`item.rs`, before the `other =>` debug arm). Still missing, each needs its own data or API: `set_item` (ItemStack item-swap API), `toggle_tooltips` (`DataComponent::TooltipDisplay`, `pumpkin-data/src/generated/data_component.rs`), `enchant_with_levels`, `set_instrument`, `set_fireworks`, `set_firework_explosion`, `set_banner_pattern`, `set_book_cover`, `set_writable_book_pages`, `set_written_book_pages`, `set_stew_effect`, `set_attributes`, `set_random_dyes`, `set_random_potion`, `set_ominous_bottle_amplifier`, `copy_name`, `copy_custom_data`, `copy_components`, `filtered`, `discard`, `exploration_map`, `fill_player_head`, `smelt`, `apply_bonus`, `apply_explosion_decay`, `copy_state`, `set_contents`, `modify_contents`, `set_loot_table`. Those need loot context (source entity, block state, container) that the command path does not carry.
- co-20e7ce row 7 (mark_dirty after item writes): FIXED, already in code (`inventory.mark_dirty()` at `item.rs` ~L152 and ~L834).
- co-20e7ce row 8 (item modify feedback): FIXED, already in code (`BlockModifyExecutor`, `EntityModifyExecutor`).
- co-20e7ce row 9 (dedup): not changed; maintenance only.

Commits: none. Per coordinator instruction, the pool supervisor commits the tree. The index.lock was held by another process during this pass. Compile not run. Edits are: `command/mod.rs` (command_block_name), `world/scoreboard.rs` (orphan_scores, OrphanScore, set_orphan_score, get_orphan_score, remove_objective), `command/commands/execute.rs` (write_stored_score), `command/commands/item.rs` (sequence and limit_count arms). The new `OrphanScore` struct has no initializer outside `scoreboard.rs`.
