# Final rows from docs/tracker/tentities-strider.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: strider.rs

- Row `strider.rs:109` happy sound: FIXED b9e4dd9eb (played from `use_player_item`). Retreat sound: STILL OPEN, trigger path not confirmed in vanilla notes.
- Row `strider.rs:81` shivering sync: STILL OPEN. M, needs temperature update in `mob_tick` plus event dispatch.
- Row `strider.rs:35` navigation: STILL OPEN. M, needs a per-mob navigation hook.
- Row `strider.rs:96` jockey spawn: STILL OPEN. M, needs jockey selection and baby-chance override.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
