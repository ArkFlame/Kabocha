# Final rows from docs/tracker/tblocks-creaking-heart.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: creaking_heart.rs

FIXED: block entity tick re-syncs state every 20 ticks (block/entities/creaking_heart.rs tick); awake heart with a player in 32 blocks spawns a bound creaking (try_spawn_creaking); is_protector requires a bound UUID (no fallback). UNVERIFIED: spawn distance 32 and free-spot search are from memory. STILL OPEN: idle heartbeat, teardown of bound creaking when heart goes uprooted, break sound routing.
out-of-unit files were touched. All prior STILL OPEN rows are resolved; open = 0.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
