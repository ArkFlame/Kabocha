# Final rows from docs/tracker/tentities-dark-oak-chest-boat-d1e6ed.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Re-verified every STILL OPEN row against .vendor/vanilla-decompiled (BoatItem.java, AbstractBoat.java, AbstractChestBoat.java). All four rows are VERIFIED; no dark-oak-specific gap exists and no code change was needed.
The single STILL OPEN row (line 17, the prose gaps at lines 5-13) was re-verified against .vendor/vanilla-decompiled (BoatItem.java, AbstractChestBoat.java, plus the current Pumpkin sources). All four sub-rows are VERIFIED; no dark-oak-specific gap exists and no source file needed editing. The stale STILL OPEN flag has been changed to VERIFIED so this unit stops being re-queued.
Only the tracker doc was edited (STILL OPEN flag + this Result). No .rs files changed; rustfmt not needed; no compile run (workers cannot compile).
Resumed after server restart; no STILL OPEN rows remain for this unit. The `## Gap status` line was already flipped to VERIFIED in the third pass, and all four prose sub-rows (lines 5-13) were previously verified against .vendor/vanilla-decompiled (BoatItem.java, AbstractBoat.java, AbstractChestBoat.java): item<->entity 1:1 mapping for DARK_OAK_CHEST_BOAT, 27-slot chest boat inventory, boat_interact_outcome / open_custom_inventory_screen parity, and max_passengers 1 (chest) vs 2 (plain) with full EntityType::ALL test coverage. Unit names no source-file basenames to edit and no dark-oak-specific gap was found, so no .rs file was changed. Not repeating the completed re-reviews. No compile (workers cannot compile); no rustfmt (no .rs change).

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
