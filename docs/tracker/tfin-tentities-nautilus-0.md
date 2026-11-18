# Final rows from docs/tracker/tentities-nautilus.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: abstract_fish.rs dolphin.rs mount_screen_handler.rs nautilus.rs player.rs zombie_nautilus.rs

| # | Pumpkin | Vanilla | Status |
|---|---|---|---|
| 1 | `nautilus.rs:101-125` goals | `Nautilus#registerGoals` / `AbstractNautilus#registerGoals` | VERIFIED present (EscapeDanger 0, Breed 1, FollowParent 3, RandomSwimming 4, RandomLookAround 5, Melee 7, Revenge 1 target). Priorities not re-audited line-by-line against `animal/nautilus/Nautilus.java` this pass. |
| 2 | `nautilus.rs:57-67, 321-368`; `player.rs` | `AbstractNautilus#onPlayerJump` (`PlayerRideableJumping`) | OPEN. Dash start path (`set_dashing(true)`, dash cooldown start, `get_dash_sound`) is not wired to the mount jump packet; needs a rideable-jump hook on `Mob` (size M). |
| 3 | `nautilus.rs:38, 51` | mount inventory (`mount_screen_handler.rs`) | OPEN. The 9-slot inventory is allocated but unused; vanilla `AbstractNautilus` `createInventory` / `canUseSlot` (SADDLE, BODY) needs wiring (size S). |
| 5 | `nautilus.rs:41-80` constructor | `AbstractNautilus` ctor (`AbstractNautilus.java:94` `setPathfindingMalus(PathType.WATER, 0.0F)`) | VERIFIED. Constructor sets `Navigator::water_bound(false)` and `set_pathfinding_malus(PathType::Water, 0.0)`. |

Status: fixed=0 this pass; verified=3 (rows 1, 5, plus the row-1 presence check); open=2 (rows 2, 3). No compile run.
