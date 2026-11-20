use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag;
use pumpkin_util::{GameMode, Hand};

pub struct SwordItem;

impl ItemMetadata for SwordItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_SWORDS.1.into()
    }
}

impl ItemBehaviour for SwordItem {
    fn normal_use_with_rotation(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        if player.world().legacy_combat() {
            let stack = player.inventory.get_stack_in_hand(hand);
            let duration = self.get_use_duration(&stack);
            player.living_entity.set_active_hand(hand, stack, duration);
        }
    }

    fn get_use_duration(&self, _stack: &ItemStack) -> i32 {
        72000
    }

    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
