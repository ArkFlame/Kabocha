use std::borrow::Cow;
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::InstrumentImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::instrument::Instrument;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::RandomImpl;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use rand::RngExt;

use crate::entity::item::ItemEntity;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        long_jump::LongJumpGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, ram_target::RamTargetGoal, revenge::RevengeGoal,
        swim::SwimGoal, tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    custom_sound::CustomSound,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use crate::item::items::bucket::fill_bucket_stack;
use crate::world::World;

const ADULT_ATTACK_DAMAGE: f64 = 2.0;
const BABY_ATTACK_DAMAGE: f64 = 1.0;
const GOAT_SCREAMING_CHANCE: f64 = 0.02;
const UNIHORN_CHANCE: f32 = 0.1;
const MAX_HEAD_Y_ROT: f32 = 15.0;

/// Vanilla `GoatAi.initLongJumpActivity`: screamers use a distinct long-jump sound.
fn goat_long_jump_sound(mob: &dyn Mob) -> Sound {
    let screaming = mob
        .cast_any()
        .downcast_ref::<GoatEntity>()
        .is_some_and(GoatEntity::is_screaming);
    if screaming {
        Sound::EntityGoatScreamingLongJump
    } else {
        Sound::EntityGoatLongJump
    }
}

pub struct GoatEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub is_screaming: AtomicBool,
    pub has_left_horn: AtomicBool,
    pub has_right_horn: AtomicBool,
}

impl GoatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut navigator = mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_can_float(true);
            navigator.set_pathfinding_malus(PathType::PowderSnow, -1.0);
            navigator.set_pathfinding_malus(PathType::DangerPowderSnow, -1.0);
        };
        let goat = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            is_screaming: AtomicBool::new(false),
            has_left_horn: AtomicBool::new(true),
            has_right_horn: AtomicBool::new(true),
        };
        let mob_arc = Arc::new(goat);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            // UNVERIFIED: ram sits in the fight slot, after panic, ahead of breeding.
            goal_selector.add_goal(2, Box::new(RamTargetGoal::default()));
            // Vanilla GoatAi.initLongJumpActivity: 600..1200 cooldown, 5x5 search, 3.5714288x jump.
            goal_selector.add_goal(
                2,
                Box::new(LongJumpGoal::new(
                    goat_long_jump_sound,
                    Sound::EntityGoatStep,
                )),
            );
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(
                3,
                Box::new(TemptGoal::with_tag(
                    1.25,
                    &tag::Item::MINECRAFT_GOAT_FOOD,
                    false,
                )),
            );
            // Vanilla GoatAi.initIdleActivity: each goat rolls its own BabyFollowAdult distance (5..=16).
            let follow_distance = rand::rng().random_range(5..=16) as f64;
            goal_selector.add_goal(
                4,
                Box::new(FollowParentGoal::with_min_distance(1.25, follow_distance)),
            );
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };
        {
            // UNVERIFIED: a hit goat turns on its attacker, which gives the ram a target.
            mob_arc
                .mob_entity
                .target_selector
                .add_goal(1, Box::new(RevengeGoal::new(true)));
        };

        mob_arc
    }

    #[must_use]
    pub fn is_screaming(&self) -> bool {
        self.is_screaming.load(Ordering::Relaxed)
    }

    pub fn set_screaming(&self, screaming: bool) {
        self.is_screaming.store(screaming, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_IS_SCREAMING_GOAT,
            screaming,
        );
    }

    #[must_use]
    pub fn has_left_horn(&self) -> bool {
        self.has_left_horn.load(Ordering::Relaxed)
    }

    pub fn set_has_left_horn(&self, has_horn: bool) {
        self.has_left_horn.store(has_horn, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_HAS_LEFT_HORN,
            has_horn,
        );
    }

    #[must_use]
    pub fn has_right_horn(&self) -> bool {
        self.has_right_horn.load(Ordering::Relaxed)
    }

    pub fn set_has_right_horn(&self, has_horn: bool) {
        self.has_right_horn.store(has_horn, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_HAS_RIGHT_HORN,
            has_horn,
        );
    }

    /// Vanilla `Goat.createHorn`: pick an instrument from the matching goat-horn tag.
    #[must_use]
    fn create_horn(&self) -> ItemStack {
        let key = if self.is_screaming() {
            tag::Instrument::MINECRAFT_SCREAMING_GOAT_HORNS
        } else {
            tag::Instrument::MINECRAFT_REGULAR_GOAT_HORNS
        };
        let mut stack = ItemStack::new(1, &Item::GOAT_HORN);
        let names = key.0;
        if names.is_empty() {
            return stack;
        }
        // Vanilla seeds a fresh `RandomSource` from the UUID hash before picking the element.
        let seed = self.get_entity().entity_uuid.as_u128() as u64;
        let index = Xoroshiro::from_seed(seed).next_bounded_i32(names.len() as i32) as usize;
        if let (Some(name), Some(instrument)) = (names.get(index), Instrument::from_name(name)) {
            stack.set_data_component(InstrumentImpl {
                instrument: Cow::Borrowed(instrument.asset_id()),
            });
        }
        stack
    }

    /// Vanilla `Goat.dropHorn`: break one horn and spawn its goat horn item.
    pub fn drop_horn(&self) -> bool {
        if self.is_baby() {
            return false;
        }
        let has_left = self.has_left_horn();
        let has_right = self.has_right_horn();
        if !has_left && !has_right {
            return false;
        }
        if !has_left {
            self.set_has_right_horn(false);
        } else if !has_right {
            self.set_has_left_horn(false);
        } else if self.get_random().random::<bool>() {
            self.set_has_left_horn(false);
        } else {
            self.set_has_right_horn(false);
        }

        let entity = self.get_entity();
        let pos = entity.pos.load();
        let world = entity.world.load_full();
        let mut random = self.get_random();
        let velocity = Vector3::new(
            random.random_range(-0.2..=0.2),
            random.random_range(0.3..=0.7),
            random.random_range(-0.2..=0.2),
        );
        let item_entity = Entity::new(world.clone(), pos, &EntityType::ITEM);
        world.spawn_entity(Arc::new(ItemEntity::new_with_velocity(
            item_entity,
            self.create_horn(),
            velocity,
            ItemEntity::DEFAULT_PICKUP_DELAY,
        )))
    }
}

impl GoatEntity {
    fn screaming_sound(&self, regular: Sound, screaming: Sound) -> Sound {
        if self.is_screaming() {
            screaming
        } else {
            regular
        }
    }
}

impl CustomSound for GoatEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(self.screaming_sound(Sound::EntityGoatDeath, Sound::EntityGoatScreamingDeath))
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(self.screaming_sound(Sound::EntityGoatHurt, Sound::EntityGoatScreamingHurt))
    }
}

impl AgeableMob for GoatEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn age_boundary_reached(&self) {
        // Vanilla `Goat.ageBoundaryReached`: babies lose their horns, adults grow them back.
        let (damage, has_horns) = if self.is_baby() {
            (BABY_ATTACK_DAMAGE, false)
        } else {
            (ADULT_ATTACK_DAMAGE, true)
        };
        self.mob_entity
            .living_entity
            .set_attribute_base(&Attributes::ATTACK_DAMAGE, damage);
        self.set_has_left_horn(has_horns);
        self.set_has_right_horn(has_horns);
    }
}

impl Animal for GoatEntity {
    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::GOAT,
        ));
        let mut random = self.get_random();
        // Vanilla `Goat.createChild`: the parent that donates the screaming trait is picked at random.
        let selected_parent_screaming = if random.random::<bool>() {
            self.is_screaming()
        } else {
            partner
                .cast_any()
                .downcast_ref::<Self>()
                .is_some_and(Self::is_screaming)
        };
        baby.set_screaming(
            selected_parent_screaming || random.random::<f64>() < GOAT_SCREAMING_CHANCE,
        );
        baby
    }

    fn play_eating_sound(&self) {
        let sound = if self.is_screaming() {
            Sound::EntityGoatScreamingEat
        } else {
            Sound::EntityGoatEat
        };
        let pitch = 0.8 + self.get_random().random::<f32>() * 0.4;
        self.play_entity_sound(sound, 1.0, pitch);
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_GOAT_FOOD)
    }
}

impl Mob for GoatEntity {
    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        _reason: crate::entity::mob::spawn::EntitySpawnReason,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        let mut random = self.get_random();
        self.set_screaming(random.random::<f64>() < GOAT_SCREAMING_CHANCE);
        self.age_boundary_reached();
        if !self.is_baby() && random.random::<f32>() < UNIHORN_CHANCE {
            if random.random::<bool>() {
                self.set_has_left_horn(false);
            } else {
                self.set_has_right_horn(false);
            }
        }
        self.mob_entity.finalize_spawn_base();
        group_data
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(self.screaming_sound(Sound::EntityGoatAmbient, Sound::EntityGoatScreamingAmbient))
    }

    fn calculate_fall_damage(&self, fall_distance: f32, damage_modifier: f32) -> i32 {
        self.mob_entity
            .living_entity
            .base_fall_damage(fall_distance, damage_modifier)
            - 10
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityGoatStep, 0.15, 1.0);
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("IsScreamingGoat", self.is_screaming());
        nbt.put_bool("HasLeftHorn", self.has_left_horn());
        nbt.put_bool("HasRightHorn", self.has_right_horn());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(screaming) = nbt.get_bool("IsScreamingGoat") {
            self.set_screaming(screaming);
        }
        // Vanilla reads these with `CompoundTag.getBoolean`, so a missing key means "no horn".
        self.set_has_left_horn(nbt.get_bool("HasLeftHorn").unwrap_or(false));
        self.set_has_right_horn(nbt.get_bool("HasRightHorn").unwrap_or(false));
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_max_head_rotation(&self) -> f32 {
        MAX_HEAD_Y_ROT
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::goat::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_IS_SCREAMING_GOAT,
            self.is_screaming(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_HAS_LEFT_HORN,
            self.has_left_horn(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::goat::DATA_HAS_RIGHT_HORN,
            self.has_right_horn(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();
        if item == &Item::BUCKET && !self.is_baby() {
            self.get_entity().world.load().play_sound_expect(
                player,
                self.screaming_sound(Sound::EntityGoatMilk, Sound::EntityGoatScreamingMilk),
                SoundCategory::Players,
                &player.position(),
            );
            fill_bucket_stack(item_stack, player, &Item::MILK_BUCKET);
            return true;
        }

        self.animal_interact(player, item_stack)
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::attributes::Attributes;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::test_support::world;

    #[tokio::test]
    async fn attack_damage_follows_the_age_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let goat = GoatEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::GOAT,
        ));
        let attack_damage = || {
            goat.mob_entity
                .living_entity
                .get_attribute_base(&Attributes::ATTACK_DAMAGE)
        };
        goat.set_baby(true);
        assert_eq!(attack_damage(), BABY_ATTACK_DAMAGE);
        assert!(!goat.has_left_horn() && !goat.has_right_horn());
        goat.set_baby(false);
        assert_eq!(attack_damage(), ADULT_ATTACK_DAMAGE);
        assert!(goat.has_left_horn() && goat.has_right_horn());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn missing_horn_keys_load_as_no_horns() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let goat = GoatEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::GOAT,
        ));
        goat.mob_read_nbt(&NbtCompound::new());
        assert!(!goat.has_left_horn() && !goat.has_right_horn());
        assert_eq!(goat.get_max_head_rotation(), MAX_HEAD_Y_ROT);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn ambient_sound_follows_the_screaming_flag() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let goat = GoatEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::GOAT,
        ));
        assert_eq!(goat.get_ambient_sound(), Some(Sound::EntityGoatAmbient));
        goat.set_screaming(true);
        assert_eq!(
            goat.get_ambient_sound(),
            Some(Sound::EntityGoatScreamingAmbient)
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn drop_horn_requires_an_adult_with_horns() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let goat = GoatEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::GOAT,
        ));
        goat.set_baby(true);
        assert!(!goat.drop_horn());
        goat.set_baby(false);
        goat.set_has_left_horn(false);
        goat.set_has_right_horn(false);
        assert!(!goat.drop_horn());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn create_horn_uses_the_screaming_tag() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let goat = GoatEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::GOAT,
        ));
        goat.set_screaming(true);
        let stack = goat.create_horn();
        assert!(
            stack
                .get_data_component::<InstrumentImpl>()
                .is_some_and(|inst| {
                    tag::Instrument::MINECRAFT_SCREAMING_GOAT_HORNS
                        .0
                        .contains(&inst.instrument.as_ref())
                })
        );
        world.level.shutdown().await;
        Ok(())
    }
}
