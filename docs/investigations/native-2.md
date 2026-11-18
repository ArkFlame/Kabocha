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

## Second pass (2026-11-18, after coordinator update)

Edits in tree (not committed: git index.lock contention, supervisor commits the tree):

- wo-ea95b7 gap 1 (End spike seed): FIXED. `crates/pumpkin-world/src/generation/proto_chunk.rs` gains a `world_seed` field set in `ProtoChunk::new` from `noise_gen.random_config.seed` / `flat_gen.seed` (Custom: 0), and `impl GenerationCache for ProtoChunk` overrides `world_seed()`. The chunk-system `Cache` already threaded the seed in the Features stage (`generation_cache.rs:756`), so the pipeline was correct; this closes the ProtoChunk fallback to 0. Dragon fight already receives `level.seed` (`world/mod.rs:662`). Compile check needed: no new struct initializer exists besides `ProtoChunk::new`.
- ne-254fb9 row 2 (config disconnect Component): FIXED (present). `crates/pumpkin-core/src/net/java/mod.rs:699` sends `CConfigDisconnect::new(reason)` with the TextComponent; `pumpkin-protocol/src/java/client/config/config_disconnect.rs` writes it as a component.
- cushion mount/dismount events: NOT A GAP (already emitted, `entity/mod.rs:6214`, `:6467`).
- en-f688a8 row 7 (kill fallback): NOT A GAP. Matches `Entity.kill` (Entity.java:765-768); no change.
- en-e4505c row 3 (cancellable dismount): NOT A GAP. Intentional per unit note; vanilla removes regardless of cancelled dismount.
- en-db7d33 row 5: NOT A GAP (false positive, owner NBT round trip matches).
- en-dae447 row 5 (f64 vs f32 random): NOT A GAP. Decided: differs only in the bit stream of the cosmetic distribution, not in any protocol or gameplay value.
- birch boat row 5 (movement): NOT A GAP. Vanilla has no server-side paddle path; paddles are client-driven (`set_paddles` is metadata only).

Still OPEN with exact seams (not implemented in this pass):

- entityInside sweep: vanilla `Entity.checkInsideBlocks` (Entity.java:1951-2050, visitedBlocks + step-based effect collector) vs Pumpkin `check_block_collision` (`entity/mod.rs:5282`, outline-based) and `check_inside_blocks` (`entity/mod.rs:3300`). Needs `OnEntityCollisionArgs` in `block/mod.rs:419` extended and every impl updated (buttons, pressure plates, tripwire, detector rail, eyeblossom, wither rose, lily pad, crops x5, torch flower, boat block_effects).
- fluid push: `base_tick_without_fluid_push` (`entity/mod.rs:7104`) exists and is used by boat.rs:584; not re-audited against `Entity.updateFluidOnEyes`/`push` (Entity.java).
- dimension change dedup player.rs vs world/mod.rs: not examined.
- difficulty damage scaling: not examined.
- message signature chain (ne-61496d): needs per-player profile key storage and SignedMessageChain unpacker in `net/java/play/chat_message.rs`; crypto deps (rsa, ed25519-dalek, sha2) are in pumpkin-core Cargo.toml.
- Other rows (bl-556f10 31/35/75, co-122988 7/8, co-ee4197 8, da-2842f9 gap 3, da-734ab8, da-c13d0e x2, en-db7d33 already listed, li-03c9dd 1/4, se-870386, wo-ea95b7 gap 2, world-natu 7): unchanged from the first pass; exact files are in the unit notes.

Status: PARTIAL. 1 Rust edit, 4 rows FIXED (present or edited), 6 NOT A GAP decisions, the rest OPEN. The coordinator's full list is not complete.
