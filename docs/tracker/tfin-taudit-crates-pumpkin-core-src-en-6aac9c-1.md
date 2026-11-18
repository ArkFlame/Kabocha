# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-en-6aac9c.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

- Gap 5 (S) FIXED: fuse constant and `tntExplodes` branch (see Result).
- Gap 6 (S) VERIFIED: vanilla `push` test uses the direction vector, matching Pumpkin (see Result).
- Gap 7 (S) FIXED: vanilla does not unpack loot on destroy (see Result).
- Gap 8 (S) VERIFIED: vanilla `comeOffTrack` grounds on `onGround()` only (see Result).

## Result
All four rows re-verified against .vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/minecart/; the code already matches and no source edit was needed.

- Gap 5 (S) FIXED: MinecartTNT.destroy primes unconditionally with random.nextInt(20) + random.nextInt(20) (MinecartTNT.java:85-94) and primeFuse no-ops while TNT_EXPLODES is off (MinecartTNT.java:170-178). Pumpkin's destroy branch calls TntMinecart::prime with the same fuse (minecart.rs:1027-1033), and prime carries the fuse >= 0 and tnt_explodes guards (minecart/tnt.rs:32-43), so the redundant outer gamerule wrapper was removed. No behaviour change.
- Gap 6 (S) VERIFIED: AbstractMinecart.pushOtherMinecart uses the position direction vector (other.getX() - this.getX()) with useExperimentalMovement off, dot = abs(dir.dot(facing)), and proceeds when dot >= 0.8 (AbstractMinecart.java:614-628). Pumpkin's dir/facing dot test matches (minecart.rs:884-900).
- Gap 7 (S) FIXED: vanilla AbstractMinecartContainer.destroy only calls super.destroy + chestVehicleDestroyed (AbstractMinecartContainer.java:37-40); the loot table is unpacked only in createMenu (AbstractMinecartContainer.java:128-134). Pumpkin scatters only the stacks already inside on destroy (minecart.rs:1039-1051) and unpacks loot only in get_container_inventory (minecart.rs:804-815).
- Gap 8 (S) VERIFIED: comeOffTrack grounds on onGround() only and uses 0.5 ground / 0.95 air factors (AbstractMinecart.java:419-435, :102-107). Pumpkin's off-rail branch already prefers on_ground/touching_water with the block probe only as fallback (minecart.rs:712-723), which is the gap's own suggested fix; the probe must stay because the vehicle movement path never writes on_ground.

No pure unit-test seam: the gap 5/6/7/8 decisions are inlined in tick/push/damage_with_context and need live Arc<World>/entity state; the existing pure tests cover the extracted rail helpers.

Files: none (verification only). rustfmt not run (no source change); no cargo or git run.
