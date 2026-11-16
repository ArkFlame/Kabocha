use std::sync::Arc;

use pumpkin_data::{
    Block, Enchantment, damage::DamageType, data_component_impl::EquipmentSlot, effect::StatusEffect,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;

use crate::block::{
    BlockBehaviour, OnEntityStepArgs, OnNeighborUpdateArgs, OnScheduledTickArgs, PlacedArgs,
    blocks::bubble_column,
};
use crate::world::World;

/// Vanilla `MagmaBlock.BUBBLE_COLUMN_CHECK_DELAY`.
const BUBBLE_COLUMN_CHECK_DELAY: u32 = 20;

#[pumpkin_block("minecraft:magma_block")]
pub struct MagmaBlock;

impl BlockBehaviour for MagmaBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        schedule_bubble_column_check(args.world, *args.position);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        // Vanilla only reacts to the block above becoming water; we cannot tell the update
        // direction here, so a water block above is the equivalent check.
        if args.world.get_block(&args.position.up()) == &Block::WATER {
            schedule_bubble_column_check(args.world, *args.position);
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if args.world.get_block(args.position) != &Block::MAGMA_BLOCK {
            return;
        }
        bubble_column::update_column(args.world, args.position.up());
    }

    fn on_entity_step(&self, args: OnEntityStepArgs<'_>) {
        {
            // Vanilla only invokes `stepOn` for the block the entity actually stands on, so a
            // non-solid block above (snow layer, carpet) shields the magma block below it.
            if args.below_supporting_block {
                return;
            }

            // Only living entities take damage
            let Some(living_entity) = args.entity.get_living_entity() else {
                return;
            };

            let ent = args.entity.get_entity();

            // Don't damage if sneaking
            if args.entity.is_stepping_carefully() {
                return;
            }

            // Fire immune entities don't take damage
            if ent.is_fire_immune() {
                return;
            }

            let has_frost_walker = {
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
            if has_frost_walker {
                return;
            }

            if living_entity
                .get_effect(&StatusEffect::FIRE_RESISTANCE)
                .is_some()
            {
                return;
            }

            // Apply damage
            args.entity.damage(args.entity, 1.0, DamageType::HOT_FLOOR);
        }
    }
}

fn schedule_bubble_column_check(world: &Arc<World>, position: BlockPos) {
    world.schedule_block_tick(
        &Block::MAGMA_BLOCK,
        position,
        BUBBLE_COLUMN_CHECK_DELAY,
        TickPriority::Normal,
    );
}
