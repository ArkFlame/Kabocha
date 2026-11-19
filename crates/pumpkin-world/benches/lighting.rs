use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockStateId;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::ProtoChunk;
use pumpkin_world::chunk_system::{Cache, Chunk, StagedChunkEnum};
use pumpkin_world::generation::generator::WorldGenerator;
use pumpkin_world::generation::get_world_gen;
use pumpkin_world::lighting::LightEngine;
use pumpkin_world::world::WorldPortalExt;
use std::hint::black_box;

const SEED: Seed = Seed(42);

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

fn make_world_gen() -> WorldGenerator {
    *get_world_gen(SEED, Dimension::OVERWORLD, false, Vec::new(), String::new())
}

/// Builds a 3x3 cache of proto chunks advanced to `Features`, the stage right before lighting.
fn setup_cache(world_gen: &WorldGenerator, block_registry: &BlockRegistry) -> Cache {
    let radius = 1;
    let mut cache = Cache::new(-radius, -radius, radius * 2 + 1);

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            cache
                .chunks
                .push(Chunk::Proto(Box::new(ProtoChunk::new(dx, dz, world_gen))));
        }
    }

    let pipeline = [
        StagedChunkEnum::Biomes,
        StagedChunkEnum::StructureStart,
        StagedChunkEnum::StructureReferences,
        StagedChunkEnum::Noise,
        StagedChunkEnum::Surface,
        StagedChunkEnum::Carvers,
        StagedChunkEnum::Features,
    ];
    for stage in pipeline {
        cache.advance(
            stage,
            world_gen,
            block_registry,
            &LightingEngineConfig::Default,
        );
    }
    cache
}

fn bench_initialize_light(c: &mut Criterion) {
    let world_gen = make_world_gen();
    let block_registry = BlockRegistry;
    let mut engine = LightEngine::new();

    let mut group = c.benchmark_group("lighting");
    group.sample_size(10);
    group.bench_function("initialize_light_3x3", |b| {
        b.iter_batched(
            || setup_cache(&world_gen, &block_registry),
            |mut cache| {
                engine.initialize_light(&mut cache, &LightingEngineConfig::Default, true);
                black_box(cache);
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_initialize_light);
criterion_main!(benches);
