use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{
    Axis, CreakingHeartLikeProperties, CreakingHeartState, PaleOakWoodLikeProperties,
};
use pumpkin_data::environment_attribute::EnvironmentAttribute;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;

use crate::block::entities::creaking_heart::CreakingHeartBlockEntity;
use crate::block::{
    BlockBehaviour, BrokenArgs, GetComparatorOutputArgs, OnNeighborUpdateArgs, OnPlaceArgs,
    PlacedArgs,
};
use crate::entity::EntityBase;

#[pumpkin_block("minecraft:creaking_heart")]
pub struct CreakingHeartBlock;

impl CreakingHeartBlock {
    fn active_state(
        world: &crate::world::World,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> CreakingHeartState {
        if world
            .environment_attributes()
            .get_value_bool(EnvironmentAttribute::GameplayCreakingActive, pos)
        {
            CreakingHeartState::Awake
        } else {
            CreakingHeartState::Dormant
        }
    }

    fn is_aligned_pale_oak_log(state_id: BlockStateId, axis: Axis) -> bool {
        let block = state_id.to_block();
        block.has_tag(&tag::Block::MINECRAFT_PALE_OAK_LOGS)
            && PaleOakWoodLikeProperties::from_state_id(state_id).axis == axis
    }

    fn check_active_logs(
        world: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
        axis: Axis,
    ) -> bool {
        let (pos_a, pos_b) = match axis {
            Axis::X => (pos.west(), pos.east()),
            Axis::Y => (pos.down(), pos.up()),
            Axis::Z => (pos.north(), pos.south()),
        };

        let state_a = world.get_block_state_id(&pos_a);
        let state_b = world.get_block_state_id(&pos_b);

        Self::is_aligned_pale_oak_log(state_a, axis) && Self::is_aligned_pale_oak_log(state_b, axis)
    }

    /// Vanilla CreakingHeartBlock updateState: re-evaluates CREAKING_HEART_STATE from the two
    /// aligned pale oak logs and the creaking_active environment attribute.
    pub fn update_state(
        world: &std::sync::Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
    ) {
        let state_id = world.get_block_state_id(position);
        let mut props = CreakingHeartLikeProperties::from_state_id(state_id);

        let new_state = if Self::check_active_logs(world.as_ref(), position, props.axis) {
            if props.creaking_heart_state == CreakingHeartState::Uprooted {
                world.play_sound(
                    Sound::BlockCreakingHeartSpawn,
                    SoundCategory::Blocks,
                    &position.to_f64(),
                );
            }
            Self::active_state(world, position)
        } else {
            CreakingHeartState::Uprooted
        };

        if props.creaking_heart_state != new_state {
            props.creaking_heart_state = new_state;
            world.set_block_state(
                position,
                props.to_state_id(state_id.to_block()),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }
}

impl BlockBehaviour for CreakingHeartBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = CreakingHeartLikeProperties::from_state_id(args.block.default_state.id);
        props.axis = match args.direction {
            pumpkin_data::BlockDirection::North | pumpkin_data::BlockDirection::South => Axis::Z,
            pumpkin_data::BlockDirection::East | pumpkin_data::BlockDirection::West => Axis::X,
            pumpkin_data::BlockDirection::Up | pumpkin_data::BlockDirection::Down => Axis::Y,
        };
        props.creaking_heart_state = CreakingHeartState::Uprooted;
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(CreakingHeartBlockEntity::new(*args.position)));
        Self::update_state(args.world, args.position);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        Self::update_state(args.world, args.position);
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        // Vanilla `CreakingHeartBlockEntity#getAnalogOutputSignal`: signal strength falls off
        // with the distance between the heart and its bound creaking, 15 close to 0 at 32.
        // Vanilla `hasAnalogOutputSignal` is always true; no bound creaking reads 0.
        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return Some(0);
        };
        let uuid = block_entity
            .as_any()
            .downcast_ref::<CreakingHeartBlockEntity>()
            .and_then(|heart| heart.creaking_uuid.load());
        let Some(uuid) = uuid else {
            return Some(0);
        };
        let Some(creaking) = args.world.get_entity_by_uuid(uuid) else {
            return Some(0);
        };
        let pos = creaking.get_entity().pos.load();
        let dx = pos.x - (f64::from(args.position.0.x) + 0.5);
        let dy = pos.y - (f64::from(args.position.0.y) + 0.5);
        let dz = pos.z - (f64::from(args.position.0.z) + 0.5);
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let signal = 15.0 - (distance * 15.0 / 32.0).floor();
        Some(signal.clamp(0.0, 15.0) as u8)
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        // Vanilla `CreakingHeartBlockEntity#getAnalogOutputSignal`: signal strength falls off
        // with the distance between the heart and its bound creaking, 15 close to 0 at 32.
        // Vanilla `hasAnalogOutputSignal` is always true; no bound creaking reads 0.
        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return Some(0);
        };
        let uuid = block_entity
            .as_any()
            .downcast_ref::<CreakingHeartBlockEntity>()
            .and_then(|heart| heart.creaking_uuid.load());
        let Some(uuid) = uuid else {
            return Some(0);
        };
        let Some(creaking) = args.world.get_entity_by_uuid(uuid) else {
            return Some(0);
        };
        let pos = creaking.get_entity().pos.load();
        let dx = pos.x - (f64::from(args.position.x) + 0.5);
        let dy = pos.y - (f64::from(args.position.y) + 0.5);
        let dz = pos.z - (f64::from(args.position.z) + 0.5);
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let signal = 15.0 - (distance * 15.0 / 32.0).floor();
        Some(signal.clamp(0.0, 15.0) as u8)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        args.world.play_sound(
            Sound::BlockCreakingHeartBreak,
            SoundCategory::Blocks,
            &args.position.to_f64(),
        );
    }
}
