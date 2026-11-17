# audit / crates/pumpkin-core/src/command/commands/particle.rs

Target: `JavaMinecraftVersion::V_26_3`. Reference: decompiled Paper/Canvas tree in
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`
(`server/commands/ParticleCommand.java`, `commands/arguments/ParticleArgument.java`,
`server/level/ServerLevel.java#sendParticles`, `core/particles/*`).

The file has no TODO/FIXME markers; the five "vanilla" hits the tracker counted are doc
comments. The command tree, argument types (`Vec3Argument.vec3()`, `vec3(false)`,
`floatArg(0.0)`, `integer(0)`, `EntityArgument.players()`), permission level and the
`force`/`normal` literals already match vanilla.

Fixed in this pass:

- `particle.rs:176-185` - `sculk_charge` and `shriek` wrote an empty payload; vanilla
  `SculkChargeParticleOptions.STREAM_CODEC` is a FLOAT `roll` and
  `ShriekParticleOption.STREAM_CODEC` a VAR_INT `delay`. Old clients mis-decoded the rest
  of the packet. Covered by two unit tests.
- `particle.rs:248-250` - the success message used `format!("{particle:?}").to_lowercase()`
  (`dustcolortransition`); vanilla uses the registry key (§
  `BuiltInRegistries.PARTICLE_TYPE.getKey(...)`), now `minecraft:dust_color_transition`.
- `crates/pumpkin-command/src/argument_types/particle.rs:66-70` - suggestions were built
  from `Debug` lowercase over `0..=124`, so ids 125-127 (`block_crumble`, `firefly`,
  `sulfur_cube_goo`) were missing and every multi-word name was unusable. Now
  `map_while(Particle::from_id)` + `to_name()`, namespaced like vanilla `suggestResource`.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| `particle.rs:157-197`, `pumpkin-command/src/argument_types/particle.rs:52-60` | `ParticleArgument.readParticle` parses options with the particle codec and throws `particle.invalidOptions` when a required field is missing | Missing options silently fall back to hard-coded defaults (`Dust` red, `EntityEffect` -1, block state 0, `ItemStack::EMPTY`, `roll` 0, `delay` 0); unknown `Properties` are dropped by `block_state_id`'s `filter_map` | Validate required fields per particle type in the argument parser (which owns the error type) and return `ERROR_INVALID_OPTIONS` (`translation::java::PARTICLE_INVALIDOPTIONS`) | M |
| `particle.rs:157-162` | `SpellParticleOption.STREAM_CODEC` = INT colour + FLOAT power (`ParticleTypes.EFFECT`, `INSTANT_EFFECT`); `ColorParticleOption` = INT colour only since 1.21.4/1.21.2 | `effect`/`instant_effect` send no payload; `entity_effect`/`tinted_leaves` send the colour unconditionally, which breaks clients older than the type change. One packet is built for every viewer, so the payload cannot be version-gated | Build the packet per viewer version (`ParticleOptions::encode_for`, `remap_particle_id_for_version`) or drop the packet, as `pumpkin-protocol/.../particle.rs` already models for `trail`/`spell` | M |
| `particle.rs:157-197` | `TrailParticleOption` requires `target` (3 doubles), `color` (INT) and `duration` (VAR_INT) | `trail` options are ignored, so the client reads the following fields as into the payload | Reuse `pumpkin_protocol::...::ParticleOptions::Trail` with the 1.21.2+ gate | M |
| `particle.rs:157-197` | `VibrationParticleOption` requires `destination` (position source) plus `arrival_in_ticks` | `vibration` options are ignored | Encode the position source; no helper exists yet | L |
| `particle.rs:157-197` | `dust_pillar`, `block_crumble`, `item_cobweb`, `geyser*`, `firefly`, `sulfur_cube_goo` payloads | Not encoded; which of these are `BlockParticleOption`/`ItemParticleOption`/simple is unverified for `V_26_3` | Check each `ParticleTypes` entry against the target assets and extend the match | M |
| `particle.rs:225-241` | `ServerLevel#sendParticles(ServerPlayer, ...)` uses `player.blockPosition().closerToCenterThan(pos, force ? 512 : 32)` (block centre, strict `<`) and `player.level() != this` | Uses the exact player position with `<=`; half a block of difference at the radius edge | Compare against `block_position + 0.5` with strict `<` | S |
| `particle.rs:107-131` (`item_stack`) | `ItemParticleOption.CODEC` uses `ItemStackTemplate.CODEC` (item id + count, no components) | Uses the full `ItemStack` reader, so components in SNBT are accepted where vanilla rejects them | Decode as `ItemStackTemplate` once such a type exists | S |
| `crates/pumpkin-protocol/src/java/client/play/particle.rs:280-320` | 26.x `ClientboundLevelParticlesPacket`: particle options first, three per-axis speeds, `count` VAR_INT, trailing `randomization_type` | `CParticle` writes particle id + payload last, one `max_speed`, `count` as i32 BE, no randomization type (out of this file's scope, flagged while reading `ParticleCommand`) | Needs a dedicated protocol audit with byte-exact tests against a vanilla packet dump | L |
