use crate::block::entities::sculk_catalyst::SculkCatalystBlockEntity;
use crate::block::{BlockBehaviour, BlockMetadata, OnPlaceArgs, OnScheduledTickArgs};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{BlockId, BlockStateId, block_properties::SculkCatalystLikeProperties};
use rand::RngExt;
use pumpkin_world::world::BlockFlags;

pub struct SculkCatalystBlock;

impl BlockMetadata for SculkCatalystBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SCULK_CATALYST].into()
    }
}

impl BlockBehaviour for SculkCatalystBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = SculkCatalystLikeProperties::default(args.block);
        props.bloom = false;
        props.to_state_id(args.block)
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = SculkCatalystLikeProperties::from_state_id(state_id);
        if props.bloom {
            props.bloom = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            // Vanilla `SculkCatalystBlock.tick` plays the bloom sound when the
            // animation ends, at volume 2.0 and a random pitch.
            let pitch = rand::rng().random_range(0.6f32..1.0);
            args.world.play_sound_fine(
                Sound::BlockSculkCatalystBloom,
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
                2.0,
                pitch,
            );
        }
        if let Some(entity) = args.world.get_block_entity(args.position)
            && let Some(catalyst) = entity.as_any().downcast_ref::<SculkCatalystBlockEntity>()
        {
            catalyst.end_bloom();
        }
    }
}
