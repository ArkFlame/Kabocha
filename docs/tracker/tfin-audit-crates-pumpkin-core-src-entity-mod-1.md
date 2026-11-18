# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-entity-mod.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- 5 dead tick_block_underneath stub (entity/mod.rs:3078): STILL OPEN, M.
- 1 TemperatureVariantAnimal Bedrock `climate_variant` (entity/mod.rs:972): STILL OPEN. `climate_variant` is a Bedrock Molang property id with no key in `pumpkin-protocol`/`pumpkin-data`; needs a generated property table plus a spawn-biome temperature hook. Cross-crate, left for a protocol/data unit. TODO comment kept.
- 1 TemperatureVariantAnimal Bedrock `climate_variant` (entity/mod.rs:997): STILL OPEN. `climate_variant` is a Bedrock Molang property whose numeric id is absent from `pumpkin-protocol`/`pumpkin-data`; filling `synced_properties` needs a generated property table plus a spawn-biome-temperature hook in `mob/mod.rs` (outside this unit). One-line reason: cross-crate, blocked on Bedrock property-id data. TODO comment kept.
- 1 TemperatureVariantAnimal Bedrock `climate_variant` (entity/mod.rs:997): STILL OPEN. TODO kept; `climate_variant` is a Bedrock Molang property whose numeric id is absent from `pumpkin-protocol`/`pumpkin-data`. One-line reason: filling `synced_properties` needs a generated Bedrock property-id table plus a spawn-biome-temperature hook in `mob/mod.rs`, both cross-crate/outside this unit. No source file in scope.

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
