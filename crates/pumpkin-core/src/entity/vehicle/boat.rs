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

use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::CustomNameImpl;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::translation;
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::text::TextComponent;

use block_effects::BoatMovement;
use physics::BoatStatus;

const FRONT_SEAT_OFFSET: f32 = 0.2;
const BACK_SEAT_OFFSET: f32 = -0.6;
const ANIMAL_SEAT_SHIFT: f32 = 0.2;
const CHEST_BOAT_CONTAINER_SIZE: usize = 27;
const TIME_TO_EJECT: f32 = 60.0;
const BUBBLE_COLUMN_DELAY: i32 = 60;
const BOAT_MAX_PASSENGERS: usize = 2;
const CHEST_BOAT_MAX_PASSENGERS: usize = 1;
const PADDLE_STEP: f32 = std::f32::consts::FRAC_PI_8;
const PADDLE_CUE: f32 = std::f32::consts::FRAC_PI_4;

/// Vanilla AbstractBoat.tick plays a stroke as the paddle position crosses PI/4 inside a
/// single PI/8 step, so a continuously paddling boat strokes twice per revolution.
const fn paddle_sound_cue(position: f32) -> bool {
    position % std::f32::consts::TAU <= PADDLE_CUE
        && (position + PADDLE_STEP) % std::f32::consts::TAU >= PADDLE_CUE
}

/// Vanilla AbstractBoat.getPaddleSound: water in fluid, land beached, silent in the air.
const fn paddle_sound_for_status(status: BoatStatus) -> Option<Sound> {
    match status {
        BoatStatus::InWater | BoatStatus::UnderWater | BoatStatus::UnderFlowingWater => {
            Some(Sound::EntityBoatPaddleWater)
        }
        BoatStatus::OnLand => Some(Sound::EntityBoatPaddleLand),
        BoatStatus::InAir => None,
    }
}

/// Checks whether an entity type is a chest boat.
///
/// Vanilla distinguishes `ChestBoat` from Boat by class. Generated entity resource names
/// carry that distinction in the _`chest_boat` / `bamboo_chest_raft` suffix, so a new wood
/// needs no extra allow-list entry here.
#[must_use]
pub fn is_chest_boat(entity_type: &EntityType) -> bool {
    entity_type.resource_name.ends_with("_chest_boat")
        || entity_type.resource_name == "bamboo_chest_raft"
}

/// Vanilla `AbstractChestBoat` always allocates a 27-slot container while a plain `AbstractBoat`
/// has none; this is the sole input to `has_container_inventory` for boats.
#[must_use]
pub fn boat_container_size(entity_type: &EntityType) -> Option<usize> {
    is_chest_boat(entity_type).then_some(CHEST_BOAT_CONTAINER_SIZE)
}

#[must_use]
pub fn is_boat(entity_type: &EntityType) -> bool {
    entity_type.has_tag(&tag::EntityType::C_BOATS) || is_chest_boat(entity_type)
}

/// Vanilla `AbstractBoat.getMaxPassengers` returns 2, `AbstractChestBoat` overrides it with 1.
#[must_use]
pub fn max_passengers(entity_type: &EntityType) -> usize {
    if is_chest_boat(entity_type) {
        CHEST_BOAT_MAX_PASSENGERS
    } else {
        BOAT_MAX_PASSENGERS
    }
}

/// Vanilla `AbstractBoat.getDropItem`: prefer the explicit boat item mapping, then the registry
/// key fallback shared with the pickup path.
#[must_use]
pub fn boat_drop_item(entity_type: &EntityType) -> Option<&'static Item> {
    crate::item::items::boat::BoatItem::entity_to_item(entity_type)
        .or_else(|| Item::from_registry_key(entity_type.resource_name))
}

/// Reports whether destroying a boat drops its item.
///
/// Vanilla `VehicleEntity.hurtServer`: an instabuild hit discards the boat without calling
/// `destroy`, so no item drops even when it carries a custom name (a name is only copied in the
/// non-creative `destroy` path).
#[must_use]
pub const fn drops_item_when_destroyed(creative: bool, _named: bool) -> bool {
    !creative
}

/// Vanilla `AbstractBoat.canAddPassenger`: a boat whose eye is in water takes no passenger.
#[must_use]
pub fn is_eye_in_water(entity: &Entity) -> bool {
    let pos = entity.pos.load();
    let eye_y = pos.y + entity.get_eye_height();
    let eye_pos = BlockPos::floored(pos.x, eye_y, pos.z);
    let world = entity.world.load();
    let (fluid, state) = world.get_fluid_and_fluid_state(&eye_pos);
    fluid.has_tag(&tag::Fluid::MINECRAFT_WATER)
        && eye_y < f64::from(eye_pos.0.y as f32 + world.get_fluid_height(&eye_pos, fluid, &state))
}

/// Vanilla AbstractChestBoat.interact: the container is offered when the ride attempt did
/// not consume the click, i.e. on secondary use or when no seat is free.
const fn chest_opens_container(
    is_sneaking: bool,
    can_add_passenger: bool,
    has_inventory: bool,
) -> bool {
    has_inventory && (is_sneaking || !can_add_passenger)
}

/// Server-side outcome of a boat click, mirroring the vanilla `interact` chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoatInteractOutcome {
    Mount,
    OpenContainer,
    Nothing,
}

/// Pure model of vanilla `AbstractBoat.interact` followed by `AbstractChestBoat.interact`.
///
/// Vanilla `canAddPassenger` checks only the seat count and whether the eye is in water. The
/// out-of-control counter and an existing vehicle gate `startRiding`, so neither may make a
/// chest boat open its container on a plain use.
#[must_use]
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "pure mirror of the vanilla interact chain; each bool is a distinct vanilla condition"
)]
const fn boat_interact_outcome(
    secondary_use: bool,
    out_of_control: bool,
    eye_in_water: bool,
    player_has_vehicle: bool,
    seat_available: bool,
    has_container: bool,
) -> BoatInteractOutcome {
    let can_add_passenger = seat_available && !eye_in_water;
    if can_add_passenger && !secondary_use && !out_of_control && !player_has_vehicle {
        BoatInteractOutcome::Mount
    } else if chest_opens_container(secondary_use, can_add_passenger, has_container) {
        BoatInteractOutcome::OpenContainer
    } else {
        BoatInteractOutcome::Nothing
    }
}

/// Vanilla AbstractBoat.getPassengerAttachmentPoint seat selection. Shared by the engine's
/// `passenger_attachment_point` and the mount-positioning path so the two cannot drift.
fn passenger_seat_offset(
    single_passenger_x_offset: f32,
    index: usize,
    passenger_count: usize,
    is_animal: bool,
) -> f32 {
    if passenger_count > 1 {
        let mut offset = if index == 0 {
            FRONT_SEAT_OFFSET
        } else {
            BACK_SEAT_OFFSET
        };
        if is_animal {
            offset += ANIMAL_SEAT_SHIFT;
        }
        offset
    } else {
        single_passenger_x_offset
    }
}

/// Full passenger attachment point matching vanilla AbstractBoat.getPassengerAttachmentPoint,
/// including the yRot(-getYRot * `DEG_TO_RAD`) applied to (0, rideHeight, offset).
pub(super) fn passenger_attachment(
    entity: &Entity,
    index: usize,
    passenger_count: usize,
    is_animal: bool,
) -> Option<Vector3<f64>> {
    let seats = entity.entity_type.boat?;
    let offset = passenger_seat_offset(
        seats.single_passenger_x_offset,
        index,
        passenger_count,
        is_animal,
    );
    Some(crate::entity::attachment_point(
        [0.0, seats.ride_height, f64::from(offset)],
        1.0,
        entity.yaw.load(),
    ))
}

pub struct BoatEntity {
    pub vehicle: VehicleEntity,
    ticks_underwater: AtomicCell<f32>,
    status: AtomicCell<Option<BoatStatus>>,
    water_level: AtomicCell<f64>,
    land_friction: AtomicCell<f32>,
    last_yd: AtomicCell<f64>,
    last_yaw: AtomicCell<f32>,
    yaw_delta: AtomicCell<f32>,
    movements: Mutex<VecDeque<BoatMovement>>,
    left_paddle_moving: AtomicBool,
    right_paddle_moving: AtomicBool,
    paddle_positions: [AtomicCell<f32>; 2],
    bubble_column_time: AtomicCell<i32>,
    bubble_column_drag_down: AtomicBool,
    bubble_column_above: AtomicBool,
    inventory: Option<Arc<MinecartInventory>>,
}

impl BoatEntity {
    #[must_use]
    pub fn new(mut entity: Entity) -> Self {
        entity.blocks_building = true;
        let yaw = entity.yaw.load();
        let inventory = boat_container_size(entity.entity_type)
            .map(|size| Arc::new(MinecartInventory::new(size)));
        Self {
            vehicle: VehicleEntity::new(entity),
            ticks_underwater: AtomicCell::new(0.0),
            status: AtomicCell::new(None),
            water_level: AtomicCell::new(0.0),
            land_friction: AtomicCell::new(0.0),
            last_yd: AtomicCell::new(0.0),
            last_yaw: AtomicCell::new(yaw),
            yaw_delta: AtomicCell::new(0.0),
            movements: Mutex::new(VecDeque::new()),
            left_paddle_moving: AtomicBool::new(false),
            right_paddle_moving: AtomicBool::new(false),
            paddle_positions: [AtomicCell::new(0.0), AtomicCell::new(0.0)],
            bubble_column_time: AtomicCell::new(0),
            bubble_column_drag_down: AtomicBool::new(false),
            bubble_column_above: AtomicBool::new(false),
            inventory,
        }
    }

    /// Vanilla `AbstractChestBoat.remove`: scatter the chest slots on any destroying removal
    /// (break, void, `/kill`, discard). `claim_drops` makes the scatter run at most once.
    fn scatter_chest_contents(&self) {
        let Some(container) = &self.inventory else {
            return;
        };
        if !container.claim_drops() {
            return;
        }
        let world = self.vehicle.entity.world.load();
        let position = self.vehicle.entity.pos.load();
        container.unpack_loot(&crate::world::loot::LootContextParameters {
            world: Some(&world),
            position: Some(position),
            ..Default::default()
        });
        let inventory: Arc<dyn Inventory> = container.clone();
        world.scatter_inventory_at(position, &inventory);
    }

    pub fn open_custom_inventory_screen(&self, player: &Arc<Player>) -> bool {
        // Vanilla AbstractChestBoat.openCustomInventoryScreen has no rider, spectator or
        // alive check: player.openMenu(this) succeeds even when another player rides the
        // boat, and a spectator can view a chest boat that has no loot table. Pumpkin mirrors
        // vanilla's only spectator restriction in the screen-handler layer, which refuses
        // loot-table containers to spectators, so do not gate spectators here. The alive check
        // covers a boat removed before the screen opens; ongoing validity is handled by the
        // screen's validity closure.
        if !self.vehicle.entity.is_alive() {
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
        // Vanilla `AbstractChestBoat.interact` emits CONTAINER_OPEN with the player as the
        // vibration source (`GameEvent.Context.of(player)`), then angers nearby piglins.
        let opened = container::open(&self.vehicle.entity, player, inventory, title, false);
        if opened {
            let world = self.vehicle.entity.world.load_full();
            world.emit_game_event_with_source(
                pumpkin_data::game_event::GameEvent::ContainerOpen.name(),
                self.vehicle.entity.pos.load(),
                Some(player.clone()),
            );
            crate::entity::mob::piglin_ai::PiglinAi::on_guarded_block_interacted(&world, player);
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

    /// Vanilla AbstractBoat.tick paddle-stroke loop: the sound is placed one unit to each side
    /// of the hull, perpendicular to the boat heading, with a randomised pitch.
    fn play_paddle_sounds(&self) {
        let entity = &self.vehicle.entity;
        if entity.is_silent() {
            return;
        }
        let Some(sound) = self.status.load().and_then(paddle_sound_for_status) else {
            return;
        };
        // Vanilla getViewVector(1.0F) for a level boat is (-sin(yaw), 0, cos(yaw)); the
        // stroke offsets use its perpendicular, side 0 to starboard and side 1 to port.
        let yaw = f64::from(entity.yaw.load()).to_radians();
        let pos = entity.pos.load();
        let world = entity.world.load();
        for (side, position) in self.paddle_positions.iter().enumerate() {
            let moving = if side == 0 {
                self.left_paddle_moving.load(Ordering::Relaxed)
            } else {
                self.right_paddle_moving.load(Ordering::Relaxed)
            };
            if !moving {
                position.store(0.0);
                continue;
            }
            let stroke = position.load();
            if paddle_sound_cue(stroke) {
                let (dx, dz) = if side == 0 {
                    (yaw.cos(), yaw.sin())
                } else {
                    (-yaw.cos(), -yaw.sin())
                };
                let pitch = 0.8 + rand::random::<f32>() * 0.4;
                world.play_sound_fine(
                    sound,
                    SoundCategory::Neutral,
                    &Vector3::new(pos.x + dx, pos.y, pos.z + dz),
                    1.0,
                    pitch,
                );
            }
            position.store(stroke + PADDLE_STEP);
        }
    }

    fn send_wobble_metadata(&self) {
        self.vehicle.send_wobble_metadata();
    }

    fn tick_underwater(&self, status: BoatStatus) {
        // Vanilla `AbstractBoat.tick` keeps `outOfControlTicks` for the 60-tick rider eject and
        // the boarding gate; only the bubble-column path writes `DATA_ID_BUBBLE_TIME`, so a
        // submerged boat must not overwrite it from this counter.
        let previous = self.ticks_underwater.load();
        let ticks = if status.is_underwater() {
            previous + 1.0
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
        // Vanilla `AbstractBoat.getControllingPassenger` returns the first passenger only when
        // it is a `Player` (otherwise `super` returns null), so a mob rider never steers.
        self.get_controlling_passenger().is_some()
    }

    /// Vanilla `AbstractBoat.tickBubbleColumn` applies the launch/sink impulse only after the
    /// 60-tick counter has elapsed, then clears it ready for the next contact.
    fn tick_bubble_column(&self) {
        let entity = &self.vehicle.entity;
        // Vanilla `tickBubbleColumn` drops the countdown as soon as the boat is no longer above a
        // column, so leaving the column cancels the pending launch or sink.
        if !self.bubble_column_above.swap(false, Ordering::Relaxed) {
            if self.bubble_column_time.load() != 0 {
                self.bubble_column_time.store(0);
                entity.set_synced_data(pumpkin_data::tracked_data::boat::ID_BUBBLE_TIME, 0i32);
            }
            return;
        }
        let time = self.bubble_column_time.load();
        if time <= 0 {
            return;
        }
        let remaining = time - 1;
        self.bubble_column_time.store(remaining);
        entity.set_synced_data(pumpkin_data::tracked_data::boat::ID_BUBBLE_TIME, remaining);
        entity.velocity_dirty.store(true, Ordering::SeqCst);
        if remaining > 0 {
            return;
        }
        let drag_down = self.bubble_column_drag_down.load(Ordering::Relaxed);
        // Vanilla ejects riders on a downward column, then broadcasts BOAT_SINK/BOAT_LAUNCH so
        // tracking clients reproduce the impulse; only the server-authoritative velocity is
        // skipped for a ridden boat (Pumpkin leaves that movement to the controlling client).
        if drag_down {
            entity.eject_passengers();
        }
        let world = entity.world.load();
        world.broadcast_entity_event(
            entity,
            if drag_down {
                EntityStatus::BoatSink
            } else {
                EntityStatus::BoatLaunch
            },
            ActorEventID::None,
        );
        if self.is_client_controlled() {
            return;
        }
        let mut velocity = entity.velocity.load();
        velocity.y = bubble_column_vertical_velocity(drag_down, velocity.y);
        entity.velocity.store(velocity);
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
    fn on_removed(&self) {
        // Vanilla `AbstractChestBoat.remove` scatters the container only when the removal reason
        // destroys the entity (KILLED / DISCARDED), not on chunk unload or dimension change.
        // `World::remove_entity` records the reason before invoking this hook.
        if self
            .vehicle
            .entity
            .removal_reason
            .load()
            .is_some_and(|reason| reason.should_destroy())
        {
            // Vanilla `AbstractBoat.remove` drops the lead on a destroying removal, so a broken
            // or /kill-ed leashed boat never strands its leash on a gone holder.
            if self.vehicle.entity.is_leashed() {
                self.vehicle.entity.drop_leash();
            }
            self.scatter_chest_contents();
        }
    }

    fn passenger_attachment_point(
        &self,
        passenger: &dyn EntityBase,
        index: usize,
    ) -> Option<Vector3<f64>> {
        let entity = self.get_entity();
        let passenger_count = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        let is_animal = passenger
            .get_mob()
            .and_then(|mob| mob.as_animal())
            .is_some();
        passenger_attachment(entity, index, passenger_count, is_animal)
    }

    fn get_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
            .filter(|passenger| passenger.get_player().is_some())
            .cloned()
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        // Vanilla AbstractBoat.addAdditionalSaveData writes the leash before
        // AbstractChestBoat appends the container items.
        self.vehicle.entity.write_leash_data(nbt);
        if let Some(inventory) = &self.inventory {
            inventory.write_nbt(nbt);
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.vehicle.entity.read_leash_data(nbt);
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

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let status = self.get_status();
        let old_status = self.status.swap(Some(status));
        self.tick_underwater(status);
        self.vehicle.tick();
        let entity = &self.vehicle.entity;
        // Vanilla `AbstractBoat.deltaRotation` is the per-tick yaw change from steering.
        // Pumpkin's boat yaw comes from the client's vehicle-move packet, so derive the
        // same value from consecutive server-side yaw samples for non-player riders.
        let yaw = entity.yaw.load();
        let delta = wrap_degrees(yaw - self.last_yaw.load());
        self.last_yaw.store(yaw);
        self.yaw_delta.store(delta);
        entity.update_last_pos();
        // Vanilla Boat.tick starts with super.tick -> Entity.baseTick, which applies the fluid
        // current to every boat. A player-steered boat leaves movement to its client, so the
        // server skips the push there to avoid fighting it; empty and mob-ridden boats are
        // server authoritative and must drift with the current.
        if self.is_client_controlled() {
            entity.base_tick_without_fluid_push(caller);
        } else {
            entity.base_tick(caller);
        }
        if entity.is_removed() {
            return;
        }

        // Vanilla `Entity.baseTick` ticks the leash for every leashable entity; boats are leashable.
        let _ = entity.tick_leash(caller);

        // Vanilla leaves ridden boat movement to its controlling player's client.
        if self.is_client_controlled() {
            entity.velocity.store(Vector3::default());
            // Vanilla zeroes a client-authoritative boat's delta movement, and its
            // `checkFallDamage` override stores that zero into `lastYd` from each
            // vehicle-move packet. Mirror it so `get_water_level_above` never reads a
            // pre-mount velocity once the boat becomes server-controlled again.
            self.last_yd.store(0.0);
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
        self.tick_bubble_column();
        self.play_paddle_sounds();
        entity.tick_block_collisions(caller);
    }

    fn init_data_tracker(&self) {
        self.send_wobble_metadata();
    }

    fn can_hit(&self) -> bool {
        self.vehicle.entity.is_alive()
    }

    // Vanilla `AbstractBoat.canBeCollidedWith` returns true unconditionally.
    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn on_above_bubble_column(&self, drag_down: bool, _pos: &BlockPos) {
        // Vanilla `AbstractBoat.onAboveBubbleColumn` arms the countdown once; the impulse
        // comes later from `tickBubbleColumn`, not on every tick of contact.
        self.bubble_column_drag_down
            .store(drag_down, Ordering::Relaxed);
        self.bubble_column_above.store(true, Ordering::Relaxed);
        if self.bubble_column_time.load() == 0 {
            self.bubble_column_time.store(BUBBLE_COLUMN_DELAY);
            self.vehicle.entity.set_synced_data(
                pumpkin_data::tracked_data::boat::ID_BUBBLE_TIME,
                BUBBLE_COLUMN_DELAY,
            );
        }
        self.vehicle
            .entity
            .velocity_dirty
            .store(true, Ordering::SeqCst);

        // Vanilla `AbstractBoat.onAboveBubbleColumn` also emits GameEvent.SPLASH at a 1% roll
        // while the boat is not fully submerged, using the controlling passenger as the
        // vibration source, alongside the splash sound and particle.
        if !self.status.load().is_some_and(BoatStatus::is_underwater)
            && rand::random::<u32>().is_multiple_of(100)
        {
            let entity = &self.vehicle.entity;
            let pos = entity.pos.load();
            let world = entity.world.load_full();
            world.emit_game_event_with_source(
                pumpkin_data::game_event::GameEvent::Splash.name(),
                pos,
                self.get_controlling_passenger(),
            );
            world.play_sound_fine(
                Sound::EntityGenericSplash,
                SoundCategory::Neutral,
                &pos,
                1.0,
                0.8 + rand::random::<f32>() * 0.4,
            );
            world.spawn_particle(
                Vector3::new(
                    pos.x + rand::random::<f64>(),
                    pos.y + 0.7,
                    pos.z + rand::random::<f64>(),
                ),
                Vector3::new(0.0, 0.0, 0.0),
                0.0,
                1,
                Particle::Splash,
            );
        }
    }

    fn on_inside_bubble_column(&self, drag_down: bool) {
        // Vanilla boats do not override `onInsideBubbleColumn`, so the capped entity push applies.
        let entity = &self.vehicle.entity;
        crate::block::blocks::bubble_column::push_entity(entity, drag_down, false);
        entity.velocity_dirty.store(true, Ordering::SeqCst);
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        // Vanilla `VehicleEntity.hurtServer` checks `isRemoved` before `isInvulnerableToBase`,
        // so an already-removed boat neither mutates damage nor drops its contents again.
        if self.vehicle.entity.is_removed() {
            return true;
        }
        // Datapack invulnerability and fire immunity apply before any wobble or chest contents.
        if self.vehicle.entity.is_invulnerable_to(&damage_type, cause) {
            return false;
        }
        // Vanilla `VehicleEntity.hurtServer` has no passenger-immunity branch: a rider's own
        // projectile can damage the boat it rides, so no such early return is added here.
        let creative = source
            .and_then(EntityBase::get_player)
            .is_some_and(|player| player.gamemode.load() == GameMode::Creative);
        let was_alive = self.vehicle.entity.is_alive();

        let damaged = self.vehicle.damage_with_context(amount, source);

        // Vanilla `VehicleEntity.hurtServer`: a destroying break runs `destroy` (chest contents
        // plus the named boat item); an instabuild hit discards directly, dropping nothing.
        // The break is read from the post-hit removal state so it cannot drift from the vehicle
        // destroy threshold.
        let named = self.vehicle.entity.custom_name.load().is_some();
        if was_alive
            && self.vehicle.entity.is_removed()
            && drops_item_when_destroyed(creative, named)
        {
            let world = self.vehicle.entity.world.load();
            // Vanilla `AbstractChestBoat.remove` scatters the 27 slots on any destroying
            // removal, independent of `doEntityDrops`; only the boat item is gamerule-gated.
            self.scatter_chest_contents();
            if world.level_info.load().game_rules.entity_drops {
                // Vanilla `AbstractBoat.destroy` skips the item drop for explosion damage and
                // copies the entity custom name onto the dropped item.
                if !damage_type.has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_IS_EXPLOSION)
                    && let Some(item) = boat_drop_item(self.vehicle.entity.entity_type)
                {
                    let mut stack = ItemStack::new(1, item);
                    if let Some(custom_name) = self.vehicle.entity.custom_name.load().as_ref() {
                        stack.set_data_component(CustomNameImpl {
                            name: custom_name.clone(),
                        });
                    }
                    world.drop_stack_at(self.vehicle.entity.pos.load(), stack);
                }
            }
        }

        damaged
    }

    fn interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        // Vanilla `Entity.interact` handles leads for every `Leashable`; `AbstractBoat`
        // implements it, so a lead ties a boat to the player or unties it on second use.
        if item_stack.item == &Item::LEAD {
            let entity = &self.vehicle.entity;
            let world = entity.world.load();
            let holder = entity.get_leash_holder();
            if holder
                .as_ref()
                .is_some_and(|holder| holder.get_entity().entity_id == player.entity_id())
            {
                if !entity.unleash() {
                    return false;
                }
                if player.gamemode.load() != GameMode::Creative {
                    entity.spawn_at_location(ItemStack::new(1, &Item::LEAD));
                }
                world.play_sound(
                    Sound::ItemLeadUntied,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
                return true;
            }
            if holder.is_none() {
                let diff = entity.pos.load() - player.get_entity().pos.load();
                if diff.length_squared()
                    <= Entity::LEASH_SNAP_DISTANCE * Entity::LEASH_SNAP_DISTANCE
                {
                    let player_base = player.clone() as Arc<dyn EntityBase>;
                    if entity.leash_to(player_base) {
                        if player.gamemode.load() != GameMode::Creative {
                            item_stack.decrement(1);
                        }
                        world.play_sound(
                            Sound::ItemLeadTied,
                            SoundCategory::Neutral,
                            &entity.pos.load(),
                        );
                        return true;
                    }
                }
            }
        }

        // Vanilla `AbstractBoat.interact` then `AbstractChestBoat.interact` decide between
        // mounting, opening the container and doing nothing. Keep the server-side chain in one
        // pure helper so the out-of-control and already-riding cases never open a chest boat.
        let seat_available = self
            .vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
            < max_passengers(self.vehicle.entity.entity_type);
        let outcome = boat_interact_outcome(
            player.get_entity().is_sneaking(),
            self.ticks_underwater.load() >= TIME_TO_EJECT,
            is_eye_in_water(&self.vehicle.entity),
            player.get_entity().has_vehicle(),
            seat_available,
            self.inventory.is_some(),
        );
        match outcome {
            BoatInteractOutcome::OpenContainer => return self.open_custom_inventory_screen(player),
            BoatInteractOutcome::Nothing => return false,
            BoatInteractOutcome::Mount => {}
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
    fn on_rider_positioned(&self, passenger: &dyn EntityBase) {
        // Vanilla `AbstractBoat.positionRider` applies the turn delta and then `clampRotation`.
        // A player steers itself and an entity in `can_turn_in_boats` keeps its own rotation.
        let rider = passenger.get_entity();
        if passenger.get_player().is_some()
            || rider
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_CAN_TURN_IN_BOATS)
        {
            return;
        }
        let boat_yaw = self.get_entity().yaw.load();
        let passenger_count = self
            .get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        let body_yaw = rider_body_yaw(
            boat_yaw,
            passenger_count,
            max_passengers(self.get_entity().entity_type),
            passenger
                .get_mob()
                .is_some_and(|mob| mob.as_animal().is_some()),
            rider.entity_id,
        );
        let delta = self.yaw_delta.load();
        rider.body_yaw.store(body_yaw);
        let Some(mob) = passenger.get_mob() else {
            let yaw = clamp_rider_yaw(rider.yaw.load() + delta, body_yaw);
            rider.yaw.store(yaw);
            rider.head_yaw.store(yaw);
            return;
        };
        // Vanilla `Mob` riders take the hull heading as body yaw (`setYRot` then
        // `clampHeadRotationToBody`), so only the head turns inside the mob's own limit.
        rider.yaw.store(body_yaw);
        rider.head_yaw.store(clamp_head_to_body(
            rider.head_yaw.load() + delta,
            body_yaw,
            mob.get_max_head_rotation(),
        ));
    }

    fn on_passenger_turned(&self, passenger: &dyn EntityBase) {
        // Vanilla `AbstractBoat.onPassengerTurned`: re-apply `clampRotation` when a passenger
        // turns and fold the clamp adjustment into its previous-yaw so interpolation stays smooth.
        let rider = passenger.get_entity();
        if passenger.get_player().is_some()
            || rider
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_CAN_TURN_IN_BOATS)
        {
            return;
        }
        let boat_yaw = self.get_entity().yaw.load();
        let passenger_count = self
            .get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len();
        let body_yaw = rider_body_yaw(
            boat_yaw,
            passenger_count,
            max_passengers(self.get_entity().entity_type),
            passenger
                .get_mob()
                .is_some_and(|mob| mob.as_animal().is_some()),
            rider.entity_id,
        );
        rider.body_yaw.store(body_yaw);
        let adjustment = passenger.get_mob().map_or_else(
            || {
                let yaw = rider.yaw.load();
                let clamped = clamp_rider_yaw(yaw, body_yaw);
                rider.yaw.store(clamped);
                rider.head_yaw.store(clamped);
                rider_clamp_adjustment(yaw, body_yaw)
            },
            |mob| {
                // Vanilla sets the mob yaw to the hull heading and returns zero, then clamps its head.
                rider.yaw.store(body_yaw);
                rider.head_yaw.store(clamp_head_to_body(
                    rider.head_yaw.load(),
                    body_yaw,
                    mob.get_max_head_rotation(),
                ));
                0.0
            },
        );
        rider.y_rot_o.store(rider.y_rot_o.load() + adjustment);
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    // Vanilla `AbstractBoat.supportQuadLeash` renders the lead to four attachment points.
    fn support_quad_leash(&self) -> bool {
        true
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

/// Vertical velocity vanilla `AbstractBoat.handleBubbleColumnEffect` sets once the column
/// countdown expires: a whirlpool adds -0.7 to the current y, an updraft sets y to 0.6.
#[must_use]
pub fn bubble_column_vertical_velocity(drag_down: bool, current_y: f64) -> f64 {
    if drag_down { current_y - 0.7 } else { 0.6 }
}

/// Vanilla `AbstractBoat.calculatePassengerBodyYRot`: an animal in a full boat sits sideways.
#[must_use]
pub fn rider_body_yaw(
    boat_yaw: f32,
    passenger_count: usize,
    max_passengers: usize,
    is_animal: bool,
    entity_id: i32,
) -> f32 {
    if is_animal && passenger_count == max_passengers {
        boat_yaw + if entity_id % 2 == 0 { 90.0 } else { 270.0 }
    } else {
        boat_yaw
    }
}

/// Vanilla `Mob.clampHeadRotationToBody`: keeps the head within `max` degrees of the body yaw.
#[must_use]
pub fn clamp_head_to_body(head_yaw: f32, body_yaw: f32, max: f32) -> f32 {
    let delta = wrap_degrees(body_yaw - head_yaw);
    let clamped = delta.clamp(-max, max);
    head_yaw + delta - clamped
}

/// Largest yaw offset, in degrees, a non-player rider may turn away from the boat's heading.
const RIDER_MAX_TURN_DEGREES: f32 = 105.0;

/// Returns the rider yaw after clamping it to within `RIDER_MAX_TURN_DEGREES` of the boat yaw.
#[must_use]
pub fn clamp_rider_yaw(rider_yaw: f32, boat_yaw: f32) -> f32 {
    let offset = wrap_degrees(rider_yaw - boat_yaw);
    let clamped = offset.clamp(-RIDER_MAX_TURN_DEGREES, RIDER_MAX_TURN_DEGREES);
    rider_yaw + clamped - offset
}

/// Vanilla `AbstractBoat.clampRotation`'s return value (`yDelta`): how far a rider's yaw was
/// pulled back toward the hull heading. `onPassengerTurned` folds this into the rider's `yRotO`.
#[must_use]
pub fn rider_clamp_adjustment(rider_yaw: f32, boat_yaw: f32) -> f32 {
    clamp_rider_yaw(rider_yaw, boat_yaw) - rider_yaw
}

#[cfg(test)]
mod tests {
    use super::{
        CHEST_BOAT_CONTAINER_SIZE, MinecartInventory, PADDLE_CUE, PADDLE_STEP, boat_container_size,
        boat_drop_item, clamp_head_to_body, clamp_rider_yaw, drops_item_when_destroyed,
        is_chest_boat, max_passengers, paddle_sound_cue, paddle_sound_for_status, rider_body_yaw,
        rider_clamp_adjustment,
    };
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_data::sound::Sound;
    use pumpkin_inventory::Inventory;
    use pumpkin_nbt::compound::NbtCompound;

    #[test]
    fn rider_yaw_clamps_to_boat_heading_window() {
        assert!((clamp_rider_yaw(10.0, 0.0) - 10.0).abs() < 1e-4);
        // Offset 200 wraps to -160 and clamps to -105, so the rider lands at yaw 255 (-105 wrapped).
        assert!((clamp_rider_yaw(200.0, 0.0) - 255.0).abs() < 1e-4);
    }

    #[test]
    fn rider_yaw_follows_boat_yaw_delta_then_clamps() {
        // A rider aligned with the boat follows its turn when the boat yaw changes by a delta.
        assert!((clamp_rider_yaw(100.0 + 10.0, 110.0) - 110.0).abs() < 1e-4);
        // The delta accumulates even while the rider stays inside the turn window.
        assert!((clamp_rider_yaw(100.0 + 20.0, 100.0) - 120.0).abs() < 1e-4);
        // The delta is applied before the +/-105 clamp keeps the rider within the hull window.
        assert!((clamp_rider_yaw(250.0 + 5.0, 105.0) - 210.0).abs() < 1e-4);
    }

    #[test]
    fn rider_clamp_adjustment_feeds_previous_yaw() {
        // A rider 200 degrees off the hull wraps to -160 and is pulled to -105, a +55 nudge.
        assert!((rider_clamp_adjustment(200.0, 0.0) - 55.0).abs() < 1e-4);
        // Inside the +/-105 window the clamp does nothing, so the previous yaw is unchanged.
        assert!(rider_clamp_adjustment(10.0, 0.0).abs() < 1e-4);
    }

    #[test]
    fn animal_in_a_full_boat_sits_sideways() {
        // Vanilla AbstractBoat.calculatePassengerBodyYRot offsets the body of a full-boat animal.
        assert!((rider_body_yaw(90.0, 2, 2, true, 4) - 180.0).abs() < 1e-4);
        assert!((rider_body_yaw(90.0, 2, 2, true, 5) - 360.0).abs() < 1e-4);
        // A lone animal or a non-animal rider keeps the hull heading.
        assert!((rider_body_yaw(90.0, 1, 2, true, 4) - 90.0).abs() < 1e-4);
        assert!((rider_body_yaw(90.0, 2, 2, false, 4) - 90.0).abs() < 1e-4);
        // A head outside the mob limit snaps back towards the body, inside it stays put.
        assert!((clamp_head_to_body(200.0, 0.0, 75.0) - 285.0).abs() < 1e-4);
        assert!((clamp_head_to_body(30.0, 0.0, 75.0) - 30.0).abs() < 1e-4);
    }

    #[test]
    fn bubble_column_impulse_matches_vanilla_boat_effect() {
        assert!((super::bubble_column_vertical_velocity(true, 0.2) - (-0.5)).abs() < 1e-9);
        assert!((super::bubble_column_vertical_velocity(false, -0.3) - 0.6).abs() < 1e-9);
    }

    #[test]
    fn paddle_stroke_cue_fires_twice_per_revolution() {
        // Vanilla crosses PI/4 between PI/8 -> PI/4 and again one full turn later.
        assert!(!paddle_sound_cue(0.0));
        assert!(paddle_sound_cue(PADDLE_STEP));
        assert!(paddle_sound_cue(PADDLE_CUE));
        assert!(!paddle_sound_cue(PADDLE_CUE + PADDLE_STEP));
        assert!(!paddle_sound_cue(std::f32::consts::TAU));
        assert!(paddle_sound_cue(std::f32::consts::TAU + PADDLE_STEP));
    }

    #[test]
    fn paddle_sound_follows_boat_status() {
        use super::BoatStatus;
        assert_eq!(
            paddle_sound_for_status(BoatStatus::InWater),
            Some(Sound::EntityBoatPaddleWater)
        );
        assert_eq!(
            paddle_sound_for_status(BoatStatus::UnderWater),
            Some(Sound::EntityBoatPaddleWater)
        );
        assert_eq!(
            paddle_sound_for_status(BoatStatus::UnderFlowingWater),
            Some(Sound::EntityBoatPaddleWater)
        );
        assert_eq!(
            paddle_sound_for_status(BoatStatus::OnLand),
            Some(Sound::EntityBoatPaddleLand)
        );
        assert_eq!(paddle_sound_for_status(BoatStatus::InAir), None);
    }

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
            if entity_type.boat.is_some() {
                assert_eq!(
                    max_passengers(entity_type),
                    if expected { 1 } else { 2 },
                    "{}",
                    entity_type.resource_name
                );
            }
        }
    }

    #[test]
    fn creative_boat_drops_no_item_even_when_named() {
        // Vanilla VehicleEntity.hurtServer discards the vehicle on an instabuild hit without
        // calling destroy, so a creative hit never drops the item even when named; the custom
        // name is only copied onto the drop in the non-creative destroy path.
        assert!(drops_item_when_destroyed(false, false));
        assert!(drops_item_when_destroyed(false, true));
        assert!(!drops_item_when_destroyed(true, false));
        assert!(!drops_item_when_destroyed(true, true));
    }

    #[test]
    fn every_boat_entity_type_resolves_a_drop_item() {
        for entity_type in EntityType::ALL {
            if entity_type.boat.is_none() {
                continue;
            }
            let resolved = boat_drop_item(entity_type);
            assert!(
                resolved.is_some(),
                "no drop item for {}",
                entity_type.resource_name
            );
            if let Some(item) = resolved {
                assert_eq!(item.registry_key, entity_type.resource_name);
            }
        }
    }

    #[test]
    fn poplar_chest_boat_has_container_but_plain_boat_does_not() {
        // Vanilla AbstractChestBoat always allocates a 27-slot container; Boat has none.
        assert_eq!(CHEST_BOAT_CONTAINER_SIZE, 27);
        assert_eq!(
            boat_container_size(&EntityType::POPLAR_CHEST_BOAT),
            Some(CHEST_BOAT_CONTAINER_SIZE)
        );
        assert_eq!(boat_container_size(&EntityType::POPLAR_BOAT), None);
    }

    #[test]
    fn chest_boat_items_round_trip_through_last_slot() {
        // Vanilla ContainerHelper.saveAllItems/loadAllItems preserves the highest slot (26)
        // of a 27-slot chest boat across the entity custom NBT save/load path.
        let source = MinecartInventory::new(CHEST_BOAT_CONTAINER_SIZE);
        source.set_stack(26, ItemStack::new(2, &Item::POWERED_RAIL));

        let mut nbt = NbtCompound::new();
        source.write_nbt(&mut nbt);
        let items = nbt.get_list("Items");
        assert!(items.is_some_and(|slots| slots.len() == 1));

        let restored = MinecartInventory::new(CHEST_BOAT_CONTAINER_SIZE);
        restored.read_nbt(&nbt);
        let stack = restored.get_stack(26);
        assert_eq!(stack.get_item().id, Item::POWERED_RAIL.id);
        assert_eq!(stack.item_count, 2);
        assert!(restored.get_stack(25).is_empty());
    }

    #[test]
    fn chest_contents_scatter_claim_is_single_use() {
        // Vanilla AbstractChestBoat.remove scatters its slots once via chestVehicleDestroyed.
        // claim_drops gates that scatter when both the damage path and on_removed fire for the
        // same break, so a repeated removal cannot drop the stored items twice.
        let container = MinecartInventory::new(CHEST_BOAT_CONTAINER_SIZE);
        assert!(container.claim_drops());
        assert!(!container.claim_drops());
        assert!(!container.claim_drops());
    }
}

#[cfg(test)]
mod seat_and_container_tests {
    use super::{
        ANIMAL_SEAT_SHIFT, BACK_SEAT_OFFSET, BoatInteractOutcome, FRONT_SEAT_OFFSET,
        boat_interact_outcome, chest_opens_container, passenger_seat_offset,
    };
    use pumpkin_data::entity::EntityType;

    #[test]
    fn chest_boat_offers_container_only_when_no_seat_or_secondary_use() {
        assert!(!chest_opens_container(false, true, true));
        assert!(chest_opens_container(true, true, true));
        assert!(chest_opens_container(false, false, true));
        assert!(!chest_opens_container(false, false, false));
    }

    #[test]
    fn boat_seat_offset_follows_vanilla_single_multi_rider_and_animal_shift() {
        assert_eq!(passenger_seat_offset(0.15, 0, 1, false), 0.15);
        assert_eq!(passenger_seat_offset(0.15, 0, 1, true), 0.15);
        assert_eq!(passenger_seat_offset(0.15, 0, 2, false), FRONT_SEAT_OFFSET);
        assert_eq!(passenger_seat_offset(0.15, 1, 2, false), BACK_SEAT_OFFSET);
        assert_eq!(
            passenger_seat_offset(0.15, 0, 2, true),
            FRONT_SEAT_OFFSET + ANIMAL_SEAT_SHIFT
        );
        assert_eq!(
            passenger_seat_offset(0.15, 1, 2, true),
            BACK_SEAT_OFFSET + ANIMAL_SEAT_SHIFT
        );
    }
    #[test]
    fn boat_stat_classification_covers_every_boat_entity_type() {
        for entity_type in EntityType::ALL {
            if entity_type.boat.is_some() {
                assert!(super::is_boat(entity_type), "{}", entity_type.resource_name);
            }
        }
    }

    #[test]
    fn chest_boat_interact_follows_vanilla_mount_container_nothing_chain() {
        use BoatInteractOutcome::{Mount, Nothing, OpenContainer};
        // Plain use with a free seat mounts, matching a successful `startRiding`.
        assert_eq!(
            boat_interact_outcome(false, false, false, false, true, true),
            Mount
        );
        // Secondary use opens the container instead of mounting.
        assert_eq!(
            boat_interact_outcome(true, false, false, false, true, true),
            OpenContainer
        );
        // A full boat or an eye in water disables `canAddPassenger` and opens the chest.
        assert_eq!(
            boat_interact_outcome(false, false, false, false, false, true),
            OpenContainer
        );
        assert_eq!(
            boat_interact_outcome(false, false, true, false, true, true),
            OpenContainer
        );
        // Out of control or already riding returns PASS: neither mount nor container.
        assert_eq!(
            boat_interact_outcome(false, true, false, false, true, true),
            Nothing
        );
        assert_eq!(
            boat_interact_outcome(false, false, false, true, true, true),
            Nothing
        );
        // A plain boat has no container to open in those same cases.
        assert_eq!(
            boat_interact_outcome(false, true, false, false, true, false),
            Nothing
        );
        assert_eq!(
            boat_interact_outcome(true, false, false, false, true, false),
            Nothing
        );
    }
}
