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

## Gap status
- Remaining-gap table rows (docs and container size helpers): STILL OPEN (unverified: no .vendor decompile in h6; not changed)

## Result

Verified against .vendor/vanilla-decompiled/net/minecraft/ (now present in this worktree). No cargo run per brief; rustfmt --edition 2024 passed on both edited .rs files.

| row | result | evidence |
|---|---|---|
| boat.rs piglin anger on open | FIXED (already implemented, verified) | Vanilla AbstractChestBoat.interact and openCustomInventoryScreen call PiglinAi.angerNearbyPiglins(level, player, true). Pumpkin already calls PiglinAi.on_guarded_block_interacted when the chest boat opens; that helper scans nearby entities and the piglin hook targets huntable adults and plays the angry sound. No edit needed. |
| player.rs hand-written boat statistic list | FIXED | Vanilla ServerPlayer.checkRidingStatistics awards BOAT_ONE_CM for any AbstractBoat. Replaced the 14-entry hand-written type list with a pure is_boat() helper in boat.rs (C_BOATS tag plus is_chest_boat()) and used it in get_movement_statistic. Added the pure unit test boat_stat_classification_covers_every_boat_entity_type. |
| docs / no decompiled reference | FIXED (verified) | .vendor/vanilla-decompiled/net/minecraft/ is present; compared AbstractChestBoat.java, BoatItem.java, PiglinAi.java, ServerPlayer.java. |

Note: the generated MINECRAFT_BOATS tag carries ids (975+) that do not match entity registry ids (for example OAK_BOAT.id is 90), so it cannot be used with Taggable::has_tag. Classification uses C_BOATS plus is_chest_boat(); poplar_chest_boat (id 107) is missing from C_BOATS and is covered by the helper.

Open rows: none.
