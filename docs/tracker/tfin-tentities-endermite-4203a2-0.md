# Final rows from docs/tracker/tentities-endermite-4203a2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

2-4. STILL OPEN - not attempted this pass (persistence gate needs reference check, RNG/impact point not observable, client-side velocity packets are a protocol-level change).
Row 3 (pearl RNG / impact point): STILL OPEN. No decompiled `EnderPearl` class exists in `.vendor/`; the spawn-position claim (pearl `pos` vs impact point) cannot be confirmed, and the RNG difference (`rand::random::<f32>()` vs entity RNG) is statistically equivalent and not observable. Needs the reference source before any change.
Row 4 (client-side particle velocity): STILL OPEN. Requires a per-particle velocity packet (protocol-level), outside the files in this unit; cosmetic/client-visible only.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
