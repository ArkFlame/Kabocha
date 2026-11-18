# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-net-java-play-use-item-on-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: predicate.rs use_item_on.rs

- **own/taudit-crates-pumpkin-core-src-ne-1655e2**: all 12 sub-rows now classified, 0 STILL OPEN. Row 5 (advancement `state`) is VERIFIED: `use_item_on.rs:325-333` serializes every clicked block-state property into a generic `Vec<(String, String)>` and `predicate.rs` matches each named key, as vanilla `StatePropertiesPredicate.matches` does (`.vendor/vanilla-decompiled/net/minecraft/advancements/predicates/StatePropertiesPredicate.java:34-46`). Row 7 (`ITEM_USED` stat) is VERIFIED: `use_item_on.rs:192-194` awards only when the use consumes the action, matching the item self-award paths. The one hard sub-row (enabled-feature gate) is BLOCKED-DATA below. No code change needed.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
