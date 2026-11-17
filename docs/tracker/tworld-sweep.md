# pumpkin-world parity sweep

Hand-written constants in `crates/pumpkin-world` checked against vanilla 1.21.x from memory.
Structure set spacing, separation, salt and biome lists come from `pumpkin-data` (generated from `assets/`) and are out of scope here.

| id | file | divergence | vanilla | fix | status |
|---|---|---|---|---|---|
| W1 | crates/pumpkin-world/src/generation/structure/placement.rs (`should_generate_frequency`, Default) | `get_large_feature_with_salt` called as `(seed, salt, chunk_x, chunk_z)` but signature is `(seed, x, z, blend)`, so salt was used as x and chunk_x as z | `setLargeFeatureWithSalt(seed, chunkX, chunkZ, salt)` | argument order corrected, regression test `default_frequency_reduction_uses_chunk_then_salt_order` | UNVERIFIED (no cargo run per h9 policy) |

Reviewed without divergence found: `LegacyType1`/`LegacyType2` frequency reduction, `get_structure_chunk_in_region` (spacing minus separation bound), cave carver room/tunnel branching and tunnel distance, canyon width factor init.
Not reviewed with confidence: canyon `update_vertical_radius` constants, dimension min_y/height (no hand-written constant found in `dimension.rs`; height comes from level config).
