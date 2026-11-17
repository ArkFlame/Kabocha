use crate::entity::player::Player;
use crate::item::registry::ItemRegistry;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_util::GameMode;

pub struct MaceItem;

impl MaceItem {
    /// Vanilla `MaceItem` is identified by its item class, so a mace-behaviour item is
    /// recognised through the registry rather than a hardcoded item id.
    #[must_use]
    pub fn is_mace(registry: &ItemRegistry, item: &Item) -> bool {
        registry
            .get_pumpkin_item(item.id)
            .is_some_and(|behaviour| behaviour.as_any().is::<MaceItem>())
    }
}

impl ItemMetadata for MaceItem {
    fn ids() -> Box<[u16]> {
        [Item::MACE.id].into()
    }
}
impl ItemBehaviour for MaceItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::MaceItem;
    use crate::item::items::default_registry;
    use pumpkin_data::item::Item;

    #[test]
    fn mace_behaviour_is_identified_by_registry_class() {
        let registry = default_registry();
        assert!(MaceItem::is_mace(&registry, &Item::MACE));
        assert!(!MaceItem::is_mace(&registry, &Item::STICK));
    }
}
