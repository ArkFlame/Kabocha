# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-e958f4.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: legacy.rs locate.rs mod.rs

| Bedrock success arguments (`locate.rs`) | VERIFIED: `send_success` uses `translate_cross_with` with `bedrock_success_args` (`locate.rs:183-223`). The cross-cutting split exists in `pumpkin-util/src/text/mod.rs` (`bedrock_with` field threaded at :1055-1065, :1127-1130, :1254-1267, :1459-1466; test :2286) and is read in `legacy.rs:273`. The tracker's "STILL OPEN" note is stale. |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
