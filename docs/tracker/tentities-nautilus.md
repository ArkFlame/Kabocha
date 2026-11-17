# entities / Nautilus — re-review audit (2026-11-16)

Scope reviewed: `crates/pumpkin-core/src/entity/passive/nautilus.rs` (493 lines),
its shared helpers used by `zombie_nautilus.rs`, the mount-jump path in
`crates/pumpkin-core/src/entity/player.rs:7528-7546`, and registration
`crates/pumpkin-core/src/entity/type.rs:238`.

Source changed since the previous assessment by
`a36dbf86c feat(entity): implement zombie nautilus` and
`efaa388d8 fix(entity): remove the duplicate nautilus controlling passenger`.
No decompiled vanilla tree is present here (`Kabocha/.vendor/` is empty), so each
item names the vanilla class it must be checked against. Items tagged **verify**
need that source before editing; the rest are proven by in-tree evidence.

## Gaps

| # | Rust location | Vanilla reference | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `nautilus.rs:41-55` (`NautilusEntity::new`) | `Nautilus#registerGoals` / `AbstractNautilus#registerGoals` | No goals are ever registered for the nautilus (no `add_goal` in the file), unlike `zombie_nautilus.rs:79+`. A wild nautilus never floats, panics, breeds, follows a parent, is tempted or strolls; it stands still. | Port the vanilla goal list with vanilla priorities and control flags, same shape as `zombie_nautilus.rs`. | L |
| 2 | `nautilus.rs:57-67, 321-368`; `player.rs:7528-7546` | `AbstractNautilus#onPlayerJump` (`PlayerRideableJumping`) | Dash can never start: the only `set_dashing` call with `false` is the stop path at `:353-355`, and `dash_cooldown` is never assigned a start value (only NBT load and decrement), so the 35-tick cutoff and `EntityNautilusDashReady` are unreachable. The mount jump packet path only routes vehicles through `mob.as_abstract_horse()`, which the nautilus does not implement. | Add a rideable-jump hook on `Mob` (can_jump/handle_start_jump), implement it in `NautilusEntity` to play `get_dash_sound()`, set the vanilla dash cooldown and `set_dashing(true)`, and accept it in `Player::start_riding_jump`. | M |
| 3 | `nautilus.rs:38, 51` | mount inventory (`crates/pumpkin-inventory/src/mount_screen_handler.rs`) | The 9-slot `inventory` is allocated but never read or written: no NBT, no equip path, no screen handler. Dead weight or a half-wired Bedrock mount screen. | Vanilla nautilus carries saddle + body armour only. Either delete the field or wire it like camel/horse (`open_mount_screen`). | S |
| 4 | `nautilus.rs:158-164` (`Animal::is_food`) | `AbstractNautilus#isFood` -> `ItemTags.NAUTILUS_FOOD` | Food is hardcoded to shell/pufferfish/cod/salmon/tropical fish, so datapack tag changes are ignored. **Blocked**: no `nautilus_food` item tag is emitted in `pumpkin-data/src/generated/tag.rs` (needs Extractor assets). | Use the item tag once generated. | S (blocked) |
| 5 | `nautilus.rs:41-55` | `Nautilus` constructor, `setPathfindingMalus(PathType.WATER, 0.0F)` | No water pathfinding setup, while the other aquatic mobs do it (`abstract_fish.rs:35-36`, `dolphin.rs:83-88`, `zombie_nautilus.rs:59-67`). **verify** whether vanilla `Nautilus` also uses a water-bound/amphibious navigator before copying `Navigator::water_bound`. | Set the WATER malus in `new()` (plus the navigator choice vanilla uses). | S |
| 6 | `nautilus.rs` (absent) | `AbstractNautilus#tick` riding sound | `Sound::EntityNautilusRiding` (`pumpkin-data/src/generated/sound.rs:120`) is never played. **verify** the vanilla interval/volume/condition (ridden by a player). | Play it from the nautilus tick. | S |
| 7 | `crates/pumpkin-core/src/world/natural_spawner.rs` (absent) | nautilus spawn placement | Neither `Nautilus` nor `ZombieNautilus` appears in the natural spawner, although biome tags exist (`tag.rs:30248 spawns_coral_variant_zombie_nautilus`). | Register both with spawn group/weight/biome tags (verify vanilla weights). | M |
| 8 | `nautilus.rs:258-278` | `TamableAnimal#addAdditionalSaveData` | Hand-rolled taming state: writes non-vanilla `IsTame`/`DashCooldown` keys and duplicates the owner, while `ZombieNautilusEntity` uses `TamableData`. Two representations of one vanilla concept. **verify** whether vanilla persists the dash cooldown at all. | Move to `TamableData` (tame derived from owner) and drop the extra keys. | S |
| 9 | `nautilus.rs:442-456` | `Animal#mobInteract` food healing | Heal is `2.0 * nutrition`. **verify** the vanilla heal value for the nautilus food set before changing. | Match the vanilla heal amount. | S |

## Checked and already correct

- Slot gating (`nautilus.rs:477-482`) matches `canUseSlot`: extra slots only when alive, adult and tamed.
- Dispenser equipping (`nautilus.rs:214-218`) is `BODY`/`SADDLE` plus `canPickUpLoot`.
- Equip sound (`nautilus.rs:484-493`) uses the underwater/land saddle equip pair.
- Rider effect (`nautilus.rs:330-351`): breath of the nautilus, 60-tick duration, refreshed every 40 world ticks, ambient.
- Dash stop at cooldown < 35 and the dash-ready sound when it reaches 0 (`nautilus.rs:353-368`).
- Controlling passenger goes through `saddled_controlling_passenger(None)` (`nautilus.rs:228-230`); the duplicate was removed in `efaa388d8` and nothing else registers one.
- Silent step (`nautilus.rs:248`); baby/land ambient, hurt, death, eat and swim sound selection.
- Feeding consumes nautilus-bucket food by filling a water bucket (`nautilus.rs:401-414`).
