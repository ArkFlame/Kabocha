use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};

use pumpkin_data::Block;
use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_inventory::beacon_screen_handler::{
    DATA_LEVELS, DATA_PRIMARY, DATA_SECONDARY, NUM_DATA_VALUES, decode_effect, encode_effect,
    filter_effect,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::ChunkHeightmapType;

use crate::block::entities::{BlockEntity, PropertyDelegate};
use crate::entity::player::advancement::trigger::AdvancementTrigger;
use crate::world::World;

const MAX_LEVELS: i32 = 4;
const BLOCKS_CHECK_PER_TICK: i32 = 10;

/// One `BeaconBeamOwner.Section`: a run of beam blocks sharing the (possibly averaged) colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeamSection {
    color: u32,
    height: i32,
}

impl BeamSection {
    const fn new(color: u32) -> Self {
        Self { color, height: 1 }
    }

    const fn increase_height(&mut self) {
        self.height += 1;
    }

    #[must_use]
    pub const fn color(&self) -> u32 {
        self.color
    }

    #[must_use]
    pub const fn height(&self) -> i32 {
        self.height
    }
}

#[derive(Default)]
struct BeamState {
    /// Sections built by the scan in progress (`checkingBeamSections`).
    checking: Vec<BeamSection>,
    /// Last completed scan's sections (`beamSections`).
    sections: Vec<BeamSection>,
}

/// One scanned column block, as seen by `BeaconBlockEntity.tick`.
#[derive(Clone, Copy)]
enum BeamObservation {
    /// A `BeaconBeamBlock` with its diffuse colour.
    Beam(u32),
    /// A transparent block the beam passes through, growing the current section.
    Pass,
    /// An opaque block, bedrock excepted, that cuts the beam.
    Blocked,
}

/// `ARGB.average`: per-channel integer average of two packed ARGB colours.
const fn argb_average(lhs: u32, rhs: u32) -> u32 {
    let alpha = u32::midpoint(lhs >> 24, rhs >> 24);
    let red = u32::midpoint((lhs >> 16) & 0xFF, (rhs >> 16) & 0xFF);
    let green = u32::midpoint((lhs >> 8) & 0xFF, (rhs >> 8) & 0xFF);
    let blue = u32::midpoint(lhs & 0xFF, rhs & 0xFF);
    (alpha << 24) | (red << 16) | (green << 8) | blue
}

/// The `BeaconBeamBlock.getColor().getTextureDiffuseColor()` for a beam block.
fn beam_color(block: &Block) -> u32 {
    if block == &Block::BEACON {
        return DyeColor::White.texture_diffuse_color();
    }
    block
        .name
        .strip_suffix("_stained_glass")
        .or_else(|| block.name.strip_suffix("_stained_glass_pane"))
        .and_then(DyeColor::by_name)
        .unwrap_or(DyeColor::White)
        .texture_diffuse_color()
}

/// Applies one scanned block to `sections`, mirroring vanilla's per-block branch.
/// Returns `false` when the beam is cut and `sections` has been cleared.
fn append_beam_block(sections: &mut Vec<BeamSection>, observation: BeamObservation) -> bool {
    match observation {
        BeamObservation::Beam(color) => {
            if sections.len() <= 1 {
                sections.push(BeamSection::new(color));
            } else if let Some(last) = sections.last_mut() {
                if last.color == color {
                    last.increase_height();
                } else {
                    let averaged = argb_average(last.color, color);
                    sections.push(BeamSection::new(averaged));
                }
            }
            true
        }
        BeamObservation::Pass => {
            if sections.is_empty() {
                return false;
            }
            if let Some(last) = sections.last_mut() {
                last.increase_height();
            }
            true
        }
        BeamObservation::Blocked => {
            sections.clear();
            false
        }
    }
}

#[cfg(test)]
fn build_beam_sections(observations: &[BeamObservation]) -> Vec<BeamSection> {
    let mut sections = Vec::new();
    for observation in observations {
        if !append_beam_block(&mut sections, *observation) {
            break;
        }
    }
    sections
}

pub struct BeaconBlockEntity {
    pub position: BlockPos,
    /// Encoded via [`encode_effect`]: `0` = none, otherwise `effect id + 1`.
    primary_effect: AtomicI32,
    secondary_effect: AtomicI32,
    levels: AtomicI32,
    dirty: AtomicBool,
    /// Result of the last completed column scan: `true` when nothing opaque blocks the beam.
    beam_active: AtomicBool,
    /// Whether the scan in progress has no beam yet: set on reset, cleared once the scan
    /// passes the beacon itself, set again by an opaque block.
    checking_beam_blocked: AtomicBool,
    /// Beam colour/height sections tracked by the same scan as vanilla's
    /// `checkingBeamSections`/`beamSections`.
    beam: Mutex<BeamState>,
    last_check_y: AtomicI32,
    /// Set by `BlockEntity::set_world`; lets `set_property` play the select sound from the
    /// world handle instead of deferring it to the next tick.
    world: OnceLock<Weak<World>>,
    metadata: super::ContainerMetadata,
}

/// Blocks the beam passes through and takes its colour from (`BeaconBeamBlock` in vanilla).
fn is_beam_block(block: &Block) -> bool {
    block == &Block::BEACON
        || block.name.ends_with("_stained_glass")
        || block.name.ends_with("_stained_glass_pane")
}

impl BeaconBlockEntity {
    pub const ID: &'static str = "minecraft:beacon";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            primary_effect: AtomicI32::new(0),
            secondary_effect: AtomicI32::new(0),
            levels: AtomicI32::new(0),
            dirty: AtomicBool::new(false),
            beam_active: AtomicBool::new(false),
            checking_beam_blocked: AtomicBool::new(false),
            beam: Mutex::new(BeamState::default()),
            last_check_y: AtomicI32::new(position.0.y - 1),
            world: OnceLock::new(),
            metadata: super::ContainerMetadata::default(),
        }
    }

    #[must_use]
    pub fn levels(&self) -> i32 {
        self.levels.load(Ordering::Relaxed)
    }

    /// Mirrors `BeaconBeamOwner.getBeamSections`: empty until the beacon has levels.
    #[must_use]
    pub fn get_beam_sections(&self) -> Vec<BeamSection> {
        if self.levels.load(Ordering::Relaxed) == 0 {
            return Vec::new();
        }
        self.beam
            .lock()
            .map_or_else(|_| Vec::new(), |state| state.sections.clone())
    }

    fn beam_state(&self) -> MutexGuard<'_, BeamState> {
        self.beam
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[must_use]
    pub fn primary_effect(&self) -> Option<&'static StatusEffect> {
        decode_effect(self.primary_effect.load(Ordering::Relaxed))
    }

    #[must_use]
    pub fn secondary_effect(&self) -> Option<&'static StatusEffect> {
        decode_effect(self.secondary_effect.load(Ordering::Relaxed))
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn play_sound(world: &World, position: BlockPos, sound: Sound) {
        world.play_block_sound(sound, SoundCategory::Blocks, position);
    }

    /// Mirrors `BeaconBlockEntity.updateBase`: the number of complete pyramid layers below.
    fn update_base(&self, world: &World) -> i32 {
        let BlockPos(pos) = self.position;
        let mut levels = 0;
        for step in 1..=MAX_LEVELS {
            let layer_y = pos.y - step;
            if layer_y < world.min_y {
                break;
            }
            let layer_ok = (pos.x - step..=pos.x + step).all(|x| {
                (pos.z - step..=pos.z + step).all(|z| {
                    world
                        .get_block(&BlockPos::new(x, layer_y, z))
                        .has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEACON_BASE_BLOCKS)
                })
            });
            if !layer_ok {
                break;
            }
            levels = step;
        }
        levels
    }

    /// Mirrors `BeaconBlockEntity.applyEffects`.
    fn apply_effects(&self, world: &World, levels: i32) {
        let Some(primary) = self.primary_effect() else {
            return;
        };
        let secondary = self.secondary_effect();
        let same = secondary.is_some_and(|s| s.id == primary.id);

        let range = f64::from(levels * 10 + 10);
        let base_amp = u8::from(levels >= MAX_LEVELS && same);
        let duration = (9 + levels * 2) * 20;
        let pos = self.position.0.to_f64();
        let bounds = BoundingBox::new_array(
            [pos.x - range, pos.y - range, pos.z - range],
            [
                pos.x + 1.0 + range,
                pos.y + 1.0 + range + f64::from(world.dimension.height),
                pos.z + 1.0 + range,
            ],
        );
        let effect = |effect_type, amplifier| Effect {
            effect_type,
            duration,
            amplifier,
            ambient: true,
            show_particles: true,
            show_icon: true,
            blend: false,
        };

        for player in world.players.load().iter() {
            // Vanilla `applyEffects` does not filter spectators.
            if !bounds.intersects(&player.living_entity.entity.bounding_box.load()) {
                continue;
            }
            player.add_effect(effect(primary, base_amp));
            if levels >= MAX_LEVELS
                && !same
                && let Some(secondary) = secondary
            {
                player.add_effect(effect(secondary, 0));
            }
        }
    }

    /// Vanilla `BeaconBlockEntity.tick`: `new AABB(x, y, z, x, y - 4, z).inflate(10, 5, 10)`.
    fn construct_beacon_bounds(position: BlockPos) -> BoundingBox {
        let BlockPos(pos) = position;
        BoundingBox::new_array(
            [
                f64::from(pos.x) - 10.0,
                f64::from(pos.y) - 4.0 - 5.0,
                f64::from(pos.z) - 10.0,
            ],
            [
                f64::from(pos.x) + 10.0,
                f64::from(pos.y) + 5.0,
                f64::from(pos.z) + 10.0,
            ],
        )
    }

    /// Fires the `construct_beacon` criterion for players near a freshly activated beacon.
    fn trigger_construct_beacon(&self, world: &World, levels: i32) {
        let bounds = Self::construct_beacon_bounds(self.position);
        for player in world.players.load().iter() {
            if !bounds.intersects(&player.living_entity.entity.bounding_box.load()) {
                continue;
            }
            player.trigger_advancement(AdvancementTrigger::ConstructBeacon { level: levels });
        }
    }

    fn effect_from_nbt(nbt: &NbtCompound, key: &str) -> i32 {
        encode_effect(filter_effect(
            nbt.get_string(key)
                .and_then(StatusEffect::from_minecraft_name),
        ))
    }

    fn effect_to_nbt(nbt: &mut NbtCompound, key: &str, encoded: i32) {
        if let Some(effect) = decode_effect(encoded) {
            nbt.put_string(key, effect.minecraft_name.to_string());
        }
    }
}

impl PropertyDelegate for BeaconBlockEntity {
    fn get_property(&self, index: i32) -> i32 {
        match index {
            DATA_LEVELS => self.levels.load(Ordering::Relaxed),
            DATA_PRIMARY => self.primary_effect.load(Ordering::Relaxed),
            DATA_SECONDARY => self.secondary_effect.load(Ordering::Relaxed),
            _ => 0,
        }
    }

    fn set_property(&self, index: i32, value: i32) {
        match index {
            DATA_LEVELS => self.levels.store(value, Ordering::Relaxed),
            DATA_PRIMARY => {
                // Vanilla fires BEACON_POWER_SELECT from the dataAccess.set handler while the
                // beam is up, not on a later tick.
                if self.beam_active.load(Ordering::Relaxed)
                    && let Some(world) = self.world.get().and_then(std::sync::Weak::upgrade)
                {
                    Self::play_sound(&world, self.position, Sound::BlockBeaconPowerSelect);
                }
                self.primary_effect.store(
                    encode_effect(filter_effect(decode_effect(value))),
                    Ordering::Relaxed,
                );
            }
            DATA_SECONDARY => self.secondary_effect.store(
                encode_effect(filter_effect(decode_effect(value))),
                Ordering::Relaxed,
            ),
            _ => return,
        }
        self.mark_dirty();
    }

    fn get_properties_size(&self) -> i32 {
        NUM_DATA_VALUES
    }
}

impl BlockEntity for BeaconBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self {
        let mut entity = Self::new(position);
        entity.metadata = super::ContainerMetadata::from_nbt(nbt);
        entity.primary_effect.store(
            Self::effect_from_nbt(nbt, "primary_effect"),
            Ordering::Relaxed,
        );
        entity.secondary_effect.store(
            Self::effect_from_nbt(nbt, "secondary_effect"),
            Ordering::Relaxed,
        );
        // `Levels` is saved but, as in vanilla, not loaded: a reloaded beacon starts at 0 and
        // replays its activation on the first pulse.
        entity
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        Self::effect_to_nbt(
            nbt,
            "primary_effect",
            self.primary_effect.load(Ordering::Relaxed),
        );
        Self::effect_to_nbt(
            nbt,
            "secondary_effect",
            self.secondary_effect.load(Ordering::Relaxed),
        );
        nbt.put_int("Levels", self.levels.load(Ordering::Relaxed));
        self.metadata.write_nbt(nbt);
    }

    fn apply_item_components(&self, stack: &ItemStack) {
        self.metadata.apply_item_components(stack);
    }

    fn collect_item_components(&self, stack: &mut ItemStack) {
        self.metadata.collect_name_only(stack);
    }

    fn collect_pick_item_components(&self, stack: &mut ItemStack) {
        self.metadata.collect_item_components(stack);
    }

    fn get_lock(&self) -> Option<pumpkin_data::data_component_impl::LockImpl> {
        self.metadata.get_lock()
    }

    fn get_custom_name(&self) -> Option<pumpkin_util::text::TextComponent> {
        self.metadata.get_custom_name()
    }

    /// Mirrors `BeaconBlockEntity.tick`.
    fn tick(&self, world: &Arc<World>) {
        let BlockPos(pos) = self.position;
        let mut last_check_y = self.last_check_y.load(Ordering::Relaxed);
        if last_check_y < pos.y {
            self.checking_beam_blocked.store(true, Ordering::Relaxed);
            last_check_y = pos.y - 1;
            self.beam_state().checking.clear();
        }
        // `get_heightmap_height` is the top block; vanilla's `getHeight` is the block above it.
        let last_set_block =
            world.get_heightmap_height(ChunkHeightmapType::WorldSurface, pos.x, pos.z) + 1;

        let mut beam = self.beam_state();
        let mut checked = 0;
        while checked < BLOCKS_CHECK_PER_TICK && last_check_y < last_set_block {
            let check_pos = BlockPos::new(pos.x, last_check_y + 1, pos.z);
            let block = world.get_block(&check_pos);
            let observation = if is_beam_block(block) {
                BeamObservation::Beam(beam_color(block))
            } else {
                let opaque = world.get_block_state(&check_pos).opacity >= 15;
                // Anything opaque above the beacon (bedrock excepted) cuts the beam.
                if opaque && block != &Block::BEDROCK {
                    BeamObservation::Blocked
                } else {
                    BeamObservation::Pass
                }
            };
            if check_pos.0.y == pos.y && matches!(observation, BeamObservation::Beam(_)) {
                self.checking_beam_blocked.store(false, Ordering::Relaxed);
            }
            if !append_beam_block(&mut beam.checking, observation) {
                self.checking_beam_blocked.store(true, Ordering::Relaxed);
                last_check_y = last_set_block;
                break;
            }
            last_check_y += 1;
            checked += 1;
        }
        drop(beam);

        let previous_levels = self.levels.load(Ordering::Relaxed);
        let beam_active = self.beam_active.load(Ordering::Relaxed);
        if world.get_world_age() % 80 == 0 {
            // Vanilla recomputes the pyramid and applies effects only while the beam is
            // unblocked (empty beamSections means blocked); an indoor beacon grants no effects.
            if beam_active {
                let levels = self.update_base(world);
                if levels != previous_levels {
                    self.levels.store(levels, Ordering::Relaxed);
                    self.mark_dirty();
                }
            }
            let levels = self.levels.load(Ordering::Relaxed);
            if levels > 0 && beam_active {
                self.apply_effects(world, levels);
                Self::play_sound(world, self.position, Sound::BlockBeaconAmbient);
            }
        }

        if last_check_y >= last_set_block {
            last_check_y = world.min_y - 1;
            self.beam_active.store(
                !self.checking_beam_blocked.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
            // Vanilla commits `checkingBeamSections` to `beamSections` when a scan completes.
            {
                let mut beam = self.beam_state();
                let checking = std::mem::take(&mut beam.checking);
                beam.sections = checking
            };
            let was_active = previous_levels > 0;
            let levels = self.levels.load(Ordering::Relaxed);
            let is_active = levels > 0;
            if !was_active && is_active {
                Self::play_sound(world, self.position, Sound::BlockBeaconActivate);
                self.trigger_construct_beacon(world, levels);
            } else if was_active && !is_active {
                Self::play_sound(world, self.position, Sound::BlockBeaconDeactivate);
            }
        }
        self.last_check_y.store(last_check_y, Ordering::Relaxed);
    }

    fn on_block_replaced(self: Arc<Self>, world: &Arc<World>, position: &BlockPos) {
        // Vanilla `setRemoved` plays this regardless of whether the beacon was lit.
        Self::play_sound(world, *position, Sound::BlockBeaconDeactivate);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let name = self
            .metadata
            .get_custom_name()
            .map(pumpkin_util::text::TextComponent::get_text);
        Some(build_beacon_actor(
            self.position,
            self.primary_effect().map_or(0, StatusEffect::to_bedrock_id),
            self.secondary_effect()
                .map_or(0, StatusEffect::to_bedrock_id),
            name.as_deref(),
        ))
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn to_property_delegate(self: Arc<Self>) -> Option<Arc<dyn PropertyDelegate>> {
        Some(self)
    }

    fn set_world(&self, world: Weak<World>) {
        let _ = self.world.set(world);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Builds the Bedrock block-actor NBT for beacons.
#[must_use]
pub fn build_beacon_actor(
    position: BlockPos,
    primary: i32,
    secondary: i32,
    custom_name: Option<&str>,
) -> NbtCompound {
    super::bedrock::BedrockBlockActorBuilder::new("Beacon", position)
        .movable(false)
        .custom_name(custom_name)
        .put_int("primary", primary.max(0))
        .put_int("secondary", secondary.max(0))
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beacon_actor_builder() {
        let pos = BlockPos::new(5, 64, 5);
        let nbt = build_beacon_actor(pos, 1, 1, None);
        assert_eq!(nbt.get_string("id"), Some("Beacon"));
        assert_eq!(nbt.get_int("primary"), Some(1));
        assert_eq!(nbt.get_int("secondary"), Some(1));
        assert_eq!(nbt.get_bool("isMovable"), Some(false));
    }

    #[test]
    fn bedrock_actor_uses_bedrock_effect_ids() {
        let position = BlockPos::new(5, 64, 5);
        let mut nbt = NbtCompound::new();
        for (primary, secondary, expected_primary, expected_secondary) in [
            ("", "", 0, 0),
            ("minecraft:speed", "minecraft:regeneration", 1, 10),
        ] {
            nbt.put_string("primary_effect", primary.to_string());
            nbt.put_string("secondary_effect", secondary.to_string());
            nbt.put_string("CustomName", "Beacon".to_string());
            let beacon = BeaconBlockEntity::from_nbt(&nbt, position);
            let actor = beacon.bedrock_block_actor_data(Block::BEACON.default_state.id);
            assert_eq!(
                actor.as_ref().and_then(|actor| actor.get_int("primary")),
                Some(expected_primary)
            );
            assert_eq!(
                actor.as_ref().and_then(|actor| actor.get_int("secondary")),
                Some(expected_secondary)
            );
            assert_eq!(
                actor
                    .as_ref()
                    .and_then(|actor| actor.get_string("CustomName")),
                Some("Beacon")
            );
        }
    }

    #[test]
    fn bedrock_actor_reads_component_custom_name() {
        let position = BlockPos::new(5, 64, 5);
        let mut name = NbtCompound::new();
        name.put_string("text", "Beacon".to_string());
        let mut nbt = NbtCompound::new();
        nbt.put("CustomName", pumpkin_nbt::tag::NbtTag::Compound(name));
        let beacon = BeaconBlockEntity::from_nbt(&nbt, position);
        let actor = beacon.bedrock_block_actor_data(Block::BEACON.default_state.id);
        assert_eq!(
            actor
                .as_ref()
                .and_then(|actor| actor.get_string("CustomName")),
            Some("Beacon")
        );
    }

    #[test]
    fn nbt_keeps_effects_and_drops_levels_on_load() {
        let position = BlockPos::new(5, 64, 5);
        let mut nbt = NbtCompound::new();
        nbt.put_string("primary_effect", "minecraft:speed".to_string());
        nbt.put_string("secondary_effect", "minecraft:regeneration".to_string());
        nbt.put_int("Levels", 4);
        let beacon = BeaconBlockEntity::from_nbt(&nbt, position);
        assert_eq!(beacon.levels(), 0);
        assert_eq!(
            beacon.primary_effect().map(|e| e.id),
            Some(StatusEffect::SPEED.id)
        );

        let mut written = NbtCompound::new();
        beacon.write_nbt(&mut written);
        assert_eq!(
            written.get_string("primary_effect"),
            Some("minecraft:speed")
        );
        assert_eq!(
            written.get_string("secondary_effect"),
            Some("minecraft:regeneration")
        );
        assert_eq!(written.get_int("Levels"), Some(0));
    }

    #[test]
    fn construct_beacon_bounds_inflate_matches_vanilla() {
        let bounds = BeaconBlockEntity::construct_beacon_bounds(BlockPos::new(5, 64, 5));
        assert_eq!(bounds.min.x, -5.0);
        assert_eq!(bounds.min.y, 55.0);
        assert_eq!(bounds.min.z, -5.0);
        assert_eq!(bounds.max.x, 15.0);
        assert_eq!(bounds.max.y, 69.0);
        assert_eq!(bounds.max.z, 15.0);
    }

    #[test]
    fn beam_sections_merge_colour_and_height() {
        let sections = build_beam_sections(&[
            BeamObservation::Beam(0x00FFFFFF),
            BeamObservation::Beam(0x00FFFFFF),
            BeamObservation::Beam(0x00FFFFFF),
            BeamObservation::Pass,
            BeamObservation::Pass,
        ]);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].color(), 0x00FFFFFF);
        assert_eq!(sections[0].height(), 1);
        assert_eq!(sections[1].color(), 0x00FFFFFF);
        assert_eq!(sections[1].height(), 4);
    }

    #[test]
    fn beam_sections_average_on_colour_change() {
        let sections = build_beam_sections(&[
            BeamObservation::Beam(0x00FFFFFF),
            BeamObservation::Beam(0x00FFFFFF),
            BeamObservation::Beam(0x00000000),
            BeamObservation::Pass,
        ]);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].color(), 0x00FFFFFF);
        assert_eq!(sections[1].color(), 0x00FFFFFF);
        assert_eq!(sections[2].color(), 0x007F7F7F);
        assert_eq!(sections[2].height(), 2);
    }

    #[test]
    fn beam_sections_cut_on_opaque_block() {
        let sections = build_beam_sections(&[
            BeamObservation::Beam(0x00FFFFFF),
            BeamObservation::Beam(0x00FFFFFF),
            BeamObservation::Blocked,
        ]);
        assert!(sections.is_empty());
    }

    #[test]
    fn argb_average_halves_each_channel() {
        assert_eq!(argb_average(0x00FF0000, 0x000000FF), 0x007F007F);
    }

    #[test]
    fn beam_color_uses_dye_diffuse_color() {
        assert_eq!(
            beam_color(&Block::BEACON),
            DyeColor::White.texture_diffuse_color()
        );
        assert_eq!(
            beam_color(&Block::RED_STAINED_GLASS),
            DyeColor::Red.texture_diffuse_color()
        );
    }
}
