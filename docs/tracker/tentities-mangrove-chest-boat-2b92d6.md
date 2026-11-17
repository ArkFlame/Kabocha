# entities / Mangrove Chest Boat re-review

Scope: `crates/pumpkin-core/src/entity/vehicle/boat.rs`, `crates/pumpkin-core/src/item/items/boat.rs`, `crates/pumpkin-core/src/entity/type.rs`, `crates/pumpkin-core/src/entity/player.rs`. No `.vendor/` decompiled reference present in this worktree; comparison is against known vanilla `AbstractChestBoat` behaviour.

Verified wired: `is_chest_boat` includes `MANGROVE_CHEST_BOAT` (boat.rs:53); chest boat container size 27 and max passengers 1; open-inventory gating (alive, not spectator, empty or ridden by opener); title key; CONTAINER_OPEN game event; NBT read/write of inventory; container drops on break (boat.rs ~389-398); item to entity mapping (item/items/boat.rs:36); mangrove chest boat in the boat item list (item/items/boat.rs:86); entity type registration (type.rs:343); boat-one-cm statistic (player.rs:4423).

Remaining gaps:

| file:line | vanilla class | suggested fix | size |
|---|---|---|---|
| crates/pumpkin-core/src/entity/vehicle/boat.rs:150-151 | `AbstractChestBoat.interact` angers nearby piglins on open | Needs piglin anger AI hook; documented in code as unimplemented | M |
| crates/pumpkin-core/src/entity/player.rs:4406-4427 | `BoatItem`/statistics boat tag | Hand-written list duplicates `is_chest_boat` and boat tags; replace with tag or helper to avoid drift when a boat type is added | S |
| docs (tracker) | n/a | No `.vendor/` decompiled reference in this worktree, so the vanilla comparison is unverified against source | S |

Not compiled or tested (no cargo run per brief).

## Gapwave2-2 status (h3)
- boat.rs:150-151 piglin anger on open: STILL OPEN (M, no piglin anger hook in AI)
- player.rs:4406-4427 hand-written boat list: STILL OPEN (S, the list is the workaround for missing c:boats entries, blocked on codegen)
- docs row: UNVERIFIED (no .vendor decompiled reference)
