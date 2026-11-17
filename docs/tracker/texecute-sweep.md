# execute.rs sweep (c3)

Scope: `crates/pumpkin-core/src/command/commands/execute.rs`, read in full (production code and tests).
Method: read-only review against the existing comments and the vanilla behaviour they cite. No vanilla decompile was consulted in this pass.

## Reviewed, no divergence found
- `as`, `at`, `positioned as`, `rotated as`, `on *`: source copies name, display name and entity together.
- `in`, `rotated`, `align`, `anchored`, `facing` (pos and entity): facing uses the source anchor; both angles wrapped.
- `if`/`unless` entity, block, loaded, dimension, biome, score (matches and all six comparisons), blocks all/masked, data block/entity/storage, stopwatch: terminal executors return counts or 1 as the numeric/conditional executors describe; missing-score handling for `unless` passes.
- `store result|success` for score, bossbar, entity/block/storage data: read-only criteria error, inactive objective ignored, scaled narrowing per type.
- `summon`, `on passengers`, `on origin`: entity adoption of name and display name.

## Open questions (unverified, not changed)
1. `positioned <pos>` resets `entity_anchor` to feet (comment cites vanilla). Confirm `CommandSourceStack#withPosition` keeps the anchor.
2. `if predicate` / `unless predicate` have no terminal `executes`, unlike `if block` / `if loaded`. Confirm vanilla requires a `run` after them.
3. `positioned over ocean_floor` falls back to WorldSurface (documented gap in code).
4. `StoreNumberType::create_tag` short/byte narrowing (`as i32 as i16/i8`) matches Java `(short)(int)` only if vanilla routes through int; not checked against decompiled source.

## Cosmetic
- `execute_unless_data_*` use `!check_data_matches(..)?.is_some()`; `is_none()` reads better. Left as-is (clippy `nonminimal_bool` is allowed at file level).
