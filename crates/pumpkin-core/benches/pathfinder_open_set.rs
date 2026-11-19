use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_core::entity::ai::pathfinder::binary_heap::BinaryHeap;
use pumpkin_core::entity::ai::pathfinder::node::Node;
use pumpkin_util::math::position::BlockPos;
use std::hint::black_box;

/// A 32x32 patch of open nodes, close to the open set of one A* search.
const GRID: i32 = 32;
const MULTIPLIER: u32 = 2_654_435_761;
const F_SCORE_BUCKETS: u32 = 1000;
const F_SCORE_SCALE: f32 = 10.0;

/// Deterministic pseudo-random f scores, so every run inserts the same order.
fn make_nodes() -> Vec<Node> {
    let mut nodes = Vec::with_capacity((GRID * GRID) as usize);
    for x in 0..GRID {
        for z in 0..GRID {
            let index = (x * GRID + z) as u32;
            let mut node = Node::new(BlockPos::new(x, 64, z));
            let bucket = (index.wrapping_mul(MULTIPLIER) >> 8) % F_SCORE_BUCKETS;
            node.f = bucket as f32 / F_SCORE_SCALE;
            nodes.push(node);
        }
    }
    nodes
}

fn bench_open_set(c: &mut Criterion) {
    let nodes = make_nodes();

    let mut group = c.benchmark_group("pathfinder_open_set");
    group.sample_size(20);
    group.bench_function("insert_then_pop_1024", |b| {
        b.iter(|| {
            let mut heap = BinaryHeap::with_capacity(nodes.len());
            for node in &nodes {
                heap.insert(*node);
            }
            while let Some(node) = heap.pop() {
                black_box(node);
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_open_set);
criterion_main!(benches);
