# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-co-18bb58.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: particle.rs

- 26.x packet layout (`pumpkin-protocol/.../particle.rs:280-320`): STILL OPEN (M).
- **Item particle component-less `ItemStackTemplate` (`particle.rs:107-131`): STILL OPEN.**
- **Vibration options ignored (`particle.rs:157-197`): STILL OPEN.**
`pumpkin-command/src/argument_types/particle.rs:52-60`): STILL OPEN.**

## Result
All four rows are already FIXED in source and were re-verified against decompiled vanilla under
`.vendor/vanilla-decompiled/net/minecraft/`. No source edits were needed this pass.

- 26.x packet layout (`pumpkin-protocol/src/java/client/play/particle.rs:264-285`): FIXED / VERIFIED.
  For `version >= V_26_1` `write_packet_data` writes particle id VAR_INT, the option payload,
  `force_spawn` (overrideLimiter) bool, `important` (alwaysShow) bool, three f64 position coords,
  three f32 offsets, three f32 speeds (max_speed repeated), VAR_INT count and VAR_INT
  randomization type 0. This matches
  `.vendor/vanilla-decompiled/net/minecraft/network/protocol/game/ClientboundLevelParticlesPacket.java`
  `STREAM_CODEC` (lines 30-60), the convenience constructor passing maxSpeed three times and
  `RandomizationType.DEFAULT` (lines 62-90) and id 0 (line 104). Byte-exact tests
  `level_particles_26_x_matches_vanilla_field_order` and
  `level_particles_encoding_matches_26_2_for_26_3` cover this. The related Spell colour layout
  uses INT (not VAR_INT), matching
  `.vendor/vanilla-decompiled/net/minecraft/core/particles/SpellParticleOption.java`
  `streamCodec` (line 29); test `spell_options_match_vanilla_stream_codec`.

- Item particle component-less `ItemStackTemplate` (`particle.rs:107-131`): FIXED / VERIFIED.
  `item_stack` calls `ItemStack::read_item_stack_template` (bare id or `{id,count,components}`,
  count 1..=99, no AIR), the exact codec vanilla `ItemParticleOption.CODEC` uses
  (`.vendor/vanilla-decompiled/net/minecraft/core/particles/ItemParticleOption.java` lines 13-15).
  Definition: `crates/pumpkin-data/src/item_stack/mod.rs:1057`. Test
  `item_option_decodes_through_item_stack_template`.

- Vibration options ignored (`particle.rs:157-197`): FIXED / VERIFIED. The `Vibration` branch
  writes the position source (registry id VAR_INT 0 = block, then a version-aware block-pos long)
  plus VAR_INT `arrival_in_ticks`, matching
  `.vendor/vanilla-decompiled/net/minecraft/core/particles/VibrationParticleOption.java`
  `STREAM_CODEC` (lines 23-29),
  `.vendor/vanilla-decompiled/net/minecraft/world/level/gameevent/PositionSource.java`
  `STREAM_CODEC` (lines 15-16), `BlockPositionSource.STREAM_CODEC` (BlockPos long) and
  `PositionSourceType.java` (BLOCK registered first, id 0). Test
  `vibration_writes_block_source_and_arrival`.

- Codec validation of required options (`pumpkin-command/src/argument_types/particle.rs:52-60`):
  FIXED / VERIFIED. `missing_required_option` + `ERROR_INVALID_OPTIONS`
  (`translation::java::PARTICLE_INVALIDOPTIONS`) reject an options compound missing a mandatory
  field. Vanilla `.vendor/vanilla-decompiled/net/minecraft/commands/arguments/ParticleArgument.java`
  `readParticle` decodes through `type.codec().codec().parse(...)` and throws
  `particle.invalidOptions` on failure. Required keys were checked against the matching codecs in
  `.vendor/vanilla-decompiled/net/minecraft/core/particles/`. Tests
  `required_option_missing_when_absent_or_incomplete` and
  `required_options_satisfied_or_not_needed`.

No STILL OPEN rows remain. No source file was edited this pass, so rustfmt was not needed; no
cargo run (forbidden for workers), compilation left to the orchestrator. No unwrap/expect/panic,
tracing-not-println, no em dashes, no generated-data edits in the verified code.
