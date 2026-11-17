# TODO sweep (core, world, protocol)

Scope: `pumpkin-core`, `pumpkin-world`, `pumpkin-protocol` src, excluding generated code,
`command/commands/*`, fluid, redstone, `net/bedrock`, `protocol/java`, bee/goat.
Triage: deferred unless a small, confident vanilla-parity or robustness fix (<60 lines).

## Fixed
- `pumpkin-world/src/chunk/mod.rs`: removed bare `// TODO` above the chunk size constants (no content, stale).

## Deferred
- `pumpkin-core/src/net/chunk_sender.rs:575` chunk lifecycle built twice; needs send-path refactor.
- `pumpkin-core/src/net/proxy/mod.rs:6` proxy trait; design item, no behaviour gap.
- `pumpkin-core/src/net/java/play/cookie_response.rs:6` plugin access to cookie responses; feature, needs plugin API.
- `pumpkin-core/src/plugin/mod.rs:481` remove plugin command nodes on reload; needs dispatcher API.
- `pumpkin-core/src/world/time.rs:127` bedrock frozen-time question; unverified vanilla behaviour.
- `pumpkin-core/src/command/mod.rs:383` command block custom names; blocked on naming support.
- `pumpkin-core/src/world/mod.rs:2443,2444` limit block-break packet recipients and light updates; touches broadcast path.
- `pumpkin-core/src/world/mod.rs:3298` spawn-in-chunk check; needs chunk-level API.
- `pumpkin-core/src/world/mod.rs:4754` structured logging for connect line; style only.
- `pumpkin-core/src/world/mod.rs:4812,5275,5981` world spawn compass, difficulty/xp bar, ender dragon; features.
- `pumpkin-core/src/entity/ai/goal/track_target.rs:88`, `melee_attack.rs:254`, `vex.rs:105`, `skeleton/mod.rs:77`, `shulker.rs:110`, `active_target.rs:48`: mob AI, owned by mob-AI rules; needs audit.
- `pumpkin-core/src/entity/living.rs:1258,1949,2297,2580,5537`: death config, raycast quirk, gravity, unknown; no confident fix.
- `pumpkin-core/src/entity/mod.rs:948,1063,1402,1442,3147`: climate variant, defrost, team colour, entity scaffolding, on_stepped_on hook; features.
- `pumpkin-core/src/entity/player.rs` (14 markers): portal cooldown, spawn position, enchant reduction, world change events, hurt scaling, atomics; each needs vanilla audit.
- `pumpkin-core/src/entity/tnt.rs:26`, `strider.rs:98`, `predicate/mod.rs:34`, `mob/mod.rs:3498`: unverified vanilla details or missing SpawnReason.
- `pumpkin-core/src/entity/living.rs:5537`: data-driven table; needs data pack work.
- `pumpkin-core/src/block/entities/mob_spawner.rs:231`, `trial_spawner.rs:1166`: SpawnData NBT; feature.
- `pumpkin-core/src/block/entities/hopper.rs:140`: set block state on chunk load; needs load-path change, not verified.
- `pumpkin-core/src/block/blocks/shulker_box.rs:109`: closing-state open check; needs audit.
- `pumpkin-core/src/block/blocks/infested.rs:158`: onStacksDropped; refactor.
- `pumpkin-core/src/block/blocks/piston/piston.rs:422`: unconditional break event; cancellable event semantics need design.
- `pumpkin-core/src/data/banned_ip.rs:41`, `banned_player.rs:45`, `whitelist.rs:28`: list validation; rules not specified.
- `pumpkin-world/src/lib.rs:55`: in-file benches question; not a fix.
- `pumpkin-world/src/generation/height_provider.rs:59`: min > max observed; root cause unknown.
- `pumpkin-world/src/chunk/format/mod.rs:282`: vanilla absent-section reconstruction inside band; needs audit.
- `pumpkin-world/src/generation/noise/router/density_function/test.rs:109,127`: test coverage and extracted values.
- `pumpkin-world/src/chunk/mod.rs:977`: heightmap tracking; needs design.
- `pumpkin-world/src/chunk_system/schedule.rs:321`: `restore_ready_tasks` is live (called at line 1687); marker kept.
- `pumpkin-world/src/chunk_system/chunk_loading.rs:14`, `chunk_system/mod.rs:4`: design notes.
- `pumpkin-world/src/generation/surface/terrain.rs:236`: cache temperature; perf only.
- `pumpkin-world/src/generation/noise/router/surface_height_sampler.rs:43`: flat map idea; perf only.
- `pumpkin-world/src/chunk/palette.rs:1282`: testing note; design question.
- `pumpkin-protocol` markers (lib.rs:335, codec/recipe.rs:31, codec/data_component.rs:371, java/client play `teleport_entity.rs`, `player_chat_message.rs`, `player_info_update.rs`, `player_position.rs`, `spawn_entity.rs`, `dialog.rs`): custom-impl and enum notes; bedrock enum TODOs (`bedrock/*`) are wire-enum refactors out of scope.
