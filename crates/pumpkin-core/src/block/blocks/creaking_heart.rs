use std::sync::Arc;

use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{
    Axis, CreakingHeartLikeProperties, CreakingHeartState, PaleOakWoodLikeProperties,
};
use pumpkin_data::environment_attribute::EnvironmentAttribute;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro};
use pumpkin_world::world::BlockFlags;

use crate::block::entities::creaking_heart::CreakingHeartBlockEntity;
use crate::block::{
    BlockBehaviour, BrokenArgs, GetComparatorOutputArgs, OnNeighborUpdateArgs, OnPlaceArgs,
    PlacedArgs,
};

use crate::entity::experience_orb::ExperienceOrbEntity;

#[pumpkin_block("minecraft:creaking_heart")]
pub struct CreakingHeartBlock;

impl CreakingHeartBlock {
    fn active_state(
        world: &crate::world::World,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> CreakingHeartState {
        if world
            .environment_attributes()
            .get_value_bool(EnvironmentAttribute::GameplayCreakingActive, pos)
        {
            CreakingHeartState::Awake
        } else {
            CreakingHeartState::Dormant
        }
    }

    fn is_aligned_pale_oak_log(state_id: BlockStateId, axis: Axis) -> bool {
        let block = state_id.to_block();
        block.has_tag(&tag::Block::MINECRAFT_PALE_OAK_LOGS)
            && PaleOakWoodLikeProperties::from_state_id(state_id).axis == axis
    }

    fn check_active_logs(
        world: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
        axis: Axis,
    ) -> bool {
        let (pos_a, pos_b) = match axis {
            Axis::X => (pos.west(), pos.east()),
            Axis::Y => (pos.down(), pos.up()),
            Axis::Z => (pos.north(), pos.south()),
        };

        let state_a = world.get_block_state_id(&pos_a);
        let state_b = world.get_block_state_id(&pos_b);

        Self::is_aligned_pale_oak_log(state_a, axis) && Self::is_aligned_pale_oak_log(state_b, axis)
    }

    /// Vanilla `CreakingHeartBlock` updateState: re-evaluates `CREAKING_HEART_STATE` from the two
    /// aligned pale oak logs and the `creaking_active` environment attribute.
    pub fn update_state(
        world: &std::sync::Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
    ) {
        let state_id = world.get_block_state_id(position);
        let mut props = CreakingHeartLikeProperties::from_state_id(state_id);

        let has_logs = Self::check_active_logs(world.as_ref(), position, props.axis);
        // Vanilla updateCreakingState: a heart with a bound protector stays awake/dormant when it
        // loses its logs, so the creaking is never orphaned; only an unbound heart can uproot.
        let block_entity = world.get_block_entity(position);
        let bound = block_entity
            .as_ref()
            .and_then(|entity| entity.as_any().downcast_ref::<CreakingHeartBlockEntity>())
            .is_some_and(|heart| heart.creaking_uuid.load().is_some());

        let new_state = if Self::should_uproot(has_logs, bound) {
            CreakingHeartState::Uprooted
        } else {
            Self::active_state(world, position)
        };

        if props.creaking_heart_state != new_state {
            props.creaking_heart_state = new_state;
            world.set_block_state(
                position,
                props.to_state_id(state_id.to_block()),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }

    /// Vanilla updateCreakingState: uproot only when there are no logs and no bound creaking.
    const fn should_uproot(has_logs: bool, bound: bool) -> bool {
        !has_logs && !bound
    }
}

impl BlockBehaviour for CreakingHeartBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = CreakingHeartLikeProperties::from_state_id(args.block.default_state.id);
        props.axis = match args.direction {
            pumpkin_data::BlockDirection::North | pumpkin_data::BlockDirection::South => Axis::Z,
            pumpkin_data::BlockDirection::East | pumpkin_data::BlockDirection::West => Axis::X,
            pumpkin_data::BlockDirection::Up | pumpkin_data::BlockDirection::Down => Axis::Y,
        };
        props.creaking_heart_state = CreakingHeartState::Uprooted;
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(CreakingHeartBlockEntity::new(*args.position)));
        Self::update_state(args.world, args.position);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        Self::update_state(args.world, args.position);
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        // Vanilla `CreakingHeartBlock#getAnalogOutputSignal`: an UPROOTED heart always reads 0,
        // even if a stale binding survives (for example after loading from NBT).
        if CreakingHeartLikeProperties::from_state_id(args.state.id).creaking_heart_state
            == CreakingHeartState::Uprooted
        {
            return Some(0);
        }
        // Vanilla `CreakingHeartBlockEntity#getAnalogOutputSignal`: no bound creaking reads 0,
        // and the block entity caches it so comparator neighbours update when it changes.
        Some(
            args.world
                .get_block_entity(args.position)
                .and_then(|entity| entity.as_any().downcast_ref::<CreakingHeartBlockEntity>())
                .map_or(0, CreakingHeartBlockEntity::output_signal),
        )
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        // Vanilla CreakingHeartBlock#tryAwardExperience: only natural (world-gen) hearts drop
        // 20-24 experience, skipped for creative and spectator players. Silk Touch does not
        // block it, since the heart is not `dropExperienceWhenMined`.
        let props = CreakingHeartLikeProperties::from_state_id(args.state.id);
        // Vanilla `Block#popExperience` is gated on the `block_drops` gamerule and spawns at the
        // block center (`Vec3.atCenterOf`), not the block corner.
        if !props.natural
            || !args.world.level_info.load().game_rules.block_drops
            || matches!(
                args.player.gamemode.load(),
                GameMode::Creative | GameMode::Spectator
            )
        {
            return;
        }
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        // Vanilla `nextIntBetweenInclusive(20, 24)`.
        let amount = 20 + random.next_bounded_i32(5) as u32;
        ExperienceOrbEntity::spawn(args.world, args.position.to_centered_f64(), amount);
    }
}

#[cfg(test)]
mod tests {
    use super::CreakingHeartBlock;

    #[test]
    fn bound_heart_never_uproots_without_logs() {
        assert!(CreakingHeartBlock::should_uproot(false, false));
        assert!(!CreakingHeartBlock::should_uproot(false, true));
        assert!(!CreakingHeartBlock::should_uproot(true, false));
        assert!(!CreakingHeartBlock::should_uproot(true, true));
    }
}
