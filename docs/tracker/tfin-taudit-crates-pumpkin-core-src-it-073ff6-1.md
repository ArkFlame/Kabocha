# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-it-073ff6.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: use_item.rs

- Gap 5: STILL OPEN: root fix is in net/java/play/use_item.rs and touches every air-use item and the Bedrock path; cross-file, not a single S fix.
- Gap 6: STILL OPEN: world-border half of Player.mayUseItemAt needs the vanilla method body to port exactly.
- Gap 7: STILL OPEN: needs a pumpkin-gametest scenario; no cargo run permitted in this pass.
- Gap 8: STILL OPEN: the non-animal arm needs the vanilla AgeableMob/getBreedOffspring guard for 26.3 before changing.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
