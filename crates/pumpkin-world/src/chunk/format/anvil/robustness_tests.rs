//! Deterministic never-panic checks for anvil chunk records over seeded bytes.

use bytes::Bytes;

use super::{AnvilChunkData, Compression};

const ROUNDS: usize = 4_000;
const MAX_LEN: usize = 512;
const COMPRESSION_IDS: [u8; 5] = [1, 2, 3, 4, 127];

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

#[test]
fn anvil_chunk_record_never_panics_on_seeded_input() {
    let mut rng = XorShift(0xA076_1D64_78BD_642F);

    for round in 0..ROUNDS {
        let bytes = if round % 2 == 0 {
            random_bytes(&mut rng, MAX_LEN)
        } else {
            // A record with a plausible length prefix and compression byte, so the
            // decompression path runs on random payloads instead of failing early.
            let body = random_bytes(&mut rng, MAX_LEN);
            let declared = if rng.below(2) == 0 {
                rng.next_u64() as u32
            } else {
                (body.len() as u32).wrapping_add(1)
            };
            let mut record = declared.to_be_bytes().to_vec();
            record.push(COMPRESSION_IDS[rng.below(COMPRESSION_IDS.len())]);
            record.extend(body);
            record
        };
        let _ = AnvilChunkData::from_bytes(Bytes::from(bytes));
    }
}

#[test]
fn anvil_decompression_never_panics_on_random_payloads() {
    let mut rng = XorShift(0xE703_7ED1_A0B4_28DB);

    for _ in 0..ROUNDS {
        let payload = random_bytes(&mut rng, MAX_LEN);
        for compression in [
            Compression::GZip,
            Compression::ZLib,
            Compression::LZ4,
            Compression::Custom,
        ] {
            let _ = compression.decompress_data(&payload);
        }
    }
}

#[test]
fn anvil_rejects_chunk_length_prefix_beyond_record() -> Result<(), String> {
    // Declares about 2 GiB of chunk data behind a 2-byte record.
    let bytes = Bytes::from_static(&[0x7f, 0xff, 0xff, 0xff, 0x02, 0x00]);
    if AnvilChunkData::from_bytes(bytes).is_ok() {
        return Err("oversized chunk length prefix was accepted".to_string());
    }
    Ok(())
}
