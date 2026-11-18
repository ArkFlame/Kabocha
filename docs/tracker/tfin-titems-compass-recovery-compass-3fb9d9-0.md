# Final rows from docs/tracker/titems-compass-recovery-compass-3fb9d9.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: block_pos.rs compass.rs setworldspawn.rs

- In-scope files verified unchanged: `setworldspawn.rs` uses `get_spawnable_pos` for both `pos` variants and `floored_v` for the self path (vanilla `SetWorldSpawnCommand.java:22-28`); `compass.rs` `use_on_block` matches `CompassItem.java:46-69`; `block_pos.rs` `is_valid_block_pos` + boundary test present. All rows closed: no STILL OPEN, no BLOCKED-DATA.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
