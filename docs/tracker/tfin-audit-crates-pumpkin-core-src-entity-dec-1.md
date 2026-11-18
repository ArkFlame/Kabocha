# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-dec.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: area_effect_cloud.rs armor_stand.rs splash_potion.rs

3. Paper events: STILL OPEN (out of unit). `PlayerArmorStandManipulateEvent` already fires cancellably in
Re-checked both STILL OPEN rows against `.vendor/vanilla-decompiled/net/minecraft/world/entity/decoration/ArmorStand.java` and the current Pumpkin sources (armor_stand.rs, area_effect_cloud.rs, splash_potion.rs). No source file changed: every in-scope row is already fixed and matches the decompiled call chain (marker gravity 0.0, spawn-protection/mayInteract guard, potion immunity honoured by cloud/potion consumers, can-tick override, read_flags, main-hand drop, wobble/death game events, interaction slot fallbacks, is_ignoring_block_triggers). No new pure seam, so no unit test added.
STILL OPEN (out of unit, one-line reasons):
STILL OPEN (one-line reason): the WIT records `entity-death-event` (v0.1 event.wit:414,

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
