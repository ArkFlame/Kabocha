# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-mod-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

No STILL OPEN rows remain. Per-row terminal disposition (full detail in the child trackers' final Result sections and in the third-pass section below):
Re-verified every row still marked STILL OPEN across the five child trackers against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/` and the live checkout. No `.rs` source needed an edit this pass: every fixable row was resolved in earlier passes. Compilation and rustfmt remain the orchestrator gate (workers cannot run cargo or rustfmt).
No STILL OPEN rows remain. Compile-pit replay: no source code added this pass, so no import/helper/macro/struct-initializer/clone/clippy-pedantic issues introduced; no unwrap/expect/panic; no em dashes; no generated-data edits.

## Result
## Result (FINAL PASS, 2026-11-18)

Terminal disposition for the five child trackers of `tpar-crates-pumpkin-core-src-entity-mod-rs.md`. Independently re-read the relevant Pumpkin source (`entity/mod.rs`, `entity/projectile/owner.rs`, `entity/projectile/ender_pearl.rs`, `entity/vehicle/boat.rs`) and re-checked every row against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/`. Every fixable row was already applied in earlier passes and is present in the current checkout; the only non-FIXED rows are two BLOCKED-DATA subsystems. No new code was needed this pass, so no rustfmt was required.

### 1fba75 - `entity/projectile/owner.rs`
- Projectile owner NBT persistence (`OWNER_UUID`/`OWNER_ID`): FIXED, VERIFIED (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/Projectile.java:154-175,180-185`); `EntityBase::write_nbt`/`read_nbt_non_mut`/`remove_nbt` use the `get_projectile_owner()` helpers.
- EntityReference weak/UUID pre-registration: FIXED, VERIFIED (`Entity::self_weak` seeds `EntityReference::new`; `get()` upgrades the Weak before the UUID lookup).
- `is_pickable` predicate: FIXED, VERIFIED. `EntityBase::is_pickable` = `is_alive() && can_hit()` (entity/mod.rs:713-715), used at owner.rs:213-215, matching vanilla `isPickable = !isRemoved() && collides` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:4153-4154`) and `EntitySelector.CAN_BE_PICKED` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/EntitySelector.java:19`).
- Vanish check: FIXED, VERIFIED (`ProjectileOwner::can_hit` uses `Player::can_see`, Paper `ServerPlayer#canSee`).
- `restore_from` hook: FIXED, VERIFIED (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/Projectile.java:180-185`).
- `lastDeflectedBy` id: VERIFIED, equivalent to vanilla's nullable reference (ids are not reused within a session).
- Paper `setCollidable` / `LivingEntity.collides`: BLOCKED-DATA. Vanilla keeps `public boolean collides = true` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:299`) read by `isPickable` (`:4154`) and the `// Paper - setCollidable API` path (`:4170`). Exact missing subsystem: a `collides: bool` field on `Entity` (default true) wired into `is_pickable`, a `set_collidable(bool)` setter, and a Paper-style plugin/WIT API. No reachable caller today, so behaviour is equivalent; requires WIT/plugin API changes (out of scope).

### e4505c - `entity/projectile/ender_pearl.rs` (rows 1-6)
- Row 1 portal particle burst: FIXED, VERIFIED (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java:165-175`).
- Row 2 teleport sound for non-player owners: VERIFIED (closed false positive), ThrownEnderpearl.java:258-264.
- Row 3 cancellable dismount: FIXED, VERIFIED. `remove_passenger_internal` gained a `cancellable: bool` parameter (entity/mod.rs:6432); `remove_passenger_before_teleport` passes `false` (entity/mod.rs:6389-6391), so `EntityDismountEvent`/`VehicleExitEvent` cancellation is ignored on the teleport path, matching vanilla `entity.unRide(true)` (ThrownEnderpearl.java:101). `remove_passenger` still passes `true` (entity/mod.rs:6383), preserving normal cancellable semantics.
- Row 4 connection-open gate: FIXED, VERIFIED (ThrownEnderpearl.java:211; `should_spawn_monsters()` world/mod.rs:8346-8351 matches `SPAWN_MOBS && SPAWN_MONSTERS && !PEACEFUL`).
- Row 5 endermite gate: FIXED, VERIFIED (ThrownEnderpearl.java:199,234).
- Row 6 death entity status: FIXED, VERIFIED (ThrownEnderpearl.java:304-306).

### f688a8 - `entity/mod.rs` (rows 1-7)
- Row 1 display-name team decoration: FIXED, VERIFIED (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:6231`, `.vendor/vanilla-decompiled/net/minecraft/world/scores/PlayerTeam.java:166`); `format_name_for_team` (entity/mod.rs:258) applied in `get_display_name` (entity/mod.rs:1475-1497). Pure unit tests at entity/mod.rs:7376-7378.
- Row 2 igniting clears freeze: FIXED, VERIFIED (Entity.java:1168-1173, 4083); `set_on_fire_for_ticks` calls `set_frozen_ticks(0)` (entity/mod.rs:1128-1131).
- Row 3 Bedrock `climate_variant`: BLOCKED-DATA. Exact missing generated field: the Bedrock synced-property id (u32) for `minecraft:climate_variant`. `PropertySyncData` (`crates/pumpkin-protocol/src/bedrock/client/set_actor_data.rs:161-164`) is a raw `HashMap<u32,i32>`/`HashMap<u32,f32>` with no property-id registry (the `entity_data_key` constants are ActorDataIDs, a different numbering). Needs a generated Bedrock entity-property table plus a spawn-biome-temperature hook; Java has only data-driven variants (`TemperatureVariants.java`), no such property. Guessing an id would corrupt the AddActor packet. Client-only Bedrock visual effect.
- Row 4 `tick_block_underneath` stub: FIXED, VERIFIED. StepOn behaviour lives in `LivingEntity.tick` via `block_registry.on_entity_step`; the dead `on_stepped_on` stub is absent from entity/mod.rs.
- Row 5 teleport broadcast to destination chunk: FIXED, VERIFIED (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:6057`); `Entity::teleport` sends `CEntityPositionSync` through `world.send_to_tracking_players` to current viewers (entity_tracker.rs:1002-1011).
- Row 6 `set_rotation` normalisation: FIXED, VERIFIED (Entity.java:913); entity/mod.rs:4507-4514 normalises yaw with `yaw % 360.0` and zeroes non-finite yaw.
- Row 7 kill fallback: VERIFIED (Entity.java:765-768); living entities take `DamageType::GENERIC_KILL`, non-living emit `GameEvent::EntityDie` plus container scatter.

### entity-mod - `entity/mod.rs` (rows 1-6)
- Row 1 Bedrock `climate_variant`: BLOCKED-DATA (same missing generated Bedrock property-id registry for `minecraft:climate_variant` plus a spawn-biome-temperature hook; entity/mod.rs:1010-1015 TODO kept). Java-side variant parity is already satisfied (`select_for_biome` in pig.rs/cow.rs/chicken.rs matching `PigVariants.bootstrap`, Pig.java:321-325).
- Rows 2-6: VERIFIED against Entity.java - igniting clears freeze, team name decoration, non-living kill emits GameEvent::EntityDie, no dead tick_block_underneath stub, stopRiding emits EntityDismount. No source change needed.

### mangrove-chest-boat - `entity/vehicle/boat.rs` (rows 1-3)
- Row 1 chest contents scatter on any destroying removal: FIXED, VERIFIED (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java:71-77,65-68`; `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/ContainerEntity.java:84-91`); `BoatEntity::on_removed` (boat.rs:488-506) checks `should_destroy()` and calls `scatter_chest_contents` (boat.rs:272-288), guarded by `claim_drops`.
- Row 2 shared `getMaxPassengers` helper: FIXED, VERIFIED (AbstractChestBoat.java:47-50); `max_passengers` (boat.rs:95-101) is the single source of truth (CHEST_BOAT_MAX_PASSENGERS = 1, BOAT_MAX_PASSENGERS = 2), reused at entity/mod.rs:1350-1357.
- Row 3 interact decides result on a failed mount: FIXED, VERIFIED (AbstractChestBoat.java:80-97,100-106); pure `chest_opens_container` (boat.rs:135-141) feeds `boat_interact_outcome` (boat.rs:161-177), used by `interact` (boat.rs:823-835), with pure unit tests (boat.rs:1217-1285).

No STILL OPEN rows remain. Compile-pit replay: no `.rs` source added or changed this pass, so no import/helper/macro/struct-initializer/clone/clippy-pedantic issues introduced; no unwrap/expect/panic/println; no em dashes; no generated-data edits. Cargo and rustfmt remain the orchestrator gate.

Status: PASS fixed=0 open=0 blocked-data=3 files=docs/tracker/tfin-ar-crates-pumpkin-core-src-entity-mod-rs-0.md (two distinct missing subsystems: the Paper `collides`/`set_collidable` plugin API and the generated Bedrock `minecraft:climate_variant` property-id registry; every other row VERIFIED)
