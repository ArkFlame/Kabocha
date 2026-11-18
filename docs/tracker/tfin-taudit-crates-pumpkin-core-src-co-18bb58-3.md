# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-18bb58.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: particle.rs

- **Codec validation of required options** (`pumpkin-command/src/argument_types/particle.rs`): VERIFIED. `missing_required_option` + `ERROR_INVALID_OPTIONS` (`translation::java::PARTICLE_INVALIDOPTIONS`) reject an options compound missing a mandatory field, matching vanilla `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ParticleArgument.java` (`readParticle` decodes through `type.codec().codec().parse(...)` and throws `particle.invalidOptions`). Required keys checked against the matching codecs in `.vendor/vanilla-decompiled/net/minecraft/core/particles/`. Tests: `required_option_missing_when_absent_or_incomplete`, `required_options_satisfied_or_not_needed`.
- **26.x packet layout** (`pumpkin-protocol/src/java/client/play/particle.rs`): VERIFIED. For `version >= V_26_1`: particle id + payload first, two bools, f64 position, three f32 offsets, three f32 speeds, VAR_INT count, VAR_INT randomization type 0. Matches `.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundLevelParticlesPacket.java` (`STREAM_CODEC` and convenience constructor). Tests: `level_particles_26_x_matches_vanilla_field_order`, `level_particles_encoding_matches_26_2_for_26_3`.

## Result

Final pass. Every row is closed: all fixes are present in code and verified against `.vendor/vanilla-decompiled/net/minecraft/`. No BLOCKED-DATA rows (no missing generated `pumpkin-data` field or subsystem). No cargo run (forbidden for workers); compilation and rustfmt of touched files left to the orchestrator.

- **Codec validation of required options** (`pumpkin-command/src/argument_types/particle.rs`): VERIFIED (vanilla `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ParticleArgument.java`, `core/particles/*`).
- **26.x packet layout** (`pumpkin-protocol/src/java/client/play/particle.rs`): VERIFIED (vanilla `.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundLevelParticlesPacket.java`).

No open rows remain. No source file was edited this pass; both fixes were already present in code and match vanilla.
