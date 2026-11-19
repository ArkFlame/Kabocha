use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::CustomNameImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;

pub struct NameTagItem;

impl ItemMetadata for NameTagItem {
    fn ids() -> Box<[u16]> {
        [Item::NAME_TAG.id].into()
    }
}

impl ItemBehaviour for NameTagItem {
    fn use_on_entity(
        &self,
        item: &mut ItemStack,
        _player: &Player,
        entity: Arc<dyn EntityBase>,
    ) -> BlockActionResult {
        let raw_entity = entity.get_entity();
        if let Some(living) = entity.get_living_entity()
            && raw_entity.entity_type.saveable
            && let Some(name) = item.get_data_component::<CustomNameImpl>()
        {
            // Vanilla returns SUCCESS for any named, serializable target, but only names and
            // consumes the tag while the target is alive.
            if living.entity.is_alive() {
                raw_entity.set_custom_name(name.name.clone());
                // Vanilla names any serializable LivingEntity (including ArmorStand), but only mobs
                // can be made persistent.
                if let Some(mob) = entity.get_mob() {
                    mob.get_mob_entity()
                        .persistence_required
                        .store(true, Ordering::Relaxed);
                }
                // Vanilla `NameTagItem#interactLivingEntity` calls `ItemStack#shrink` without a creative check.
                item.decrement(1);
            }
            return BlockActionResult::SuccessServer;
        }
        BlockActionResult::Pass
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
