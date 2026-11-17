mod block_effects;
mod dismount;
mod passengers;
mod physics;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam::atomic::AtomicCell;

use crate::entity::player::Player;
use crate::entity::vehicle::minecart::container::{self, MinecartInventory};
use crate::entity::vehicle::vehicle::VehicleEntity;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::server::Server;
use crate::world::game_event::GameEventContext;

use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::CustomNameImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::translation;
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use block_effects::BoatMovement;
use physics::BoatStatus;

const FRONT_SEAT_OFFSET: f32 = 0.2;
const BACK_SEAT_OFFSET: f32 = -0.6;
const ANIMAL_SEAT_SHIFT: f32 = 0.2;
const CHEST_BOAT_CONTAINER_SIZE: usize = 27;
const TIME_TO_EJECT: f32 = 60.0;
const MAX_PASSENGERS: usize = 2;

#[must_use]
pub const fn is_chest_boat(entity_type: &EntityType) -> bool {
    entity_type.id == EntityType::ACACIA_CHEST_BOAT.id
        || entity_type.id == EntityType::BAMBOO_CHEST_RAFT.id
        || entity_type.id == EntityType::BIRCH_CHEST_BOAT.id
        || entity_type.id == EntityType::CHERRY_CHEST_BOAT.id
        || entity_type.id == EntityType::DARK_OAK_CHEST_BOAT.id
        || entity_type.id == EntityType::JUNGLE_CHEST_BOAT.id
        || entity_type.id == EntityType::MANGROVE_CHEST_BOAT.id
        || entity_type.id == EntityType::OAK_CHEST_BOAT.id
        || entity_type.id == EntityType::PALE_OAK_CHEST_BOAT.id
        || entity_type.id == EntityType::POPLAR_CHEST_BOAT.id
        || entity_type.id == EntityType::SPRUCE_CHEST_BOAT.id
}

pub struct BoatEntity {
    pub vehicle: VehicleEntity,
    ticks_underwater: AtomicCell<f32>,
    status: AtomicCell<Option<BoatStatus>>,
    water_level: AtomicCell<f64>,
    land_friction: AtomicCell<f32>,
    last_yd: AtomicCell<f64>,
    movements: Mutex<VecDeque<BoatMovement>>,
    left_paddle_moving: AtomicBool,
    right_paddle_moving: AtomicBool,
    inventory: Option<Arc<MinecartInventory>>,
}

impl BoatEntity {
    #[must_use]
    pub fn new(mut entity: Entity) -> Self {
        entity.blocks_building = true;
        let inventory = is_chest_boat(entity.entity_type)
            .then(|| Arc::new(MinecartInventory::new(CHEST_BOAT_CONTAINER_SIZE)));
        Self {
            vehicle: VehicleEntity::new(entity),
            ticks_underwater: AtomicCell::new(0.0),
            status: AtomicCell::new(None),
            water_level: AtomicCell::new(0.0),
            land_friction: AtomicCell::new(0.0),
            last_yd: AtomicCell::new(0.0),
            movements: Mutex::new(VecDeque::new()),
            left_paddle_moving: AtomicBool::new(false),
            right_paddle_moving: AtomicBool::new(false),
            inventory,
        }
    }

    pub fn open_custom_inventory_screen(&self, player: &Arc<Player>) -> bool {
        // Vanilla AbstractChestBoat.openCustomInventoryScreen opens only when the boat is
        // alive and either empty or ridden by the opener; ChestBoat.interact also rejects
        // spectators. A boat occupied by someone else cannot be looted.
        if player.is_spectator() || !self.vehicle.entity.is_alive() {
            return false;
        }
        let passengers = self
            .vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let opener_is_rider = passengers
            .iter()
            .any(|passenger| passenger.get_entity().entity_id == player.entity_id());
        let occupied_by_other = !passengers.is_empty() && !opener_is_rider;
        drop(passengers);
        if occupied_by_other {
            return false;
        }
        let Some(inventory) = &self.inventory else {
            return false;
        };
        let entity_type = self.vehicle.entity.entity_type;
        let bedrock_key = if entity_type.id == EntityType::BAMBOO_CHEST_RAFT.id {
            translation::bedrock::ENTITY_CHEST_RAFT_NAME
        } else {
            translation::bedrock::ENTITY_CHEST_BOAT_NAME
        };
        let title = TextComponent::translate_cross(
            format!("entity.minecraft.{}", entity_type.resource_name),
            bedrock_key,
            [],
        );
        let opened = container::open(&self.vehicle.entity, player, inventory, title, false);
        if opened {
            let world = self.vehicle.entity.world.load();
            world.emit_game_event_with_context(
                GameEvent::ContainerOpen.name(),
                self.vehicle.entity.pos.load(),
                &GameEventContext {
                    source_entity: Some(player.as_ref()),
                },
            );
        }
        opened
    }

    pub fn set_paddles(&self, left: bool, right: bool) {
        self.left_paddle_moving.store(left, Ordering::Relaxed);
        self.right_paddle_moving.store(right, Ordering::Relaxed);

        self.vehicle.entity.send_meta_data(
            &[
                Metadata::new(pumpkin_data::tracked_data::boat::ID_PADDLE_LEFT, left),
                Metadata::new(pumpkin_data::tracked_data::boat::ID_PADDLE_RIGHT, right),
            ],
            None,
        );
    }

    fn send_wobble_metadata(&self) {
        self.vehicle.send_wobble_metadata();
    }

    fn tick_underwater(&self, status: BoatStatus) {
        let ticks = if status.is_underwater() {
            self.ticks_underwater.load() + 1.0
        } else {
            0.0
        };
        self.ticks_underwater.store(ticks);
        if ticks < TIME_TO_EJECT {
            return;
        }

        let passengers = self
            .vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for passenger in passengers.iter().rev() {
            self.vehicle
                .entity
                .remove_passenger(passenger.get_entity().entity_id);
        }
    }

    fn is_client_controlled(&self) -> bool {
        // Vanilla `AbstractBoat.isControlledByLocalInstance` goes through
        // `getControllingPassenger`, i.e. the first passenger that is a `LivingEntity`.
        self.get_controlling_passenger()
            .is_some_and(|passenger| passenger.get_player().is_some())
    }

    pub(in crate::entity) fn can_push(&self, other: &Entity, is_boat: bool) -> bool {
        let bounds = self.vehicle.entity.bounding_box.load();
        let other_bottom = other.bounding_box.load().min.y;
        if is_boat {
            other_bottom < bounds.max.y
        } else {
            other_bottom <= bounds.min.y
        }
    }
}

impl EntityBase for BoatEntity {
    fn passenger_attachment_point(
        &self,
        passenger: &dyn EntityBase,
        index: usize,
    ) -> Option<Vector3<f64>> {
        let entity = self.get_entity();
        let seats = entity.entity_type.boat?;
        let mut offset = seats.single_passenger_x_offset;
        let passenger_count = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        if passenger_count > 1 {
            offset = if index == 0 {
                FRONT_SEAT_OFFSET
            } else {
                BACK_SEAT_OFFSET
            };
            if passenger
                .get_mob()
                .and_then(|mob| mob.as_animal())
                .is_some()
            {
                offset += ANIMAL_SEAT_SHIFT;
            }
        }
        Some(crate::entity::attachment_point(
            [0.0, seats.ride_height, f64::from(offset)],
            1.0,
            entity.yaw.load(),
        ))
    }

    fn get_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
            .filter(|passenger| passenger.get_living_entity().is_some())
            .cloned()
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(inventory) = &self.inventory {
            inventory.write_nbt(nbt);
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        if let Some(inventory) = &self.inventory {
            inventory.read_nbt(nbt);
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.vehicle.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn get_vehicle_entity(&self) -> Option<&VehicleEntity> {
        Some(&self.vehicle)
    }

    fn has_container_inventory(&self) -> bool {
        self.inventory.is_some()
    }

    fn get_container_inventory(&self) -> Option<Arc<dyn Inventory>> {
        let container = self.inventory.as_ref()?;
        if container.has_loot_table() {
            let world = self.vehicle.entity.world.load();
            container.unpack_loot(&crate::world::loot::LootContextParameters {
                world: Some(&world),
                position: Some(self.vehicle.entity.pos.load()),
                ..Default::default()
            });
        }
        Some(container.clone() as Arc<dyn Inventory>)
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        let status = self.get_status();
        let old_status = self.status.swap(Some(status));
        self.tick_underwater(status);
        self.vehicle.tick();
        let entity = &self.vehicle.entity;
        entity.tick(caller, server);
        if entity.is_removed() {
            return;
        }

        // Vanilla leaves ridden boat movement to its controlling player's client.
        if self.is_client_controlled() {
            entity.velocity.store(Vector3::default());
        } else {
            if self.left_paddle_moving.load(Ordering::Relaxed)
                || self.right_paddle_moving.load(Ordering::Relaxed)
            {
                self.set_paddles(false, false);
            }
            let surface_offset = self.float_boat(old_status, status);
            self.move_boat(caller, surface_offset);
        }
        self.apply_effects_from_blocks();
        entity.tick_block_collisions(caller);
    }

    fn init_data_tracker(&self) {
        self.send_wobble_metadata();
    }

    fn can_hit(&self) -> bool {
        self.vehicle.entity.is_alive()
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        let creative = source
            .and_then(EntityBase::get_player)
            .is_some_and(|player| player.gamemode.load() == GameMode::Creative);
        let will_break = self.vehicle.entity.is_alive()
            && (creative || self.vehicle.get_damage() + amount * 10.0 > 40.0);

        let damaged = self.vehicle.damage_with_context(amount, source);

        // Vanilla `AbstractBoat#hurt`: a destroying break runs `destroy` (chest contents plus the
        // boat item) only when it is not an instabuild hit, so a creative break drops nothing.
        if will_break && !creative && self.vehicle.entity.is_removed() {
            let world = self.vehicle.entity.world.load();
            if world.level_info.load().game_rules.entity_drops {
                let position = self.vehicle.entity.block_pos.load();
                if let Some(container) = &self.inventory
                    && container.claim_drops()
                {
                    container.unpack_loot(&crate::world::loot::LootContextParameters {
                        world: Some(&world),
                        position: Some(self.vehicle.entity.pos.load()),
                        ..Default::default()
                    });
                    let inventory: Arc<dyn Inventory> = container.clone();
                    world.scatter_inventory_at(self.vehicle.entity.pos.load(), &inventory);
                }
                let resource_name = self.vehicle.entity.entity_type.resource_name;
                if let Some(item) = Item::from_registry_key(resource_name) {
                    let mut stack = ItemStack::new(1, item);
                    if let Some(custom_name) = self.vehicle.entity.custom_name.load().as_ref() {
                        stack.set_data_component(CustomNameImpl {
                            name: custom_name.clone(),
                        });
                    }
                    world.drop_stack(&position, stack);
                }
            }
        }

        damaged
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        // Vanilla `AbstractBoat.interact` gates boarding on `outOfControlTicks < 60`
        // and secondary use only; it never probes the eye fluid.
        let can_add_passenger = self.ticks_underwater.load() < TIME_TO_EJECT
            && !player.get_entity().has_vehicle()
            && self
                .vehicle
                .entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len()
                < MAX_PASSENGERS;

        if self.inventory.is_some() && (!can_add_passenger || player.get_entity().is_sneaking()) {
            return self.open_custom_inventory_screen(player);
        }

        if player.get_entity().is_sneaking() || !can_add_passenger {
            return false;
        }

        let world = self.vehicle.entity.world.load();
        let Some(vehicle) = world.get_entity_by_id(self.vehicle.entity.entity_id) else {
            return false;
        };

        let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
            return false;
        };

        self.vehicle
            .entity
            .add_passenger(vehicle, passenger as Arc<dyn EntityBase>);

        self.vehicle.entity.has_passenger(player.entity_id())
    }

    fn set_paddle_state(&self, left: bool, right: bool) {
        self.set_paddles(left, right);
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_pushable(&self) -> bool {
        true
    }

    fn get_gravity(&self) -> f64 {
        if self.vehicle.entity.has_no_gravity() {
            0.0
        } else {
            physics::DEFAULT_GRAVITY
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_chest_boat;
    use pumpkin_data::entity::EntityType;

    #[test]
    fn chest_boat_detection_covers_every_chest_boat_entity_type() {
        for entity_type in EntityType::ALL {
            let expected = entity_type.resource_name.ends_with("_chest_boat")
                || entity_type.resource_name == "bamboo_chest_raft";
            assert_eq!(
                is_chest_boat(entity_type),
                expected,
                "{}",
                entity_type.resource_name
            );
        }
    }
}
