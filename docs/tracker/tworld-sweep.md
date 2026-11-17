# pumpkin-world parity sweep

Hand-written constants in `crates/pumpkin-world` checked against vanilla 1.21.x from memory.
Structure set spacing, separation, salt and biome lists come from `pumpkin-data` (generated from `assets/`) and are out of scope here.

| id | file | divergence | vanilla | fix | status |
|---|---|---|---|---|---|
| W1 | crates/pumpkin-world/src/generation/structure/placement.rs (`should_generate_frequency`, Default) | `get_large_feature_with_salt` called as `(seed, salt, chunk_x, chunk_z)` but signature is `(seed, x, z, blend)`, so salt was used as x and chunk_x as z | `setLargeFeatureWithSalt(seed, chunkX, chunkZ, salt)` | argument order corrected (master 197c6255d), regression test `default_frequency_reduction_uses_chunk_then_salt_order` | UNVERIFIED (no cargo run per h9 policy) |

Reviewed without divergence found (from vanilla 1.21.x memory, UNVERIFIED, no cargo run):
- `LegacyRand` (java.util.Random): `from_seed` XOR `0x5DEECE66D` + 48-bit mask, LCG `*0x5DEECE66D + 0xB`, `next_f32` (24 bits, 2^-24), `next_f64` (26+27 bits, 2^-53), `next_i64` (hi<<32 + lo), bounded int power-of-two and rejection branches. Existing known-value tests cover these.
- `get_large_feature_with_salt` / `get_region_seed`: multipliers 341873128712 and 132897987541, `+ seed + salt` (region test value 474797819485 checks out by hand).
- `get_large_feature_seed` (setLargeFeatureSeed): two `nextLong`s from `setSeed(seed)`, `x*l ^ z*m ^ seed`. Duplicated in `carver/mod.rs` and `random/mod.rs`; same logic.
- `get_population_seed` (setDecorationSeed, `| 1` on both longs), `get_decorator_seed` (`+ index + 10000*step`).
- `seed_slime_chunk`: `(seed + x*x*4987142 + x*5947611 + z*z*4392871 + z*389711) ^ 987234911` with int-overflow on the x/z products, matching Java precedence.
- Frequency methods `LegacyType1` (`x ^ (z<<4)` widened, `nextInt()` then `nextInt(1/freq)==0`) and `LegacyType2` (salt 10387320, `nextFloat < freq`). `get_structure_chunk_in_region` (spacing minus separation bound).
- Light: `decayed` (`max(1, opacity)` step), `sky_descended` (15 straight down through opacity 0), out-of-world above = 15, below = 0.
- Region file: 4096-byte sectors, 2-sector header, location `offset = entry >> 8`, `count = entry & 0xFF`, compression ids 1 GZip / 2 ZLib / 3 none / 4 LZ4 / 127 custom, `.mcc` threshold 256 sectors with 128 flag. NBT keys `DataVersion`, `xPos`, `Status`, `sections` present; data version 5023.
- Cave carver room/tunnel branching and tunnel distance, canyon width factor init.

Confidence gaps (not changed):
- `LegacyType3` uses `next_f64() < frequency`. Vanilla may use `nextFloat`; not confirmed from memory.
- Canyon `update_vertical_radius` constants.
- Dimension min_y/height (no hand-written constant found in `dimension.rs`; height comes from level config).
- Biome blending and noise router wiring: not audited in this pass.
