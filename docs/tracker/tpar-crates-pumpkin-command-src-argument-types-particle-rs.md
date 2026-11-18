# Parity rows for crates/pumpkin-command/src/argument_types/particle.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-co-18bb58** [partial/OWN-OPEN] own audit: 16 open, 12 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-co-18bb58.md) docs/tracker/taudit-crates-pumpkin-core-src-co-18bb58.md

## Result
No rows left STILL OPEN. One row, closed VERIFIED (third pass).

- **own/taudit-crates-pumpkin-core-src-co-18bb58 [partial/OWN-OPEN] -> VERIFIED.**
  The referenced audit (docs/tracker/taudit-crates-pumpkin-core-src-co-18bb58.md)
  now ends with no STILL OPEN rows: its two last-open rows (codec validation of
  required options in pumpkin-command/src/argument_types/particle.rs, and the 26.x
  ClientboundLevelParticlesPacket layout in pumpkin-protocol) are FIXED / VERIFIED
  by other units. Direct verification of this unit's file against decompiled
  vanilla .vendor/vanilla-decompiled/net/minecraft/:
  - commands/arguments/ParticleArgument.java:29-34,60-83 - ERROR_UNKNOWN_PARTICLE
    (particle.notFound) and ERROR_INVALID_OPTIONS (particle.invalidOptions) match;
    options are parsed as SNBT right after the id and a missing mandatory key is
    a parse error (missing_required_option + ERROR_INVALID_OPTIONS in parse()).
  - commands/arguments/ParticleArgument.java:28 - examples() third entry corrected
    to "particle{foo:bar}" to match vanilla EXAMPLES ["foo","foo:bar","particle{foo:bar}"].
  - commands/arguments/ParticleArgument.java:86-89 - suggestions via
    minecraft:{to_name} (map_while(Particle::from_id)) match suggestResource.
  - core/particles/CODECs - missing_required_option keys match every required
    field: block_state (BlockParticleOption), color+scale (DustParticleOptions),
    from_color/to_color/scale (DustColorTransitionOptions), color
    (ColorParticleOption), roll (SculkChargeParticleOptions), delay
    (ShriekParticleOption), item (ItemParticleOption), destination+arrival_in_ticks
    (VibrationParticleOption), target+color+duration (TrailParticleOption),
    water_blocks (GeyserParticleOptions), water_blocks+burst_impulse_base
    (GeyserBaseParticleOptions). effect/instant_effect stay all-optional like
    SpellParticleOption (optionalFieldOf("color",-1)).

Files edited: crates/pumpkin-command/src/argument_types/particle.rs (one-line
examples parity fix). Two pure unit tests cover the validation seam; no
unwrap/expect/panic. No cargo run (forbidden for workers); compilation left to
the orchestrator.
