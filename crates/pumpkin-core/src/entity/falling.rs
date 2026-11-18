use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::BlockId;
use pumpkin_data::BlockState;
use pumpkin_data::BlockStateId;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::CUpdateBlock;
use pumpkin_protocol::java::client::play::CBlockUpdate;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::resource_location::ToResourceLocation;
use pumpkin_world::generation::structure::template::{BlockStateResolver, PaletteEntry};
use pumpkin_world::world::BlockFlags;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32};
use std::sync::{Arc, Mutex, atomic::Ordering};

use crate::{
    block::{
        blocks::{anvil::AnvilBlock, falling::FallingBlock},
        entities::block_entity_from_nbt,
        registry::can_replace_with_other_block,
    },
    entity::{Entity, EntityBase, living::LivingEntity},
    plugin::api::events::entity::entity_change_block::EntityChangeBlockEvent,
    server::Server,
    world::World,
};

/// Vanilla `Entity.getAirDrag`.
const AIR_DRAG: f64 = 0.98;
/// Vanilla `FallingBlockEntity.tick`: drop after this many ticks while outside the build height.
const TIME_OUT_OF_WORLD: i32 = 100;
/// Vanilla `FallingBlockEntity.tick`: drop after this many ticks while still falling.
const TIME_MAX: i32 = 600;
/// Vanilla `FallingBlockEntity` hurt values for anvils and falling pointed dripstone.
const ANVIL_DAMAGE_PER_DISTANCE: f32 = 2.0;
const DRIPSTONE_DAMAGE_PER_DISTANCE: f32 = 2.0;
const FALLING_DAMAGE_MAX: f32 = 40.0;
/// Vanilla `FallingBlockEntity`: an anvil hit chance is `0.05 + steps * 0.05`.
const ANVIL_DAMAGE_BASE_CHANCE: f32 = 0.05;

fn state_to_nbt(state_id: BlockStateId) -> NbtCompound {
    let block = Block::from_state_id(state_id);
    let properties = block
        .properties(state_id)
        .map(|props| props.to_props())
        .unwrap_or_default();
    PaletteEntry::with_properties(block.to_resource_location(), properties).to_nbt_compound()
}

/// Vanilla falls back to sand when `BlockState` is missing or unknown.
fn state_from_nbt(nbt: &NbtCompound) -> BlockStateId {
    nbt.get_compound("BlockState")
        .and_then(|state| PaletteEntry::from_nbt_compound(state).ok())
        .and_then(|entry| BlockStateResolver::resolve_simple(&entry))
        .map_or(Block::SAND.default_state.id, |state| state.id)
}

pub struct FallingEntity {
    entity: Entity,
    block_state_id: AtomicCell<BlockStateId>,
    time: AtomicI32,
    /// Vanilla `FallingBlockEntity.dropItem`.
    drop_item: AtomicBool,
    /// Vanilla `FallingBlockEntity.cancelDrop`: land as nothing, neither block nor item.
    cancel_drop: AtomicBool,
    /// Vanilla `FallingBlockEntity.blockData`, NBT `TileEntityData`.
    block_data: Mutex<Option<NbtCompound>>,
    /// Accumulated fall distance in blocks, stored as `f32` bits.
    fall_distance: AtomicU32,
}

impl FallingEntity {
    pub const fn new(mut entity: Entity, block_state_id: BlockStateId) -> Self {
        entity.blocks_building = true;
        Self {
            entity,
            block_state_id: AtomicCell::new(block_state_id),
            time: AtomicI32::new(0),
            drop_item: AtomicBool::new(true),
            cancel_drop: AtomicBool::new(false),
            block_data: Mutex::new(None),
            fall_distance: AtomicU32::new(0),
        }
    }

    fn fall_distance(&self) -> f32 {
        f32::from_bits(self.fall_distance.load(Ordering::Relaxed))
    }

    fn add_fall_distance(&self, fallen: f32) {
        if fallen > 0.0 {
            let total = self.fall_distance() + fallen;
            self.fall_distance.store(total.to_bits(), Ordering::Relaxed);
        }
    }

    /// Vanilla `FallingBlockEntity.disableDrop`: land as nothing, neither block nor item.
    pub fn disable_drop(&self) {
        self.cancel_drop.store(true, Ordering::Relaxed);
    }

    /// Replaced the current Block and Spawns a new Falling one (synchronous)
    pub fn replace_spawn(
        world: &Arc<World>,
        position: BlockPos,
        block_state: BlockStateId,
    ) -> Option<Arc<Self>> {
        let block = Block::from_state_id(block_state);
        // Vanilla copies the state with `WATERLOGGED` cleared onto the entity.
        // `None` means the state has no water to clear.
        let falling_state = block
            .set_waterlogged(block_state, false)
            .unwrap_or(block_state);
        // Vanilla `FallingBlockEntity.fall`: leave `state.getFluidState().createLegacyBlock()`.
        let leftover = if block.is_waterlogged(block_state) {
            Block::WATER.default_state.id
        } else {
            Block::AIR.default_state.id
        };

        // `block_state` can differ from the world's (scaffolding passes its recomputed state), so the
        // write below compares against the cell as it was before the event.
        let observed = world.get_block_state_id(&position);
        let spawn_pos = position.0.to_f64().add_raw(0.5, 0.0, 0.5);
        let entity = Entity::new(world.clone(), spawn_pos, &EntityType::FALLING_BLOCK);
        if world.cancelled_by_plugin(|| {
            EntityChangeBlockEvent::new(
                entity.entity_id,
                position,
                Block::from_state_id(leftover).to_resource_location(),
                world.clone(),
            )
        }) {
            return None;
        }

        // An event handler may have run long enough for another task to change the source block.
        world.set_block_state_if(&position, leftover, BlockFlags::NOTIFY_ALL, |current| {
            current == observed
        })?;
        entity
            .data
            .store(i32::from(falling_state.as_u16()), Ordering::Relaxed);
        let falling = Arc::new(Self::new(entity, falling_state));
        world.spawn_entity_non_save(falling.clone());
        Some(falling)
    }

    /// Vanilla merges `TileEntityData` over the landed block's block entity.
    fn apply_block_data(&self, world: &Arc<World>, pos: BlockPos) {
        let block_data = self
            .block_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(data) = block_data.as_ref() else {
            return;
        };
        let Some(block_entity) = world.get_block_entity(&pos) else {
            return;
        };
        let mut merged = NbtCompound::new();
        block_entity.write_internal(&mut merged);
        for (key, value) in &data.child_tags {
            // Position and id stay those of the landed block entity.
            if !matches!(key.as_ref(), "x" | "y" | "z" | "id") {
                merged.child_tags.insert(key.clone(), value.clone());
            }
        }
        if let Some(reloaded) = block_entity_from_nbt(&merged) {
            world.add_block_entity(reloaded);
        }
    }

    // TODO: vanilla also calls `Fallable.onBrokenAfterFall` before the drop.
    fn drop_as_item(&self) {
        let world = self.entity.world.load();
        if self.drop_item.load(Ordering::Relaxed)
            && world.level_info.load().game_rules.entity_drops
            && let Some(item) =
                Item::from_id(Block::from_state_id(self.block_state_id.load()).item_id)
        {
            self.entity.spawn_at_location(ItemStack::new(1, item));
        }
    }
}

impl FallingEntity {
    /// Hurts entities in the landing box and rolls anvil wear. Returns `true` when the anvil breaks.
    fn damage_landing_entities(
        &self,
        caller: &dyn EntityBase,
        world: &World,
        block: &Block,
        state_id: &mut BlockStateId,
    ) -> bool {
        let entity = &self.entity;
        let Some((per_distance, max_damage, damage_type)) = landing_hurt(block) else {
            return false;
        };
        let fall_distance = self.fall_distance();
        let damage = falling_block_damage(fall_distance, per_distance, max_damage);
        if damage > 0.0 {
            for target in world.get_all_at_box(&entity.bounding_box.load()) {
                if target.get_entity().entity_id != entity.entity_id {
                    target.damage(caller, damage, damage_type);
                }
            }
        }
        if is_anvil(block)
            && damage > 0.0
            && rand::random::<f32>() < anvil_damage_chance(fall_distance)
        {
            match AnvilBlock::damage(*state_id) {
                Some(damaged) => *state_id = damaged,
                None => return true,
            }
        }
        false
    }
}

impl EntityBase for FallingEntity {
    fn can_hit(&self) -> bool {
        !self.get_entity().is_removed()
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;
        if self.block_state_id.load() == BlockStateId::AIR {
            entity.remove();
            return;
        }
        // `Time` comes from NBT; wraps like vanilla's `time++` instead of overflowing.
        let time = self.time.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        let mut velo = entity.velocity.load();
        velo.y -= self.get_gravity();

        entity.velocity.store(velo);

        let start_y = entity.pos.load().y;
        // TODO: vanilla treats concrete powder in water as still falling (`isStuckInWater`, plus a source-fluid clip above speed 1).
        entity.move_entity(caller, velo);
        self.add_fall_distance((start_y - entity.pos.load().y) as f32);
        entity.tick_block_collisions(caller);

        // Preserve collision and block-interaction velocity changes (for example,
        // cobweb slowdown) instead of overwriting them with the pre-move velocity.
        let velo = entity.velocity.load();
        if entity.on_ground.load(Ordering::Relaxed) {
            entity
                .velocity
                .store(entity.velocity.load().multiply(0.7, -0.5, 0.7));
            let world = entity.world.load();
            let landing_pos = self.entity.block_pos.load();
            let (current_block, current_state) = world.get_block_and_state(&landing_pos);
            // Vanilla waits until a piston has finished moving a block through the landing cell.
            if current_block != &Block::MOVING_PISTON {
                let mut state_id = self.block_state_id.load();
                let block = Block::from_state_id(state_id);

                if shatters_on_landing(block) {
                    world.sync_world_event(
                        WorldEvent::ParticlesDestroyBlock,
                        landing_pos,
                        i32::from(state_id.as_u16()),
                    );
                    if let Some(sound) = shatter_sound(block) {
                        world.play_sound(sound, SoundCategory::Blocks, &landing_pos.to_f64());
                    }
                    self.entity.remove();
                    return;
                }

                let anvil_destroyed =
                    self.damage_landing_entities(caller, &world, block, &mut state_id);
                let block = Block::from_state_id(state_id);

                let (below_block, below_state) = world.get_block_and_state(&landing_pos.down());
                let may_replace = can_replace_with_other_block(current_block, current_state);
                let would_continue_falling =
                    FallingBlock::can_fall_through(below_state, below_block);
                let would_survive = world.block_registry.can_place_at(
                    None,
                    Some(&**world),
                    &**world,
                    None,
                    block,
                    state_id.to_state(),
                    &landing_pos,
                    None,
                    None,
                ) && !would_continue_falling;
                if self.cancel_drop.load(Ordering::Relaxed) {
                    // Vanilla `cancelDrop`: the entity just disappears.
                    // TODO: vanilla also calls `Fallable.onBrokenAfterFall` (break particles and sound, anvil destroy sound).
                } else if !anvil_destroyed && may_replace && would_survive {
                    if let Some(concrete) = FallingBlock::concrete_of(block)
                        && FallingBlock::should_solidify(&**world, &landing_pos)
                    {
                        state_id = concrete.default_state.id;
                    }
                    let place_block = Block::from_state_id(state_id);
                    // TODO: vanilla waterlogs only for source water (`Fluids.WATER`), this tag also matches flowing water.
                    if place_block.is_waterloggable()
                        && world
                            .get_fluid(&landing_pos)
                            .has_tag(&tag::Fluid::MINECRAFT_WATER)
                        && let Some(waterlogged) = place_block.set_waterlogged(state_id, true)
                    {
                        state_id = waterlogged;
                    }
                    if world.cancelled_by_plugin(|| {
                        EntityChangeBlockEvent::new(
                            entity.entity_id,
                            landing_pos,
                            Block::from_state_id(state_id).to_resource_location(),
                            world.clone(),
                        )
                    }) || world
                        .set_block_state_if(&landing_pos, state_id, BlockFlags::NOTIFY_ALL, |now| {
                            now == current_state.id
                        })
                        .is_none()
                    {
                        // Vanilla drops the block when `setBlock` refuses; here also when the cell changed under the event.
                        self.drop_as_item();
                    } else {
                        self.apply_block_data(&world, landing_pos);
                        // block updates to watchers before the despawn, else a invisible block gap until the tick flush.
                        let placed = world.get_block_state_id(&landing_pos);
                        world.send_to_tracking_players_editioned(
                            entity,
                            &CBlockUpdate::new(landing_pos, i32::from(placed.as_u16()).into()),
                            &CUpdateBlock::new(landing_pos, BlockState::to_be_network_id(placed)),
                        );
                    }
                } else if !anvil_destroyed {
                    self.drop_as_item();
                }
                self.entity.remove();
            }
        } else {
            let pos = entity.block_pos.load();
            let world = entity.world.load();
            let out_of_world = pos.0.y <= world.get_bottom_y() || pos.0.y > world.get_top_y();
            if (time > TIME_OUT_OF_WORLD && out_of_world) || time > TIME_MAX {
                self.drop_as_item();
                self.entity.remove();
            }
        }

        entity.velocity.store(
            entity
                .velocity
                .load()
                .multiply(AIR_DRAG, AIR_DRAG, AIR_DRAG),
        );
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_compound("BlockState", state_to_nbt(self.block_state_id.load()));
        nbt.put_int("Time", self.time.load(Ordering::Relaxed));
        nbt.put_bool("DropItem", self.drop_item.load(Ordering::Relaxed));
        nbt.put_bool("CancelDrop", self.cancel_drop.load(Ordering::Relaxed));
        if let Some(data) = self
            .block_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            nbt.put_compound("TileEntityData", data.clone());
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        let state_id = state_from_nbt(nbt);
        self.block_state_id.store(state_id);
        self.entity
            .data
            .store(i32::from(state_id.as_u16()), Ordering::Relaxed);
        self.time
            .store(nbt.get_numeric_int("Time").unwrap_or(0), Ordering::Relaxed);
        self.drop_item
            .store(nbt.get_bool("DropItem").unwrap_or(true), Ordering::Relaxed);
        self.cancel_drop.store(
            nbt.get_bool("CancelDrop").unwrap_or(false),
            Ordering::Relaxed,
        );
        *self
            .block_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            nbt.get_compound("TileEntityData").cloned();
    }

    fn init_data_tracker(&self) {
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::falling_block::START_POS,
            self.entity.block_pos.load(),
        );
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
    fn damage(&self, _caller: &dyn EntityBase, _amount: f32, _damage_type: DamageType) -> bool {
        false
    }

    fn get_gravity(&self) -> f64 {
        0.04
    }

    fn entity_bedrock_spawn_metadata(
        &self,
    ) -> Option<pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList> {
        let mut meta =
            pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList::new();
        let runtime_id = BlockState::to_be_network_id(self.block_state_id.load());
        meta.set(
            pumpkin_protocol::bedrock::client::set_actor_data::entity_data_key::VARIANT,
            pumpkin_protocol::bedrock::client::set_actor_data::MetadataValue::Int(
                runtime_id as i32,
            ),
        );
        Some(meta)
    }

    fn bedrock_y_offset(&self) -> f64 {
        0.49
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn shatters_on_landing(block: &Block) -> bool {
    block.id == BlockId::SUSPICIOUS_SAND || block.id == BlockId::SUSPICIOUS_GRAVEL
}

fn is_anvil(block: &Block) -> bool {
    block == &Block::ANVIL || block == &Block::CHIPPED_ANVIL || block == &Block::DAMAGED_ANVIL
}

/// Landing hurt settings: `(damage per fall step, max damage, damage source)`.
fn landing_hurt(block: &Block) -> Option<(f32, f32, DamageType)> {
    if is_anvil(block) {
        Some((
            ANVIL_DAMAGE_PER_DISTANCE,
            FALLING_DAMAGE_MAX,
            DamageType::FALLING_ANVIL,
        ))
    } else if block == &Block::POINTED_DRIPSTONE {
        Some((
            DRIPSTONE_DAMAGE_PER_DISTANCE,
            FALLING_DAMAGE_MAX,
            DamageType::FALLING_STALACTITE,
        ))
    } else {
        None
    }
}

/// Vanilla `Mth.ceil(fallDistance - 1)`, or `None` when no impact happens.
fn fall_steps(fall_distance: f32) -> Option<f32> {
    let steps = (fall_distance - 1.0).ceil();
    (steps >= 0.0).then_some(steps)
}

fn falling_block_damage(fall_distance: f32, per_distance: f32, max_damage: f32) -> f32 {
    fall_steps(fall_distance).map_or(0.0, |steps| (steps * per_distance).floor().min(max_damage))
}

fn anvil_damage_chance(fall_distance: f32) -> f32 {
    fall_steps(fall_distance).map_or(0.0, |steps| {
        ANVIL_DAMAGE_BASE_CHANCE + steps * ANVIL_DAMAGE_BASE_CHANCE
    })
}

fn shatter_sound(block: &Block) -> Option<Sound> {
    if block.id == BlockId::SUSPICIOUS_SAND {
        Some(Sound::BlockSuspiciousSandBreak)
    } else if block.id == BlockId::SUSPICIOUS_GRAVEL {
        Some(Sound::BlockSuspiciousGravelBreak)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::block_properties::Facing;

    #[test]
    fn suspicious_blocks_shatter_with_their_sound() {
        assert!(shatters_on_landing(&Block::SUSPICIOUS_SAND));
        assert!(shatters_on_landing(&Block::SUSPICIOUS_GRAVEL));
        assert_eq!(
            shatter_sound(&Block::SUSPICIOUS_SAND),
            Some(Sound::BlockSuspiciousSandBreak)
        );
        assert_eq!(
            shatter_sound(&Block::SUSPICIOUS_GRAVEL),
            Some(Sound::BlockSuspiciousGravelBreak)
        );
    }

    #[test]
    fn falling_damage_follows_vanilla_steps_and_cap() {
        // No impact below one block of fall.
        assert_eq!(falling_block_damage(0.5, 2.0, 40.0), 0.0);
        assert_eq!(falling_block_damage(1.0, 2.0, 40.0), 0.0);
        // ceil(3 - 1) = 2 steps, 2 damage each.
        assert_eq!(falling_block_damage(3.0, 2.0, 40.0), 4.0);
        // ceil(2.5 - 1) = 2 steps.
        assert_eq!(falling_block_damage(2.5, 2.0, 40.0), 4.0);
        // Capped at the max.
        assert_eq!(falling_block_damage(100.0, 2.0, 40.0), 40.0);
    }

    #[test]
    fn anvil_damage_chance_grows_per_step() {
        assert!((anvil_damage_chance(0.5) - 0.05).abs() < 1e-6);
        assert!((anvil_damage_chance(1.0) - 0.05).abs() < 1e-6);
        assert!((anvil_damage_chance(3.0) - 0.15).abs() < 1e-6);
    }

    #[test]
    fn landing_hurt_covers_anvils_and_dripstone_only() {
        assert!(is_anvil(&Block::ANVIL));
        assert!(is_anvil(&Block::CHIPPED_ANVIL));
        assert!(is_anvil(&Block::DAMAGED_ANVIL));
        assert!(landing_hurt(&Block::SAND).is_none());
        assert_eq!(
            landing_hurt(&Block::ANVIL).map(|h| h.2),
            Some(DamageType::FALLING_ANVIL)
        );
        assert_eq!(
            landing_hurt(&Block::POINTED_DRIPSTONE).map(|h| h.2),
            Some(DamageType::FALLING_STALACTITE)
        );
    }

    #[test]
    fn ordinary_falling_blocks_place_themselves() {
        for block in [
            &Block::SAND,
            &Block::GRAVEL,
            &Block::RED_SAND,
            &Block::WHITE_CONCRETE_POWDER,
        ] {
            assert!(!shatters_on_landing(block));
            assert_eq!(shatter_sound(block), None);
        }
    }

    fn wrapped(state: BlockStateId) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_compound("BlockState", state_to_nbt(state));
        nbt
    }

    #[test]
    fn plain_state_round_trips() {
        let state = Block::GRAVEL.default_state.id;
        assert_eq!(state_from_nbt(&wrapped(state)), state);
    }

    #[test]
    fn state_with_properties_round_trips() {
        let state = Block::REPEATER
            .from_properties(&[pumpkin_data::Property::Facing(Facing::South)])
            .to_state_id(&Block::REPEATER);
        assert_eq!(state_from_nbt(&wrapped(state)), state);
    }

    #[test]
    fn missing_or_unknown_state_is_sand() {
        assert_eq!(
            state_from_nbt(&NbtCompound::new()),
            Block::SAND.default_state.id
        );

        let mut unknown = NbtCompound::new();
        let mut entry = NbtCompound::new();
        entry.put_string("id", "minecraft:not_a_block".to_string());
        unknown.put_compound("BlockState", entry);
        assert_eq!(state_from_nbt(&unknown), Block::SAND.default_state.id);
    }
}
