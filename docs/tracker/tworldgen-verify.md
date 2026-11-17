# tworldgen-verify

Scope: `crates/pumpkin-world` against decompiled vanilla in `.vendor/vanilla-decompiled/net/minecraft/world/level/`.

| id | area | divergence | vanilla | fix | status |
|---|---|---|---|---|---|
| W1 | structure frequency reduction | Default reducer passed chunk coords before salt, vanilla passes salt as the first coordinate | `AbstractSpreadingStructurePlacement.probabilityReducer`: `setLargeFeatureWithSalt(seed, salt, sourceX, sourceZ)` | `should_generate_frequency` Default now calls `get_large_feature_with_salt(seed, salt, chunk_x, chunk_z)`; test renamed to `..._salt_then_chunk_order` | fixed, unverified by chunk dump |
| W2 | node evaluator bb dims (pumpkin-core, from tpath-sweep) | `get_bb_width`/`get_bb_height` used ceil | `NodeEvaluator.prepare`: `Mth.floor(getBbWidth()+1)`, `Mth.floor(getBbHeight()+1)` | floor; zombie test now expects 1 and 2 | fixed |
| W3 | fluid and falling tick delays (pumpkin-core) | none | water 5, lava 30 / 10 fast lava, FallingBlock delay 2 | none needed | matches |
| W4 | legacy_type_2 / legacy_type_1 / legacy_type_3 reducers | none | `setLargeFeatureWithSalt(seed, sourceX, sourceZ, 10387320)`, pillager `nextInt` path, `setLargeFeatureSeed` | none needed; `saltOverride` (Paper config) not modelled | matches for null override |

Open, not audited in this pass: lighting engine, heightmap types, POI ticks, biome source noise, noise surface rules, structure spacing salts (these come from `assets/` through codegen).
