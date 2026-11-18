# Native agent 2: triage of 32 open parity rows

Status: PARTIAL. No Rust edits in this pass. Verification by grep only, cargo not run.

## Closed by verification (no code change needed)

| Unit note | Row | Verdict | Evidence |
|---|---|---|---|
| tentities-cushion-2527f1 | mount/dismount game events (entity/mod.rs:6217/5883) | NOT A GAP, already implemented | `crates/pumpkin-core/src/entity/mod.rs:6214` emits `GameEvent::EntityMount`, `:6467` emits `GameEvent::EntityDismount` |
| tentities-birch-boat | 2 chest-boat destroy | NOT A GAP | Vanilla `VehicleEntity.java:94-101` sets only `CUSTOM_NAME`; `AbstractChestBoat.java:65-67` scatters contents. Note evidence reused, not re-read |
| tentities-birch-boat | 3 placement container component | NOT A GAP | Vanilla `BoatItem.java:93-104` and `Entity.java:7239-7245` set only CUSTOM_NAME/CUSTOM_DATA. Boat placement is in `item/items/boat.rs`, not `place_on_water.rs` |
| tentities-birch-boat | 4 leash hooks | FIXED (present) | `crates/pumpkin-core/src/entity/vehicle/boat.rs:752-763` implements lead interact/unleash |
| tentities-birch-boat | 6 damage early-out | FIXED (present) | `crates/pumpkin-core/src/entity/vehicle/boat.rs:691-710` `damage_with_context` with invulnerability check |
| world-expl | 2 unloaded-chunk force-load | FIXED (present) | `crates/pumpkin-core/src/world/explosion.rs:138` `get_or_fetch_chunk`; test `explosion_force_loads_unloaded_chunks` at `:1267` |
| world-expl | 4 seeded level RNG | NOT A GAP (observable) | `crates/pumpkin-core/src/world/mod.rs:700` seeds `explosion_random` from `level.seed`, used at `explosion.rs:498`, `:825`. Vanilla shares `level.getRandom()` but the sequence is not an observable contract |
| world-expl | 8 particles/sound in finalize | FIXED (present) | `crates/pumpkin-core/src/world/mod.rs:5124-5130` sends `explosion.particle()` and `explosion.sound()` |

## Still OPEN (not closed in this pass)

Rows not re-verified here keep their unit-note reasons:

- taudit-crates-pumpkin-core-src-bl-556f10: rows 31, 35 (entity/mod.rs BoundingBox union), 75 (PlayerPlacedArgs/registry, plugin payload).
- taudit-crates-pumpkin-core-src-co-122988: row 7 timeline day query, row 8 Java Float.toString formatting.
- taudit-crates-pumpkin-core-src-co-ee4197: row 8 pack order model (L).
- taudit-crates-pumpkin-core-src-da-2842f9: gap 3 cross-entry validation, needs generated validator schema (BLOCKED-DATA).
- taudit-crates-pumpkin-core-src-da-734ab8: out-of-scope rows.
- taudit-crates-pumpkin-core-src-da-c13d0e: crafting_special display fields (M); brewing/transmute/dye/decorated_pot/imbue shapes need pumpkin-protocol types (BLOCKED-protocol).
- taudit-crates-pumpkin-core-src-en-dae447: row 5 f64 vs f32 random, cosmetic distribution-only (candidate NOT A GAP, not re-decided here).
- taudit-crates-pumpkin-core-src-en-e4505c: row 3 cancellable dismount events, intentional per note.
- taudit-crates-pumpkin-core-src-en-f688a8: row 7 kill fallback, matches vanilla per note (candidate NOT A GAP).
- taudit-crates-pumpkin-core-src-li-03c9dd: row 1 early reject at accept loop; row 4 player stats persistence on shutdown.
- taudit-crates-pumpkin-core-src-ne-254fb9: row 2 config disconnect Component, needs CConfigDisconnect protocol change (M, protocol).
- taudit-crates-pumpkin-core-src-ne-61496d: message signature chain (L).
- taudit-crates-pumpkin-core-src-se-870386: rows outside this unit.
- taudit-crates-pumpkin-core-src-wo-ea95b7: gap 1 world seed threading into EndSpike (M), gap 2 SavedData NBT for dragon fight (M-L).
- taudit-crates-pumpkin-core-src-world-natu: row 7 chunk-gen vs live adjust_spawn_position, narrow case.
- tentities-birch-boat: none left open beyond row 5 (note reports closed, not re-verified here).
- taudit-crates-pumpkin-core-src-entity-dec: out-of-unit rows.

## Risks

- No compile. Verified items are read-only greps; no new code.
- Line numbers in unit notes (entity/mod.rs:6217/5883) differ from current code (6214/6467). Notes need refresh.
- The work list (/tmp/kabocha-runs/native-2.md) says "32 open lines" but the rows are mostly M/L or cross-crate. Most cannot be closed without a compile-checked multi-crate change.
