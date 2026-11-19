use std::sync::Arc;
use std::sync::atomic::Ordering;

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use pumpkin_world::block::BlockStateCodec;

use crate::entity::EntityBase;
use crate::entity::decoration::armor_stand::ArmorStandEntity;
use crate::world::{BlockFlags, World};

use super::BlockEntity;

pub struct PistonBlockEntity {
    pub position: BlockPos,
    pub pushed_block_state: &'static BlockState,
    pub facing: BlockDirection,
    pub current_progress: AtomicCell<f32>,
    pub last_progress: AtomicCell<f32>,
    pub last_ticked: AtomicCell<i64>,
    pub extending: bool,
    pub source: bool,
}

impl PistonBlockEntity {
    pub const ID: &'static str = "minecraft:piston";

    const fn movement_direction(&self) -> BlockDirection {
        if self.extending {
            self.facing
        } else {
            self.facing.opposite()
        }
    }

    /// Vanilla's `getAmountExtended`: how far back from the block's final position
    /// the visual is at a given animation progress. Negative for extending.
    fn amount_extended(&self, progress: f32) -> f32 {
        if self.extending {
            progress - 1.0
        } else {
            1.0 - progress
        }
    }

    fn dir_vec(dir: BlockDirection, scale: f64) -> Vector3<f64> {
        let off = dir.to_offset();
        Vector3::new(
            f64::from(off.x) * scale,
            f64::from(off.y) * scale,
            f64::from(off.z) * scale,
        )
    }

    /// Vanilla `Direction.from3DDataValue`: `abs(data % 6)`, so out-of-range ids wrap
    /// instead of falling back to `Down`.
    const fn facing_from_data_value(data: i32) -> BlockDirection {
        match data % 6 {
            0 => BlockDirection::Down,
            1 | -1 => BlockDirection::Up,
            2 | -2 => BlockDirection::North,
            3 | -3 => BlockDirection::South,
            4 | -4 => BlockDirection::West,
            _ => BlockDirection::East,
        }
    }

    /// Vanilla `moveStuckEntities`: a horizontal honey block drags grounded entities
    /// standing on top of it along for the whole move.
    fn drag_stuck_entities(&self, world: &Arc<World>, new_progress: f32) {
        if self.pushed_block_state.id.to_block() != &Block::HONEY_BLOCK {
            return;
        }
        let motion_dir = self.movement_direction();
        if !motion_dir.is_horizontal() {
            return;
        }
        let sticky_top = self
            .pushed_block_state
            .get_block_collision_shapes()
            .fold(0.0f64, |top, shape| top.max(shape.max.y));
        let zone = BoundingBox::new(
            Vector3::new(0.0, sticky_top, 0.0),
            Vector3::new(1.0, 1.500_001, 1.0),
        )
        .at_pos(self.position)
        .shift(Self::dir_vec(
            self.facing,
            f64::from(self.amount_extended(self.last_progress.load())),
        ));
        let delta = f64::from(new_progress - self.last_progress.load());
        for entity in world.get_entities_at_box(&zone) {
            let e = entity.get_entity();
            let pos = e.pos.load();
            let grounded = e.on_ground.load(Ordering::Relaxed);
            let inside = pos.x >= zone.min.x
                && pos.x <= zone.max.x
                && pos.z >= zone.min.z
                && pos.z <= zone.max.z;
            if !grounded || !inside || entity.get_player().is_some() {
                continue;
            }
            Self::move_entity(e, motion_dir, delta);
        }
    }

    /// Ports vanilla `PistonBlockEntity.pushEntities`: pushes entities whose
    /// bounding box intersects the moving block's swept volume during this tick.
    fn push_entities(&self, world: &Arc<World>, new_progress: f32) {
        let last = self.last_progress.load();
        let delta = f64::from(new_progress - last);
        if delta <= 0.0 {
            return;
        }

        let motion_dir = self.movement_direction();
        let amount = f64::from(self.amount_extended(last));
        let shift = Self::dir_vec(self.facing, amount);
        let motion = Self::dir_vec(motion_dir, delta);

        // Vanilla sweeps the moved block's collision shape, not a full cube.
        // The head BE uses the 4-pixel piston-head slab; a pushed block uses its
        // own collision shapes (stairs, slabs and fences then push less volume).
        let shifted: Vec<BoundingBox> = if self.source {
            vec![
                Self::head_shape_aabb(BoundingBox::from_block(&self.position), self.facing)
                    .shift(shift),
            ]
        } else {
            let shapes: Vec<BoundingBox> = self
                .pushed_block_state
                .get_block_collision_shapes_at(&self.position)
                .map(|shape| shape.at_pos(self.position).shift(shift))
                .collect();
            if shapes.is_empty() {
                return;
            }
            shapes
        };

        // Query against the union of all swept shapes (vanilla uses shape.bounds()).
        let bounds = shifted
            .iter()
            .skip(1)
            .fold(shifted[0], |acc, shape| acc.minmax(shape));
        let swept_bounds = bounds.stretch(motion);

        for entity in world.get_entities_at_box(&swept_bounds) {
            let e = entity.get_entity();
            if e.no_physics.load(Ordering::Relaxed) || ignores_piston_push(entity.as_ref()) {
                continue;
            }
            // Player movement is client-authoritative; vanilla still nudges them
            // via Entity.move(PISTON), but we skip them here to avoid teleport jank.
            if entity.get_player().is_some() {
                continue;
            }
            // Vanilla `moveCollidedEntities` bounces entities off a moving slime block.
            if self.pushed_block_state.id.to_block() == &Block::SLIME_BLOCK {
                let off = motion_dir.to_offset();
                let mut velocity = e.velocity.load();
                match motion_dir {
                    BlockDirection::East | BlockDirection::West => velocity.x = f64::from(off.x),
                    BlockDirection::Up | BlockDirection::Down => velocity.y = f64::from(off.y),
                    BlockDirection::South | BlockDirection::North => velocity.z = f64::from(off.z),
                }
                e.set_velocity(velocity);
            }

            let entity_aabb = e.bounding_box.load();
            let mut intersection: f64 = 0.0;
            for shape in &shifted {
                let swept = shape.stretch(motion);
                if !swept.intersects(&entity_aabb) {
                    continue;
                }
                intersection =
                    intersection.max(Self::intersection_size(swept, motion_dir, entity_aabb));
                if intersection >= delta {
                    break;
                }
            }
            if intersection <= 0.0 {
                continue;
            }
            let push_amount = intersection.min(delta) + 0.01;
            Self::move_entity(e, motion_dir, push_amount);

            // For a retracting head, vanilla also shoves the entity OUT of the
            // piston body cube. Without this, entities get pulled into the
            // piston and stick to it (looks like a sticky-piston drag).
            // For a retract-head BE, self.position already IS the piston
            // block position (it replaces the piston during animation).
            if !self.extending && self.source {
                Self::push_out_of_piston_body(e, &self.position, motion_dir, delta);
            }
        }
    }

    /// Vanilla `getIntersectionSize`: how much `entity` overlaps `swept` along
    /// `motion_dir`. Positive means the entity is in the path of the moving block.
    fn intersection_size(
        swept: BoundingBox,
        motion_dir: BlockDirection,
        entity: BoundingBox,
    ) -> f64 {
        match motion_dir {
            BlockDirection::East => swept.max.x - entity.min.x,
            BlockDirection::West => entity.max.x - swept.min.x,
            BlockDirection::Up => swept.max.y - entity.min.y,
            BlockDirection::Down => entity.max.y - swept.min.y,
            BlockDirection::South => swept.max.z - entity.min.z,
            BlockDirection::North => entity.max.z - swept.min.z,
        }
    }

    fn move_entity(entity: &crate::entity::Entity, dir: BlockDirection, distance: f64) {
        let new_pos = entity.pos.load() + Self::dir_vec(dir, distance);
        entity.set_pos(new_pos);
    }

    /// Vanilla `push`: when a piston head retracts, shove entities that ended up
    /// inside the piston-body cube back out the opposite direction (slightly past
    /// the move they just got, so the net motion is essentially zero).
    fn push_out_of_piston_body(
        entity: &crate::entity::Entity,
        piston_pos: &BlockPos,
        motion_dir: BlockDirection,
        amount: f64,
    ) {
        let body_aabb = BoundingBox::from_block(piston_pos);
        let entity_aabb = entity.bounding_box.load();
        if !body_aabb.intersects(&entity_aabb) {
            return;
        }
        let back = motion_dir.opposite();
        let e = Self::intersection_size(body_aabb, back, entity_aabb) + 0.01;
        let f = Self::intersection_size(
            body_aabb,
            back,
            Self::aabb_intersection(body_aabb, entity_aabb),
        ) + 0.01;
        if (e - f).abs() < 0.01 {
            let distance = e.min(amount) + 0.01;
            Self::move_entity(entity, back, distance);
        }
    }

    /// Approximation of vanilla `PistonHeadBlock` collision shape: a 4-pixel
    /// (0.25 block) slab at the front face of the block in the facing direction.
    fn head_shape_aabb(block_aabb: BoundingBox, facing: BlockDirection) -> BoundingBox {
        const HEAD_THICKNESS: f64 = 0.25;
        let mut min = block_aabb.min;
        let mut max = block_aabb.max;
        match facing {
            BlockDirection::East => min.x = max.x - HEAD_THICKNESS,
            BlockDirection::West => max.x = min.x + HEAD_THICKNESS,
            BlockDirection::Up => min.y = max.y - HEAD_THICKNESS,
            BlockDirection::Down => max.y = min.y + HEAD_THICKNESS,
            BlockDirection::South => min.z = max.z - HEAD_THICKNESS,
            BlockDirection::North => max.z = min.z + HEAD_THICKNESS,
        }
        BoundingBox::new(min, max)
    }

    const fn aabb_intersection(a: BoundingBox, b: BoundingBox) -> BoundingBox {
        BoundingBox::new(
            Vector3::new(
                a.min.x.max(b.min.x),
                a.min.y.max(b.min.y),
                a.min.z.max(b.min.z),
            ),
            Vector3::new(
                a.max.x.min(b.max.x),
                a.max.y.min(b.max.y),
                a.max.z.min(b.max.z),
            ),
        )
    }

    pub fn finish(&self, world: &Arc<World>) {
        if self.last_progress.load() < 1.0 {
            self.current_progress.store(1.0);
            self.last_progress.store(1.0);
            let pos = self.position;
            world.remove_block_entity(&pos);
            if world.get_block(&pos) == &Block::MOVING_PISTON {
                let state = if self.source {
                    Block::AIR.default_state.id
                } else {
                    world.update_from_neighbor_shapes(self.pushed_block_state.id, &pos)
                };
                world.set_block_state(&pos, state, BlockFlags::NOTIFY_ALL);
                world.update_neighbor(&pos, state.to_block());
            }
        }
    }
}

/// Vanilla `getPistonPushReaction` is `IGNORE_ENTITY` for markers, displays, area effect
/// clouds, interactions, ominous item spawners and marker armor stands: pistons pass through them.
fn ignores_piston_push(entity: &dyn EntityBase) -> bool {
    piston_ignores_entity_type(entity.get_entity().entity_type)
        || entity
            .cast_any()
            .downcast_ref::<ArmorStandEntity>()
            .is_some_and(ArmorStandEntity::is_marker)
}

fn piston_ignores_entity_type(entity_type: &EntityType) -> bool {
    [
        EntityType::MARKER.id,
        EntityType::BLOCK_DISPLAY.id,
        EntityType::ITEM_DISPLAY.id,
        EntityType::TEXT_DISPLAY.id,
        EntityType::AREA_EFFECT_CLOUD.id,
        EntityType::INTERACTION.id,
        EntityType::OMINOUS_ITEM_SPAWNER.id,
    ]
    .contains(&entity_type.id)
}

const FACING: &str = "facing";
const BLOCK_STATE: &str = "blockState";
const LAST_PROGRESS: &str = "progress";
const EXTENDING: &str = "extending";
const SOURCE: &str = "source";

impl BlockEntity for PistonBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn tick(&self, world: &Arc<World>) {
        self.last_ticked.store(world.get_world_age());
        let current_progress = self.current_progress.load();
        self.last_progress.store(current_progress);
        if current_progress >= 1.0 {
            let pos = self.position;
            world.remove_block_entity(&pos);
            if world.get_block(&pos) == &Block::MOVING_PISTON {
                let new_state = world.update_from_neighbor_shapes(self.pushed_block_state.id, &pos);
                if new_state.to_state().is_air() {
                    // The moved block can't survive here, so it is placed and then broken with drops.
                    world.set_block_state(
                        &pos,
                        self.pushed_block_state.id,
                        BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK
                            | BlockFlags::MOVED
                            | BlockFlags::UPDATE_KNOWN_SHAPE,
                    );
                    world.break_block(&pos, None, BlockFlags::NOTIFY_ALL);
                } else {
                    let new_block = new_state.to_block();
                    let new_state = new_block
                        .set_waterlogged(new_state, false)
                        .unwrap_or(new_state);
                    world.set_block_state(
                        &pos,
                        new_state,
                        BlockFlags::NOTIFY_ALL | BlockFlags::MOVED,
                    );
                    world.update_neighbor(&pos, new_block);
                }
            }
            return;
        }
        let new_progress = (current_progress + 0.5).min(1.0);
        self.push_entities(world, new_progress);
        self.drag_stuck_entities(world, new_progress);
        self.current_progress.store(new_progress);
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let pushed_block_state = nbt
            .get_compound(BLOCK_STATE)
            .and_then(BlockStateCodec::state_from_nbt)
            .map_or(Block::AIR.default_state, BlockState::from_id);
        let facing = Self::facing_from_data_value(i32::from(nbt.get_byte(FACING).unwrap_or(0)));
        // Vanilla reads these with `getFloatOr`/`getBooleanOr`, which accept any numeric tag.
        let last_progress = nbt.get_numeric_float(LAST_PROGRESS).unwrap_or(0.0);
        let extending = nbt.get_numeric_short(EXTENDING).unwrap_or(0) != 0;
        let source = nbt.get_numeric_short(SOURCE).unwrap_or(0) != 0;
        Self {
            pushed_block_state,
            position,
            facing,
            current_progress: last_progress.into(),
            last_progress: last_progress.into(),
            last_ticked: 0.into(),
            extending,
            source,
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_compound(
            BLOCK_STATE,
            BlockStateCodec::state_to_nbt(self.pushed_block_state.id),
        );
        nbt.put_byte(FACING, self.facing.to_index() as i8);
        nbt.put_float(LAST_PROGRESS, self.last_progress.load());
        nbt.put_bool(EXTENDING, self.extending);
        nbt.put_bool(SOURCE, self.source);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        Some(nbt)
    }

    // The client animates moving blocks itself from the piston block event.
    fn update_packet_nbt(&self) -> Option<NbtCompound> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod test {
    use pumpkin_data::{Block, BlockDirection};
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

    use super::PistonBlockEntity;
    use crate::block::entities::BlockEntity;

    #[test]
    fn pistons_pass_through_non_pushable_decorations() {
        use pumpkin_data::entity::EntityType;

        use super::piston_ignores_entity_type;

        assert!(piston_ignores_entity_type(&EntityType::MARKER));
        assert!(piston_ignores_entity_type(&EntityType::ITEM_DISPLAY));
        assert!(piston_ignores_entity_type(&EntityType::INTERACTION));
        assert!(!piston_ignores_entity_type(&EntityType::ZOMBIE));
        assert!(!piston_ignores_entity_type(&EntityType::ARMOR_STAND));
    }

    #[test]
    fn pushed_block_state_survives_nbt_round_trip() {
        let state = Block::OAK_PLANKS.default_state;
        let entity = PistonBlockEntity {
            position: BlockPos::new(1, 2, 3),
            pushed_block_state: state,
            facing: BlockDirection::Up,
            current_progress: 0.5f32.into(),
            last_progress: 0.0f32.into(),
            last_ticked: 0.into(),
            extending: true,
            source: false,
        };
        let mut nbt = NbtCompound::new();
        entity.write_nbt(&mut nbt);
        let restored = PistonBlockEntity::from_nbt(&nbt, BlockPos::new(1, 2, 3));
        assert_eq!(restored.pushed_block_state.id, state.id);
    }

    #[test]
    fn facing_id_wraps_like_vanilla() {
        // Vanilla `Direction.from3DDataValue` is `abs(data % 6)`.
        assert_eq!(
            PistonBlockEntity::facing_from_data_value(7),
            BlockDirection::Up
        );
        assert_eq!(
            PistonBlockEntity::facing_from_data_value(-1),
            BlockDirection::Up
        );
        let mut nbt = NbtCompound::new();
        nbt.put_byte("facing", 4);
        let restored = PistonBlockEntity::from_nbt(&nbt, BlockPos::new(0, 0, 0));
        assert_eq!(restored.facing, BlockDirection::West);
    }

    #[test]
    fn head_shape_slab_sits_on_facing_side() {
        // Vanilla PistonHeadBlock rotates a 4-pixel platform (boxZ(16, 0, 4)) so
        // it always sits on the side the head faces, never the base side.
        let block = BoundingBox::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let cases: [(BlockDirection, usize, f64, f64); 6] = [
            (BlockDirection::East, 0, 0.75, 1.0),
            (BlockDirection::West, 0, 0.0, 0.25),
            (BlockDirection::Up, 1, 0.75, 1.0),
            (BlockDirection::Down, 1, 0.0, 0.25),
            (BlockDirection::South, 2, 0.75, 1.0),
            (BlockDirection::North, 2, 0.0, 0.25),
        ];
        for (facing, axis, expected_min, expected_max) in cases {
            let slab = PistonBlockEntity::head_shape_aabb(block, facing);
            let (min, max) = match axis {
                0 => (slab.min.x, slab.max.x),
                1 => (slab.min.y, slab.max.y),
                _ => (slab.min.z, slab.max.z),
            };
            assert!(
                (min - expected_min).abs() < 1e-6,
                "{facing:?} axis {axis} min {min}"
            );
            assert!(
                (max - expected_max).abs() < 1e-6,
                "{facing:?} axis {axis} max {max}"
            );
        }
    }
}
