//! Block state resolution from template palette entries.
//!
//! This module handles converting NBT palette entries (block name + properties)
//! to the runtime block state IDs used by the world, with support for rotation
//! and mirroring transformations.

use pumpkin_data::{Block, BlockState, BlockStateId, Mirror, Rotation};
use tracing::warn;

use super::PaletteEntry;

fn has_transformable_property(properties: &[pumpkin_data::Property]) -> bool {
    use pumpkin_data::block_properties::Property;
    properties.iter().any(|prop| {
        matches!(
            prop,
            Property::Facing(_)
                | Property::StairsShape(_)
                | Property::Hinge(_)
                | Property::RailShape(_)
                | Property::North(_)
                | Property::South(_)
                | Property::East(_)
                | Property::West(_)
                | Property::NorthWall(_)
                | Property::SouthWall(_)
                | Property::EastWall(_)
                | Property::WestWall(_)
                | Property::NorthRedstone(_)
                | Property::SouthRedstone(_)
                | Property::EastRedstone(_)
                | Property::WestRedstone(_)
                | Property::Orientation(_)
                | Property::Axis(_)
                | Property::Rotation(_)
        )
    })
}

/// Applies properties to a block, falling back to the default state when the generated
/// property lookup rejects an unmapped rotation/mirror combination.
fn state_from_properties(
    block: &Block,
    properties: &[pumpkin_data::Property],
) -> Option<BlockStateId> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        block.from_properties(properties).to_state_id(block)
    })) {
        Ok(id) => Some(id),
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str));
            if let Some(message) = message {
                warn!("Invalid template properties for {}: {message}", block.name);
            } else {
                warn!("Invalid template properties for {}", block.name);
            }
            None
        }
    }
}

/// Resolves template palette entries to block state IDs.
///
/// This resolver handles:
/// - Looking up blocks by name
/// - Applying block state properties
/// - Rotating/mirroring directional properties (facing, axis, rotation)
pub struct BlockStateResolver;

impl BlockStateResolver {
    /// Resolves a palette entry to a block state, applying rotation and mirror transforms.
    ///
    /// Returns the resolved `BlockState` or `None` if the block is not found.
    #[must_use]
    pub fn resolve(
        entry: &PaletteEntry,
        rotation: Rotation,
        mirror: Mirror,
    ) -> Option<&'static BlockState> {
        if let Some(state_id) = entry.state_id {
            if (rotation == Rotation::None && mirror == Mirror::None)
                || !has_transformable_property(&entry.properties)
            {
                return Some(BlockState::from_id(state_id));
            }
            let block = Block::from_state_id(state_id);
            let transformed_props = pumpkin_data::transform_block_properties(
                &entry.name,
                &entry.properties,
                rotation,
                mirror,
            );
            let rotated_id =
                state_from_properties(block, &transformed_props).unwrap_or(block.default_state.id);
            return Some(BlockState::from_id(rotated_id));
        }

        // Strip minecraft: prefix if present
        let block_name = entry.name.strip_prefix("minecraft:").unwrap_or(&entry.name);

        // Find the block
        let block = Block::from_name(&entry.name).or_else(|| Block::from_registry_key(block_name));

        let Some(block) = block else {
            warn!("Unknown block in template: {}", entry.name);
            return None;
        };

        // If no properties, return default state
        if entry.properties.is_empty() {
            return Some(block.default_state);
        }

        // Transform properties for rotation/mirror using the unified vanilla logic
        let state_id = if rotation == Rotation::None && mirror == Mirror::None {
            state_from_properties(block, &entry.properties).unwrap_or(block.default_state.id)
        } else {
            let transformed_props = pumpkin_data::transform_block_properties(
                &entry.name,
                &entry.properties,
                rotation,
                mirror,
            );
            state_from_properties(block, &transformed_props).unwrap_or(block.default_state.id)
        };

        Some(BlockState::from_id(state_id))
    }

    /// Resolves a palette entry without any transformation.
    #[must_use]
    pub fn resolve_simple(entry: &PaletteEntry) -> Option<&'static BlockState> {
        Self::resolve(entry, Rotation::None, Mirror::None)
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::block_properties::{
        DoorHinge, DoubleBlockHalf, Facing, Half, Property, StairsShape,
    };

    use super::*;

    #[test]
    fn malformed_rotated_property_falls_back_to_default_state() {
        let entry = PaletteEntry::with_properties(
            "minecraft:stone".to_string(),
            vec![Property::Facing(Facing::North)],
        );

        let resolved = BlockStateResolver::resolve(&entry, Rotation::Clockwise90, Mirror::None);
        assert_eq!(
            resolved.map(|state| state.id),
            Some(Block::STONE.default_state.id)
        );
    }

    #[test]
    fn resolve_simple_block() {
        let entry = PaletteEntry::new("minecraft:stone".to_string());
        let state = BlockStateResolver::resolve_simple(&entry);
        assert!(state.is_some());
    }

    #[test]
    fn resolve_with_properties() {
        let entry = PaletteEntry::with_properties(
            "minecraft:oak_stairs".to_string(),
            vec![
                Property::Facing(Facing::North),
                Property::Half(Half::Bottom),
                Property::StairsShape(StairsShape::Straight),
                Property::Waterlogged(false),
            ],
        );
        let state = BlockStateResolver::resolve_simple(&entry);
        assert!(state.is_some());
    }

    #[test]
    fn rotation_transforms_facing() {
        let entry = PaletteEntry::with_properties(
            "minecraft:furnace".to_string(),
            vec![Property::Facing(Facing::North), Property::Lit(false)],
        );

        // Get state with rotation
        let rotated = BlockStateResolver::resolve(&entry, Rotation::Clockwise90, Mirror::None);
        assert!(rotated.is_some());
    }

    #[test]
    fn unknown_block_returns_none() {
        let entry = PaletteEntry::new("minecraft:nonexistent_block".to_string());
        let state = BlockStateResolver::resolve_simple(&entry);
        assert!(state.is_none());
    }

    #[test]
    fn stairs_mirror_and_rotate_resolves_expected_state() {
        let entry = PaletteEntry::with_properties(
            "minecraft:oak_stairs".to_string(),
            vec![
                Property::Facing(Facing::North),
                Property::Half(Half::Bottom),
                Property::StairsShape(StairsShape::InnerLeft),
                Property::Waterlogged(false),
            ],
        );

        // Mirror LeftRight (north -> south, inner_left -> inner_right)
        // then rotate Clockwise90 (south -> west)
        let resolved =
            BlockStateResolver::resolve(&entry, Rotation::Clockwise90, Mirror::LeftRight).unwrap();

        // Compare with explicitly resolved target state
        let target_entry = PaletteEntry::with_properties(
            "minecraft:oak_stairs".to_string(),
            vec![
                Property::Facing(Facing::West),
                Property::Half(Half::Bottom),
                Property::StairsShape(StairsShape::InnerRight),
                Property::Waterlogged(false),
            ],
        );
        let expected = BlockStateResolver::resolve_simple(&target_entry).unwrap();
        assert_eq!(resolved.id, expected.id);
    }

    #[test]
    fn door_mirror_resolves_flipped_hinge() {
        let entry = PaletteEntry::with_properties(
            "minecraft:oak_door".to_string(),
            vec![
                Property::Facing(Facing::East),
                Property::DoubleBlockHalf(DoubleBlockHalf::Lower),
                Property::Hinge(DoorHinge::Left),
                Property::Open(false),
                Property::Powered(false),
            ],
        );

        // Mirror FrontBack (east -> west, hinge left -> right)
        let resolved =
            BlockStateResolver::resolve(&entry, Rotation::None, Mirror::FrontBack).unwrap();

        let target_entry = PaletteEntry::with_properties(
            "minecraft:oak_door".to_string(),
            vec![
                Property::Facing(Facing::West),
                Property::DoubleBlockHalf(DoubleBlockHalf::Lower),
                Property::Hinge(DoorHinge::Right),
                Property::Open(false),
                Property::Powered(false),
            ],
        );
        let expected = BlockStateResolver::resolve_simple(&target_entry).unwrap();
        assert_eq!(resolved.id, expected.id);
    }
}
