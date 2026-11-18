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
