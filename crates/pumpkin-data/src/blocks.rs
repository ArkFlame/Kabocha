use crate::{
    BlockState, BlockStateId, Property, PropertyKey,
    tag::{RegistryKey, Tag, Taggable},
};
use pumpkin_util::{
    loot_table::LootTable,
    math::{experience::Experience, position::BlockPos, vector3::Vector3},
    random::hash_block_pos,
    resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation},
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{OnceLock, RwLock};

/// Represents the static definition of a Minecraft block type.
///
/// This struct contains the base properties shared by all instances of a block
/// Data-driven attributes like `hardness` and `blast_resistance` are defined here,
/// while specific orientations or variations are stored in the associated `BlockState`.
#[derive(Debug, Clone)]
pub struct Block {
    /// The numeric ID used for internal registry mapping.
    pub id: BlockId,
    /// The unique namespaced ID (e.g., "`diamond_ore`").
    pub name: &'static str,
    /// How hard the block is to break. A value of -1.0 indicates an unbreakable block (e.g., Bedrock).
    pub hardness: f32,
    /// The block's resistance to explosions.
    pub blast_resistance: f32,
    pub map_color: u8,
    /// The friction coefficient. Default is 0.6; Ice is 0.98.
    pub slipperiness: f32,
    /// How much this block affects the speed of an entity walking on it (e.g., Soul Sand).
    pub velocity_multiplier: f32,
    /// How much this block affects an entity's jump height (e.g., Honey Blocks).
    pub jump_velocity_multiplier: f32,
    /// Vertical bounce restitution (`Block.getBounceRestitution`).
    pub bounce_restitution: f32,
    /// Fraction of fall distance reduced on landing (`Block.getFallDistanceReduction`).
    pub fall_distance_reduction: f32,
    /// The ID of the item form of this block, used for inventory and drops.
    pub item_id: u16,
    /// The initial state of the block when placed without extra data.
    pub default_state: &'static BlockState,
    /// A list of all possible valid states (properties like rotation, waterlogged, etc.) for this block.
    pub states: &'static [BlockState],
    /// Fire behavior settings. If `None`, the block is not flammable.
    pub flammable: Option<Flammable>,
    /// Defines the amount of XP dropped when the block is mined (e.g., Coal or Diamond).
    pub experience: Option<Experience>,
    /// Loot table identifier, `None` when the block has no loot table.
    pub loot_table: Option<&'static str>,
}

/// One property of a block: `index = (state - first) / stride % values.len()`.
#[derive(Debug, Clone, Copy)]
pub struct PropertySlot {
    pub key: PropertyKey,
    pub stride: u16,
    /// Values in index order, same variants as `to_props`.
    pub values: &'static [Property],
}

impl PropertySlot {
    #[must_use]
    pub const fn count(&self) -> u16 {
        self.values.len() as u16
    }

    #[must_use]
    pub fn index_of(&self, value: Property) -> Option<u16> {
        self.values
            .iter()
            .position(|slot_value| slot_value.matches_property(value))
            .map(|index| index as u16)
    }
}

struct SlotBuilder {
    key: PropertyKey,
    values: Vec<Property>,
    stride: Option<u16>,
}

/// Lazily built property slots, keyed by block id. The generated tables only store
/// the state order, so the mixed radix is derived from it on first use per block.
static PROPERTY_SLOT_CACHE: OnceLock<RwLock<HashMap<u16, &'static [PropertySlot]>>> =
    OnceLock::new();

/// Derives the slots from the generated state order. Blocks whose states are not a full
/// mixed radix of their properties (unlisted properties) get no slots.
fn build_property_slots(block: &Block) -> &'static [PropertySlot] {
    let state_count = block.states.len();
    if state_count <= 1 {
        return &[];
    }
    let Some(first) = block.properties(block.states[0].id) else {
        return &[];
    };
    let first_props = first.to_props();
    if first_props.is_empty() {
        return &[];
    }
    let mut builders: Vec<SlotBuilder> = first_props
        .iter()
        .map(|prop| SlotBuilder {
            key: prop.key(),
            values: vec![*prop],
            stride: None,
        })
        .collect();
    for (offset, state) in block.states.iter().enumerate().skip(1) {
        let Some(props) = block.properties(state.id) else {
            return &[];
        };
        for prop in props.to_props() {
            let Some(builder) = builders
                .iter_mut()
                .find(|builder| builder.key == prop.key())
            else {
                return &[];
            };
            if builder.stride.is_none() && !builder.values[0].matches_property(prop) {
                builder.stride = Some(offset as u16);
            }
            if !builder
                .values
                .iter()
                .any(|value| value.matches_property(prop))
            {
                builder.values.push(prop);
            }
        }
    }
    let mut product = 1usize;
    for builder in &builders {
        product *= builder.values.len();
    }
    if product != state_count {
        return &[];
    }
    for builder in &mut builders {
        if builder.stride.is_none() {
            builder.stride = Some(state_count as u16);
        }
    }
    builders.sort_by(|a, b| b.stride.cmp(&a.stride));
    let slots: Vec<PropertySlot> = builders
        .into_iter()
        .map(|builder| PropertySlot {
            key: builder.key,
            stride: builder.stride.unwrap_or(1),
            values: Box::leak(builder.values.into_boxed_slice()),
        })
        .collect();
    Box::leak(slots.into_boxed_slice())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnFloorPredicate {
    Default,
    Never,
    Always,
    OcelotOrParrot,
    PolarBear,
    FireImmune,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuffocationPredicate {
    Default,
    Never,
    Always,
}

/// Helper struct to ensure the validity of BlockIds parsed from external sources.
/// Every [`BlockId`] is guaranteed to correspond to a valid [`Block`].
///
/// Also enables [`Block`]-type pattern matching, even in const contexts:
/// ```rs
/// const fn to_waxed(block: &'static Block) -> Option<&'static Block> {
///     match block.id {
///         BlockId::COPPER_BLOCK => Some(Block::WAXED_COPPER_BLOCK),
///         //...
///         _ => None
///     }
/// }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BlockId(u16);

impl PartialEq<BlockId> for Block {
    fn eq(&self, other: &BlockId) -> bool {
        self.id == *other
    }
}

impl PartialEq<Block> for BlockId {
    fn eq(&self, other: &Block) -> bool {
        *self == other.id
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Block {}

impl Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Taggable for Block {
    #[inline]
    fn tag_key() -> RegistryKey {
        RegistryKey::Block
    }

    #[inline]
    fn registry_key(&self) -> &str {
        self.name
    }

    #[inline]
    fn registry_id(&self) -> u16 {
        self.id.as_u16()
    }
}

impl ToResourceLocation for &'static Block {
    fn to_resource_location(&self) -> ResourceLocation {
        self.resource_location().into_owned()
    }
}

impl FromResourceLocation for &'static Block {
    fn from_resource_location(resource_location: &ResourceLocation) -> Option<Self> {
        Block::from_registry_key(
            resource_location
                .strip_prefix("minecraft:")
                .unwrap_or(resource_location),
        )
    }
}

impl Block {
    #[must_use]
    pub const fn get_speed_factor(&self) -> f32 {
        self.velocity_multiplier
    }

    #[must_use]
    pub const fn get_jump_velocity_multiplier(&self) -> f32 {
        self.jump_velocity_multiplier
    }

    pub(crate) fn shape_offset_delta(&self, pos: &BlockPos) -> Vector3<f64> {
        let Some(shape_offset) = self.shape_offset() else {
            return Vector3::new(0.0, 0.0, 0.0);
        };

        let seed = hash_block_pos(pos.0.x, 0, pos.0.z) as u64;
        let max_horizontal = f64::from(shape_offset.max_horizontal);
        let x = (f64::from((seed & 15) as f32 / 15.0) - 0.5) * 0.5;
        let x = x.clamp(-max_horizontal, max_horizontal);
        let z = (f64::from(((seed >> 8) & 15) as f32 / 15.0) - 0.5) * 0.5;
        let z = z.clamp(-max_horizontal, max_horizontal);
        let y = match shape_offset.offset_type {
            ShapeOffsetType::Xz => 0.0,
            ShapeOffsetType::Xyz => {
                (f64::from(((seed >> 4) & 15) as f32 / 15.0) - 1.0)
                    * f64::from(shape_offset.max_vertical)
            }
        };

        // Extracted shapes are sampled at BlockPos::ZERO, where vanilla uses the
        // negative horizontal limits and, for XYZ offsets, the negative vertical
        // limit. Return only the delta from that sample.
        Vector3::new(
            x + max_horizontal,
            y + shape_offset.offset_type.origin_y(shape_offset.max_vertical),
            z + max_horizontal,
        )
    }

    #[must_use]
    pub fn is_waterlogged(&self, id: BlockStateId) -> bool {
        id.is_waterlogged()
    }

    #[must_use]
    pub fn is_waterloggable(&self) -> bool {
        self.property_slot(PropertyKey::Waterlogged).is_some()
    }

    /// Returns a new [`BlockStateId`] for the given [`BlockStateId`] with the
    /// `waterlogged` property forced to `value`. If the state already has
    /// waterlogged set to `value` or the  block does not expose a `waterlogged`
    /// property then `None` is returned.
    #[must_use]
    pub fn set_waterlogged(&self, id: BlockStateId, value: bool) -> Option<BlockStateId> {
        if id.is_waterlogged() == value {
            return None;
        }
        self.with_property(id, Property::Waterlogged(value))
    }

    /// Property layout of this block in state-index order, empty when the generated
    /// states are not a full mixed radix of their properties.
    #[must_use]
    pub fn property_slots(&self) -> &'static [PropertySlot] {
        let cache = PROPERTY_SLOT_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
        {
            let guard = cache
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(slots) = guard.get(&self.id.as_u16()) {
                return slots;
            }
        }
        let slots = build_property_slots(self);
        let mut guard = cache
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard.entry(self.id.as_u16()).or_insert(slots)
    }

    #[must_use]
    pub fn property_slot(&self, key: PropertyKey) -> Option<&'static PropertySlot> {
        self.property_slots().iter().find(|slot| slot.key == key)
    }

    /// Offset of `id` in this block's state list, `None` for another block's state.
    fn state_offset(&self, id: BlockStateId) -> Option<u16> {
        let Some(offset) = id.as_u16().checked_sub(self.states[0].id.as_u16()) else {
            return None;
        };
        if (offset as usize) < self.states.len() {
            Some(offset)
        } else {
            None
        }
    }

    /// Value index of `key` in `id`; `None` if `id` is not a state of this block.
    #[must_use]
    pub fn property_index(&self, id: BlockStateId, key: PropertyKey) -> Option<u16> {
        let offset = self.state_offset(id)?;
        let slot = self.property_slot(key)?;
        Some(offset / slot.stride % slot.count())
    }

    /// `id` with `key` set to value `index`, by arithmetic.
    #[must_use]
    pub fn with_property_index(
        &self,
        id: BlockStateId,
        key: PropertyKey,
        index: u16,
    ) -> Option<BlockStateId> {
        let offset = self.state_offset(id)?;
        let slot = self.property_slot(key)?;
        if index >= slot.count() {
            return None;
        }
        let current = offset / slot.stride % slot.count();
        let offset = offset - current * slot.stride + index * slot.stride;
        if (offset as usize) >= self.states.len() {
            return None;
        }
        BlockStateId::new(self.states[0].id.as_u16() + offset)
    }

    #[must_use]
    pub fn get_property(&self, id: BlockStateId, key: PropertyKey) -> Option<Property> {
        let index = self.property_index(id, key)?;
        self.property_slot(key)
            .map(|slot| slot.values[index as usize])
    }

    /// `id` with `value` applied; `None` if the block has no such value.
    #[must_use]
    pub fn with_property(&self, id: BlockStateId, value: Property) -> Option<BlockStateId> {
        let slot = self.property_slot(value.key())?;
        self.with_property_index(id, slot.key, slot.index_of(value)?)
    }

    /// All property values of `id`, without allocating.
    pub fn property_values(&self, id: BlockStateId) -> impl Iterator<Item = Property> {
        let slots = self.property_slots();
        self.state_offset(id).into_iter().flat_map(move |offset| {
            slots
                .iter()
                .map(move |slot| slot.values[(offset / slot.stride % slot.count()) as usize])
        })
    }

    #[must_use]
    pub fn state_from_properties(
        &'static self,
        properties: &[crate::Property],
    ) -> Option<&'static BlockState> {
        self.states.iter().find(|state| {
            let Some(state_properties) = self.properties(state.id) else {
                return properties.is_empty();
            };
            let state_properties = state_properties.to_props();

            state_properties.len() == properties.len()
                && properties.iter().all(|prop| {
                    state_properties
                        .iter()
                        .any(|state_prop| state_prop.matches_property(*prop))
                })
        })
    }

    /// Parses a single `(key, value)` state property pair against this block's valid states.
    pub fn parse_property(
        &self,
        key: &str,
        value: &str,
    ) -> Result<crate::Property, PropertyLookupError> {
        let default_props = self
            .properties(self.default_state.id)
            .ok_or(PropertyLookupError::UnknownProperty)?
            .to_props();
        let idx = default_props
            .iter()
            .position(|p| p.key().to_str() == key)
            .ok_or(PropertyLookupError::UnknownProperty)?;
        self.states
            .iter()
            .find_map(|state| {
                let props = self.properties(state.id)?.to_props();
                let prop = *props.get(idx)?;
                (prop.value_str() == value).then_some(prop)
            })
            .ok_or(PropertyLookupError::InvalidValue)
    }

    /// Returns whether this block defines a state property with `key`.
    #[must_use]
    pub fn has_property(&self, key: &str) -> bool {
        self.properties(self.default_state.id)
            .is_some_and(|props| props.to_props().iter().any(|p| p.key().to_str() == key))
    }

    /// Copies each property in `overrides` onto `base_state_id` when it belongs to this block.
    #[must_use]
    pub fn apply_properties(
        &self,
        base_state_id: BlockStateId,
        overrides: &[crate::Property],
    ) -> BlockStateId {
        if overrides.is_empty() || Self::from_state_id(base_state_id).id != self.id {
            return base_state_id;
        }
        let Some(state_props) = self.properties(base_state_id) else {
            return base_state_id;
        };
        let mut props = state_props.to_props();
        for &defined in overrides {
            if let Some(slot) = props.iter_mut().find(|p| p.key() == defined.key()) {
                *slot = defined;
            }
        }
        self.from_properties(&props).to_state_id(self)
    }

    /// Returns whether this block is solid (based on default state)
    #[must_use]
    pub const fn is_solid(&self) -> bool {
        self.default_state.is_solid()
    }

    /// Returns whether this block is air (based on default state)
    #[must_use]
    pub const fn is_air(&self) -> bool {
        self.default_state.is_air()
    }

    #[must_use]
    pub fn mirror(
        &self,
        id: BlockStateId,
        mirror: crate::block_rotation::Mirror,
    ) -> &'static BlockState {
        if mirror == crate::block_rotation::Mirror::None || self.states.len() <= 1 {
            return BlockState::from_id(id);
        }
        if let Some(props) = self.properties(id) {
            let props_vec = props.to_props();
            let transformed = crate::block_rotation::transform_block_properties(
                self.name,
                &props_vec,
                crate::block_rotation::Rotation::None,
                mirror,
            );
            let new_props = self.from_properties(&transformed);
            let new_state_id = new_props.to_state_id(self);
            BlockState::from_id(new_state_id)
        } else {
            BlockState::from_id(id)
        }
    }

    #[must_use]
    pub fn rotate(
        &self,
        id: BlockStateId,
        rotation: crate::block_rotation::Rotation,
    ) -> &'static BlockState {
        if rotation == crate::block_rotation::Rotation::None || self.states.len() <= 1 {
            return BlockState::from_id(id);
        }
        if let Some(props) = self.properties(id) {
            let props_vec = props.to_props();
            let transformed = crate::block_rotation::transform_block_properties(
                self.name,
                &props_vec,
                rotation,
                crate::block_rotation::Mirror::None,
            );
            let new_props = self.from_properties(&transformed);
            let new_state_id = new_props.to_state_id(self);
            BlockState::from_id(new_state_id)
        } else {
            BlockState::from_id(id)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyLookupError {
    UnknownProperty,
    InvalidValue,
}

#[derive(Clone, Copy)]
pub(crate) enum ShapeOffsetType {
    Xz,
    Xyz,
}

#[derive(Clone, Copy)]
pub(crate) struct ShapeOffset {
    pub offset_type: ShapeOffsetType,
    pub max_horizontal: f32,
    pub max_vertical: f32,
}

impl ShapeOffsetType {
    fn origin_y(self, max_vertical: f32) -> f64 {
        match self {
            Self::Xz => 0.0,
            Self::Xyz => f64::from(max_vertical),
        }
    }
}

impl BlockId {
    // depends on generated impl:
    // pub(crate) const BLOCK_COUNT: u16;

    /// The count of the generated (vanilla) blocks. Plugin blocks' ids follow them.
    pub const COUNT: u16 = Self::BLOCK_COUNT;

    // SAFETY: A BlockId must be a generated block (< BLOCK_COUNT) or a registered dynamic one
    // (checked by `from_raw`); the generated lookups rely on it.

    /// A generated block id. Cannot see registered blocks, use [`Self::from_raw`] for external ids.
    #[inline]
    #[must_use]
    pub const fn new(inner: u16) -> Option<Self> {
        if inner < Self::BLOCK_COUNT {
            return Some(Self(inner));
        }
        None
    }

    /// A generated block id, air for anything else. See [`Self::new`].
    #[inline]
    #[must_use]
    pub const fn new_or_air(inner: u16) -> Self {
        if inner < Self::BLOCK_COUNT {
            return Self(inner);
        }
        Self::AIR
    }

    /// The id of a registered dynamic block. Only for ids the dynamic registry handed out.
    #[inline]
    #[must_use]
    pub(crate) const fn from_dynamic_raw(inner: u16) -> Self {
        Self(inner)
    }

    /// Number of generated and registered blocks. Ids are `0..total_count()`.
    #[inline]
    #[must_use]
    pub fn total_count() -> u32 {
        crate::block_registry::total_block_count()
    }

    /// A block id of a generated or registered block.
    #[inline]
    #[must_use]
    pub fn from_raw(inner: u16) -> Option<Self> {
        (u32::from(inner) < Self::total_count()).then_some(Self(inner))
    }

    /// Like [`Self::from_raw`], air for an id that does not exist.
    #[inline]
    #[must_use]
    pub fn from_raw_or_air(inner: u16) -> Self {
        Self::from_raw(inner).unwrap_or(Self::AIR)
    }

    #[inline]
    #[must_use]
    pub fn to_block(self) -> &'static Block {
        Block::from_id(self)
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn has_tag(self, tag: Tag) -> bool {
        tag.1.contains(&self.0)
            || crate::dynamic_tag::contains_id(crate::tag::RegistryKey::Block, tag.2, self.0)
                .unwrap_or(false)
    }
}

impl From<BlockId> for u16 {
    #[inline]
    fn from(value: BlockId) -> Self {
        value.as_u16()
    }
}

impl From<Block> for BlockId {
    #[inline]
    fn from(value: Block) -> Self {
        value.id
    }
}

impl From<&Block> for BlockId {
    #[inline]
    fn from(value: &Block) -> Self {
        value.id
    }
}

impl Default for BlockId {
    #[inline]
    fn default() -> Self {
        Self::AIR
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BlockId({} = \"{}\")",
            self.0,
            Block::from_id(*self).name
        )
    }
}

#[derive(Clone, Debug)]
pub struct Flammable {
    pub spread_chance: u8,
    pub burn_chance: u8,
}

#[cfg(test)]
mod tests {
    use super::{Block, BlockId, ShapeOffsetType};
    use pumpkin_util::math::position::BlockPos;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1.0e-6, "{actual} != {expected}");
    }

    #[test]
    fn shape_offset_registry_matches_vanilla_26_2() {
        let mut xz = 0;
        let mut xyz = 0;

        for raw_id in 0..BlockId::BLOCK_COUNT {
            match Block::from_id(BlockId::new(raw_id).unwrap())
                .shape_offset()
                .map(|offset| offset.offset_type)
            {
                Some(ShapeOffsetType::Xz) => xz += 1,
                Some(ShapeOffsetType::Xyz) => xyz += 1,
                None => {}
            }
        }

        assert_eq!(xz, 34);
        assert_eq!(xyz, 5);
    }

    #[test]
    fn shape_offset_limits_match_vanilla_26_2() {
        let bamboo = Block::BAMBOO.shape_offset().unwrap();
        assert_eq!(bamboo.max_horizontal, 0.25);
        assert_eq!(bamboo.max_vertical, 0.2);

        let pointed_dripstone = Block::POINTED_DRIPSTONE.shape_offset().unwrap();
        assert_eq!(pointed_dripstone.max_horizontal, 0.125);

        let small_dripleaf = Block::SMALL_DRIPLEAF.shape_offset().unwrap();
        assert_eq!(small_dripleaf.max_vertical, 0.1);
    }

    #[test]
    fn shape_offset_delta_matches_vanilla_coordinate_hash() {
        let origin = BlockPos::new(0, 64, 0);
        let positive_extreme = BlockPos::new(-18, 64, -7);

        let origin_delta = Block::BAMBOO.shape_offset_delta(&origin);
        assert_eq!(origin_delta.x, 0.0);
        assert_eq!(origin_delta.y, 0.0);
        assert_eq!(origin_delta.z, 0.0);

        let bamboo_delta = Block::BAMBOO.shape_offset_delta(&positive_extreme);
        assert_eq!(bamboo_delta.x, 0.5);
        assert_eq!(bamboo_delta.y, 0.0);
        assert_eq!(bamboo_delta.z, 0.5);

        let speleothem_delta = Block::POINTED_DRIPSTONE.shape_offset_delta(&positive_extreme);
        assert_eq!(speleothem_delta.x, 0.25);
        assert_eq!(speleothem_delta.y, 0.0);
        assert_eq!(speleothem_delta.z, 0.25);
        assert_eq!(
            Block::SULFUR_SPIKE.shape_offset_delta(&positive_extreme),
            speleothem_delta
        );

        let xyz_delta = Block::SHORT_GRASS.shape_offset_delta(&positive_extreme);
        assert_eq!(xyz_delta.x, 0.5);
        assert_close(xyz_delta.y, 0.08);
        assert_eq!(xyz_delta.z, 0.5);

        assert_eq!(Block::STONE.shape_offset_delta(&positive_extreme).x, 0.0);
    }

    #[test]
    fn property_slots_match_generated_state_layout() {
        for raw_id in 0..BlockId::COUNT {
            let block = Block::from_id(BlockId::new(raw_id).unwrap());
            let slots = block.property_slots();
            if slots.is_empty() {
                continue;
            }
            for state in block.states {
                let props = block.properties(state.id).unwrap().to_props();
                let values: Vec<_> = block.property_values(state.id).collect();
                assert_eq!(props.len(), values.len(), "{}", block.name);
                for prop in &props {
                    assert!(
                        values.iter().any(|value| value.matches_property(*prop)),
                        "{} {prop:?}",
                        block.name
                    );
                    assert_eq!(
                        block.with_property(state.id, *prop),
                        Some(state.id),
                        "{} {prop:?}",
                        block.name
                    );
                }
                for slot in slots {
                    let index = block.property_index(state.id, slot.key).unwrap();
                    let expected = props.iter().find(|prop| prop.key() == slot.key).unwrap();
                    assert!(
                        slot.values[index as usize].matches_property(*expected),
                        "{} {:?}",
                        block.name,
                        slot.key
                    );
                    assert_eq!(
                        block.with_property_index(state.id, slot.key, index),
                        Some(state.id),
                        "{} {:?}",
                        block.name,
                        slot.key
                    );
                }
            }
        }
    }
}
