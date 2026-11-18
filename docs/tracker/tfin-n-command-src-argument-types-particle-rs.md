# Final rows from docs/tracker/tpar-crates-pumpkin-command-src-argument-types-particle-rs.md

Close each item below: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: crates/pumpkin-command/src/argument_types/particle.rs, crates/pumpkin-protocol/src/java/client/play/particle.rs.

Single carried row, closed VERIFIED (final pass). The referenced audit
(docs/tracker/taudit-crates-pumpkin-core-src-co-18bb58.md) has no STILL OPEN rows.

## Result

All rows end FIXED / VERIFIED against decompiled vanilla. No source edits were
needed this final pass; the fixes are present in the two particle.rs files and
match vanilla. No generated pumpkin-data field or missing subsystem blocks any
row, so there are no BLOCKED-DATA rows.

- **own/taudit-crates-pumpkin-core-src-co-18bb58 [partial/OWN-OPEN] -> VERIFIED.**
  Its two last-open rows are both closed in code, re-verified this pass against
  decompiled vanilla:
  - Codec validation of required options (pumpkin-command/src/argument_types/particle.rs,
    `missing_required_option` + `ERROR_INVALID_OPTIONS`): FIXED / VERIFIED.
    Vanilla .vendor/vanilla-decompiled/net/minecraft/commands/arguments/ParticleArgument.java
    lines 71-83 decode options via `type.codec().codec().parse(...)` and throw
    `particle.invalidOptions` for a missing mandatory key; the error messages
    `particle.notFound` / `particle.invalidOptions` (lines 29-34), EXAMPLES
    `"particle{foo:bar}"` (line 28) and resource-key suggestions (lines 86-89)
    all match the Rust. Tests: `required_option_missing_when_absent_or_incomplete`,
    `required_options_satisfied_or_not_needed`.
  - 26.x packet layout (pumpkin-protocol/src/java/client/play/particle.rs, the
    `version >= V_26_1` branch): FIXED / VERIFIED. Options first, then two bools
    (`overrideLimiter`, `alwaysShow`), f64 position, three f32 offsets, three f32
    speeds, VAR_INT count, VAR_INT randomization type 0. Matches vanilla
    .vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundLevelParticlesPacket.java
    `STREAM_CODEC` lines 30-60 and the convenience constructor lines 62-91
    (DEFAULT id 0 at line 104).

- All other carried rows (spell/colour, trail, vibration, block/geyser payloads,
  player-centre range, component-less item template, spell colour INT encoding):
  FIXED / VERIFIED in code per the audit's final Result. No STILL OPEN rows remain.

Re-checked the diff against the Worker compile pitfalls table: no guessed import
paths, helpers are free functions, no struct-initializer or macro-path hazards,
no moved-then-reused values, no unwrap/expect/panic in new code. No cargo run
(forbidden for workers); compilation left to the orchestrator.

Status: PASS fixed=1 open=0 files=docs/tracker/tfin-n-command-src-argument-types-particle-rs.md
