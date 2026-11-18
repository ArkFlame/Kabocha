# Parity rows for own-tworld-sweep3

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/tworld-sweep3** [partial/OWN-BLOCKED-DATA] own audit: 0 open, 4 fixed, 5 verified, 1 blocked-data (docs/tracker/tworld-sweep3.md) docs/tracker/tworld-sweep3.md

## Result
THIRD PASS 2026-11-18. Row `own/tworld-sweep3` (pointer to docs/tracker/tworld-sweep3.md): CLOSED as VERIFIED + FIXED + BLOCKED-DATA. All sub-rows verified against .vendor/vanilla-decompiled/net/minecraft/; no source changes needed this pass (every in-scope fix was already present in the tree).

Per sub-row of docs/tracker/tworld-sweep3.md:
- item 1 encompassing_bits: VERIFIED FIXED. `crates/pumpkin-util/src/lib.rs:98` `encompassing_bits(count)` returns 0 for count<=1 else `ilog2 + (not power of two)`, matching vanilla `Mth.ceillog2` (.vendor/vanilla-decompiled/net/minecraft/util/Mth.java:309-312) with `smallestEncompassingPowerOfTwo(0)=1`. Test `encompassing_bits_tests::matches_vanilla_ceillog2`.
- items 2 & 3 LastUpdate + MODELLED_ROOT_TAGS: VERIFIED FIXED. `crates/pumpkin-world/src/chunk/format/mod.rs:59` lists `LastUpdate` in MODELLED_ROOT_TAGS and `:763` writes `put_long("LastUpdate", last_update_clock())`; clock set each world tick. Round-trip test `last_update_round_trips_the_world_clock` (`:2206`). Matches vanilla chunk save writing game time every save (.vendor/vanilla-decompiled/net/minecraft/world/level/chunk/storage/RegionFile.java save path).
- Light nibble packing: VERIFIED. Pumpkin `LightContainer::index` = `y*256 + z*16 + x`, byte `index >> 1`, nibble `index & 1` (chunk/format/mod.rs:1691-1714), matching vanilla `DataLayer.getIndex/get/set` (.vendor/vanilla-decompiled/net/minecraft/world/level/chunk/DataLayer.java:40-69). Regression pin `chunk::format::tests::light_nibbles_follow_vanilla_datalayer_index_order`.
- BlendedNoise constants: VERIFIED. BASE_SCALE 684.412, LIMIT_FACTOR 0.99998474F, MAIN_FACTOR 12.75, LIMIT_FIRST_OCTAVE -15, MAIN_FIRST_OCTAVE -7 in .vendor/vanilla-decompiled/net/minecraft/world/level/levelgen/synth/BlendedNoise.java:33-37,54; SmearedPerlinNoise.range closed. No `STANDARD_DEVIATION` in vanilla BlendedNoise: not a vanilla constant.
- Bedrock palette bit table: BLOCKED-DATA. `.vendor/vanilla-decompiled/net/minecraft/` is Java Edition only; no Bedrock Edition palette reference exists. Missing subsystem: a Bedrock-format palette bit-width reference. `crates/pumpkin-world/src/chunk/palette.rs:14` `bedrock_palette_bits` cannot be verified without it.

No STILL OPEN rows remain. No .rs file touched, so rustfmt not required.
