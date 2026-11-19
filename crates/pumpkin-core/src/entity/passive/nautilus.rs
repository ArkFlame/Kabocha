use crossbeam::atomic::AtomicCell;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering},
};
use uuid::Uuid;

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::{
    data_component_impl::EquipmentSlot,
    effect::StatusEffect,
    entity::{EntityStatus, EntityType},
    item_stack::ItemStack,
    particle::Particle,
    potion::Effect,
    sound::{Sound, SoundCategory},
    tag::{self, Taggable},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::ActorEventID;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    ai::{
        control::{
            smooth_swimming_look_control::SmoothSwimmingLookControl,
            smooth_swimming_move_control::SmoothSwimmingMoveControl,
        },
        goal::{
            breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
            look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
            melee_attack::MeleeAttackGoal, random_swimming::RandomSwimmingGoal,
            revenge::RevengeGoal, tempt::TemptGoal,
        },
        pathfinder::{Navigator, node::PathType},
    },
    breath::MAX_AIR,
    custom_sound::CustomSound,
    mob::{Mob, MobEntity, equipment::equip_on_target},
    passive::{abstract_horse::player_jump_pending_scale, animal::Animal, water_animal},
    player::Player,
};

/// Vanilla `SmoothSwimmingMoveControl(this, 85, 10, 0.011F, 0.0F, true)`.
const MAX_TURN_X: i32 = 85;
const MAX_TURN_Y: i32 = 10;
const IN_WATER_SPEED_MODIFIER: f32 = 0.011;
/// Vanilla `Nautilus#handleAirSupply` dry-out damage on land.
const DRY_OUT_DAMAGE: f32 = 2.0;

pub struct NautilusEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: crate::entity::ageable::AgeableData,
    pub is_tame: AtomicBool,
    pub owner: AtomicCell<Option<Uuid>>,
    pub is_dashing: AtomicBool,
    pub dash_cooldown: AtomicI32,
}

impl NautilusEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        // Vanilla `AbstractNautilus` constructor: a water bound navigator with the
        // WATER path malus cleared.
        {
            let navigator = mob_entity
                .navigator
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *navigator = Navigator::water_bound(false);
            navigator.set_pathfinding_malus(PathType::Water, 0.0);
        };
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(SmoothSwimmingMoveControl::new(
                MAX_TURN_X,
                MAX_TURN_Y,
                IN_WATER_SPEED_MODIFIER,
                0.0,
                true,
            ));
        *mob_entity
            .look_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(SmoothSwimmingLookControl::new(10));
        let nautilus = Arc::new(Self {
            mob_entity,
            ageable_data: crate::entity::ageable::AgeableData::default(),
            is_tame: AtomicBool::new(false),
            owner: AtomicCell::new(None),
            is_dashing: AtomicBool::new(false),
            dash_cooldown: AtomicI32::new(0),
        });
        {
            // Vanilla `NautilusAi`: panic, make love, follow temptation, follow a
            // parent, random swim and random look, in priority order.
            let goals = &nautilus.mob_entity.goals_selector;
            goals.add_goal(0, EscapeDangerGoal::new(1.6));
            goals.add_goal(1, BreedGoal::new(0.4));
            goals.add_goal(
                2,
                Box::new(TemptGoal::with_tag(
                    1.3,
                    &tag::Item::MINECRAFT_NAUTILUS_FOOD,
                    false,
                )),
            );
            goals.add_goal(3, Box::new(FollowParentGoal::new(1.3)));
            goals.add_goal(4, Box::new(RandomSwimmingGoal::new(1.0, 10)));
            goals.add_goal(5, Box::new(RandomLookAroundGoal::default()));
            // Vanilla NautilusAi.IDLE: look-at target, then charge a hostile in water.
            let mob_weak = Arc::downgrade(&(nautilus.clone() as Arc<dyn Mob>));
            goals.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goals.add_goal(7, Box::new(MeleeAttackGoal::new(0.6, true)));
        };
        nautilus
            .mob_entity
            .target_selector
            .add_goal(1, Box::new(RevengeGoal::new(true)));
        nautilus
    }

    pub fn is_dashing(&self) -> bool {
        self.is_dashing.load(Ordering::Relaxed)
    }

    pub fn set_dashing(&self, dashing: bool) {
        self.is_dashing.store(dashing, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(pumpkin_data::tracked_data::nautilus::DASH, dashing);
    }

    pub fn is_tame(&self) -> bool {
        self.is_tame.load(Ordering::Relaxed)
    }

    pub fn set_tame(&self, tame: bool, owner: Option<Uuid>) {
        self.is_tame.store(tame, Ordering::Relaxed);
        self.owner.store(owner);
        // Vanilla `TamableAnimal` syncs the tame flag and the owner instead of a
        // separate boolean.
        let entity = &self.mob_entity.living_entity.entity;
        let flags = if tame {
            crate::entity::passive::tamable::TAME_FLAG as i8
        } else {
            0
        };
        entity.set_synced_data(pumpkin_data::tracked_data::nautilus::DATA_FLAGS_ID, flags);
        entity.set_synced_data(
            pumpkin_data::tracked_data::nautilus::DATA_OWNERUUID_ID,
            owner,
        );
    }

    pub fn get_ambient_sound(&self) -> Sound {
        let is_baby = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0;
        let is_water = self.get_entity().is_under_water();
        if is_baby {
            if is_water {
                Sound::EntityBabyNautilusAmbient
            } else {
                Sound::EntityBabyNautilusAmbientLand
            }
        } else if is_water {
            Sound::EntityNautilusAmbient
        } else {
            Sound::EntityNautilusAmbientLand
        }
    }

    pub fn get_dash_sound(&self) -> Sound {
        let is_water = self.get_entity().is_under_water();
        if is_water {
            Sound::EntityNautilusDash
        } else {
            Sound::EntityNautilusDashLand
        }
    }

    pub fn get_dash_ready_sound(&self) -> Sound {
        let is_water = self.get_entity().is_under_water();
        if is_water {
            Sound::EntityNautilusDashReady
        } else {
            Sound::EntityNautilusDashReadyLand
        }
    }

    pub fn get_eat_sound(&self) -> Sound {
        let is_baby = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0;
        if is_baby {
            Sound::EntityBabyNautilusEat
        } else {
            Sound::EntityNautilusEat
        }
    }
}

impl Animal for NautilusEntity {
    /// Vanilla `Nautilus#getBreedOffspring`: a tame parent makes a tame baby with the same owner.
    fn get_breed_offspring(&self, _partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::NAUTILUS,
        ));
        if self.is_tame() {
            baby.set_tame(true, self.owner.load());
        }
        baby
    }

    fn play_eating_sound(&self) {
        self.make_sound(if self.mob_entity.living_entity.is_baby() {
            Sound::EntityBabyNautilusEat
        } else {
            Sound::EntityNautilusEat
        });
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        // Vanilla `AbstractNautilus#isFood`: a wild adult is tempted and tamed with
        // the taming tag, every other nautilus eats the food tag.
        let item_tag = if !self.is_tame() && !self.mob_entity.living_entity.is_baby() {
            &tag::Item::MINECRAFT_NAUTILUS_TAMING_ITEMS
        } else {
            &tag::Item::MINECRAFT_NAUTILUS_FOOD
        };
        item_stack.item.has_tag(item_tag)
    }
}

impl CustomSound for NautilusEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        let is_water = entity.is_under_water();
        Some(if is_baby {
            if is_water {
                Sound::EntityBabyNautilusHurt
            } else {
                Sound::EntityBabyNautilusHurtLand
            }
        } else if is_water {
            Sound::EntityNautilusHurt
        } else {
            Sound::EntityNautilusHurtLand
        })
    }

    fn death_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        let is_water = entity.is_under_water();
        Some(if is_baby {
            if is_water {
                Sound::EntityBabyNautilusDeath
            } else {
                Sound::EntityBabyNautilusDeathLand
            }
        } else if is_water {
            Sound::EntityNautilusDeath
        } else {
            Sound::EntityNautilusDeathLand
        })
    }
}

impl crate::entity::ageable::AgeableMob for NautilusEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Mob for NautilusEntity {
    fn as_ageable(&self) -> Option<&dyn crate::entity::ageable::AgeableMob> {
        Some(self)
    }

    /// Vanilla `AbstractNautilus#checkSpawnObstruction`: only unobstructed water.
    fn check_spawn_obstruction(&self, world: &crate::world::World) -> bool {
        world.is_unobstructed(self.get_entity())
    }

    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot == EquipmentSlot::BODY
            || *slot == EquipmentSlot::SADDLE
            || self.mob_entity.can_pick_up_loot()
    }

    fn is_tamed(&self) -> bool {
        self.is_tame()
    }

    fn get_owner_uuid(&self) -> Option<Uuid> {
        self.owner.load()
    }

    fn mob_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.saddled_controlling_passenger(None)
    }

    /// Vanilla `AbstractNautilus#isPushedByFluid` is `false`.
    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn requires_custom_persistence(&self) -> bool {
        self.get_entity().has_vehicle() || self.is_tame()
    }

    /// Vanilla `AbstractNautilus#removeWhenFarAway` always despawns wild ones.
    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        true
    }

    // Vanilla `PlayerRideableJumping.canJump` is `isSaddled()`; the jump input path
    // already guarantees a controlling rider, so the saddle gate is equivalent here.
    fn can_rider_jump(&self) -> bool {
        self.saddled_controlling_passenger(None).is_some()
    }

    fn handle_rider_start_jump(&self, jump_scale: i32) {
        if self.dash_cooldown.load(Ordering::Relaxed) > 0 {
            return;
        }
        let entity = &self.mob_entity.living_entity.entity;
        entity.world.load().play_sound(
            self.get_dash_sound(),
            SoundCategory::Neutral,
            &entity.pos.load(),
        );
        let momentum = if entity.is_in_water() { 1.2 } else { 0.5 };
        let amount = f64::from(player_jump_pending_scale(jump_scale));
        // Vanilla `executeRidersJump` dashes along the rider's `getLookAngle()`,
        // scaled by the movement speed attribute and the block speed factor.
        let speed = self
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::MOVEMENT_SPEED);
        let factor = f64::from(self.get_block_speed_factor());
        let look = self.saddled_controlling_passenger(None).map_or_else(
            || entity.get_looking_vector(),
            |rider| rider.get_looking_vector(),
        );
        entity.add_velocity(look * (momentum * amount * speed * factor));
        self.dash_cooldown.store(40, Ordering::Relaxed);
        self.set_dashing(true);
    }

    fn mob_get_swim_sound(&self) -> Option<Sound> {
        let is_baby = self
            .mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0;
        Some(if is_baby {
            Sound::EntityBabyNautilusSwim
        } else {
            Sound::EntityNautilusSwim
        })
    }

    /// Vanilla `AbstractNautilus` steps silently.
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {}

    /// Vanilla `Nautilus#baseTick`: the generic air rules run first, then the land dry-out rule.
    fn base_tick(&self, caller: &dyn EntityBase) {
        let pre_tick_air_supply = self.get_entity().air_supply.load(Ordering::Relaxed);
        self.get_mob_entity().living_entity.tick_air_supply(caller);
        if !self.get_mob_entity().is_no_ai() {
            water_animal::handle_air_supply(
                self,
                pre_tick_air_supply,
                caller,
                self.get_entity().is_in_water(),
                MAX_AIR,
                DRY_OUT_DAMAGE,
                DamageType::DRY_OUT,
            );
        }
    }

    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        // Vanilla `TamableAnimal` persists only `Owner`; taming is derived from
        // it and the dash cooldown stays transient.
        if let Some(owner) = self.owner.load() {
            nbt.put_uuid("Owner", owner);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.mob_entity
            .read_legacy_saddle(nbt.get_bool("Saddled") == Some(true));
        match nbt.get_uuid("Owner") {
            Some(owner) => self.set_tame(true, Some(owner)),
            None => self.set_tame(false, None),
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(
            pumpkin_data::tracked_data::nautilus::DASH,
            self.is_dashing(),
        );
        let flags = if self.is_tame() {
            crate::entity::passive::tamable::TAME_FLAG as i8
        } else {
            0
        };
        entity.set_synced_data(pumpkin_data::tracked_data::nautilus::DATA_FLAGS_ID, flags);
        entity.set_synced_data(
            pumpkin_data::tracked_data::nautilus::DATA_OWNERUUID_ID,
            self.owner.load(),
        );
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        tick_abstract_nautilus(
            &self.mob_entity,
            &self.dash_cooldown,
            self.is_dashing(),
            |dashing| self.set_dashing(dashing),
            || self.get_dash_ready_sound(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        interact_abstract_nautilus(
            self,
            player,
            item_stack,
            self.is_tame(),
            |owner| self.set_tame(true, Some(owner)),
            || self.get_eat_sound(),
        )
    }

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        nautilus_can_use_slot(self, slot)
    }

    fn get_equip_sound(&self, slot: &EquipmentSlot) -> Option<Sound> {
        nautilus_equip_sound(self.get_entity(), slot)
    }
}

pub(super) fn tick_abstract_nautilus(
    mob_entity: &MobEntity,
    dash_cooldown: &AtomicI32,
    is_dashing: bool,
    set_dashing: impl FnOnce(bool),
    dash_ready_sound: impl FnOnce() -> Sound,
) {
    let entity = &mob_entity.living_entity.entity;

    if let Ok(passengers) = entity.passengers.try_lock()
        && let Some(passenger) = passengers.first()
        && let Some(player) = passenger.cast_any().downcast_ref::<Player>()
    {
        let world = entity.world.load();
        let has_effect = player
            .living_entity
            .has_effect(&StatusEffect::BREATH_OF_THE_NAUTILUS);
        if (!has_effect || world.get_world_age() % 40 == 0)
            && let Some(p) = world.get_player_by_uuid(player.gameprofile.id)
        {
            p.add_effect(Effect {
                effect_type: &StatusEffect::BREATH_OF_THE_NAUTILUS,
                duration: 60,
                amplifier: 0,
                ambient: true,
                show_particles: true,
                show_icon: true,
                blend: true,
            });
        }
    }

    if is_dashing && dash_cooldown.load(Ordering::Relaxed) < 35 {
        set_dashing(false);
    }

    let cooldown = dash_cooldown.load(Ordering::Relaxed);
    if cooldown > 0 {
        let next = cooldown - 1;
        dash_cooldown.store(next, Ordering::Relaxed);
        if next == 0 {
            entity.world.load().play_sound(
                dash_ready_sound(),
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
        }
    }

    if entity.touching_water.load(Ordering::Relaxed) {
        let velo = entity.velocity.load();
        let speed = velo.length();
        let prob = (speed * 2.0).clamp(0.15, 1.0);
        if rand::random::<f64>() < prob {
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.spawn_particle(
                pos + Vector3::new(0.0, 0.25, 0.0),
                Vector3::new(0.4, 0.4, 0.4),
                0.5,
                2,
                Particle::Bubble,
            );
        }
    }
}

pub(super) fn interact_abstract_nautilus<M: Animal>(
    mob: &M,
    player: &Arc<Player>,
    item_stack: &mut ItemStack,
    is_tame: bool,
    on_tamed: impl FnOnce(Uuid),
    eat_sound: impl FnOnce() -> Sound,
) -> bool {
    let mob_entity = mob.get_mob_entity();
    let entity = &mob_entity.living_entity.entity;

    let consume_nautilus_food = |item_stack: &mut ItemStack| {
        if item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_NAUTILUS_BUCKET_FOOD)
        {
            crate::item::items::bucket::fill_bucket_stack(
                item_stack,
                player,
                &pumpkin_data::item::Item::WATER_BUCKET,
            );
        } else {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        }
    };

    if !is_tame && mob.is_food(item_stack) {
        consume_nautilus_food(item_stack);
        let world = entity.world.load();
        if rand::random::<u32>().is_multiple_of(3) && mob.approve_taming(player) {
            on_tamed(player.gameprofile.id);
            mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stop();
            world.send_entity_status(
                entity,
                EntityStatus::TamingSucceeded,
                ActorEventID::TamingSucceeded,
            );
        } else {
            world.send_entity_status(
                entity,
                EntityStatus::TamingFailed,
                ActorEventID::TamingFailed,
            );
        }
        world.play_sound(eat_sound(), SoundCategory::Neutral, &entity.pos.load());
        return true;
    }

    if !item_stack.is_empty()
        && mob.is_food(item_stack)
        && mob_entity.living_entity.health.load() < mob_entity.living_entity.get_max_health()
    {
        let heal = item_stack
            .get_data_component::<pumpkin_data::data_component_impl::FoodImpl>()
            .map_or(1.0, |food| 2.0 * food.nutrition as f32);
        consume_nautilus_food(item_stack);
        mob_entity.living_entity.heal(heal);
        entity
            .world
            .load()
            .play_sound(eat_sound(), SoundCategory::Neutral, &entity.pos.load());
        return true;
    }

    if is_tame && !player.get_entity().is_sneaking() {
        if equip_on_target(mob, player, item_stack) {
            return true;
        }

        if !mob.is_food(item_stack) {
            let world = player.world();
            if let Some(vehicle) = world.get_entity_by_id(entity.entity_id)
                && let Some(passenger) = world.get_player_by_id(player.entity_id())
            {
                entity.add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
                return true;
            }
        }
    }

    mob.animal_interact(player, item_stack)
}

pub(super) fn nautilus_can_use_slot(mob: &dyn Mob, slot: &EquipmentSlot) -> bool {
    (*slot != EquipmentSlot::SADDLE && *slot != EquipmentSlot::BODY)
        || (mob.get_entity().is_alive()
            && mob.as_ageable().is_none_or(|a| !a.is_baby())
            && mob.is_tamed())
}

pub(super) fn nautilus_equip_sound(entity: &Entity, slot: &EquipmentSlot) -> Option<Sound> {
    if *slot != EquipmentSlot::SADDLE {
        return None;
    }
    Some(if entity.is_under_water() {
        Sound::ItemNautilusSaddleUnderwaterEquip
    } else {
        Sound::ItemNautilusSaddleEquip
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn taming_persists_as_vanilla_owner_only() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let nautilus = NautilusEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &pumpkin_data::entity::EntityType::NAUTILUS,
        ));
        let owner = Uuid::from_u128(7);
        nautilus.dash_cooldown.store(140, Ordering::Relaxed);
        nautilus.set_tame(true, Some(owner));

        let mut nbt = NbtCompound::new();
        nautilus.mob_write_nbt(&mut nbt);

        assert!(nbt.get_int("DashCooldown").is_none());
        assert!(nbt.get_bool("IsTame").is_none());
        assert_eq!(nbt.get_uuid("Owner"), Some(owner));

        let restored = NautilusEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &pumpkin_data::entity::EntityType::NAUTILUS,
        ));
        restored.mob_read_nbt(&nbt);
        assert!(restored.is_tame());

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn wild_adult_only_eats_taming_items() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let nautilus = NautilusEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &pumpkin_data::entity::EntityType::NAUTILUS,
        ));
        let cooked_cod = ItemStack::new(1, &pumpkin_data::item::Item::COOKED_COD);
        let pufferfish = ItemStack::new(1, &pumpkin_data::item::Item::PUFFERFISH);

        assert!(!nautilus.is_food(&cooked_cod));
        assert!(nautilus.is_food(&pufferfish));

        nautilus.set_tame(true, Some(Uuid::from_u128(1)));
        assert!(nautilus.is_food(&cooked_cod));

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn tame_parent_breeds_tame_baby_with_the_same_owner() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let nautilus = NautilusEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &pumpkin_data::entity::EntityType::NAUTILUS,
        ));
        let partner = NautilusEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &pumpkin_data::entity::EntityType::NAUTILUS,
        ));
        let owner = Uuid::from_u128(11);
        nautilus.set_tame(true, Some(owner));

        let offspring = nautilus.get_breed_offspring(partner.as_ref());
        let baby = offspring
            .cast_any()
            .downcast_ref::<NautilusEntity>()
            .ok_or("offspring is not a nautilus")?;
        assert!(baby.is_tame());
        assert_eq!(baby.owner.load(), Some(owner));

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn land_nautilus_dries_out_once_its_air_runs_out() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let nautilus = NautilusEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &pumpkin_data::entity::EntityType::NAUTILUS,
        ));
        nautilus
            .get_entity()
            .air_supply
            .store(-19, Ordering::Relaxed);

        Mob::base_tick(nautilus.as_ref(), nautilus.as_ref());

        assert_eq!(nautilus.get_entity().air_supply.load(Ordering::Relaxed), 0);
        world.level.shutdown().await;
        Ok(())
    }
}
