# pumpkin-world parity sweep 3

Scope: `crates/pumpkin-world/src` (excluding `generation/structure/placement.rs`). Read-only sweep, no code changes. Vanilla 1.21.x knowledge, UNVERIFIED against decompiled sources.

## Checked, matches vanilla

| Area | Location | Result |
|---|---|---|
| Block palette bits | `chunk/palette.rs` `MIN_INDEX_BITS=4`, `MAX_INDEX_BITS=8`, `BLOCK_DISK_MIN_BITS=4` | matches (4 min, 8 heterogeneous max, global above) |
| Biome palette bits | `BIOME_DISK_MIN_BITS=0`, `BIOME_NETWORK_MIN_MAP_BITS=1`, `BIOME_NETWORK_MAX_MAP_BITS=3` | matches |
| Direct palette threshold | `global_palette_bits` `.max(9)` | matches (direct above 8 bits) |
| Heightmap packing | `chunk/mod.rs` `ChunkHeightmaps::set/get`, 9 bits, 7 per long, 37 longs | matches; stored value `y - min_y + 1`, unset returns `min_y - 1` |
| Region header math | `chunk/format/anvil.rs` `SECTOR_BYTES=4096`, `HEADER_BYTES=8192`, `location>>8` offset, `EXTERNAL_CHUNK_THRESHOLD=256`, `EXTERNAL_STREAM_FLAG=128` | matches |
| Zlib header check | `anvil.rs` CMF method 8, `u16 % 31 == 0` | matches |
| Section Y | `chunk/format/mod.rs` `section_y` accepts Byte/Short/Int/Long | matches; lowest biome section fallback clamps to 0 |
| Light attenuation | `lighting/mod.rs` `decayed` = `level - max(1, opacity)`, `sky_descended` keeps 15 through opacity 0 | matches |
| Sky fill | `lighting/sky_fill.rs` fills only sections strictly above the highest surface | conservative, no over-fill |
| Quart coords | `generation/mod.rs` `biome_coords` `>>2`, `<<2`, section `>>4` | matches |
| Fuzzy biome offset | `generation/biome.rs` `x-2`, `&3 * 0.25`, permutations | matches |
| Lerp family | `pumpkin-util/src/math/mod.rs` `lerp`, `clamped_lerp`, `clamped_map`, `lerp2`, `lerp3`, `catmullrom`, `smoothstep`, `smoothstep_derivative`, `triangle_wave`, `rot_lerp`, `floor_div`, `floor_mod` | matches |
| Perlin constants | `pumpkin-util/src/noise/perlin.rs` `INPUT_FACTOR`, `TARGET_DEVIATION`, `ROUND_OFF`, `SHIFT_UP_EPSILON`, gradient table, `skip(262)` for empty legacy octaves, `0.1*(1+1/(span+1))` deviation | matches vanilla as recalled |
| Blended noise octaves | `generation/noise/router/density_function/noise.rs` limit `-15`, main `-7` | matches as recalled; other constants UNVERIFIED |
| World data version | `chunk/format/anvil.rs` `WORLD_DATA_VERSION=5023` equals `MAXIMUM_SUPPORTED_WORLD_DATA_VERSION` | consistent |

## Divergences logged, not fixed

1. FIXED. `pumpkin-util/src/lib.rs` `encompassing_bits` now returns 0 for 0 and 1 (vanilla `Mth.ceillog2`), removing the `ilog2(0)` panic. Palette callers that divide by bits keep 1 bit via `.max(1)` in `chunk/palette.rs` (`bedrock_palette_bits`, heterogeneous `bits_per_entry`). Test: `encompassing_bits_tests::matches_vanilla_ceillog2`.
2. OPEN (needs game time threaded through `ChunkIO::save_chunks` -> `file_manager` -> `SingleChunkDataSerializer::to_bytes`, 4+ signatures, left logged). Chunk save does not write `LastUpdate`. Vanilla writes the world game time on every save; this server only preserves an existing value through the passthrough of unmodelled root tags. Fixing needs the game time at serialization, which the writer does not receive. Low impact on load.
3. OPEN, same root cause as item 2. `chunk/format/mod.rs` `MODELLED_ROOT_TAGS` does not list `LastUpdate`, so it rides through verbatim. Same root cause as item 2.

## Not checked

- `generation/structure/placement.rs` (owned elsewhere).
- Full `BlendedNoise` constants (`684.412`, `12.75`, `0.999_984_74`, `SMEARED_RANGE_BASE`) and `STANDARD_DEVIATION`: no decompiled reference consulted, kept.
- Light nibble packing and `LightContainer` layout, bedrock palette bit table.
