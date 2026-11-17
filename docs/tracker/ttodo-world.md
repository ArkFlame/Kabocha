# TODO triage: world, net, data, block, pumpkin-world, wasm-host gametest/player

Scope: TODO/FIXME/HACK/XXX in the g4 paths (excluding generation_cache.rs, structure/placement.rs and the net/java/play edits owned by other workers).
Result: 0 resolved, all left. Each is larger than the <40-line bar, needs an API that does not exist yet, or is a question rather than a code fix.

## pumpkin-core
- world/time.rs:127: Bedrock frozen-time question. Needs a Bedrock packet decision.
- world/mod.rs:2446: send block-break only to players with chunk loaded. Needs a chunk-watcher query per viewer.
- world/mod.rs:2447: send light updates directly on break. Needs lighting-update wiring.
- world/mod.rs:3303: canSpawnEntitiesInChunk. Needs the spawn-chunk gate, a separate port.
- world/mod.rs:4759: structured logging for join. Touches the log surface for a cosmetic change.
- world/mod.rs:4817: world spawn compass. Needs spawn-point and compass tracking.
- world/mod.rs:5280: difficulty, exp bar, status effects on respawn. Multi-feature.
- world/mod.rs:5986: ender dragon handling. Entity-specific, large.
- net/chunk_sender.rs:575: chunk lifecycle built twice. Refactor across commit_batch.
- net/proxy/mod.rs:6: proxy trait. Design change, no caller needs it yet.
- net/bedrock/play/player_action.rs:278: bare TODO, no spec.
- net/java/play/cookie_response.rs:6: plugin access to cookie responses. Needs plugin event design.
- data/banned_player.rs:45, data/banned_ip.rs:41, data/whitelist.rs:28: validate lists. Needs a defined validation rule per vanilla file format.
- block/entities/trial_spawner.rs:1166: load SpawnData NBT into mob. Needs mob NBT API.
- block/entities/mob_spawner.rs:235: SpawnData entity/custom_spawn_rules/equipment. Same blocker.
- block/blocks/piston/piston.rs:422: vanilla destroys unconditionally. Changes BlockBreakEvent cancel semantics, needs an event-policy decision.

## pumpkin-world
- generation/height_provider.rs:59: why min > max. Needs a reproduction against vanilla heights.
- generation/surface/terrain.rs:236: cache. Perf change, needs a benchmark.
- generation/noise/router/surface_height_sampler.rs:43: flat map for aquifer sampler. Perf refactor.
- chunk_system/mod.rs:4: bare TODO.
- chunk_system/schedule.rs:321: "will remove at some point". Needs removal condition.
- chunk/format/mod.rs:300: reconstruct absent section inside band (vanilla behaviour). Needs a save-format fixture.
- chunk/mod.rs:976: heightmap update tracking. Needs the tracking design.
- chunk/palette.rs:1282: own network/disk serialization testing. Open question.
- lib.rs:55: in-file benches. Tooling question.
- generation/noise/router/density_function/test.rs:109, :127: all dimensions/routers, extractor values. Needs the extractor (assets gap).

## pumpkin-wasm-host-v0_2
- gametest.rs (about 40 sites): "make async (list it in bindings.rs) once implemented". Needs WIT bindings changes across host, SDK and runtime together, so it is out of one worker's scope.
- player.rs:3434: relative teleport flags and passengers. Needs passenger teleport handling.
