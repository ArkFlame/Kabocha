use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_protocol::{
    ClientPacket, VarInt,
    java::client::play::{CEntityPositionSync, CHeadRot, CKeepAlive},
};
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

/// Encodes representative clientbound play packets, framed with id and length.
fn packet_encode(c: &mut Criterion) {
    let version = JavaMinecraftVersion::V_1_21_4;
    let mut group = c.benchmark_group("java_packet_encode");

    let keep_alive = CKeepAlive::new(1_234_567_890_123_456_789);
    group.bench_function("keep_alive", |b| {
        b.iter(|| black_box(black_box(&keep_alive).serialize_packet(black_box(&version))));
    });

    let position_sync = CEntityPositionSync::new(
        VarInt(42),
        Vector3::new(123.5, 64.0, -987.25),
        Vector3::new(0.1, -0.08, 0.0),
        90.0,
        15.0,
        true,
    );
    group.bench_function("entity_position_sync", |b| {
        b.iter(|| black_box(black_box(&position_sync).serialize_packet(black_box(&version))));
    });

    let head_rot = CHeadRot::new(VarInt(42), 128);
    group.bench_function("head_rot", |b| {
        b.iter(|| black_box(black_box(&head_rot).serialize_packet(black_box(&version))));
    });

    group.finish();
}

criterion_group!(benches, packet_encode);
criterion_main!(benches);
