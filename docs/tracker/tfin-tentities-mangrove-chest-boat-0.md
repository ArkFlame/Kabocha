# Final rows from docs/tracker/tentities-mangrove-chest-boat.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Three rows from the source tracker, re-verified against the decompiled vanilla tree.

## Result
All three rows from the source tracker `docs/tracker/tentities-mangrove-chest-boat.md` are FIXED in the tree; no source edits were needed this pass. Verified via read/grep against `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/`.

- Row 1 (chest contents scatter on any destroying removal): FIXED, VERIFIED.
  `AbstractChestBoat.remove` (AbstractChestBoat.java:71-77) scatters the 27 slots for every `reason.shouldDestroy()`, and `AbstractBoat.remove` (AbstractBoat.java:790-805) drops the leash on the same destroying removals. Pumpkin `BoatEntity::on_removed` (boat.rs:488-506) checks `removal_reason.should_destroy()` then calls `drop_leash` and `scatter_chest_contents` (boat.rs:272-288), with `claim_drops` (minecart/container.rs:39) limiting the scatter to once; the damage path also scatters (boat.rs:746), matching `AbstractChestBoat.destroy` (:65-68).
- Row 2 (shared `getMaxPassengers` helper): FIXED, VERIFIED.
  `AbstractChestBoat.getMaxPassengers` = 1 (AbstractChestBoat.java:47-50) and `AbstractBoat.getMaxPassengers` = 2 (AbstractBoat.java:840-842). Pumpkin's single source of truth `boat.rs::max_passengers` (boat.rs:95-101, using `CHEST_BOAT_MAX_PASSENGERS` = 1 and `BOAT_MAX_PASSENGERS` = 2) is reused by the mob-push mounting path (entity/mod.rs:1353-1354). No behaviour change.
- Row 3 (interact deciding result on a failed mount): FIXED, VERIFIED.
  `AbstractChestBoat.interact` (AbstractChestBoat.java:80-97) plus `AbstractBoat.interact` (:779-788) and `canAddPassenger` (:836-838) open the container whenever the ride attempt did not consume the click. The pure seam `chest_opens_container` (boat.rs:135-141) feeds `boat_interact_outcome` (boat.rs:161-177), used by `interact` (boat.rs:823-835); pure unit tests cover it (boat.rs:1218-1285). `open_custom_inventory_screen` (boat.rs:290-328) keeps only the alive guard, matching vanilla (:100-106), which has no rider/spectator check.

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-tentities-mangrove-chest-boat-0.md
