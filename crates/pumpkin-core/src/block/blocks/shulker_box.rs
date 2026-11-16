use std::sync::Arc;
use std::sync::Mutex;

use crate::block::{
    GetComparatorOutputArgs, GetScreenHandlerFactoryArgs, OnPlaceArgs, OnSyncedBlockEventArgs,
    PlacedArgs,
};
use crate::block::{
    registry::BlockActionResult,
    {BlockBehaviour, NormalUseArgs},
};

use crate::block::entities::shulker_box::ShulkerBoxBlockEntity;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::Facing;
use pumpkin_data::translation;
use pumpkin_inventory::Inventory;
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

struct ShulkerBoxScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for ShulkerBoxScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let handler = create_generic_9x3(sync_id, player_inventory, self.0.clone(), player);
        let screen_handler_arc = Arc::new(Mutex::new(handler));

        Some(screen_handler_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_SHULKERBOX,
            translation::bedrock::CONTAINER_SHULKERBOX
        )
    }
}

#[pumpkin_block_from_tag("minecraft:shulker_boxes")]
pub struct ShulkerBoxBlock;

type EndRodLikeProperties = pumpkin_data::block_properties::EndRodLikeProperties;

/// Vanilla `Shulker.getProgressDeltaAabb(1.0, facing, 0.0, 0.5, bottom_center)`,
/// deflated by `1e-6`: the space the opening lid sweeps through.
fn lid_open_box(position: &BlockPos, facing: Facing) -> BoundingBox {
    let dir = match facing {
        Facing::North => Vector3::new(0.0, 0.0, -1.0),
        Facing::East => Vector3::new(1.0, 0.0, 0.0),
        Facing::South => Vector3::new(0.0, 0.0, 1.0),
        Facing::West => Vector3::new(-1.0, 0.0, 0.0),
        Facing::Up => Vector3::new(0.0, 1.0, 0.0),
        Facing::Down => Vector3::new(0.0, -1.0, 0.0),
    };
    let mut min = Vector3::new(-0.5, 0.0, -0.5);
    let mut max = Vector3::new(0.5, 1.0, 0.5);
    // `expandTowards(dir * 0.5)` then `contract(-dir)`: the box moves half a block
    // and shrinks to the far half of the neighbouring block.
    min = min.add_raw(dir.x.min(0.0) * 0.5, dir.y.min(0.0) * 0.5, dir.z.min(0.0) * 0.5);
    max = max.add_raw(dir.x.max(0.0) * 0.5, dir.y.max(0.0) * 0.5, dir.z.max(0.0) * 0.5);
    min = min.add_raw(dir.x.max(0.0), dir.y.max(0.0), dir.z.max(0.0));
    max = max.add_raw(dir.x.min(0.0), dir.y.min(0.0), dir.z.min(0.0));
    let center = Vector3::new(
        f64::from(position.0.x) + 0.5,
        f64::from(position.0.y),
        f64::from(position.0.z) + 0.5,
    );
    BoundingBox::new(
        min.add_raw(center.x, center.y, center.z),
        max.add_raw(center.x, center.y, center.z),
    )
    .expand_all(-1.0e-6)
}

impl BlockBehaviour for ShulkerBoxBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = EndRodLikeProperties::default(args.block);
        props.facing = args.direction.to_facing().opposite();
        props.to_state_id(args.block)
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        // On the server, we don't need the Animation steps for now, because the client is responsible for that.
        // TODO: Do not open the shulker box when it is currently closing
        args.r#type == Self::OPEN_ANIMATION_EVENT_TYPE
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let barrel_block_entity = ShulkerBoxBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(barrel_block_entity));
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        // Vanilla `ShulkerBoxBlock.canOpen`: a closed box cannot open when a
        // collidable block occupies the space its lid sweeps through.
        let facing =
            EndRodLikeProperties::from_state_id(args.world.get_block_state_id(args.position))
                .facing;
        if !args.world.is_space_empty(lid_open_box(args.position, facing)) {
            return BlockActionResult::Success;
        }

        if let Some(factory) = self.get_screen_handler_factory(GetScreenHandlerFactoryArgs {
            server: args.server,
            world: args.world,
            block: args.block,
            position: args.position,
            player: args.player,
        }) {
            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::OpenShulkerBox as i32,
                1,
            );
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));
        }

        BlockActionResult::Success
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        let block_entity = args.world.get_block_entity(args.position)?;
        let inventory = block_entity.get_inventory()?;
        Some(Box::new(ShulkerBoxScreenFactory(inventory)))
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        crate::block::container_comparator_output(&args)
    }
}

impl ShulkerBoxBlock {
    pub const OPEN_ANIMATION_EVENT_TYPE: u8 = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lid_open_box_sweeps_the_space_above() {
        let pos = BlockPos::new(3, 64, -2);
        let space = lid_open_box(&pos, Facing::Up);
        assert!(space.min.x >= 3.0 && space.max.x <= 4.0);
        assert!(space.min.z >= -2.0 && space.max.z <= -1.0);
        assert!(space.min.y >= 65.0 && space.max.y <= 65.5);
    }

    #[test]
    fn lid_open_box_sweeps_the_space_below_when_facing_down() {
        let pos = BlockPos::new(0, 64, 0);
        let space = lid_open_box(&pos, Facing::Down);
        assert!(space.min.y >= 63.5 && space.max.y <= 64.0);
    }
}
