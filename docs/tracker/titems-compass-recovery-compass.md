# items / Compass & Recovery Compass (RE-REVIEW)

Target: JavaMinecraftVersion::V_26_3. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft` (`world/item/CompassItem.java`,
`world/item/component/LodestoneTracker.java`).

Reviewed source change: `1c14e29ee` (wired remaining 26.3 data components) added
`Player::tick_inventory_item_components`, so two lodestone validators now ran every player
tick: that one and `tick_compasses` from `95a1277ce` (#4567).

Fixed in this pass:

- `entity/player.rs` `tick_compasses` (old lines 6936-6997) and its call in the tick
  (old line 3625) removed. It duplicated `tick_inventory_item_components` (same
  `LODESTONE_TRACKER` clear, same content update) but (a) ignored `tracker.tracked`, which
  `LodestoneTracker#tick` requires, and (b) validated through
  `PortalPoiStorage::exists`, which calls `get_or_load_region` (disk IO under the world POI
  mutex, per compass per tick) and unbinds a compass whose lodestone block still exists but
  has no POI entry (e.g. a `/setblock` lodestone, which never runs
  `LodestoneBlock::placed`). Paper's `LodestoneTracker#tick` checks the loaded chunk's block
  state and must not load chunks; `tick_inventory_item_components` already does exactly
  that, off-hand included (`equipment_slots` covers `OFF_HAND_SLOT`).

Verified as matching 26.3, no change needed:

- `CompassItem#useOn` lock flow: lodestone-only, `LODESTONE_COMPASS_LOCK` on
  `SoundSource.PLAYERS`, a single non-creative stack gets the component in place, otherwise
  `transmuteCopy(COMPASS, 1)` + `consume(1)` + `Inventory#add` with drop fallback. Order
  (`consume` before `add`) matches the reference.
- `LodestoneTracker#tick` keeps the target when `target.dimension != level.dimension` and
  while the target chunk is unloaded; it clears to `(empty, tracked=true)` otherwise.
  `tick_inventory_item_components` matches both.
- Recovery compass: not bindable at a lodestone (`use_on_block` gates on
  `Item::COMPASS`); `last_death_location` is written on death as
  `GlobalPos(dimension, floored block pos)` and sent in `CLogin`/`CRespawn` through
  `PlayerSpawnData` (1.19+ field), which is all the server owes the client.
- Needle direction, lodestone-compass glint (`CompassItem#isFoil`) and the
  `item.minecraft.lodestone_compass` name are computed client-side from the component, so
  the server does not reproduce them.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `entity/player.rs:6822-6831` | `LodestoneTracker#tick` returns the empty tracker when `!level.isInWorldBounds(pos)` | No bounds check; clearing relies on `get_block` returning a non-lodestone for an out-of-world Y | Add a world-bounds guard before reading the block | S |
| `entity/player.rs:753,1129,4845-4853` | No vanilla counterpart | `compass_target` plus `set_compass_target`/`get_compass_target` has no reader and only ever stores `None`; leftover from the spawn-compass TODO at `world/mod.rs:4589` | Delete the field and both accessors | S |
| `item/items/compass.rs:55-66` | `CompassItem#useOn` branches on `player.hasInfiniteMaterials()` (abilities `instabuild`, true for creative and spectator) | Branches on `gamemode != Creative`; spectator never reaches `use_on_block`, so no observable difference today | Use an ability/`has_infinite_materials` check when one exists | S |
