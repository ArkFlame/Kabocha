# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-18bb58.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: particle.rs

- **26.x packet layout (`pumpkin-protocol/.../particle.rs:280-320`): STILL OPEN.**
row was not itself marked STILL OPEN and is unchanged; it still needs per-viewer packet
Re-checked the STILL OPEN rows against
(`pumpkin-command/src/argument_types/particle.rs:52-60`): STILL OPEN.**

## Result

- **26.x packet layout (`pumpkin-protocol/src/java/client/play/particle.rs:264-285`): VERIFIED.**
  For `version >= V_26_1`, `write_packet_data` writes particle id VAR_INT, option payload,
  `force_spawn` (overrideLimiter) bool, `important` (alwaysShow) bool, three f64 position
  coords, three f32 offsets, three f32 speeds (max_speed repeated), VAR_INT count and
  VAR_INT randomization type 0 (RandomizationType.DEFAULT). Matches
  `.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundLevelParticlesPacket.java`
  `STREAM_CODEC` field order (lines 30-60) and the convenience constructor (lines 62-91, id 0
  at line 104). Per-viewer colour gate for `entity_effect`/`tinted_leaves`/`flash` is handled
  in `crates/pumpkin-core/src/command/commands/particle.rs` (`color_particle_payload`, test
  `colour_payload_is_stripped_for_pre_1_21_4_viewers`); per-viewer packet construction uses
  `ParticleOptions::encode_for`. Tests: `level_particles_26_x_matches_vanilla_field_order`,
  `level_particles_encoding_matches_26_2_for_26_3`.

- **Codec validation of required options (`pumpkin-command/src/argument_types/particle.rs:33-67`): VERIFIED.**
  `missing_required_option` + `ERROR_INVALID_OPTIONS` (`translation::java::PARTICLE_INVALIDOPTIONS`)
  reject an options compound missing a mandatory field. Matches
  `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ParticleArgument.java`
  (`readParticle`, lines 71-83) decoding via `type.codec().codec().parse(...)` and throwing
  `particle.invalidOptions` on failure. Required keys checked against the matching codecs in
  `.vendor/vanilla-decompiled/net/minecraft/core/particles/` (DustParticleOptions,
  DustColorTransitionOptions, ColorParticleOption, SculkChargeParticleOptions, ShriekParticleOption,
  ItemParticleOption, VibrationParticleOption, TrailParticleOption, GeyserParticleOptions,
  GeyserBaseParticleOptions, BlockParticleOption). Tests:
  `required_option_missing_when_absent_or_incomplete`,
  `required_options_satisfied_or_not_needed`.

No source edits this pass; all fixes already present and verified against vanilla. No cargo run
(forbidden for workers); compilation left to the orchestrator.
