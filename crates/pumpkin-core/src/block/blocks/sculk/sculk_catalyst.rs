use crate::block::entities::sculk_catalyst::SculkCatalystBlockEntity;
use crate::block::{BlockBehaviour, BlockMetadata, BrokenArgs, OnPlaceArgs, OnScheduledTickArgs};
use crate::entity::experience_orb::ExperienceOrbEntity;
use pumpkin_data::{
    BlockId, BlockStateId, Enchantment, block_properties::SculkCatalystLikeProperties,
};
use pumpkin_util::GameMode;
use pumpkin_world::world::BlockFlags;

/// Vanilla `SculkCatalystBlock.xpRange` (`ConstantInt.of(5)`).
const SCULK_CATALYST_XP: u32 = 5;

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
        }
        if let Some(entity) = args.world.get_block_entity(args.position)
            && let Some(catalyst) = entity.as_any().downcast_ref::<SculkCatalystBlockEntity>()
        {
            catalyst.end_bloom();
        }
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        // Vanilla `SculkCatalystBlock.getExpDrop`: `tryDropExperience` returns 0 with Silk Touch,
        // and `popExperience` is gated on the `block_drops` gamerule. Creative players get no drop.
        if args.player.gamemode.load() == GameMode::Creative
            || !args.world.level_info.load().game_rules.block_drops
            || args
                .player
                .inventory()
                .held_item()
                .get_enchantment_level(&Enchantment::SILK_TOUCH)
                > 0
        {
            return;
        }
        ExperienceOrbEntity::spawn(
            args.world,
            args.position.to_centered_f64(),
            SCULK_CATALYST_XP,
        );
    }
}
