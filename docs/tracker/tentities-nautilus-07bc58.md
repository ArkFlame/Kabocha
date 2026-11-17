# entities / Nautilus — re-review audit (2026-11-16, worktree 07bc58)

Scope: `crates/pumpkin-core/src/entity/passive/nautilus.rs` (489 lines), the shared
helpers it exports for `zombie_nautilus.rs`, entity construction
(`crates/pumpkin-core/src/entity/type.rs:238`) and the spawn path
(`crates/pumpkin-core/src/entity/spawn_util.rs:157`).
Source changed since the previous assessment by `a36dbf86c feat(entity): implement
zombie nautilus` and `efaa388d8 fix(entity): remove the duplicate nautilus
controlling passenger`. No decompiled vanilla tree is present (`.vendor/` empty), so
each item names the vanilla class to check; **verify** marks items that need it.

## Fixed in this pass

- `nautilus.rs:247` `check_spawn_obstruction`: the `Mob` default rejects any bounding
  box containing liquid, so a water-only spawn (`type.rs:771`) was always obstructed
  at `spawn_util.rs:157` and the nautilus could never spawn. Now uses
  `water_animal::check_spawn_obstruction` (`is_unobstructed`), like `zombie_nautilus.rs:241`.
- Taming state moved to `TamableData` (`TamableAnimal` impl). The nautilus tracked
  data already carries `DATA_FLAGS_ID`/`DATA_OWNERUUID_ID` (ids 18/19 in
  `pumpkin-data/src/generated/tracked_data.rs`), but the hand-rolled
  `is_tame`/`owner` pair never synced them, persisted a non-vanilla `IsTame` key and
  skipped the tame advancement. `write_tamable_nbt`/`read_tamable_nbt` still read
  legacy `IsTame`.
- `Animal::is_food` (`nautilus.rs:148`) now uses `tag::Item::MINECRAFT_NAUTILUS_FOOD`
  (generated at `pumpkin-data/src/generated/tag.rs:25961`) instead of a hardcoded
  shell + raw-fish list. The tag was previously missing, which is why the old audit
  marked this blocked; it exists now. Note it contains cooked fish and fish buckets
  and does **not** contain `nautilus_shell`.

## Gaps

| # | Rust location | Vanilla reference | Gap | Suggested fix | Size |
|---|---|---|---|---|---|
| 1 | `nautilus.rs:41-60` (`NautilusEntity::new`) | `Nautilus#registerGoals` / `AbstractNautilus#registerGoals` | No goals are registered (no `add_goal` in the file), unlike `zombie_nautilus.rs:79`. A wild nautilus never floats, panics, breeds, follows a parent, is tempted or strolls. | Port the vanilla goal list with vanilla priorities and control flags, same shape as `zombie_nautilus.rs`. **verify** the exact list. | L |
| 2 | `nautilus.rs:59-65, 317-364`; `player.rs` mount-jump path | `AbstractNautilus#onPlayerJump` (`PlayerRideableJumping`) | Dash never starts: `set_dashing(true)` is never called and `dash_cooldown` is only read from NBT or decremented, so the 35-tick cut-off and `EntityNautilusDashReady` are dead code. The mount jump packet only routes vehicles through `as_abstract_horse()`, which the nautilus does not implement. | Add a rideable-jump hook on `Mob` (can_jump / start_jump), implement it on `NautilusEntity` (play `get_dash_sound()`, set the vanilla cooldown, `set_dashing(true)`), accept it in `Player::start_riding_jump`. | M |
| 3 | `nautilus.rs:392-444` (`interact_abstract_nautilus`) | `AbstractNautilus#mobInteract`, `TamableAnimal#isTamingItem` | The untamed branch tames with any `nautilus_food` item, but `MINECRAFT_NAUTILUS_TAMING_ITEMS` (`tag.rs:25979`, pufferfish + bucket) exists and the zombie variant uses it. Cod therefore tames a nautilus. | Give the helper an `is_taming_item` predicate and use it in the `!is_tame` branch. | S |
| 4 | `nautilus.rs:37, 49` | vanilla has saddle + body only | The 9-slot `inventory` is never read or written (no NBT, no equip path, no screen handler). | Delete the field, or wire it like camel/horse if the Bedrock mount screen needs it. | S |
| 5 | `nautilus.rs:41-60` | `Nautilus` constructor, `setPathfindingMalus(PathType.WATER, 0.0F)` | No water pathfinding setup, while `zombie_nautilus.rs:62-67` and the other aquatic mobs do it. **verify** whether vanilla `Nautilus` is water-bound or amphibious before choosing the navigator. | Set the WATER malus in `new()` (plus the navigator vanilla uses). | S |
| 6 | `nautilus.rs` (absent) | `AbstractNautilus#tick` riding sound | `Sound::EntityNautilusRiding` is never played. **verify** interval/volume/condition (ridden by a player). | Play it from the nautilus tick when a player rides it. | S |
| 7 | `world/natural_spawner.rs` (absent) | nautilus spawn placement | Neither `Nautilus` nor `ZombieNautilus` appears in the natural spawner, although `type.rs:771` already implements the nautilus spawn rules. | Register both with their spawn group/weight/biome tags. **verify** the vanilla weights. | M |
| 8 | `nautilus.rs` (absent) | `WaterAnimal#isPushedByFluid` | `mob_is_pushed_by_fluids` is not overridden, unlike every other water mob (`abstract_fish.rs:323`, `dolphin.rs:297`, `turtle.rs:318`, `zombie_nautilus.rs:237`). **verify** that `AbstractNautilus` inherits the `false` override. | Return `false`. | S |
| 9 | `nautilus.rs` (absent) | `Mob#requiresCustomPersistence` / `removeWhenFarAway` | A tamed or saddled nautilus despawns like a wild one; `zombie_nautilus.rs:249-255` keeps tamed/mounted instances. **verify** the vanilla rule for the tamed mount. | Mirror the zombie variant once verified. | S |
| 10 | `nautilus.rs:443-449` | `Animal#mobInteract` heal amount | Heal is `2.0 * nutrition`. **verify** the vanilla heal factor for `nautilus_food`. | Match the vanilla amount. | S |

## Checked and correct

- Slot gating (`nautilus.rs:473-478`), dispenser equipping (`:214-218`), saddle equip
  sound pair (`:480-489`).
- Rider effect: breath of the nautilus, 60 ticks, refreshed every 40 world ticks,
  ambient (`:326-347`).
- Dash stop at cooldown < 35 and dash-ready sound at 0 (`:349-364`).
- Silent step (`:244`), baby/land ambient, hurt, death, eat and swim sound selection.
- Feeding consumes nautilus-bucket food by filling a water bucket (`:397-410`).
