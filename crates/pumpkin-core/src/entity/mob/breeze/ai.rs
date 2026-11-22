use pumpkin_data::environment_attribute::Activity;

use super::{BreezeEntity, movement};
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainState,
    activity::ActivityData,
    behavior::{DoNothing, LookAtTargetSink, Swim, Timed, random_stroll, run_one},
    memory::{MemoryStatus, PackedMemories, types},
};
use crate::entity::mob::Mob;

pub(super) fn make_brain(breeze: &BreezeEntity, packed: &PackedMemories) -> Brain {
    let mut brain = BrainProvider {
        memory_types: &[],
        sensor_types: &[],
        activities,
    }
    .make_brain(breeze, packed);
    brain.set_default_activity(Activity::Fight);
    brain.use_default_activity();
    brain
}

pub(super) fn update_activity(brain: &mut BrainState) {
    brain.set_active_activity_to_first_valid(&[Activity::Fight, Activity::Idle]);
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(Swim::new(0.8))),
                Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
            ],
        ),
        ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![
                movement::slide_to_target_sink(),
                Box::new(run_one(vec![
                    (Box::new(DoNothing::new(20, 100)), 1),
                    (Box::new(random_stroll::stroll(0.6)), 2),
                ])),
            ],
        ),
        ActivityData::with_priority_start_and_conditions(
            Activity::Fight,
            0,
            vec![movement::long_jump(), movement::slide()],
            vec![
                (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
                (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            ],
        ),
    ]
}
