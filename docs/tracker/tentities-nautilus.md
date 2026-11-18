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
| 1 | `nautilus.rs:41-55` (`NautilusEntity::new`) | `Nautilus#registerGoals` / `AbstractNautilus#registerGoals` | No goals are ever registered for the nautilus (no `add_goal` in the file), unlike `zombie_nautilus.rs:79+`. A wild nautilus never floats, panics, breeds, follows a parent, is tempted or strolls; it stands still. | Port the vanilla goal list with vanilla priorities and control flags, same shape as `zombie_nautilus.rs`. | L, STILL OPEN: size L, skipped per wave brief |
| 2 | `nautilus.rs:57-67, 321-368`; `player.rs:7528-7546` | `AbstractNautilus#onPlayerJump` (`PlayerRideableJumping`) | Dash can never start: the only `set_dashing` call with `false` is the stop path at `:353-355`, and `dash_cooldown` is never assigned a start value (only NBT load and decrement), so the 35-tick cutoff and `EntityNautilusDashReady` are unreachable. The mount jump packet path only routes vehicles through `mob.as_abstract_horse()`, which the nautilus does not implement. | Add a rideable-jump hook on `Mob` (can_jump/handle_start_jump), implement it in `NautilusEntity` to play `get_dash_sound()`, set the vanilla dash cooldown and `set_dashing(true)`, and accept it in `Player::start_riding_jump`. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 3 | `nautilus.rs:38, 51` | mount inventory (`crates/pumpkin-inventory/src/mount_screen_handler.rs`) | The 9-slot `inventory` is allocated but never read or written: no NBT, no equip path, no screen handler. Dead weight or a half-wired Bedrock mount screen. | Vanilla nautilus carries saddle + body armour only. Either delete the field or wire it like camel/horse (`open_mount_screen`). | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 4 | `nautilus.rs:158-164` (`Animal::is_food`) | `AbstractNautilus#isFood` -> `ItemTags.NAUTILUS_FOOD` | Food is hardcoded to shell/pufferfish/cod/salmon/tropical fish, so datapack tag changes are ignored. **Blocked**: no `nautilus_food` item tag is emitted in `pumpkin-data/src/generated/tag.rs` (needs Extractor assets). | Use the item tag once generated. | S (blocked) |
| 5 | `nautilus.rs:41-55` | `Nautilus` constructor, `setPathfindingMalus(PathType.WATER, 0.0F)` | No water pathfinding setup, while the other aquatic mobs do it (`abstract_fish.rs:35-36`, `dolphin.rs:83-88`, `zombie_nautilus.rs:59-67`). **verify** whether vanilla `Nautilus` also uses a water-bound/amphibious navigator before copying `Navigator::water_bound`. | Set the WATER malus in `new()` (plus the navigator choice vanilla uses). | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 6 | `nautilus.rs` (absent) | `AbstractNautilus#tick` riding sound | `Sound::EntityNautilusRiding` (`pumpkin-data/src/generated/sound.rs:120`) is never played. **verify** the vanilla interval/volume/condition (ridden by a player). | Play it from the nautilus tick. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 7 | `crates/pumpkin-core/src/world/natural_spawner.rs` (absent) | nautilus spawn placement | Neither `Nautilus` nor `ZombieNautilus` appears in the natural spawner, although biome tags exist (`tag.rs:30248 spawns_coral_variant_zombie_nautilus`). | Register both with spawn group/weight/biome tags (verify vanilla weights). | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 8 | `nautilus.rs:258-278` | `TamableAnimal#addAdditionalSaveData` | Hand-rolled taming state: writes non-vanilla `IsTame`/`DashCooldown` keys and duplicates the owner, while `ZombieNautilusEntity` uses `TamableData`. Two representations of one vanilla concept. **verify** whether vanilla persists the dash cooldown at all. | Move to `TamableData` (tame derived from owner) and drop the extra keys. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| 9 | `nautilus.rs:442-456` | `Animal#mobInteract` food healing | Heal is `2.0 * nutrition`. **verify** the vanilla heal value for the nautilus food set before changing. | Match the vanilla heal amount. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Checked and already correct

- Slot gating (`nautilus.rs:477-482`) matches `canUseSlot`: extra slots only when alive, adult and tamed.
- Dispenser equipping (`nautilus.rs:214-218`) is `BODY`/`SADDLE` plus `canPickUpLoot`.
- Equip sound (`nautilus.rs:484-493`) uses the underwater/land saddle equip pair.
- Rider effect (`nautilus.rs:330-351`): breath of the nautilus, 60-tick duration, refreshed every 40 world ticks, ambient.
- Dash stop at cooldown < 35 and the dash-ready sound when it reaches 0 (`nautilus.rs:353-368`).
- Controlling passenger goes through `saddled_controlling_passenger(None)` (`nautilus.rs:228-230`); the duplicate was removed in `efaa388d8` and nothing else registers one.
- Silent step (`nautilus.rs:248`); baby/land ambient, hurt, death, eat and swim sound selection.
- Feeding consumes nautilus-bucket food by filling a water bucket (`nautilus.rs:401-414`).

## Result

| # | Result |
|---|---|
| 1 | FIXED. Ported the vanilla `NautilusAi` list into `NautilusEntity::new`: panic 1.6 (core), `BreedGoal` 0.4, `TemptGoal` 1.3 on `MINECRAFT_NAUTILUS_FOOD`, `FollowParentGoal` 1.3, `RandomSwimmingGoal` 1.0/10, `RandomLookAroundGoal`. |
| 2 | FIXED. The `PlayerRideableJumping` hook already reached `NautilusEntity`; `handle_rider_start_jump` now also scales the impulse by `MOVEMENT_SPEED` and the block speed factor, as vanilla `executeRidersJump` does. Same correction applied in `zombie_nautilus.rs`. |
| 3 | FIXED. The dead 9-slot `inventory` field is deleted (vanilla `getInventoryColumns` is 0, so this mob has no mount screen). `zombie_nautilus.rs` had the same dead field and lost it too. |
| 4 | FIXED. The `nautilus_food` and `nautilus_taming_items` tags are generated, so `is_food` uses them: vanilla `AbstractNautilus#isFood` picks the taming tag for a wild adult and the food tag otherwise. |
| 5 | FIXED. `new()` installs `Navigator::water_bound(false)`, the WATER malus of 0.0, `SmoothSwimmingMoveControl(85, 10, 0.011, 0.0, true)` and `SmoothSwimmingLookControl(10)`, all from the vanilla constructor. |
| 6 | LEFT OPEN. Nothing in the decompiled tree reads `SoundEvents.NAUTILUS_RIDING`: `AbstractNautilus` only calls `startRiding` and `applyEffects`, and the client-side riding loop is not vendored. Needs the client source before adding a sound with an unknown interval. |
| 7 | VERIFIED, no change. Natural spawning is data driven: the spawner reads `biome.spawners.water_creature` (nine biomes list `minecraft:nautilus`, weight 2, 1-1) and `check_spawn_rules` already has the nautilus water/sea-level branch. `zombie_nautilus` is `MONSTER`/`Unrestricted` and absent from the biome assets, so it needs no registration. |
| 8 | FIXED. `mob_write_nbt` writes only `Owner` (vanilla `TamableAnimal`); `mob_read_nbt` derives taming from it, dropping the non-vanilla `IsTame` key. Tame state and owner now sync through `DATA_FLAGS_ID`/`DATA_OWNERUUID_ID` on tame and on tracker init. |
| 9 | VERIFIED, no change. `TamableAnimal#feed` heals `2.0 * nutrition` with a `1.0` fallback, matching the code. |

Also picked up in the same vanilla file: `isPushedByFluid` false, `checkSpawnObstruction` via `is_unobstructed`, `removeWhenFarAway` always true, and `requiresCustomPersistence` true when tamed.

Tests: replaced the stale dash-cooldown NBT test with `taming_persists_as_vanilla_owner_only` (owner-only NBT plus read-back) and added `wild_adult_only_eats_taming_items` (cooked cod is not food for a wild adult, pufferfish is, a tamed nautilus eats both). Not compiled here, per the wave brief.



## Result (re-review 2026-11-18, decompiled vanilla present)

Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled/net/minecraft/`,
now populated. All fixes recorded in the earlier Result section are present in the
code and match vanilla; the three "verify" rows were re-checked with source:

| # | Result |
|---...|
| 5 | VERIFIED, no change. `AbstractNautilus` ctor uses `SmoothSwimmingMoveControl(85,10,0.011,0,true)`, `SmoothSwimmingLookControl(10)`, `setPathfindingMalus(WATER,0.0)` and `createNavigation` -> `WaterBoundPathNavigation`; `NautilusEntity::new` matches all four. |
| 6 | LEFT OPEN, confirmed client-only. Full decompiled-tree search finds `SoundEvents.NAUTILUS_RIDING` only registered (`SoundEvents.java:139`) and never played; the nautilus riding loop is client-side and not vendored. The dash sound IS played server-side in `handle_rider_start_jump`, matching vanilla `handleStartJump`. No server-side seam to fix. |
| 7 | VERIFIED, no change. `checkNautilusSpawnRules` (sea-25..sea-5, water below+above) matches `entity/type.rs:786-798`; nautilus is data-driven in `water_creature` biome spawns. Zombie nautilus is MONSTER/Unrestricted and absent from biome assets. |
| 9 | VERIFIED, no change. `TamableAnimal.feed` heals `2.0 * nutrition` (fallback `1.0`) at `TamableAnimal.java:135-140`, nautilus calls `feed(...,2.0,1.0)`, and the Rust heal matches. |

Rows 1-4 and 8 remain as recorded above (FIXED); the code was re-read and matches
those results. No new code changes this pass: the fixes and the two unit tests
(`taming_persists_as_vanilla_owner_only`, `wild_adult_only_eats_taming_items`) are
already in place. Helper paths and struct initializers were re-checked against the
Worker compile pitfalls table.
