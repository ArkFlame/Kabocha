# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-556f10.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: registry.rs

| `player_placed` receives the raw clicked face while `on_place` receives the opposite face | `registry.rs:923` (`face`) vs `registry.rs:797` (`final_face`) | `setPlacedBy` has no face parameter; overrides that care use the block state | Document what `PlayerPlacedArgs.direction` means (unused by every implementation) and pass `final_face` if it is meant to match `OnPlaceArgs` | S. STILL OPEN: intended meaning of `PlayerPlacedArgs.direction` is undecided and no implementation reads it, so either choice is a guess |
| `BlockFertilizeEvent` is fired with `vec![]` before `perform_bonemeal`, so plugins never see the states the growth changed | `registry.rs:577-591` (`bone_meal`) | Bukkit `CraftEventFactory.callBlockFertilizeEvent` passes the list of changed block states to `BlockFertilizeEvent.getBlocks()` | Let the `perform_bonemeal` path collect changed positions (or diff the world before/after) and fire the event with them; Paper region protection reads that list | S-M (plugin API). STILL OPEN: changes the `BlockFertilizeEvent` payload and needs a plugin-API decision |
| `TestBlock` registered twice, burning a behaviour index | `registry.rs:320` and `registry.rs:382` | n/a | delete the second `register(TestBlock)` | trivial. STILL OPEN: trivial, outside the S/M scope of this pass |
Audited all STILL OPEN rows against .vendor/vanilla-decompiled/net/minecraft/

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
