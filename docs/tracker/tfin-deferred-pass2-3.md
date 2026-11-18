# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bucket.rs mod.rs

- taudit-crates-pumpkin-core-src-it-0d4e6b.md: - Row 6 (event payload lacks face/stack, bucket.rs:642-654/726-736): STILL OPEN. Additive WIT
- taudit-crates-pumpkin-core-src-ne-254fb9.md: - Row 3 (stonecutter-only recipe packet, mod.rs 772-781): STILL OPEN, M, needs generated recipe property sets in `pumpkin-data`/`recipe_helper`, outside this unit.
- taudit-crates-pumpkin-core-src-net-bedroc.md: | 1 await_teleport / Teleport ack | STILL OPEN. Vanilla handleAcceptTeleportation clears the pending teleport by id; Bedrock's ack needs a new Teleport value in SPlayerAuthInput (pumpkin-protocol) plus id-based clearing in await_teleport. That is a protocol packet + cross-crate change outside this u
- taudit-crates-pumpkin-core-src-net-java-m.md: - Row 3 (UPDATE_RECIPES item property sets): STILL OPEN - writing real `itemSets` needs generated

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
