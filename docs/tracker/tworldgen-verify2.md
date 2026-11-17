# tworldgen-verify2

Continues `tworldgen-verify.md` (x5). Scope: `crates/pumpkin-world` vs `.vendor/vanilla-decompiled/net/minecraft/world/level/`.

| id | area | divergence | vanilla | fix | status |
|---|---|---|---|---|---|
| V1 | live OCEAN_FLOOR (generation_cache) | `ocean_floor_height_exclusive` returned the WORLD_SURFACE height, so water columns read as floor | `Heightmap.Types.OCEAN_FLOOR` uses `MATERIAL_MOTION_BLOCKING`, liquids excluded; default `minY` | column scan below surface via `ocean_floor_from_column`; pure tests | fixed, UNVERIFIED (not compiled, no gametest) |
| V2 | heightmap update rules | none | `Heightmap.update` / `primeHeightmaps` | none needed | matches (opaque/non-opaque branches, search below, minY default) |
| V3 | heightmap types | Pumpkin stores 3 maps (WORLD_SURFACE, MOTION_BLOCKING, MOTION_BLOCKING_NO_LEAVES); no OCEAN_FLOOR or WG-persisted map in chunk data | 6 types, OCEAN_FLOOR is LIVE_WORLD | none; structural (chunk NBT format) | open, not fixed |
| V4 | heightmap predicates | `blocks_movement` = is_solid minus cobweb/bamboo; is_liquid for fluids | `BLOCKS_MOTION_IN_HEIGHTMAP` tag = `#blocks_motion` | none; equivalent for shipped tags | matches |
| V5 | POI tickets and valid ranges | none | `PoiTypes` (meeting 32/6, home 1/1, bee nest 0/1) | none; generated table matches | matches |
| V6 | biome search | none | `Climate.RTree` fitness = sum of squared distances over 6 params + offset | none; `RTreeLeaf::distance` sums 7 dims | matches |
| V7 | sky/block propagation | `propagate_level` decays by max(1, opacity); sky 15 straight down undimmed | `LightEngine.getOpacity` = max(1, dampening); `SkyLightEngine` source handling | none | matches for the simple opacity path |
| V8 | light dampening into a face | `get_light_dampening_into` ignores direction and from-state, returns 16 only when `to` is solid-render | `Shapes.mergedFaceOccludes(from, to, dir)` | not fixed: from-state and direction are needed to match, which is more than 40 lines and needs shape data | open, possible divergence (for example stone to air) |
| V9 | tick queue ordering | none | `ScheduledTick.DRAIN_ORDER` = trigger, priority, subTickOrder; unique (type,pos) | none; `OrderedTick` ordering and `queued_ticks` dedupe match | matches |
| V10 | fluid and falling tick delays | none | see x5 W3 | none | matches (x5) |

Not audited in this pass: noise router and surface rule data (generated from `assets/`), SpawnPlacements heightmap use in pumpkin-core.

Commands: none run (workers do not compile). Changed file: `crates/pumpkin-core/src/world/generation_cache.rs`.
