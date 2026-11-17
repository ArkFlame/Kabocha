use pumpkin_data::data_component_impl::ContainerImpl;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomImpl, get_seed};
use std::any::Any;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{array::from_fn, sync::Arc};

use crate::block::entities::BlockEntity;
use crate::block::viewer::{ViewerCountListener, ViewerCountTracker, ViewerCountTrackerExt};
use crate::world::World;
use pumpkin_inventory::{Clearable, Inventory, sync_write_items_to_nbt};

pub struct ShulkerBoxBlockEntity {
    pub position: BlockPos,
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,
    pub comparator_dirty: AtomicBool,
    pub animation: std::sync::Mutex<AnimationState>,

    // Viewer
    pub viewers: ViewerCountTracker,
    pub metadata: super::ContainerMetadata,
}

impl BlockEntity for ShulkerBoxBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut shulker_box = Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            animation: std::sync::Mutex::new(AnimationState::default()),
            viewers: ViewerCountTracker::new(),
            metadata: super::ContainerMetadata::from_nbt(nbt),
        };

        pumpkin_inventory::sync_read_items_from_nbt(
            nbt,
            shulker_box
                .items
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );

        shulker_box
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_inventory_nbt(nbt, true);
        self.metadata.write_nbt(nbt);
    }

    fn tick(&self, world: &Arc<World>) {
        if let Ok(mut animation) = self.animation.lock() {
            animation.step();
        }
        self.viewers
            .update_viewer_count::<Self>(self, world, &self.position);
    }

    fn on_block_replaced(self: Arc<Self>, _world: &Arc<World>, _position: &BlockPos) {
        // Shulker boxes retain items when broken
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn collect_item_components(&self, stack: &mut ItemStack) {
        self.metadata.collect_item_components(stack);
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let contents: Vec<(u8, ItemStack)> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| !item.is_empty())
            .map(|(slot, item)| (slot as u8, item.clone()))
            .collect();

        if !contents.is_empty() {
            stack.set_data_component(ContainerImpl { items: contents });
        }
    }

    fn apply_item_components(&self, stack: &ItemStack) {
        self.metadata.apply_item_components(stack);
        if let Some(container) = stack.get_data_component::<ContainerImpl>() {
            let mut items = self
                .items
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            for (slot, item) in &container.items {
                if let Some(target) = items.get_mut(*slot as usize) {
                    *target = item.clone();
                }
            }
        }
        self.mark_dirty();
    }

    fn get_lock(&self) -> Option<pumpkin_data::data_component_impl::LockImpl> {
        self.metadata.get_lock()
    }

    fn get_custom_name(&self) -> Option<pumpkin_util::text::TextComponent> {
        self.metadata.get_custom_name()
    }

    fn drops_for_creative_player(&self) -> bool {
        !self.is_empty()
    }

    fn is_comparator_dirty(&self) -> bool {
        self.comparator_dirty.load(Ordering::Relaxed)
    }

    fn clear_comparator_dirty(&self) {
        self.comparator_dirty.store(false, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(items) = self.items.try_read() {
            sync_write_items_to_nbt(items.as_slice(), &mut nbt);
        }
        Some(nbt)
    }

    fn client_update_tag(&self, _saved: NbtCompound) -> NbtCompound {
        NbtCompound::new()
    }

    fn bedrock_block_actor_data(
        &self,
        state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let facing =
            match pumpkin_data::block_properties::ShulkerBoxProperties::from_state_id(state_id)
                .facing
            {
                pumpkin_data::block_properties::Facing::Down => 0,
                pumpkin_data::block_properties::Facing::Up => 1,
                pumpkin_data::block_properties::Facing::North => 2,
                pumpkin_data::block_properties::Facing::South => 3,
                pumpkin_data::block_properties::Facing::West => 4,
                pumpkin_data::block_properties::Facing::East => 5,
            };
        Some(build_shulker_box_actor(self.position, facing, None))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ViewerCountListener for ShulkerBoxBlockEntity {
    fn on_container_open(&self, world: &Arc<World>, position: &BlockPos) {
        Self::play_sound(world, position, 1);
        world.emit_game_event(
            pumpkin_data::game_event::GameEvent::ContainerOpen.name(),
            position.to_centered_f64(),
        );
    }

    fn on_container_close(&self, world: &Arc<World>, position: &BlockPos) {
        Self::play_sound(world, position, 0);
        world.emit_game_event(
            pumpkin_data::game_event::GameEvent::ContainerClose.name(),
            position.to_centered_f64(),
        );
    }

    fn on_viewer_count_update(&self, world: &Arc<World>, position: &BlockPos, _old: u16, new: u16) {
        world.add_synced_block_event(*position, Self::OPEN_ANIMATION_EVENT_TYPE, new as u8);
    }
}

/// Lid animation state, mirroring vanilla `ShulkerBoxBlockEntity.AnimationStatus`
/// with the progress kept as a fraction of the fully open lid.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnimationState {
    pub status: AnimationStatus,
    pub progress: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AnimationStatus {
    #[default]
    Closed,
    Opening,
    Opened,
    Closing,
}

impl AnimationState {
    /// Progress added or removed per tick (vanilla `updateAnimation`).
    pub const STEP: f32 = 0.1;

    /// Applies the open-count block event (`data` = viewer count). Returns false when
    /// the event is dropped: vanilla ignores an OPENING event while the lid is closing.
    pub fn on_open_count(&mut self, open_count: u8) -> bool {
        if open_count == 0 {
            self.status = AnimationStatus::Closing;
            return true;
        }
        if self.status == AnimationStatus::Closing {
            return false;
        }
        self.status = AnimationStatus::Opening;
        true
    }

    /// Advances the lid by one tick.
    pub fn step(&mut self) {
        match self.status {
            AnimationStatus::Opening => {
                self.progress = (self.progress + Self::STEP).min(1.0);
                if self.progress >= 1.0 {
                    self.status = AnimationStatus::Opened;
                }
            }
            AnimationStatus::Closing => {
                self.progress = (self.progress - Self::STEP).max(0.0);
                if self.progress <= 0.0 {
                    self.status = AnimationStatus::Closed;
                }
            }
            AnimationStatus::Opened | AnimationStatus::Closed => {}
        }
    }

    /// True once the lid is fully open (vanilla `getAnimationProgress(1.0F) >= 1.0F`).
    #[must_use]
    pub fn is_fully_open(&self) -> bool {
        self.progress >= 1.0
    }
}

impl ShulkerBoxBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const OPEN_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:shulker_box";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            animation: std::sync::Mutex::new(AnimationState::default()),
            viewers: ViewerCountTracker::new(),
            metadata: super::ContainerMetadata::default(),
        }
    }

    fn play_sound(world: &World, position: &BlockPos, viewer_count: i32) {
        let sound = if viewer_count > 0 {
            Sound::BlockShulkerBoxOpen
        } else {
            Sound::BlockShulkerBoxClose
        };

        // Vanilla `ShulkerBoxBlockEntity.startOpen`/`stopOpen`: volume 0.5 and a
        // random pitch in [0.9, 1.0) at the block centre.
        let mut rng = Xoroshiro::from_seed(get_seed());
        world.play_sound_fine(
            sound,
            SoundCategory::Blocks,
            &position.to_centered_f64(),
            0.5,
            rng.next_f32() * 0.1 + 0.9,
        );
    }
}

const SHULKER_SLOTS: [usize; ShulkerBoxBlockEntity::INVENTORY_SIZE] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26,
];

impl Inventory for ShulkerBoxBlockEntity {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> bool {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.iter().all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items[slot].clone()
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let removed = std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone());
        self.mark_dirty();
        removed
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let res = if !items[slot].is_empty() && amount > 0 {
            items[slot].split(amount)
        } else {
            ItemStack::EMPTY.clone()
        };
        self.mark_dirty();
        res
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items[slot] = stack;
        self.mark_dirty();
    }

    fn is_valid_slot_for(&self, _slot: usize, stack: &ItemStack) -> bool {
        !stack
            .item
            .has_tag(&pumpkin_data::tag::Item::MINECRAFT_SHULKER_BOXES)
    }

    fn is_sided(&self) -> bool {
        true
    }

    fn get_slots_for_face(&self, _direction: pumpkin_data::BlockDirection) -> &'static [usize] {
        &SHULKER_SLOTS
    }

    fn on_open(&self) {
        self.viewers.open_container();
    }

    fn on_close(&self) {
        self.viewers.close_container();
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
        self.comparator_dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for ShulkerBoxBlockEntity {
    fn clear(&self) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.fill_with(|| ItemStack::EMPTY.clone());
        self.mark_dirty();
    }
}

/// Builds the Bedrock block-actor NBT for shulker boxes.
#[must_use]
pub fn build_shulker_box_actor(
    position: BlockPos,
    facing: u8,
    custom_name: Option<&str>,
) -> NbtCompound {
    super::bedrock::BedrockBlockActorBuilder::new("ShulkerBox", position)
        .movable(false)
        .custom_name(custom_name)
        .put_byte("facing", facing as i8)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_event_is_dropped_while_closing() {
        let mut state = AnimationState::default();
        assert!(state.on_open_count(1));
        assert_eq!(state.status, AnimationStatus::Opening);
        assert!(state.on_open_count(0));
        assert!(!state.on_open_count(1));
        assert_eq!(state.status, AnimationStatus::Closing);
    }

    #[test]
    fn lid_reaches_fully_open_after_ten_ticks() {
        let mut state = AnimationState::default();
        state.on_open_count(1);
        for _ in 0..9 {
            state.step();
        }
        assert!(!state.is_fully_open());
        state.step();
        assert!(state.is_fully_open());
        assert_eq!(state.status, AnimationStatus::Opened);
    }

    #[test]
    fn shulker_box_actor_builder() {
        let pos = BlockPos::new(4, 64, 4);
        let nbt = build_shulker_box_actor(pos, 1, Some("Box"));
        assert_eq!(nbt.get_string("id"), Some("ShulkerBox"));
        assert_eq!(nbt.get_byte("facing"), Some(1));
        assert_eq!(nbt.get_bool("isMovable"), Some(false));
        assert_eq!(nbt.get_string("CustomName"), Some("Box"));
    }
}
