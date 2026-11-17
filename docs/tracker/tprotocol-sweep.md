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

## Open (not fixed)

- `codec/bitset.rs` `Bitset<N>`: `get`/`set` assert `index <= N` (should be `< N`), and the
  read format is 7-bit groups with continuation bits, which is not a vanilla wire format.
  Only Bedrock `PlayerAuthInput` uses it. Bedrock is out of scope here. Confidence in the
  format mismatch: medium. Needs a Bedrock-side check.
- `codec/bit_set.rs` `BitSet::MAX_LENGTH = 1024` longs on read. Vanilla bounds it only by
  the packet size. Low impact, not changed.
