use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;

use crate::entity::ai::behavior::fleeing::{self, Flee};
use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::bow_attack::BowAttackGoal;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        Controls, Goal, look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        evoker::IllagerSpell,
        patrol::{LongDistancePatrolGoal, PatrolData, PatrollingMonster},
        raider::{
            self, HoldGroundAttackGoal, ObtainRaidLeaderBannerGoal, PathfindToRaidGoal, Raider,
            RaiderCelebrationGoal, RaiderData, RaiderMoveThroughVillageGoal,
        },
    },
};

const ILLUSIONER_FLEES: &[Flee] = &[raider::flee_creaking(3, 1.0, 1.2)];

const ILLUSIONER_HUNTS: &[Hunt] = &[
    Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true).forget_unseen_after(300),
    Hunt::new(3, TargetMatch::VILLAGER_LIKE, false).forget_unseen_after(300),
    Hunt::new(3, TargetMatch::Type(&EntityType::IRON_GOLEM), false).forget_unseen_after(300),
];

/// Vanilla illusioner bow: `RangedBowAttackGoal<>(this, 0.5, 20, 15.0)`.
const BOW_ATTACK_SPEED: f64 = 0.5;
const BOW_ATTACK_INTERVAL: i32 = 20;
const BOW_ATTACK_RANGE: f32 = 15.0;

/// `SpellcasterUseSpellGoal#getCastingTime` for both illusioner spells.
const SPELL_CASTING_TIME: i32 = 20;
const BLINDNESS_SPELL_INTERVAL: i32 = 180;
const BLINDNESS_DURATION: i32 = 400;
const MIRROR_SPELL_INTERVAL: i32 = 340;
/// Client-side illusions expire after 60 seconds in vanilla.
const ILLUSION_LIFETIME: i32 = 60 * 20;

pub struct IllusionerEntity {
    pub mob_entity: MobEntity,
    pub raider_data: RaiderData,
    spell_casting_tick_count: AtomicI32,
    current_spell: AtomicU8,
    illusions_until_tick: AtomicI32,
}

impl IllusionerEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let illusioner = Self {
            mob_entity,
            raider_data: RaiderData::default(),
            spell_casting_tick_count: AtomicI32::new(0),
            current_spell: AtomicU8::new(IllagerSpell::None as u8),
            illusions_until_tick: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(illusioner);
        let mob_weak: Weak<Self> = Arc::downgrade(&mob_arc);
        let mob_interface_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        // Like other illagers, the illusioner is armed with a bow.
        {
            let living = &mob_arc.mob_entity.living_entity;
            let mut equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            equipment.put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::BOW));
        }

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(IllusionerCastingSpellGoal::new(mob_weak.clone())));
            goal_selector.add_goal(2, Box::new(ObtainRaidLeaderBannerGoal));
            goal_selector.add_goal(3, Box::new(HoldGroundAttackGoal::new(10.0)));
            fleeing::apply(goal_selector, ILLUSIONER_FLEES);
            goal_selector.add_goal(4, Box::new(LongDistancePatrolGoal::new(0.7, 0.595)));
            goal_selector.add_goal(4, Box::new(RaiderMoveThroughVillageGoal::new(1.05)));
            goal_selector.add_goal(4, Box::new(PathfindToRaidGoal::default()));
            goal_selector.add_goal(4, Box::new(IllusionerMirrorSpellGoal::new(mob_weak.clone())));
            goal_selector.add_goal(5, Box::new(RaiderCelebrationGoal));
            goal_selector.add_goal(5, Box::new(IllusionerBlindnessSpellGoal::new(mob_weak)));
            goal_selector.add_goal(
                6,
                Box::new(BowAttackGoal::new(
                    BOW_ATTACK_SPEED,
                    BOW_ATTACK_INTERVAL,
                    BOW_ATTACK_RANGE,
                )),
            );
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_interface_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(1, raider::revenge_goal());
            hunting::apply(target_selector, &mob_arc.mob_entity, ILLUSIONER_HUNTS);
        };

        mob_arc
    }

    pub fn is_casting_spell(&self) -> bool {
        self.spell_casting_tick_count.load(Ordering::Relaxed) > 0
    }

    pub fn set_is_casting_spell(&self, spell: IllagerSpell) {
        self.current_spell.store(spell as u8, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(
            pumpkin_data::tracked_data::illusioner::SPELL_CASTING_ID,
            spell as u8 as i8,
        );
    }

    pub fn get_current_spell(&self) -> IllagerSpell {
        IllagerSpell::from_u8(self.current_spell.load(Ordering::Relaxed))
    }

    pub fn get_spell_casting_time(&self) -> i32 {
        self.spell_casting_tick_count.load(Ordering::Relaxed)
    }

    pub fn set_spell_casting_time(&self, ticks: i32) {
        self.spell_casting_tick_count
            .store(ticks, Ordering::Relaxed);
    }

    fn tick_count(&self) -> i32 {
        self.mob_entity
            .living_entity
            .entity
            .tick_count
            .load(Ordering::Relaxed)
    }

    fn has_illusion(&self) -> bool {
        self.tick_count() < self.illusions_until_tick.load(Ordering::Relaxed)
    }
}

impl Mob for IllusionerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_patrolling_monster(&self) -> Option<&dyn PatrollingMonster> {
        Some(self)
    }

    fn as_raider(&self) -> Option<&dyn Raider> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_raider_nbt(nbt);
        nbt.put_int("SpellTicks", self.get_spell_casting_time());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_raider_nbt(nbt);
        if let Some(ticks) = nbt.get_int("SpellTicks") {
            self.set_spell_casting_time(ticks);
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let ticks = self.spell_casting_tick_count.load(Ordering::Relaxed);
        if ticks > 0 {
            self.spell_casting_tick_count
                .store(ticks - 1, Ordering::Relaxed);
        }
    }
}

impl PatrollingMonster for IllusionerEntity {
    fn get_patrol_data(&self) -> &PatrolData {
        &self.raider_data.patrol_data
    }
}

impl Raider for IllusionerEntity {
    fn get_raider_data(&self) -> &RaiderData {
        &self.raider_data
    }

    fn get_celebrate_sound(&self) -> Sound {
        Sound::EntityEvokerCelebrate
    }
}

pub struct IllusionerCastingSpellGoal {
    illusioner: Weak<IllusionerEntity>,
}

impl IllusionerCastingSpellGoal {
    #[must_use]
    pub const fn new(illusioner: Weak<IllusionerEntity>) -> Self {
        Self { illusioner }
    }
}

impl Goal for IllusionerCastingSpellGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(illusioner) = self.illusioner.upgrade() else {
            return false;
        };
        illusioner.is_casting_spell()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(illusioner) = self.illusioner.upgrade() else {
            return false;
        };
        illusioner.is_casting_spell()
    }

    fn start(&mut self, _mob: &dyn Mob) {
        if let Some(illusioner) = self.illusioner.upgrade() {
            illusioner
                .mob_entity
                .living_entity
                .entity
                .velocity
                .store(pumpkin_util::math::vector3::Vector3::new(0.0, 0.0, 0.0));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(illusioner) = self.illusioner.upgrade() {
            illusioner.set_is_casting_spell(IllagerSpell::None);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct IllusionerMirrorSpellGoal {
    illusioner: Weak<IllusionerEntity>,
    warmup_delay: i32,
    next_attack_tick: i32,
}

impl IllusionerMirrorSpellGoal {
    #[must_use]
    pub const fn new(illusioner: Weak<IllusionerEntity>) -> Self {
        Self {
            illusioner,
            warmup_delay: 0,
            next_attack_tick: 0,
        }
    }
}

impl Goal for IllusionerMirrorSpellGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(illusioner) = self.illusioner.upgrade() else {
            return false;
        };
        if illusioner.is_casting_spell() || illusioner.has_illusion() {
            return false;
        }
        if illusioner.tick_count() < self.next_attack_tick {
            return false;
        }
        illusioner.mob_entity.get_target().is_some()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.warmup_delay > 0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.warmup_delay = SPELL_CASTING_TIME;
        if let Some(illusioner) = self.illusioner.upgrade() {
            illusioner.set_spell_casting_time(SPELL_CASTING_TIME);
            self.next_attack_tick = illusioner.tick_count() + MIRROR_SPELL_INTERVAL;
            illusioner.set_is_casting_spell(IllagerSpell::Disappear);
            illusioner
                .mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityIllusionerPrepareMirror);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.warmup_delay -= 1;
        if self.warmup_delay == 0 {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return;
            };
            let tick = illusioner.tick_count();
            illusioner
                .illusions_until_tick
                .store(tick + ILLUSION_LIFETIME, Ordering::Relaxed);
            illusioner
                .mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityIllusionerCastSpell);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(illusioner) = self.illusioner.upgrade() {
            illusioner.set_is_casting_spell(IllagerSpell::None);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct IllusionerBlindnessSpellGoal {
    illusioner: Weak<IllusionerEntity>,
    warmup_delay: i32,
    next_attack_tick: i32,
    last_target_id: i32,
}

impl IllusionerBlindnessSpellGoal {
    #[must_use]
    pub const fn new(illusioner: Weak<IllusionerEntity>) -> Self {
        Self {
            illusioner,
            warmup_delay: 0,
            next_attack_tick: 0,
            last_target_id: i32::MIN,
        }
    }
}

impl Goal for IllusionerBlindnessSpellGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(illusioner) = self.illusioner.upgrade() else {
            return false;
        };
        if illusioner.is_casting_spell() || illusioner.tick_count() < self.next_attack_tick {
            return false;
        }
        let entity = &illusioner.mob_entity.living_entity.entity;
        let difficulty = entity.world.load().level_info.load().difficulty;
        if !matches!(difficulty, Difficulty::Normal | Difficulty::Hard) {
            return false;
        }
        illusioner
            .mob_entity
            .get_target()
            .is_some_and(|target| target.get_entity().entity_id != self.last_target_id)
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.warmup_delay > 0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.warmup_delay = SPELL_CASTING_TIME;
        if let Some(illusioner) = self.illusioner.upgrade() {
            illusioner.set_spell_casting_time(SPELL_CASTING_TIME);
            self.next_attack_tick = illusioner.tick_count() + BLINDNESS_SPELL_INTERVAL;
            illusioner.set_is_casting_spell(IllagerSpell::Blindness);
            illusioner
                .mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityIllusionerPrepareBlindness);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.warmup_delay -= 1;
        if self.warmup_delay == 0 {
            let Some(illusioner) = self.illusioner.upgrade() else {
                return;
            };
            let illusioner_entity = &illusioner.mob_entity.living_entity.entity;
            illusioner_entity.play_sound(Sound::EntityIllusionerCastSpell);
            let Some(target) = illusioner.mob_entity.get_target() else {
                return;
            };
            if let Some(living) = target.get_living_entity() {
                living.add_effect(Effect {
                    effect_type: &StatusEffect::BLINDNESS,
                    duration: BLINDNESS_DURATION,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }
            self.last_target_id = target.get_entity().entity_id;
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(illusioner) = self.illusioner.upgrade() {
            illusioner.set_is_casting_spell(IllagerSpell::None);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
