# protocol sweep (crates/pumpkin-protocol)

Status: UNVERIFIED. Compiled and tested from memory of vanilla 1.21.x; no cargo run in this pass.

## Fixed

- `ser/mod.rs` `write_string_bounded`: the writer checked UTF-8 byte length against the
  character bound. Vanilla `writeUtf` checks UTF-16 units against the bound and allows up to
  3x that many UTF-8 bytes (the reader already did this). Regression test added:
  `write_string_bounded_counts_utf16_units_like_vanilla`.
- `codec/lp_vector_3d.rs` extended scale tail: written and read as VarInt. Scale is up to
  2^34 after clamping, so `scale >> 2` reaches 2^32 and does not fit a VarInt. Changed to VarLong.
  Vanilla `LpVec3` also uses VarLong here (confidence: medium-high).
- `codec/lp_vector_3d.rs` `MAX_VELOCITY_CLAMP`: was 1.717_986_918_3E10, vanilla is 2^34
  (1.7179869184E10). Regression test added: `extended_scale_tail_is_varlong_and_clamps_to_vanilla_max`.

## Checked, no divergence found

- VarInt / VarLong encode, decode, overflow rejection on the 5th and 10th byte.
- BlockPos packing: 1.14+ x:26 z:26 y:12 signed, and pre-1.14 x:26 y:12 z:26.
- Angle byte packing (`pack_degrees`, `floor(deg * 256 / 360) as u8`) and unpacking (`i8 / (256/360)`).
- BitSet (VarInt length + big-endian longs, 26.3 little-endian byte form is separate).
- Network NBT: empty root name stripped for 1.20.2+ (name-less root), TAG_End for empty.
- LpVec3 header/quantisation (15-bit, bits 3/18/33), zero-vector single byte, legacy short velocity.
- CSpawnEntity field order (id, uuid, type, pos, [LpVec3 for 1.21.9+], pitch, yaw, head yaw, data, legacy velocity).
- CLogin play 1.20.2+ field order through dimension, hashed seed, game modes.

- `codec/bitset.rs` `Bitset<N>`: `get`/`set` asserted `index <= N`; now `index < N` with an
  index-out-of-range message. Tests: `last_valid_index_is_n_minus_one`, `index_n_is_out_of_range`.
  Bedrock only (`PlayerAuthInput`). UNVERIFIED (no cargo run).

## Java packet sweep (1.21.x), 15 packets

Checked field order and types against vanilla from memory. No confident divergence found, so no wire change:

- login: entity id, hardcore, dimension names, max players, view/sim distance, debug info, respawn screen, limited crafting, spawn info, secure chat. Partial check only.
- set_entity_motion (`CEntityVelocity`): VarInt id, LpVec3 1.21.9+, legacy i16 triple before.
- move_entity pos / pos_rot (`CUpdateEntityPosRot`): VarInt id, i16 deltas, u8 yaw/pitch, bool on_ground.
- level_chunk_with_light: i32 chunk xz, heightmaps, data, block entities, then light (trust edges 1.16 to 1.19.4, BitSet masks 1.17+, VarInt arrays).
- block_update: BlockPos, VarInt state.
- section_blocks_update: packed section long, suppress-light bool only 1.16 to 1.19.4, VarLong `(state << 12) | local`.
- entity_event: i32 id, i8 status.
- level_event: i32 event, BlockPos, i32 data, bool global.
- sound: holder id or inline name, category, i32 xyz, f32 volume, f32 pitch, i64 seed 1.19+.
- set_equipment: VarInt id, slot byte with 0x80 continuation, ItemStack.
- boss_event: UUID, VarInt action 0 to 5 (add, remove, health, name, style, flags).
- player_info_update: action bits 0x01 to 0x80 in vanilla enum order, data written in bit order.
- update_attributes: VarInt id, count, per property id, f64, modifiers. Modifier id string 1.21+ (UUID before). Operation is u8 here and VarInt in vanilla; the bytes match for values 0 to 127.
- set_entity_data (`CSetEntityData`): VarInt id, metadata slice. Only the outer frame was checked.

## Open (not fixed)

- `codec/bitset.rs` read format: 7-bit groups with continuation bits. This is not a vanilla
  Bedrock wire format as far as I know. Bedrock only, confidence medium. Left as is per policy.
- `codec/bit_set.rs` `BitSet::MAX_LENGTH = 1024` longs on read. Vanilla bounds it only by
  the packet size. Low impact, not changed.
