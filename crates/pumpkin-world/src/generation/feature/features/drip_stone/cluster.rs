use pumpkin_data::BlockDirection;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

// Datapack values for `minecraft:dripstone_cluster`
// (assets/datapack/data/minecraft/worldgen/feature/dripstone_cluster.json).
const RADIUS_MIN: i32 = 2;
const RADIUS_MAX: i32 = 8;
const HEIGHT_MIN: i32 = 3;
const HEIGHT_MAX: i32 = 6;
const DENSITY_MIN: f32 = 0.3;
const DENSITY_MAX: f32 = 0.7;
const CHANCE_AT_MAX_DISTANCE: f32 = 0.1;

pub struct DripstoneClusterFeature;

impl DripstoneClusterFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let radius = RADIUS_MIN + random.next_bounded_i32(RADIUS_MAX - RADIUS_MIN + 1);
        let height = HEIGHT_MIN + random.next_bounded_i32(HEIGHT_MAX - HEIGHT_MIN + 1);
        let density = DENSITY_MIN + random.next_f32() * (DENSITY_MAX - DENSITY_MIN);
        let mut generated = false;

        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let distance = ((dx * dx + dz * dz) as f32).sqrt();
                if distance > radius as f32 {
                    continue;
                }
                // Vanilla tapers the speleothem chance towards the cluster edge.
                let chance = density
                    + (CHANCE_AT_MAX_DISTANCE - density) * (distance / radius as f32);
                if random.next_f32() > chance {
                    continue;
                }
                let column_height = (((height as f32) * (1.0 - distance / (radius as f32 + 1.0)))
                    as i32)
                    .clamp(1, height);

                let mut current_pos = BlockPos::new(pos.0.x + dx, pos.0.y, pos.0.z + dz);
                let mut found_floor = false;
                while current_pos.0.y > chunk.bottom_y() as i32 + 2 {
                    if !chunk.is_air(&current_pos.0) {
                        found_floor = true;
                        break;
                    }
                    current_pos = current_pos.down();
                }
                if !found_floor {
                    continue;
                }
                let floor_pos = current_pos;
                if super::gen_dripstone(chunk, floor_pos) {
                    generated = true;
                    let stalagmite_start = floor_pos.up();
                    if chunk.is_air(&stalagmite_start.0) {
                        super::grow_pointed_dripstone(
                            chunk,
                            stalagmite_start,
                            BlockDirection::Up,
                            column_height,
                            false,
                        );
                    }
                }

                // Stalactites hang from the ceiling of the cave the origin sits in.
                let mut ceiling_pos = BlockPos::new(pos.0.x + dx, pos.0.y, pos.0.z + dz);
                let mut found_ceiling = false;
                while ceiling_pos.0.y < floor_pos.0.y + 24 {
                    if !chunk.is_air(&ceiling_pos.up().0) {
                        found_ceiling = true;
                        break;
                    }
                    ceiling_pos = ceiling_pos.up();
                }
                if !found_ceiling || ceiling_pos.0.y == floor_pos.0.y {
                    continue;
                }
                if super::gen_dripstone(chunk, ceiling_pos) {
                    generated = true;
                    let stalactite_start = ceiling_pos.down();
                    if chunk.is_air(&stalactite_start.0) {
                        super::grow_pointed_dripstone(
                            chunk,
                            stalactite_start,
                            BlockDirection::Down,
                            column_height,
                            false,
                        );
                    }
                }
            }
        }
        generated
    }
}
