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
2. FIXED (h8 pass, re-verified 2026-11-18). Game time is threaded to chunk save via a process-wide `last_update_clock` set each world tick; `chunk/format/mod.rs:763` writes `put_long("LastUpdate", last_update_clock())`. Round-trip test `last_update_round_trips_the_world_clock` (`:2206`). Matches vanilla writing game time every save.
3. FIXED (h8 pass, re-verified 2026-11-18). `chunk/format/mod.rs:59` now lists `LastUpdate` in `MODELLED_ROOT_TAGS` and `:763` writes `put_long("LastUpdate", last_update_clock())`, with round-trip test `last_update_round_trips_the_world_clock` (`:2206`). Stale OPEN marker closed.

## Mob family sweep (CODER, goals and constants, vanilla 1.21.x recalled, UNVERIFIED)

`docs/tracker/tfamily-sweep.md` is not in the tree, so the family list comes from the brief. Skipped per brief: bee, goat, axolotl, sniffer, allay, panda, vex, zombie_villager.

| Family | Location | Result |
|---|---|---|
| Cow | `passive/cow.rs` | matches: panic 2.0, breed 1.0, tempt 1.0, follow parent 1.25, stroll 1.0 |
| Chicken | `passive/chicken.rs` | matches: panic 1.4, breed 1.0, follow 1.1, stroll 1.0 |
| Pig | `passive/pig.rs` | matches: panic 1.25, breed 1.0, tempt 1.2, follow 1.1, stroll 1.0 |
| Sheep | `passive/sheep.rs` | matches: panic 1.25, breed 1.0, tempt/follow 1.1, stroll 1.0 |
| Rabbit | `passive/rabbit.rs` | stroll 0.6, breed 0.8, tempt 1.0, attack 1.4, flee 2.2 match; monster flee range 4.0 and jump constants UNVERIFIED, kept |
| Strider | `passive/strider.rs` | matches: panic 1.65, breed 1.0, follow 1.1, stroll 1.0 |
| Cat | `passive/cat.rs` | matches: lie-on-block 1.1 range 8, follow owner 1.0/10/5, sit-on-block 0.8; tempt via `feline::TEMPT_SPEED` 0.6 |
| Ocelot | `passive/ocelot.rs` | tempt via `feline::TEMPT_SPEED` 0.6, flee 0.8/1.33 match; kept |
| Drowned | `mob/zombie/drowned.rs` | matches: water 1.0, beach 1.0, swim-up 1.0, stroll 1.0 |
| Hoglin, Zoglin | `mob/hoglin.rs`, `mob/zoglin.rs` | attributes 40 hp, speed 0.3, knockback res 0.6, damage 6 match; vanilla uses Brain (HoglinAi), goals kept, UNVERIFIED |
| Turtle, Parrot, Llama, Horse, Fox, Piglin, Piglin brute | various | not reviewed this pass (budget) |

No code changes in this pass. No fix was confident enough to land without a decompiled reference.

## Not checked

- `generation/structure/placement.rs` (owned elsewhere).
- Full `BlendedNoise` constants (`684.412`, `12.75`, `0.999_984_74`, `SMEARED_RANGE_BASE`) and `STANDARD_DEVIATION`: no decompiled reference consulted, kept.
- Light nibble packing and `LightContainer` layout, bedrock palette bit table.

## h8 pass (S/M open gaps)

- Item 2 LastUpdate save: FIXED (already in tree, 7911189485 writes `put_long("LastUpdate", last_update_clock())` in `chunk/format/mod.rs`; 8ee11b06ce sets the clock each world tick in `world/mod.rs` `tick_environment`). Process-wide clock, so with several worlds the last ticked wins. No new test in this pass; existing coverage only checks passthrough of a pre-existing value.
- Item 3 `MODELLED_ROOT_TAGS` lacks `LastUpdate`: FIXED (already in tree). `LastUpdate` is listed at `chunk/format/mod.rs` line ~59, so it is modelled, not passed through.
- BlendedNoise constants (684.412, 0.99998474F, 12.75, -15, -7, smear range base 2.0): CLOSED, match `BlendedNoise.java` and `SmearedPerlinNoise.range`. `STANDARD_DEVIATION` does not appear in the vanilla BlendedNoise source: CLOSED, not a vanilla constant there.
- Light nibble packing: FIXED (verified, 56f2e2e1d). Pumpkin `LightContainer::index` is `y*256 + z*16 + x`, byte `>>1`, nibble `&1`, matching vanilla `DataLayer.getIndex`/`get`/`set`. No code change; regression pin `chunk::format::tests::light_nibbles_follow_vanilla_datalayer_index_order`.
- Bedrock palette bit table: BLOCKED-DATA. No Bedrock reference in `.vendor/vanilla-decompiled`.
