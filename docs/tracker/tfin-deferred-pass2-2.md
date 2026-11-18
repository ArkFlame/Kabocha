# Final rows from docs/tracker/deferred-pass2.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: armor_stand.rs mod.rs

- taudit-crates-pumpkin-core-src-en-cf6827.md: - Row 1 (flags not synced): OPEN, out of scope. It was BLOCKED-DATA, not STILL OPEN. The generated tracked data now exposes `DATA_CLIENT_FLAGS` (index 15), so the blocker is gone; syncing the four flag bits is a follow-up in armor_stand.rs.
- taudit-crates-pumpkin-core-src-en-f688a8.md: - Row 3 (Bedrock climate_variant): STILL OPEN. Needs the Bedrock `climate_variant` property id plus biome-temperature classification and a `PropertySyncData` entry; that property data is outside this unit and would risk generated/Bedrock data changes.
- taudit-crates-pumpkin-core-src-entity-liv.md: | 5 | STILL OPEN | Consumable-effect probability stays hardcoded; needs the datapack component data pipeline (`ConsumableImpl` is generated from assets), which is outside this unit. |
- taudit-crates-pumpkin-core-src-entity-mod.md: - 1 TemperatureVariantAnimal Bedrock `climate_variant` (entity/mod.rs:972): STILL OPEN. `climate_variant` is a Bedrock Molang property id with no key in `pumpkin-protocol`/`pumpkin-data`; needs a generated property table plus a spawn-biome temperature hook. Cross-crate, left for a protocol/data unit

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
