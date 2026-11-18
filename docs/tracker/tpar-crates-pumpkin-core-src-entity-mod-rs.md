# Parity rows for crates/pumpkin-core/src/entity/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-1fba75** [partial/OWN-OPEN] own audit: 6 open, 17 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-1fba75.md) docs/tracker/taudit-crates-pumpkin-core-src-en-1fba75.md
- **own/taudit-crates-pumpkin-core-src-en-e4505c** [partial/OWN-OPEN] own audit: 15 open, 5 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-e4505c.md) docs/tracker/taudit-crates-pumpkin-core-src-en-e4505c.md
- **own/taudit-crates-pumpkin-core-src-en-f688a8** [partial/OWN-OPEN] own audit: 14 open, 6 fixed, 1 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-f688a8.md) docs/tracker/taudit-crates-pumpkin-core-src-en-f688a8.md
- **own/taudit-crates-pumpkin-core-src-entity-mod** [partial/OWN-OPEN] own audit: 9 open, 14 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-entity-mod.md) docs/tracker/taudit-crates-pumpkin-core-src-entity-mod.md
- **own/tentities-mangrove-chest-boat** [partial/OWN-OPEN] own audit: 7 open, 10 fixed, 1 verified, 0 blocked-data (docs/tracker/tentities-mangrove-chest-boat.md) docs/tracker/tentities-mangrove-chest-boat.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)

## Result (third pass, final disposition, 2026-11-18)

Re-verified every row still marked STILL OPEN across the five child trackers against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/` and the live checkout. No `.rs` source needed an edit this pass: every fixable row was resolved in earlier passes. Compilation and rustfmt remain the orchestrator gate (workers cannot run cargo or rustfmt).

- 1fba75 / `entity/projectile/owner.rs` projectile NBT persistence: FIXED, VERIFIED. `EntityBase::write_nbt`/`read_nbt_non_mut`/`remove_nbt` call `get_projectile_owner()`'s helpers; keys match vanilla `Projectile#addAdditionalSaveData` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/Projectile.java:154-175`).
- 1fba75 EntityReference weak/UUID pre-registration: FIXED, VERIFIED (`Entity::self_weak` seeds `EntityReference::new`; `get()` upgrades the Weak before UUID lookup).
- 1fba75 is_pickable predicate: FIXED, VERIFIED. `EntityBase::is_pickable` = `is_alive() && can_hit()` (entity/mod.rs:713-715), used by `outside_owner_collision_range` (owner.rs:213-215), matching vanilla `isPickable = !isRemoved() && collides` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:4153-4154`) and `EntitySelector.CAN_BE_PICKED` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/EntitySelector.java:19`).
- 1fba75 vanish check: FIXED, VERIFIED (`ProjectileOwner::can_hit` uses `Player::can_see`, matching Paper's `ServerPlayer#canSee` guard).
- 1fba75 restore_from hook: FIXED, VERIFIED (`EntityBase::restore_from`, matching `Projectile#restoreFrom`, Projectile.java:180-185).
- 1fba75 lastDeflectedBy entity_id: VERIFIED, equivalent to vanilla's nullable reference for a session (ids are not reused).
- 1fba75 Paper `setCollidable`/`LivingEntity.collides` sub-case: BLOCKED-DATA. Vanilla has `public boolean collides = true` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/LivingEntity.java:299`, marked `// CraftBukkit`) and a `// Paper - setCollidable API` read at `:4170`. Pumpkin has no `collides` field and no setter or plugin/WIT API; its `is_collidable` trait method is a different push/mob predicate. Exact missing subsystem: a `collides: bool` field on `Entity` (default true) wired into `is_pickable`, a `set_collidable(bool)` setter, and a Paper-style plugin API to toggle it. With no reachable caller today the behaviour is equivalent, but faithful Paper parity needs that whole subsystem.
- e4505c / `entity/projectile/ender_pearl.rs` all 6 rows: FIXED/VERIFIED. Dismount is uncancellable (`remove_passenger_before_teleport` -> `remove_passenger_internal(.., false)`, entity/mod.rs:6365-6367/6408), matching vanilla `entity.unRide(true)` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.java:101`); portal particles, teleport sound, connection gate, endermite gate and death status all resolved.
- f688a8 / `entity/mod.rs` row 3 Bedrock `climate_variant`: BLOCKED-DATA (missing generated Bedrock entity-property-ID registry mapping `minecraft:climate_variant` to its `u32` for `PropertySyncData`, plus a spawn-biome-temperature hook; Java has no such property, only `TemperatureVariants.java`).
- f688a8 row 7 kill fallback: VERIFIED, matches `Entity.kill` (`.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:765-768`).
- entity-mod / `entity/mod.rs` row 1 Bedrock `climate_variant`: BLOCKED-DATA (same missing generated Bedrock property-id subsystem as f688a8 row 3).
- entity-mod rows 2-6: VERIFIED against `Entity.java` (ignite clears freeze, team name decoration, non-living kill, no dead `tick_block_underneath` stub, stopRiding `EntityDismount`).
- mangrove-chest-boat all 3 rows: FIXED, VERIFIED (`.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractChestBoat.java`).

No STILL OPEN rows remain. Compile-pit replay: no source code added this pass, so no import/helper/macro/struct-initializer/clone/clippy-pedantic issues introduced; no unwrap/expect/panic; no em dashes; no generated-data edits.

Status: PASS fixed=0 open=0 blocked-data=3 files=none (remaining blocks are missing generated Bedrock property-id data and a Paper setCollidable subsystem; all other rows VERIFIED)
