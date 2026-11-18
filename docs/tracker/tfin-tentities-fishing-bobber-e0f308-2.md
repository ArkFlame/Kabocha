# Final rows from docs/tracker/tentities-fishing-bobber-e0f308.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: fishing_bobber.rs fishing_rod.rs type.rs

- `entity/type.rs:143` (bobber arm): STILL OPEN - needs a `from_loaded` constructor and owner-NBT
- `fishing_bobber.rs` (Lured event state): STILL OPEN - needs an additive `PlayerFishState::Lured`
- `item/items/fishing_rod.rs` (Fishing event on cast): STILL OPEN - the event is already fired and
Re-verified each STILL OPEN row against the decompiled FishingHook.java and FishingRodItem.java

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
