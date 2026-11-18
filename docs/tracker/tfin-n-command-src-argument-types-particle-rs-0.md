# Final rows from docs/tracker/tpar-crates-pumpkin-command-src-argument-types-particle-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

Single carried row closed VERIFIED. Its audit (docs/tracker/taudit-crates-pumpkin-core-src-co-18bb58.md) ends with both last-open rows (codec validation of required options; 26.x packet layout) closed.

## Result
- **own/taudit-crates-pumpkin-core-src-co-18bb58**: VERIFIED.
  - Codec validation of required options (`crates/pumpkin-command/src/argument_types/particle.rs`, `missing_required_option` and `ERROR_INVALID_OPTIONS`): FIXED and VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ParticleArgument.java` (lines 29-34 messages, 71-83 option decode). Tests: `required_option_missing_when_absent_or_incomplete`, `required_options_satisfied_or_not_needed`.
  - 26.x packet layout (`crates/pumpkin-protocol/src/java/client/play/particle.rs`, `V_26_1` branch): FIXED and VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundLevelParticlesPacket.java` (STREAM_CODEC lines 30-60).
- All other carried rows (spell/colour, trail, vibration, block/geyser payloads, player-centre range, component-less item template, spell colour INT encoding): FIXED / VERIFIED per the audit's final Result.
- BLOCKED-DATA: none.
- No source edit this pass. Citations re-checked against the vendored files. No cargo run (worker denied); build status UNVERIFIED.
