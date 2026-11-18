# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-476274.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: resource_key.rs

- 1. `get_biome`/`get_advancement` suggestions (`resource_key.rs:123,141`): CLOSED (advancement and biome suggestions are in `list_suggestions`, `crates/pumpkin-core/src/command/argument_types/resource_key.rs:48-76`).
- 2. Dimension suggestions only loaded worlds: CLOSED in this pass.
- 3. `ERROR_NOT_SUMMONABLE_ENTITY`/`ERROR_INVALID` (`resource_key.rs:29`): STALE REFERENCE. The current file has no such constants (its errors are `ERROR_INVALID_ADVANCEMENT` and `ERROR_INVALID_BIOME`, lines 20-25). Not reproducible in this unit; needs a re-audit against the summon command.
- 4. No automated test for suggestions: CLOSED. `dummy_context` and `context_with` (lines 244-270) build a context without a server, and tests `dimension_suggestions_are_filtered_by_prefix` and `biome_suggestions_are_filtered_by_prefix` cover filtering.

## Result
- Item 2 (dimension suggestions): FIXED. Dimension suggestions now come from the dynamic registry (`get_all_entry_ids(DIMENSION_REGISTRY)`, the same source biomes use), so datapack dimension types are advertised even without a loaded world, matching vanilla `ResourceArgument`. Falls back to loaded worlds when the registry has no `dimension` entries. Not compiled (worker rule). Whether the server's registry carries `minecraft:dimension` entries was not confirmed at runtime; the fallback keeps the old behaviour if it does not.
- Item 3: open, stale reference (see row above).
