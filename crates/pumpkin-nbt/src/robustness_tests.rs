//! Deterministic never-panic checks for the NBT readers over seeded, mutated bytes.

use std::io::Cursor;

use crate::{Nbt, NbtCompound, deserializer::NbtReadHelperJava, tag::NbtTag};

const ROUNDS: usize = 4_000;
const MAX_RANDOM_LEN: usize = 256;

/// Named root compound holding one `Int` (`a` = 5).
const SEED_COMPOUND: &[u8] = &[
    0x0a, 0x00, 0x00, 0x03, 0x00, 0x01, b'a', 0x00, 0x00, 0x00, 0x05, 0x00,
];
/// Named root compound holding one `IntArray` (`b` = [2, 4]).
const SEED_INT_ARRAY: &[u8] = &[
    0x0a, 0x00, 0x00, 0x0b, 0x00, 0x01, b'b', 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00,
    0x00, 0x00, 0x04, 0x00,
];

/// Tiny xorshift generator, so every run sees the same byte sequences.
struct XorShift(u64);

impl XorShift {
    const fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "the modulo keeps the value below `n`"
    )]
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

fn random_bytes(rng: &mut XorShift, max_len: usize) -> Vec<u8> {
    let len = rng.below(max_len + 1);
    (0..len).map(|_| rng.next_u64() as u8).collect()
}

/// Flips a few bytes of `seed` and sometimes truncates it.
fn mutate(rng: &mut XorShift, seed: &[u8]) -> Vec<u8> {
    let mut out = seed.to_vec();
    for _ in 0..=rng.below(3) {
        if out.is_empty() {
            break;
        }
        let idx = rng.below(out.len());
        out[idx] = rng.next_u64() as u8;
    }
    if rng.below(4) == 0 {
        let cut = rng.below(out.len() + 1);
        out.truncate(cut);
    }
    out
}

#[test]
fn nbt_readers_never_panic_on_seeded_input() {
    let seeds = [SEED_COMPOUND, SEED_INT_ARRAY];
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);

    for round in 0..ROUNDS {
        let bytes = if round % 2 == 0 {
            random_bytes(&mut rng, MAX_RANDOM_LEN)
        } else {
            mutate(&mut rng, seeds[round % seeds.len()])
        };

        let _ = Nbt::read(&mut NbtReadHelperJava::new(Cursor::new(bytes.as_slice())));
        let _ = Nbt::read_unnamed(&mut NbtReadHelperJava::new(Cursor::new(bytes.as_slice())));
        let _ = NbtTag::deserialize(&mut NbtReadHelperJava::new(Cursor::new(bytes.as_slice())));
        let _ = NbtCompound::deserialize_content(&mut NbtReadHelperJava::new(Cursor::new(
            bytes.as_slice(),
        )));
    }
}

#[test]
fn nbt_rejects_array_length_prefix_above_limit() -> Result<(), String> {
    // An `IntArray` payload declaring 0x7fff_ffff elements with no data behind it.
    let bytes = [0x0b, 0x7f, 0xff, 0xff, 0xff];
    let result = NbtTag::deserialize(&mut NbtReadHelperJava::new(Cursor::new(&bytes[..])));
    if result.is_ok() {
        return Err("oversized IntArray prefix was accepted".to_string());
    }
    Ok(())
}
