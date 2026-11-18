# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-31ea5a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: jukebox.rs

- `entities/jukebox.rs` `chunk_data_nbt`/`write_nbt` ticks (was STILL OPEN): FIXED on master, verified.
- `blocks/jukebox.rs:192` `on_state_replaced` (was STILL OPEN): VERIFIED. Only updates the output signal;
Out of scope (not a tracker STILL OPEN row): Bedrock block-actor data dropping item components
behaviour matches vanilla. No row remains STILL OPEN.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
