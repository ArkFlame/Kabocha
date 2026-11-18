# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-veh.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: container.rs minecart.rs

- minecart 5 (powered-rail boost on 3D length, minecart.rs:305-327): STILL OPEN, M.
- minecart 6 (container friction, minecart/container.rs:235-277): STILL OPEN, M.
- minecart 7 (passenger repositioning, minecart.rs:575-593): STILL OPEN, S.
- minecart 8 (slope tests, minecart.rs:64-103): STILL OPEN, S, test-only.

## Result
- minecart 5 (powered-rail boost on 3D length): FIXED - powered_rail_boost and its call site use the horizontal length (hypot(x, z)) and leave Y untouched, matching vanilla horizontalDistance() at .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/OldMinecartBehavior.java:233-238. Covered by test powered_rail_boost_uses_horizontal_speed_and_preserves_y.
- minecart 6 (container friction): VERIFIED - container::velocity matches vanilla AbstractMinecartContainer.applyNaturalSlowdown at .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/AbstractMinecartContainer.java:103-115: keep 0.98 plus (15 - signal) * 0.001 without a loot table, times 0.95 in water, multiply(keep, 0, keep). No generated-data or cross-crate need.
- minecart 7 (passenger repositioning): VERIFIED - minecart passenger_attachment_point returns the lowered zero offset for Villager or WanderingTrader and position_riders seats each rider at seat minus its own vehicle offset, matching vanilla AbstractMinecart.getPassengerAttachmentPoint (AbstractMinecart.java:198-200) and Entity.positionRider (Entity.java:3540-3544).
- minecart 8 (slope tests): FIXED - added ascending_rails_snap_off_center_to_vanilla_height, which fails with the old exit order. Corrected AscendingWest and AscendingNorth exit ordering to vanilla AbstractMinecart.EXITS (AbstractMinecart.java:83-85); center-only tests still pass and off-center snaps now match vanilla OldMinecartBehavior.getPos (OldMinecartBehavior.java:304-347).
