use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId, Enchantment,
    block_properties::CampfireLikeProperties,
    damage::DamageType,
    data_component_impl::EquipmentSlot,
    effect::StatusEffect,
    fluid::Fluid,
    game_event::GameEvent,
    recipes::{CookingRecipeKind, get_cooking_recipe_with_ingredient},
    tag::{self, Taggable},
    world::WorldEvent,
};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use pumpkin_world::tick::TickPriority;

use crate::block::entities::campfire::CampfireBlockEntity;
use crate::entity::projectile::may_interact;
use crate::plugin::api::events::block::campfire_start::CampfireStartEvent;
use crate::world::World;
use crate::{
    block::{
        BlockBehaviour, BlockIsReplacing, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
        OnPlaceArgs, OnProjectileHitArgs, PathComputationType, PlacedArgs, UseWithItemArgs,
        registry::BlockActionResult,
    },
    entity::EntityBase,
};
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:campfires")]
pub struct CampfireBlock;

impl CampfireBlock {
    const SMOKE_DISTANCE: i32 = 5;

    #[must_use]
    pub fn is_lit_campfire(block: &Block, state_id: BlockStateId) -> bool {
        block.has_tag(&tag::Block::MINECRAFT_CAMPFIRES)
            && CampfireLikeProperties::from_state_id(state_id).lit
    }

    /// Vanilla `CampfireBlock.isSmokeyPos`.
    #[must_use]
    pub fn is_smokey_pos(world: &World, pos: &BlockPos) -> bool {
        let virtual_post = BoundingBox::new(
            Vector3::new(0.375, 0.0, 0.375),
            Vector3::new(0.625, 1.0, 0.625),
        );
        Self::is_smokey_pos_from(|dy| {
            let check_pos = BlockPos::new(pos.0.x, pos.0.y - dy, pos.0.z);
            let (block, state) = world.get_block_and_state(&check_pos);
            let blocks_smoke = state
                .get_block_collision_shapes_at(&check_pos)
                .any(|shape| shape.intersects(&virtual_post));
            (Self::is_lit_campfire(block, state.id), blocks_smoke)
        })
    }

    /// Pure core of `is_smokey_pos`. `state_at(dy)` reports whether the cell
    /// `dy` blocks below the smoke origin holds a lit campfire and whether its
    /// collision shape crosses the 0.375..0.625 smoke column.
    fn is_smokey_pos_from(mut state_at: impl FnMut(i32) -> (bool, bool)) -> bool {
        for dy in 1..=Self::SMOKE_DISTANCE {
            let (is_lit, blocks_smoke) = state_at(dy);
            if is_lit {
                return true;
            }
            if blocks_smoke {
                // Vanilla checks the one cell directly below the blocker and then
                // stops; it does not assume the space below the blocker is empty.
                return state_at(dy + 1).0;
            }
        }
        false
    }
}

impl BlockBehaviour for CampfireBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = CampfireBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(campfire) = block_entity.as_any().downcast_ref::<CampfireBlockEntity>()
            && let Some(recipe) = get_cooking_recipe_with_ingredient(
                args.item_stack.item,
                CookingRecipeKind::CampfireCooking,
            )
        {
            let slot = campfire.items.iter().position(|item| {
                item.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .is_empty()
            });
            let Some(slot) = slot else {
                return BlockActionResult::Consume;
            };

            // Vanilla/Paper fires CampfireStartEvent in placeFood at insertion so
            // plugins can adjust the cook time before the first tick.
            let mut cooking_time = recipe.cookingtime;
            let mut cancelled = false;
            if let Some(server) = args.world.server.upgrade() {
                let mut event = CampfireStartEvent::new(
                    *args.position,
                    args.world.clone(),
                    args.item_stack.clone(),
                    slot as u8,
                    cooking_time,
                );
                server.plugin_manager.fire_blocking(&server, &mut event);
                cooking_time = event.cooking_time;
                cancelled = event.cancelled;
            }

            let mut stored = campfire.items[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *stored = args
                .item_stack
                .split_unless_creative(args.player.gamemode.load(), 1);
            *campfire.cooking_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
            *campfire.cooking_total_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = cooking_time;
            *campfire.cooking_cancelled[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = cancelled;
            drop(stored);

            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::InteractWithCampfire as i32,
                1,
            );
            args.world.emit_game_event(
                GameEvent::BlockChange.name(),
                args.position.to_centered_f64(),
            );
            args.world.update_block_entity(&block_entity);
            return BlockActionResult::SuccessServer;
        }

        let state = args.world.get_block_state(args.position);
        let mut props = CampfireLikeProperties::from_state_id(state.id);
        if args
            .item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_DOUSES_CAMPFIRES)
            && props.lit
        {
            args.world
                .sync_world_event(WorldEvent::SoundExtinguishFire, *args.position, 0);
            args.world.emit_game_event(
                GameEvent::BlockChange.name(),
                args.position.to_centered_f64(),
            );
            props.lit = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            args.player.damage_item_in_slot(args.equipment_slot, 1);
            return BlockActionResult::Success;
        }

        BlockActionResult::PassToDefaultBlockAction
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        if CampfireLikeProperties::from_state_id(args.state.id).lit
            && let Some(living_entity) = args.entity.get_living_entity()
        {
            let has_frost_walker_enchantment = {
                let equipment = living_entity
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                equipment
                    .equipment
                    .get(&EquipmentSlot::FEET)
                    .is_some_and(|boots| {
                        boots.get_enchantment_level(&Enchantment::FROST_WALKER) != 0
                    })
            };
            let has_fire_res = living_entity
                .get_effect(&StatusEffect::FIRE_RESISTANCE)
                .is_some();
            if has_frost_walker_enchantment || has_fire_res {
                // Campfire damage is prevented by Frost Walker boots or fire resistance.
                return;
            }
            let damage_amount = if args.block == &Block::SOUL_CAMPFIRE {
                2.0
            } else {
                1.0
            };
            args.entity
                .damage(args.entity, damage_amount, DamageType::CAMPFIRE);
        }
    }

    fn on_projectile_hit(&self, args: OnProjectileHitArgs<'_>) {
        let props = CampfireLikeProperties::from_state_id(args.state.id);
        if !args.projectile.get_entity().is_on_fire()
            || !may_interact(args.projectile, args.world, args.position)
            || props.lit
            || props.waterlogged
        {
            return;
        }

        let mut lit_props = props;
        lit_props.lit = true;
        args.world.set_block_state(
            args.position,
            lit_props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL,
        );
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let is_replacing_water = matches!(args.replacing, BlockIsReplacing::Water(_));
        let mut props = CampfireLikeProperties::from_state_id(args.block.default_state.id);
        props.waterlogged = is_replacing_water;
        props.signal_fire = is_signal_fire_base_block(args.world.get_block(&args.position.down()));
        props.lit = !is_replacing_water;
        props.facing = args.player.get_entity().get_horizontal_facing();
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let mut props = CampfireLikeProperties::from_state_id(args.state_id);
        // Vanilla `CampfireBlock.getStateForNeighborUpdate` only schedules the water tick here.
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed,
                TickPriority::Normal,
            );
        }

        if args.direction == BlockDirection::Down {
            props.signal_fire =
                is_signal_fire_base_block(args.world.get_block(args.neighbor_position));
        }

        props.to_state_id(args.block)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn is_signal_fire_base_block(block: &Block) -> bool {
    block == &Block::HAY_BLOCK
}

#[cfg(test)]
mod tests {
    use super::CampfireBlock;

    #[test]
    fn smoke_scans_five_cells_and_stops_at_a_blocker() {
        // A lit campfire within range smokes.
        assert!(CampfireBlock::is_smokey_pos_from(|dy| (dy == 2, false)));
        // A lit campfire itself blocks the column but still smokes.
        assert!(CampfireBlock::is_smokey_pos_from(|dy| (dy == 1, dy == 1)));
        // A blocker with a lit campfire directly below it still smokes.
        assert!(CampfireBlock::is_smokey_pos_from(|dy| (dy == 2, dy == 1)));
        // A blocker with nothing lit below it does not smoke.
        assert!(!CampfireBlock::is_smokey_pos_from(|dy| (false, dy == 1)));
        // A lit campfire beyond the scan distance is ignored.
        assert!(!CampfireBlock::is_smokey_pos_from(|dy| (dy == 6, false)));
    }
}
