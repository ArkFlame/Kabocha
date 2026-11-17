//! Block-equality regression harness for terrain generation.
//!
//! Generates a 3x3 chunk square around the origin for a few fixed seeds, hashes every
//! block-state id of every generated chunk with FNV-1a 64 and compares the digest against
//! tests/data/block_hash.txt. When a seed has no line yet, the test fails and prints the
//! exact line to add, so the first run bootstraps the reference file.
//!
//! Reference format, one line per seed:
//!
//!     seed <n> <digest>
//!
//! The digest is FNV-1a 64 over the little-endian bytes of the u16 block-state ids, in
//! chunk order (chunk x ascending, then chunk z ascending), inside each chunk in x, z, y
//! order with y from the dimension's bottom to its top. tests/data/vanilla_block_hash.txt
//! uses the same format and is optional.
//!
//! Producing the vanilla reference: run a vanilla server with -Xint (interpreted mode, so
//! JIT floating-point/Math differences do not pollute the reference), create the world with
//! the same seed, then dump the same 3x3 chunk square to the raw little-endian u16
//! block-state stream in the order above and run it through fnv1a64 from this file. A vanilla
//! dump can also be written as "seed <n> blocks <hex>" (hex of that raw stream), which lets
//! the test report the per-block matching percentage instead of only a digest mismatch.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::panic_in_result_fn
)]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use pumpkin_data::BlockStateId;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::chunk_system::{Chunk, StagedChunkEnum, generate_single_chunk};
use pumpkin_world::generation::get_world_gen;
use pumpkin_world::world::WorldPortalExt;

const SEEDS: [u64; 3] = [0, 1, 4242];
const CHUNK_RADIUS: i32 = 1;

/// Minimal WorldPortalExt used by the generator; mirrors benches/chunk_gen.rs.
struct BlockRegistry;
impl WorldPortalExt for BlockRegistry {
    fn can_place_at(
        &self,
        _block: &pumpkin_data::Block,
        _state: &pumpkin_data::BlockState,
        _block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        _block_pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        true
    }

    fn mirror(
        &self,
        block: &pumpkin_data::Block,
        state_id: BlockStateId,
        mirror: pumpkin_data::Mirror,
    ) -> &'static pumpkin_data::BlockState {
        block.mirror(state_id, mirror)
    }

    fn rotate(
        &self,
        block: &pumpkin_data::Block,
        state_id: BlockStateId,
        rotation: pumpkin_data::Rotation,
    ) -> &'static pumpkin_data::BlockState {
        block.rotate(state_id, rotation)
    }

    fn spawn_mobs_for_chunk_generation(
        &self,
        _cache: &mut dyn pumpkin_world::generation::proto_chunk::GenerationCache,
        _biome: &'static pumpkin_data::chunk::Biome,
        _chunk_x: i32,
        _chunk_z: i32,
    ) {
    }
}

/// FNV-1a 64 over the little-endian bytes of the ids.
fn fnv1a64(ids: &[u16]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for id in ids {
        for byte in id.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(name)
}

/// Generates the 3x3 chunk square for the seed and returns every block-state id in hash order.
fn block_ids_for_seed(seed: u64) -> Vec<u16> {
    let world_gen = *get_world_gen(
        Seed(seed),
        Dimension::OVERWORLD,
        false,
        Vec::new(),
        String::new(),
    );
    let registry = BlockRegistry;

    let side = CHUNK_RADIUS * 2 + 1;
    let mut chunks = Vec::with_capacity((side * side) as usize);
    for chunk_z in -CHUNK_RADIUS..=CHUNK_RADIUS {
        for chunk_x in -CHUNK_RADIUS..=CHUNK_RADIUS {
            // Features is the last stage before Full upgrades the chunk to a level chunk;
            // the block data is final there, and Lighting never changes blocks.
            let chunk = generate_single_chunk(
                &world_gen,
                &registry,
                chunk_x,
                chunk_z,
                StagedChunkEnum::Features,
            );
            chunks.push((chunk_x, chunk_z, chunk));
        }
    }
    chunks.sort_by_key(|(x, z, _)| (*x, *z));

    let mut ids = Vec::new();
    for (_, _, chunk) in &chunks {
        let Chunk::Proto(proto) = chunk else {
            panic!("Features stage must leave the chunk as a proto chunk");
        };
        let height = i32::from(proto.height());
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..height {
                    ids.push(proto.get_block_state_raw(x, y, z).as_u16());
                }
            }
        }
    }
    ids
}

fn digest_for_seed(seed: u64) -> String {
    format!("{:016x}", fnv1a64(&block_ids_for_seed(seed)))
}

/// Reads a "seed <n> <digest>" reference file. A missing file returns None.
fn read_digests(name: &str) -> Option<HashMap<u64, String>> {
    let content = fs::read_to_string(data_path(name)).ok()?;
    let mut digests = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some("seed"), Some(seed), Some(digest)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if let Ok(seed) = seed.parse::<u64>() {
            digests.insert(seed, digest.to_owned());
        }
    }
    Some(digests)
}

/// Reads the optional raw block stream lines: "seed <n> blocks <hex>".
fn read_raw_blocks(name: &str) -> Option<HashMap<u64, Vec<u16>>> {
    let content = fs::read_to_string(data_path(name)).ok()?;
    let mut blocks = HashMap::new();
    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let (Some("seed"), Some(seed), Some("blocks"), Some(hex)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let Ok(seed) = seed.parse::<u64>() else {
            continue;
        };
        let raw = hex.as_bytes();
        if raw.len() % 4 != 0 {
            continue;
        }
        let ids = raw
            .chunks_exact(4)
            .filter_map(|pair| {
                let text = std::str::from_utf8(pair).ok()?;
                u16::from_str_radix(text, 16).ok()
            })
            .collect();
        blocks.insert(seed, ids);
    }
    Some(blocks)
}

#[test]
fn block_hash_matches_reference() {
    let reference = read_digests("block_hash.txt");

    let mut missing = Vec::new();
    for seed in SEEDS {
        let digest = digest_for_seed(seed);
        match reference.as_ref().and_then(|reference| reference.get(&seed)) {
            Some(expected) => assert_eq!(
                *expected, digest,
                "seed {seed}: terrain block hash changed, expected {expected}, got {digest}"
            ),
            None => missing.push(format!("seed {seed} {digest}")),
        }
    }

    assert!(
        missing.is_empty(),
        "tests/data/block_hash.txt has no line for some seeds; add:\n{}",
        missing.join("\n")
    );
}

#[test]
fn block_hash_matches_vanilla_dump() {
    let Some(reference) = read_digests("vanilla_block_hash.txt") else {
        return;
    };
    let raw_blocks = read_raw_blocks("vanilla_block_hash.txt").unwrap_or_default();

    for seed in SEEDS {
        let Some(expected) = reference.get(&seed) else {
            continue;
        };
        let ids = block_ids_for_seed(seed);
        let digest = format!("{:016x}", fnv1a64(&ids));

        if let Some(vanilla_ids) = raw_blocks.get(&seed) {
            let matching = ids
                .iter()
                .zip(vanilla_ids)
                .filter(|(ours, vanilla)| ours == vanilla)
                .count();
            let percentage = 100.0 * matching as f64 / vanilla_ids.len().max(1) as f64;
            println!("seed {seed}: {percentage:.3}% of blocks match the vanilla dump");
        }

        assert_eq!(
            *expected, digest,
            "seed {seed}: block hash differs from the vanilla reference, expected {expected}, got {digest}"
        );
    }
}
