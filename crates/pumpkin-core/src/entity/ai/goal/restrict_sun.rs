use super::{Controls, Goal};
use crate::entity::ai::pathfinder::PathNavigationTrait;
use crate::entity::mob::Mob;
use crate::entity::mob::sun_burn;

#[derive(Default)]
pub struct RestrictSunGoal;

impl RestrictSunGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn is_exposed(mob: &dyn Mob) -> bool {
        mob.get_entity().world.load().is_bright_outside() && !sun_burn::is_protected(mob)
    }
}

impl Goal for RestrictSunGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        Self::is_exposed(mob)
    }

    // Vanilla keeps the goal running once started (default canContinueToUse), so the
    // navigation avoid-sun flag stays on for the rest of the mob's life.
    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        true
    }

    fn start(&mut self, mob: &dyn Mob) {
        let steered = mob.get_mob_entity().steered();
        let mut navigator = steered.navigator();
        navigator.set_avoid_sun(true);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let steered = mob.get_mob_entity().steered();
        let mut navigator = steered.navigator();
        navigator.set_avoid_sun(false);
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
