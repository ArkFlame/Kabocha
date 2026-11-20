//! Stand-in names for hover ids the client's version cannot decode.

use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::remap::{
    entity_id_remap::remap_entity_id_for_version,
    item_id_remap::{remap_item_id_for_version, remap_item_id_from_version},
};

const ITEM_FALLBACK: &str = "minecraft:barrier";
const ENTITY_FALLBACK: &str = "minecraft:pig";

/// Resolves a current item name to one that exists in `version`.
#[must_use]
pub(super) fn item_name(name: &str, version: JavaMinecraftVersion) -> String {
    let Some(item) = Item::from_registry_key(name) else {
        return ITEM_FALLBACK.to_string();
    };
    let target = remap_item_id_for_version(item.id, version);
    let stand_in = remap_item_id_from_version(target, version);
    // The round trip only names a valid stand-in when it maps back to the target slot.
    if remap_item_id_for_version(stand_in, version) != target {
        return ITEM_FALLBACK.to_string();
    }
    Item::from_id(stand_in).map_or_else(
        || ITEM_FALLBACK.to_string(),
        |item| item.resource_location().into_owned(),
    )
}

/// Resolves a current entity name to one that exists in `version`.
#[must_use]
pub(super) fn entity_name(name: &str, version: JavaMinecraftVersion) -> String {
    let Some(entity) = EntityType::from_name(name) else {
        return ENTITY_FALLBACK.to_string();
    };
    let target = remap_entity_id_for_version(entity.id, version);
    // Entities are listed in id order, so the first match is the oldest name for that slot:
    // entities added after `version` share a slot with the one they replace.
    EntityType::ALL
        .iter()
        .find(|candidate| remap_entity_id_for_version(candidate.id, version) == target)
        .map_or_else(
            || ENTITY_FALLBACK.to_string(),
            |stand_in| format!("minecraft:{}", stand_in.resource_name),
        )
}
