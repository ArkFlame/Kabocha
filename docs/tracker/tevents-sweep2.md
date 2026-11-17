# Plugin event sweep 2

Audit of the events named in the sweep: PluginDisable ordering, ServiceUnregister, Vehicle collision, chunk load/unload, world load/unload. No code changed; no cargo run.

| Event | Native type | Fire site | Status | Reason |
|---|---|---|---|---|
| VehicleCollisionEvent | `events/vehicle/vehicle_collision.rs` | `entity/vehicle/vehicle.rs` `collide_entity`, `collide_block` (base fired before specific) | DONE (already wired) | Nothing to add. |
| WorldLoadEvent | `events/world/world_load.rs` | `server/mod.rs` (2 sites) | DONE (already wired) | Nothing to add. |
| WorldUnloadEvent | `events/world/world_load.rs` | `world/mod.rs` | DONE (already wired) | Nothing to add. |
| ChunkUnloadEvent | `events/world/chunk_unload.rs` | `world/mod.rs` `unload_chunk` | DONE (already wired) | Nothing to add. |
| ChunkLoad | `events/world/chunk_load.rs` | none | BLOCKED | Chunk loads complete inside `pumpkin-world` (`level.rs`, `chunk_system`), which has no callback into the core plugin manager. Needs a listener from core to world, then an async fire with the `Arc<RwLock<ChunkData>>`. WASM hosts already register it. |
| ChunkSave | none | none | BLOCKED | Same reason: saves run in `pumpkin-world` `chunk_saver`. |
| ServiceUnregisterEvent | `events/server/service_unregister.rs` | `server/mod.rs` `unregister_service`, which has no caller | BLOCKED | Services are removed inside the plugin unload sweep, under `plugin_manager.services` lock. Firing a cancellable event there risks deadlock and cancel semantics are undefined (a cancel cannot keep a service the owner is already dropping). Needs a design decision on cancel and lock release. |
| PluginDisableEvent | `events/server/plugin_disable.rs` | `server/mod.rs` `disable_plugin`, which has no caller | BLOCKED | The real disable is `plugin/mod.rs` `unload` (claimed record, `plugin_states` and `plugins` locks, then library release). Ordering must fire before the library is released and outside the locks. Needs that restructuring, not a one-line call. |

WIT: no new events added, so no WIT change. `ChunkLoad` and `ServiceUnregister` WIT status unchanged.

Tests: none added; no behaviour changed.
