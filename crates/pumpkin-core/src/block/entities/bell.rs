use crate::block::entities::BlockEntity;
use crate::entity::EntityBase;
use crate::entity::mob::raider::is_raider;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::sync::{Arc, Mutex};

/// Vanilla `BellBlockEntity.RAIDERS_HEAR_BELL_RANGE`: raiders within 48 blocks resonate the bell.
const RAIDERS_HEAR_BELL_RANGE: f64 = 48.0;
/// Vanilla resonation lasts 40 ticks; the glowing tag stays for that long.
const RESONATE_TICKS: i32 = 40;

pub struct BellBlockEntity {
    pub position: BlockPos,
    pub last_side_hit: AtomicCell<Option<HorizontalFacing>>,
    pub ring_ticks: AtomicCell<i32>,
    pub ringing: AtomicCell<bool>,
    resonating: AtomicCell<bool>,
    resonate_time: AtomicCell<i32>,
    /// Raider entity ids which were given the glowing tag by the current resonation.
    glowing_raiders: Mutex<Vec<i32>>,
}

impl BellBlockEntity {
    pub const ID: &'static str = "minecraft:bell";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            last_side_hit: AtomicCell::new(None),
            ring_ticks: AtomicCell::new(0),
            resonate_time: AtomicCell::new(0),
            resonating: AtomicCell::new(false),
            ringing: AtomicCell::new(false),
            glowing_raiders: Mutex::new(Vec::new()),
        }
    }
    pub fn activate(&self, direction: HorizontalFacing) {
        self.last_side_hit.store(Some(direction));
        if self.ringing.load() {
            self.ring_ticks.store(0);
        } else {
            self.ringing.store(true);
        }
    }
    fn nearby_raiders(world: &Arc<World>, position: BlockPos) -> Vec<Arc<dyn EntityBase>> {
        let center = position.to_f64();
        let bounds = BoundingBox::new(center, center).expand_all(RAIDERS_HEAR_BELL_RANGE);
        world.get_entities_of_type_at_box(&bounds, is_raider)
    }
    fn raiders_hear_bell(&self, world: &Arc<World>) -> bool {
        let raiders = Self::nearby_raiders(world, self.position);
        if raiders.is_empty() {
            return false;
        }
        let mut glowing = self
            .glowing_raiders
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        glowing.clear();
        for raider in raiders {
            let entity = raider.get_entity();
            entity.set_glowing(true);
            glowing.push(entity.entity_id);
        }
        true
    }
    fn stop_raider_glow(&self, world: &Arc<World>) {
        let mut glowing = self
            .glowing_raiders
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for id in glowing.drain(..) {
            if let Some(entity) = world.get_entity_by_id(id) {
                entity.get_entity().set_glowing(false);
            }
        }
    }
}

impl BlockEntity for BellBlockEntity {
    fn write_nbt(&self, _nbt: &mut NbtCompound) {}

    fn from_nbt(_nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self::new(position)
    }

    fn tick(&self, world: &Arc<World>) {
        if self.ringing.load() {
            self.ring_ticks.fetch_add(1);
        }
        if self.ring_ticks.load() >= 50 {
            self.ringing.store(false);
            self.ring_ticks.store(0);
        }
        if self.ring_ticks.load() >= 5
            && self.resonate_time.load() == 0
            && self.raiders_hear_bell(world)
        {
            self.resonating.store(true);
            world.play_sound_fine(
                Sound::BlockBellResonate,
                SoundCategory::Blocks,
                &self.position.to_f64(),
                1.0,
                1.0,
            );
        }

        if self.resonating.load() {
            if self.resonate_time.load() < RESONATE_TICKS {
                self.resonate_time.fetch_add(1);
            } else {
                self.resonating.store(false);
                self.resonate_time.store(0);
                self.stop_raider_glow(world);
            }
        }
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let direction = match self.last_side_hit.load() {
            Some(HorizontalFacing::South) => 0,
            Some(HorizontalFacing::West) => 1,
            Some(HorizontalFacing::North) => 2,
            Some(HorizontalFacing::East) => 3,
            None => 255,
        };
        Some(
            super::bedrock::BedrockBlockActorBuilder::new("Bell", self.position)
                .movable(false)
                .put_bool("Ringing", self.ringing.load())
                .put_int("Ticks", self.ring_ticks.load())
                .put_int("Direction", direction)
                .finish(),
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
