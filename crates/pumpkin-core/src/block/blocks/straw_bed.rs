use std::sync::Arc;

use crate::block::entities::bed::BedBlockEntity;
use pumpkin_data::block_properties::BedPart;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::translation;
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, NormalUseArgs,
    OnPlaceArgs, OnStateReplacedArgs, PathComputationType, PlacedArgs,
};
use crate::entity::passive::villager::VillagerEntity;
use crate::entity::{EntityBase, player::Player};
use crate::world::World;
use pumpkin_data::HorizontalFacingExt;

type BedProperties = pumpkin_data::block_properties::WhiteBedLikeProperties;

const NO_SLEEP_IDS: &[u16] = &[
    pumpkin_data::entity::EntityType::BLAZE.id,
    pumpkin_data::entity::EntityType::BOGGED.id,
    pumpkin_data::entity::EntityType::SKELETON.id,
    pumpkin_data::entity::EntityType::STRAY.id,
    pumpkin_data::entity::EntityType::WITHER_SKELETON.id,
    pumpkin_data::entity::EntityType::BREEZE.id,
    pumpkin_data::entity::EntityType::CREAKING.id,
    pumpkin_data::entity::EntityType::CREEPER.id,
    pumpkin_data::entity::EntityType::DROWNED.id,
    pumpkin_data::entity::EntityType::ENDERMITE.id,
    pumpkin_data::entity::EntityType::EVOKER.id,
    pumpkin_data::entity::EntityType::GIANT.id,
    pumpkin_data::entity::EntityType::GUARDIAN.id,
    pumpkin_data::entity::EntityType::ELDER_GUARDIAN.id,
    pumpkin_data::entity::EntityType::ILLUSIONER.id,
    pumpkin_data::entity::EntityType::OCELOT.id,
    pumpkin_data::entity::EntityType::PIGLIN.id,
    pumpkin_data::entity::EntityType::PIGLIN_BRUTE.id,
    pumpkin_data::entity::EntityType::PILLAGER.id,
    pumpkin_data::entity::EntityType::PHANTOM.id,
    pumpkin_data::entity::EntityType::RAVAGER.id,
    pumpkin_data::entity::EntityType::SILVERFISH.id,
    pumpkin_data::entity::EntityType::SPIDER.id,
    pumpkin_data::entity::EntityType::CAVE_SPIDER.id,
    pumpkin_data::entity::EntityType::VEX.id,
    pumpkin_data::entity::EntityType::VINDICATOR.id,
    pumpkin_data::entity::EntityType::WARDEN.id,
    pumpkin_data::entity::EntityType::WITCH.id,
    pumpkin_data::entity::EntityType::WITHER.id,
    pumpkin_data::entity::EntityType::ZOGLIN.id,
    pumpkin_data::entity::EntityType::ZOMBIE.id,
    pumpkin_data::entity::EntityType::ZOMBIE_VILLAGER.id,
    pumpkin_data::entity::EntityType::HUSK.id,
    pumpkin_data::entity::EntityType::ENDERMAN.id,
    pumpkin_data::entity::EntityType::ZOMBIFIED_PIGLIN.id,
];

#[pumpkin_block("minecraft:straw_bed")]
pub struct StrawBedBlock;

impl BlockBehaviour for StrawBedBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if let Some(player) = args.player {
            let facing = player.get_entity().get_horizontal_facing();
            let head_pos = args.position.offset(facing.to_offset());
            // AbstractBedBlock#getStateForPlacement requires the head position to
            // be inside the world border.
            let head_inside_border = args.world.is_none_or(|world| {
                world.worldborder.lock().is_ok_and(|border| {
                    border.is_within_bounds(f64::from(head_pos.0.x), f64::from(head_pos.0.z), 1.0)
                })
            });
            return head_inside_border
                && args
                    .block_accessor
                    .get_block_state(args.position)
                    .replaceable()
                && args.block_accessor.get_block_state(&head_pos).replaceable();
        }
        true
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut bed_props = BedProperties::default(args.block);
        bed_props.facing = args.player.get_entity().get_horizontal_facing();
        bed_props.part = BedPart::Foot;
        bed_props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let bed_entity = BedBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(bed_entity));

        let mut bed_head_props = BedProperties::default(args.block);
        bed_head_props.facing = BedProperties::from_state_id(args.state_id).facing;
        bed_head_props.part = BedPart::Head;

        let bed_head_pos = args.position.offset(bed_head_props.facing.to_offset());
        args.world.set_block_state(
            &bed_head_pos,
            bed_head_props.to_state_id(args.block),
            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
        );

        let bed_head_entity = BedBlockEntity::new(bed_head_pos);
        args.world.add_block_entity(Arc::new(bed_head_entity));
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let bed_props = BedProperties::from_state_id(args.state.id);
        let other_half_pos = if bed_props.part == BedPart::Head {
            args.position
                .offset(bed_props.facing.opposite().to_offset())
        } else {
            args.position.offset(bed_props.facing.to_offset())
        };
        let neighbor_state_id = args.world.get_block_state_id(&other_half_pos);
        if neighbor_state_id.to_block_id() != args.block.id {
            args.world.update_neighbors(&other_half_pos, None);
            return;
        }

        let is_creative = args.player.gamemode.load() == GameMode::Creative;
        let flags = if bed_props.part == BedPart::Foot && !is_creative {
            BlockFlags::NOTIFY_ALL
        } else {
            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL
        };

        args.world
            .break_block(&other_half_pos, Some(args.player), flags);
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if args.moved {
            return;
        }

        let bed_props = BedProperties::from_state_id(args.old_state_id);
        let other_half_pos = if bed_props.part == BedPart::Head {
            args.position
                .offset(bed_props.facing.opposite().to_offset())
        } else {
            args.position.offset(bed_props.facing.to_offset())
        };

        let (other_block, other_state) = args.world.get_block_and_state(&other_half_pos);
        if other_block == args.block {
            let other_props = BedProperties::from_state_id(other_state.id);
            if other_props.part != bed_props.part {
                args.world.break_block(
                    &other_half_pos,
                    None,
                    BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let mut bed_props = BedProperties::from_state_id(args.state_id);
        let neighbor_dir = if bed_props.part == BedPart::Foot {
            bed_props.facing.to_block_direction()
        } else {
            bed_props.facing.opposite().to_block_direction()
        };
        if args.direction == neighbor_dir {
            let neighbor_block = args.neighbor_state_id.to_block();
            if neighbor_block == args.block {
                let neighbor_props = BedProperties::from_state_id(args.neighbor_state_id);
                if neighbor_props.part != bed_props.part {
                    bed_props.occupied = neighbor_props.occupied;
                    return bed_props.to_state_id(args.block);
                }
            }
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        Self::use_bed(args.world, args.player, args.block, args.position)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

impl StrawBedBlock {
    pub fn destroy_after_use(world: &Arc<World>, bed_head_pos: BlockPos) {
        let (block, state) = world.get_block_and_state_id(&bed_head_pos);
        if block != &Block::STRAW_BED {
            return;
        }
        let bed_props = BedProperties::from_state_id(state);
        let (head_pos, foot_pos) = if bed_props.part == BedPart::Head {
            (
                bed_head_pos,
                bed_head_pos.offset(bed_props.facing.opposite().to_offset()),
            )
        } else {
            (
                bed_head_pos.offset(bed_props.facing.to_offset()),
                bed_head_pos,
            )
        };
        world.play_block_sound(
            Sound::BlockStrawBedBreakLeave,
            SoundCategory::Blocks,
            head_pos,
        );
        world.break_block(
            &head_pos,
            None,
            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
        );
        world.break_block(
            &foot_pos,
            None,
            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
        );
    }

    fn use_bed(
        world: &Arc<World>,
        player: &Arc<Player>,
        block: &Block,
        position: &BlockPos,
    ) -> BlockActionResult {
        let state_id = world.get_block_state_id(position);
        let bed_props = BedProperties::from_state_id(state_id);

        let (bed_head_pos, bed_foot_pos, other_half_pos) = if bed_props.part == BedPart::Head {
            (
                *position,
                position.offset(bed_props.facing.opposite().to_offset()),
                position.offset(bed_props.facing.opposite().to_offset()),
            )
        } else {
            (
                position.offset(bed_props.facing.to_offset()),
                *position,
                position.offset(bed_props.facing.to_offset()),
            )
        };

        // AbstractBedBlock#useWithoutItem ignores a half whose partner is not the
        // same block with the opposite part (e.g. a lone half from /setblock).
        let (other_block, other_state) = world.get_block_and_state(&other_half_pos);
        if other_block != block
            || BedProperties::from_state_id(other_state.id).part == bed_props.part
        {
            return BlockActionResult::Consume;
        }

        let is_dark = world.is_dark_outside();
        let can_sleep = world.dimension.bed_rule.can_sleep(is_dark);
        let can_set_spawn = world.dimension.bed_rule.can_set_spawn(is_dark);

        // AbstractBedBlock#useWithoutItem handles an occupied bed before any
        // range, obstruction or sleep rule check.
        if bed_props.occupied {
            let bed_box = BoundingBox::from_block(&bed_head_pos);
            let mut kicked_villager = false;
            for entity in
                world.get_entities_of_type_at_box(&bed_box, |t| t == &EntityType::VILLAGER)
            {
                let Some(villager) = entity.cast_any().downcast_ref::<VillagerEntity>() else {
                    continue;
                };
                if villager.is_sleeping() {
                    villager.wake_up();
                    kicked_villager = true;
                    break;
                }
            }
            if !kicked_villager {
                player.send_system_message_raw(
                    &pumpkin_macros::translate_cross!(
                        translation::java::BLOCK_MINECRAFT_BED_OCCUPIED,
                        translation::bedrock::TILE_BED_OCCUPIED
                    ),
                    true,
                );
            }
            return BlockActionResult::SuccessServer;
        }

        // ServerPlayer#getBedResult checks the reachable range before the
        // obstruction.
        if !player
            .position()
            .is_within_bounds(bed_head_pos.to_f64(), 3.0, 2.0, 3.0)
            && !player
                .position()
                .is_within_bounds(bed_foot_pos.to_f64(), 3.0, 2.0, 3.0)
        {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_TOO_FAR_AWAY,
                    translation::bedrock::TILE_BED_TOOFAR
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // ServerPlayer#bedBlocked uses Player#freeAt, i.e. `!isSuffocating`.
        if world.get_block_state(&bed_head_pos.up()).is_suffocating()
            || world.get_block_state(&bed_foot_pos.up()).is_suffocating()
        {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_OBSTRUCTED,
                    translation::bedrock::TILE_BED_OBSTRUCTED
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Set respawn point
        if can_set_spawn
            && player.set_respawn_point(
                world.dimension.clone(),
                bed_head_pos,
                player.get_entity().yaw.load(),
                player.get_entity().pitch.load(),
                false,
            )
        {
            player.send_system_message(&pumpkin_macros::translate_cross!(
                translation::java::BLOCK_MINECRAFT_SET_SPAWN,
                translation::bedrock::TILE_BED_RESPAWNSET
            ));
        }

        if !can_sleep {
            player.send_system_message_raw(
                &pumpkin_macros::translate_cross!(
                    translation::java::BLOCK_MINECRAFT_BED_NO_SLEEP,
                    translation::bedrock::TILE_BED_NOSLEEP
                ),
                true,
            );
            return BlockActionResult::SuccessServer;
        }

        // Vanilla skips the monster scan in creative and only scans the head.
        for entity in world.entities.load().iter() {
            if player.gamemode.load() == GameMode::Creative
                || !NO_SLEEP_IDS.contains(&entity.get_entity().entity_type.id)
                || entity
                    .get_mob()
                    .is_some_and(|mob| !mob.is_preventing_player_rest(player.as_ref()))
            {
                continue;
            }

            let pos = entity.get_entity().pos.load();
            if pos.is_within_bounds(bed_head_pos.to_f64(), 8.0, 5.0, 8.0) {
                player.send_system_message_raw(
                    &pumpkin_macros::translate_cross!(
                        translation::java::BLOCK_MINECRAFT_BED_NOT_SAFE,
                        translation::bedrock::TILE_BED_NOTSAFE
                    ),
                    true,
                );
                return BlockActionResult::SuccessServer;
            }
        }

        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::player::player_bed::PlayerBedEnterEvent::new(
                    player.clone(),
                    bed_head_pos,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return BlockActionResult::SuccessServer;
            }
        }

        player.sleep(bed_head_pos);
        // ServerPlayer#startSleepInBed only fires SLEPT_IN_BED when canSetSpawn.
        if can_set_spawn {
            player.trigger_advancement(
                crate::entity::player::advancement::trigger::AdvancementTrigger::SleptInBed,
            );
        }
        player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::SleepInStrawBed as i32,
            1,
        );
        crate::block::blocks::bed::BedBlock::set_occupied(true, world, block, position, state_id);

        BlockActionResult::SuccessServer
    }
}
