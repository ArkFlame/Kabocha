//! Deterministic never-panic checks for serverbound packet decoding over seeded bytes.

use pumpkin_data::packet::CURRENT_MC_VERSION;

use crate::{
    ServerPacket,
    java::server::play::{SChatCommandSigned, SClickSlot, SSetGameRule},
};

const ROUNDS: usize = 4_000;
const MAX_LEN: usize = 192;

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

fn random_bytes(rng: &mut XorShift) -> Vec<u8> {
    let len = rng.below(MAX_LEN + 1);
    (0..len).map(|_| rng.next_u64() as u8).collect()
}

/// Varint prefixes that claim extreme lengths, followed by random filler.
fn hostile_prefix(rng: &mut XorShift) -> Vec<u8> {
    const PREFIXES: [&[u8]; 4] = [
        &[0xff, 0xff, 0xff, 0xff, 0x0f],
        &[0x80, 0x80, 0x80, 0x80, 0x08],
        &[0x7f],
        &[0x00],
    ];
    let mut bytes = PREFIXES[rng.below(PREFIXES.len())].to_vec();
    bytes.extend(random_bytes(rng));
    bytes
}

#[test]
fn serverbound_packets_never_panic_on_seeded_input() {
    let mut rng = XorShift(0xD1B5_4A32_D192_ED03);

    for round in 0..ROUNDS {
        let bytes = if round % 2 == 0 {
            random_bytes(&mut rng)
        } else {
            hostile_prefix(&mut rng)
        };

        let mut slice = bytes.as_slice();
        let _ = SClickSlot::read(&mut slice, &CURRENT_MC_VERSION);
        let mut slice = bytes.as_slice();
        let _ = SChatCommandSigned::read(&mut slice, &CURRENT_MC_VERSION);
        let mut slice = bytes.as_slice();
        let _ = SSetGameRule::read(&mut slice, &CURRENT_MC_VERSION);
    }
}
