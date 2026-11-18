# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: jukebox.rs

Audit pass 4 (2026-11-18). Re-verified every row marked STILL OPEN against the decompiled
- `entities/jukebox.rs` tick particle order/payload (was STILL OPEN): verified correct, no code
- `entities/jukebox.rs` `chunk_data_nbt` / `write_nbt` ticks (was STILL OPEN): already fixed on
- `blocks/jukebox.rs:192` `on_state_replaced` delegation (was STILL OPEN): verified correct, no

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
