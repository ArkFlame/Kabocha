# Final rows from docs/tracker/tblocks-structure-block-328c1d.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Old line 52, LootTableSeed gating (not marked STILL OPEN): FIXED a real mismatch. is_randomizable_container_id listed suspicious_sand and suspicious_gravel, but vanilla BrushableBlockEntity extends BlockEntity (not RandomizableContainer); the only implementors are RandomizableContainerBlockEntity subclasses (chest/trapped_chest, barrel, dispenser/dropper, hopper, shulker boxes, crafter) plus DecoratedPotBlockEntity. Removed the two brushable ids and extended loot_seed_only_for_randomizable_containers to assert they are excluded.
Only the LootTableSeed row needed an edit; the other STILL OPEN rows were already implemented in this checkout and are now verified against vanilla. rustfmt --edition 2024 run on the touched .rs file. Compile deferred to the orchestrator.
Third pass. Re-verified every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft
Fourth pass. Re-checked every STILL OPEN row against .vendor/vanilla-decompiled/net/minecraft

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
