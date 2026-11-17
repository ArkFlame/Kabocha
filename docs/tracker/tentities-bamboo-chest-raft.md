# entities / Bamboo Chest Raft

Coverage: RE-REVIEW-UNTRACKED (source changed under the previous assessment).
Audited: 2026-11-16.

Source: `crates/pumpkin-core/src/entity/vehicle/boat.rs` (`BoatEntity` handles all 22 boat and
chest boat types, including `BAMBOO_CHEST_RAFT`), registration in `entity/type.rs:337`,
`entity/vehicle/boat.rs:36` (`is_chest_boat`), item placement in `item/items/boat.rs:44`,
loot/damage helpers in `entity/vehicle/vehicle.rs`.

## Covered (matches vanilla AbstractBoat / ChestBoat)

- 27-slot container only for chest boats/rafts (`boat.rs:31,60`), custom name + contents NBT
  (`boat.rs:167-177`), container screen titled from the entity type (`boat.rs:75-102`).
- Sneak or full/sealed boat opens the container, otherwise boards (`boat.rs:286`), matching
  `ChestBoat.interact` routing through `canAddPassenger`/`isSecondaryUseActive`.
- 60-tick underwater boarding lock (`boat.rs:215-220,286`), 1 passenger for chest rafts, 2 for
  plain rafts (`boat.rs:71`).
- Damage wobble, creative instant break, hurt metadata (`vehicle.rs:216-285`).
- Paddle metadata driven by `ServerboundPaddleBoat` (`net/java/play/paddle_boat.rs:5`).
- Bamboo raft bedrock title fallback (`boat.rs:80-88`).
- Lava ignition/destruction via fluid `entityInside` (`entity/mod.rs:6599+`).

## Fixed in this audit

- `boat.rs:208 tick` never ran `Entity.baseTick`, so boats kept `first_tick`, never decremented
  fire ticks (a burning boat burned forever), never decremented `riding_cooldown` (dismounted
  player could not re-board), never ran the portal/out-of-world checks and never updated
  `last_pos`. Now runs `base_tick_without_fluid_push` + `update_last_pos`
  (`entity/mod.rs:6585-6612`, boat.rs:208-219).

## Remaining gaps

| # | Location | Vanilla class/method | Gap | Fix | Size |
|---|---|---|---|---|---|
| 1 | `boat.rs:233` | `AbstractBoat.canBeCollidedWith` | Returns `true` unconditionally; vanilla returns `!isRemoved()` | `!self.vehicle.entity.is_removed()` | S |
| 2 | `boat.rs:286` | `AbstractBoat.interact`, `ChestBoat.interact` | No `is_alive()`/removed guard; a dead/removed boat can still board a player or open its menu | Early `PASS` when `!is_alive()` | S |
| 3 | `boat.rs:208` | `AbstractBoat.tick` -> `tickBubbleColumn` | Upward bubble columns do not lift the boat, downward ones do not sink it | Read `BubbleColumnBlock` state under the boat and set vertical velocity, then flag `velocity_dirty` | M |
| 4 | `boat.rs:256` | `ChestBoat.destroy` / `ContainerHelper.dropContents` | Contents are dropped only when `doEntityDrops` is on and never on a creative break; vanilla drops the 27 slots regardless of the gamerule (only the boat item is gamerule-gated). No `.vendor` copy to confirm ordering | Unconditional `scatter_inventory`, keep the item drop gated | S |
| 5 | `boat.rs:270` + `vehicle.rs:190` | `Boat.destroy` (hardcoded drop) | `kill_and_drop_self` also rolls `minecraft:entities/<type>` and `boat.rs` drops the item from code: a datapack table makes the boat drop twice | Skip the loot-table roll for boats (`is_chest_boat`/boat types) or drop only from one place | S |
| 6 | `boat.rs:215` | `Entity.updateFluidInteraction` push | Fluid current push deliberately skipped for boats (client predicted); vanilla boats are pushed by flowing water/in water columns | Only if server-side divergence is observed: apply the push when no controlling passenger | M |
