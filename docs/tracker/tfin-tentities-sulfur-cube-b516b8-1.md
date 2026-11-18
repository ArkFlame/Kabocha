# Final rows from docs/tracker/tentities-sulfur-cube-b516b8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: sulfur_cube.rs

All rows resolved against `.vendor/vanilla-decompiled/net/minecraft/`. No STILL OPEN rows remain.
STILL OPEN row `9728ac352` (apply sulfur cube body state on item pickup): VERIFIED. Vanilla `SulfurCube.pickUpItem` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/monster/cubemob/SulfurCube.java:720-729`) calls `onItemPickup` then `setItemSlot(BODY, split)`; the body state is applied by `SulfurCube.collectEquipmentChanges` (same file :405-457), run once per tick by the base entity. Pumpkin `pick_up_item` (`sulfur_cube.rs:903-937`) sets the BODY slot via `set_item_slot_and_drop_when_killed` (922-923) and calls `detect_body_item_change()` immediately (924), before the absorb sound; that routes to `on_body_item_changed` (402-474), which applies the same goal/archetype/attribute state set as `collectEquipmentChanges`. `mob_tick` still calls `detect_body_item_change` once per tick (839) as the vanilla collection point. No code change needed. No pure unit-test seam exists: the path needs a live MobEntity/world and `detect_body_item_change` is covered by the pickup/equip/shear integration paths.
STILL OPEN row `9728ac352` (apply sulfur cube body state on item pickup): VERIFIED against decompiled vanilla. No source change required: the fix is already present in the current tree.
row in this unit is closed; no STILL OPEN rows and no BLOCKED-DATA rows remain. All vanilla

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
