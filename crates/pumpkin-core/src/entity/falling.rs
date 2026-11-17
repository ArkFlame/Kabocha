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
use pumpkin_protocol::bedrock::client::CUpdateBlock;
use pumpkin_protocol::java::client::play::CBlockUpdate;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::{
    Arc,
    atomic::{AtomicU16, AtomicU32, Ordering},
};

use crate::{
    block::{
        blocks::{anvil::AnvilBlock, falling::FallingBlock},
        registry::can_replace_with_other_block,
    },
    entity::{Entity, EntityBase, living::LivingEntity},
    server::Server,
    world::World,
};

/// Vanilla `FallingBlockEntity` hurt values for anvils and falling pointed dripstone.
const ANVIL_DAMAGE_PER_DISTANCE: f32 = 2.0;
const DRIPSTONE_DAMAGE_PER_DISTANCE: f32 = 2.0;
const FALLING_DAMAGE_MAX: f32 = 40.0;
/// Vanilla `FallingBlockEntity`: an anvil hit chance is `0.05 + steps * 0.05`.
const ANVIL_DAMAGE_BASE_CHANCE: f32 = 0.05;

pub struct FallingEntity {
    entity: Entity,
    block_state_id: AtomicU16,
    /// Accumulated fall distance in blocks, stored as `f32` bits.
    fall_distance: AtomicU32,
}

impl FallingEntity {
    pub const fn new(mut entity: Entity, block_state_id: BlockStateId) -> Self {
        entity.blocks_building = true;
        Self {
            entity,
            block_state_id: AtomicU16::new(block_state_id.as_u16()),
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

    /// Sets the block state encoded in this entity's spawn packet.
    ///
    /// This must be called before the entity is spawned so clients render the
    /// requested block state when they receive its spawn packet.
    pub fn set_block_state(&self, block_state_id: BlockStateId) {
        self.block_state_id
            .store(block_state_id.as_u16(), Ordering::Relaxed);
        self.entity
            .data
            .store(i32::from(block_state_id.as_u16()), Ordering::Relaxed);
    }

    /// Replaces the current block with its fluid and spawns a falling one (synchronous)
    pub fn replace_spawn(world: &Arc<World>, position: BlockPos, block_state: BlockStateId) {
        let replacement = if block_state.is_waterlogged() {
            Block::WATER.default_state.id
        } else {
            Block::AIR.default_state.id
        };
        world.set_block_state(&position, replacement, BlockFlags::NOTIFY_ALL);
        let block_state = block_state
            .to_state()
            .set_waterlogged(false)
            .map_or(block_state, |state| state.id);

        let position = position.0.to_f64().add_raw(0.5, 0.0, 0.5);
        let entity = Entity::new(world.clone(), position, &EntityType::FALLING_BLOCK);
        let entity = Arc::new(Self::new(entity, block_state));
        entity.set_block_state(block_state);
        world.spawn_entity_non_save(entity);
    }
}

impl EntityBase for FallingEntity {
    fn can_hit(&self) -> bool {
        !self.get_entity().is_removed()
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;
        let mut velo = entity.velocity.load();
        velo.y -= self.get_gravity();

        entity.velocity.store(velo);

        let start_y = entity.pos.load().y;
        entity.move_entity(caller, velo);
        self.add_fall_distance((start_y - entity.pos.load().y) as f32);
        entity.tick_block_collisions(caller);

        // Preserve collision and block-interaction velocity changes (for example,
        // cobweb slowdown) instead of overwriting them with the pre-move velocity.
        let velo = entity.velocity.load();
        if entity.on_ground.load(Ordering::Relaxed) {
            entity.velocity.store(velo.multiply(0.7, -0.5, 0.7));
            let world = entity.world.load();
            let landing_pos = self.entity.block_pos.load();
            let (current_block, current_state) = world.get_block_and_state(&landing_pos);
            // Vanilla waits until a piston has finished moving a block through the landing cell.
            if current_block != &Block::MOVING_PISTON {
                let mut state_id =
                    BlockStateId::new_or_air(self.block_state_id.load(Ordering::Relaxed));
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

                let mut anvil_destroyed = false;
                if let Some((per_distance, max_damage, damage_type)) = landing_hurt(block) {
                    let fall_distance = self.fall_distance();
                    let damage = falling_block_damage(fall_distance, per_distance, max_damage);
                    if damage > 0.0 {
                        for target in world.get_all_at_box(&entity.bounding_box.load()) {
                            if target.get_entity().entity_id != entity.entity_id {
                                target.damage(caller, damage, damage_type);
                            }
                        }
                    }
                    if is_anvil(block) && rand::random::<f32>() < anvil_damage_chance(fall_distance)
                    {
                        match AnvilBlock::damage(state_id) {
                            Some(damaged) => state_id = damaged,
                            None => anvil_destroyed = true,
                        }
                    }
                }
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
                if !anvil_destroyed && may_replace && would_survive {
                    if block.has_tag(&tag::Block::MINECRAFT_CONCRETE_POWDERS)
                        && FallingBlock::should_solidify(&**world, &landing_pos)
                        && let Some(name) = block.name.strip_suffix("_powder")
                        && let Some(concrete) = Block::from_name(name)
                    {
                        state_id = concrete.default_state.id;
                    }
                    if world.is_water_source(&landing_pos)
                        && let Some(waterlogged) = state_id.to_state().set_waterlogged(true)
                    {
                        state_id = waterlogged.id;
                    }
                    let block = Block::from_state_id(state_id);
                    let mut event = crate::plugin::api::events::entity::entity_change_block::EntityChangeBlockEvent::new(
                        entity.entity_id,
                        landing_pos,
                        format!("minecraft:{}", block.name),
                        (*world).clone(),
                    );
                    if let Some(server) = world.server.upgrade() {
                        server.plugin_manager.fire_blocking(&server, &mut event);
                    }
                    if !event.cancelled {
                        world.set_block_state(&landing_pos, state_id, BlockFlags::NOTIFY_ALL);
                        // Block updates must reach watchers before the entity despawns, or an invisible
                        // gap persists until the next tick flush.
                        let placed = world.get_block_state_id(&landing_pos);
                        world.send_to_tracking_players_editioned(
                            entity,
                            &CBlockUpdate::new(landing_pos, i32::from(placed.as_u16()).into()),
                            &CUpdateBlock::new(landing_pos, BlockState::to_be_network_id(placed)),
                        );
                    }
                } else if !anvil_destroyed
                    && world.level_info.load().game_rules.entity_drops
                    && let Some(item) = Item::from_id(block.item_id)
                {
                    entity.spawn_at_location(ItemStack::new(1, item));
                }
                self.entity.remove();
            }
        }

        entity.velocity.store(velo.multiply(0.98, 0.98, 0.98));
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
        let runtime_id = BlockState::to_be_network_id(BlockStateId::new_or_air(
            self.block_state_id.load(Ordering::Relaxed),
        ));
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
        assert_eq!(anvil_damage_chance(0.5), 0.0);
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
}
