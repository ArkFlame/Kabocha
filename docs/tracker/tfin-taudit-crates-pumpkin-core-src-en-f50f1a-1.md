# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-f50f1a.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: ai.rs

| Play-dead regeneration duration | `ai.rs:39,353-364` grants 200 ticks at amplifier 0 | wiki states Regeneration I while playing dead; duration is not documented in code we can read here | keep amplifier 0 (wiki: Regeneration I); only shorten to 100 ticks if 1.21.x source confirms | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
- All STILL OPEN rows resolved and verified against decompiled vanilla; see ## Result below.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
