mod chest;
pub mod command_block;
pub(crate) mod container;
mod furnace;
mod hopper;
mod rideable;
mod spawner;
mod tnt;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_protocol::java::server::play::SPlayerInput;
use rand::RngExt;

use crate::entity::projectile::abstract_arrow::is_abstract_arrow;
use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
};
use pumpkin_data::Block;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{
    PoweredRailLikeProperties, RailLikeProperties, RailShape, RailShapeStraight,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::vehicle::vehicle::VehicleEntity;
use chest::ChestMinecart;
use command_block::CommandBlockMinecart;
use container::MinecartInventory;
use furnace::FurnaceMinecart;
use hopper::HopperMinecart;
use rideable::RideableMinecart;
use spawner::SpawnerMinecart;
use tnt::TntMinecart;

const fn get_exits(
    shape: pumpkin_data::block_properties::RailShape,
) -> (Vector3<f64>, Vector3<f64>) {
    use pumpkin_data::block_properties::RailShape;
    match shape {
        RailShape::NorthSouth => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(0.0, 0.0, 1.0)),
        RailShape::EastWest => (Vector3::new(-1.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
        // Vanilla stores ascending exits with the low end one block BELOW (Direction.below()),
        // so the high end sits at y=0 and the low end at y=-1 relative to the rail block.
        RailShape::AscendingEast => (Vector3::new(-1.0, -1.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
        RailShape::AscendingWest => (Vector3::new(1.0, -1.0, 0.0), Vector3::new(-1.0, 0.0, 0.0)),
        RailShape::AscendingNorth => (Vector3::new(0.0, -1.0, 1.0), Vector3::new(0.0, 0.0, -1.0)),
        RailShape::AscendingSouth => (Vector3::new(0.0, -1.0, -1.0), Vector3::new(0.0, 0.0, 1.0)),
        RailShape::SouthEast => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, 0.0, 0.0)),
        RailShape::SouthWest => (Vector3::new(0.0, 0.0, 1.0), Vector3::new(-1.0, 0.0, 0.0)),
        RailShape::NorthWest => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(-1.0, 0.0, 0.0)),
        RailShape::NorthEast => (Vector3::new(0.0, 0.0, -1.0), Vector3::new(1.0, 0.0, 0.0)),
    }
}

/// Vanilla `OldMinecartBehavior.getSlowdownFactor`: 0.997 with a passenger, else 0.96.
const fn rail_slowdown_factor(has_passengers: bool) -> f64 {
    if has_passengers { 0.997 } else { 0.96 }
}

/// Vanilla `OldMinecartBehavior.moveAlongTrack` move: scale by 0.75 with a rider, then clamp each
/// horizontal axis to `max_speed` (already halved in water). y is cleared.
fn on_rail_move_velocity(
    velocity: Vector3<f64>,
    max_speed: f64,
    has_passengers: bool,
) -> Vector3<f64> {
    let scale = if has_passengers { 0.75 } else { 1.0 };
    Vector3::new(
        (velocity.x * scale).clamp(-max_speed, max_speed),
        0.0,
        (velocity.z * scale).clamp(-max_speed, max_speed),
    )
}

/// Vanilla `OldMinecartBehavior.moveAlongTrack` unpowered-rail halt: horizontal speed
/// under 0.03 stops the cart, otherwise horizontal speed is halved and y is cleared.
fn unpowered_rail_halt(velocity: Vector3<f64>) -> Vector3<f64> {
    if velocity.x.hypot(velocity.z) < 0.03 {
        Vector3::new(0.0, 0.0, 0.0)
    } else {
        Vector3::new(velocity.x * 0.5, 0.0, velocity.z * 0.5)
    }
}

/// Vanilla `OldMinecartBehavior` rail shape: plain rails read `RailLikeProperties`, powered and
/// activator rails the straight subset.
fn rail_shape_of(state_id: BlockStateId, plain_rail: bool) -> RailShape {
    if plain_rail {
        return RailLikeProperties::from_state_id(state_id).shape;
    }
    match PoweredRailLikeProperties::from_state_id(state_id).shape {
        RailShapeStraight::NorthSouth => RailShape::NorthSouth,
        RailShapeStraight::EastWest => RailShape::EastWest,
        RailShapeStraight::AscendingEast => RailShape::AscendingEast,
        RailShapeStraight::AscendingWest => RailShape::AscendingWest,
        RailShapeStraight::AscendingNorth => RailShape::AscendingNorth,
        RailShapeStraight::AscendingSouth => RailShape::AscendingSouth,
    }
}

/// Vanilla `OldMinecartBehavior` powered rail at rest: a flat rail nudges 0.02 toward a redstone
/// conductor on its axis (west before east, north before south). Other shapes return `None`.
fn stopped_powered_rail_velocity(
    velocity: Vector3<f64>,
    shape: RailShape,
    rail_pos: &BlockPos,
    is_conductor: impl Fn(&BlockPos) -> bool,
) -> Option<Vector3<f64>> {
    let (mut dx, mut dz) = (velocity.x, velocity.z);
    match shape {
        RailShape::EastWest => {
            if is_conductor(&rail_pos.west()) {
                dx = STOPPED_RAIL_NUDGE;
            } else if is_conductor(&rail_pos.east()) {
                dx = -STOPPED_RAIL_NUDGE;
            }
        }
        RailShape::NorthSouth => {
            if is_conductor(&rail_pos.north()) {
                dz = STOPPED_RAIL_NUDGE;
            } else if is_conductor(&rail_pos.south()) {
                dz = -STOPPED_RAIL_NUDGE;
            }
        }
        _ => return None,
    }
    Some(Vector3::new(dx, velocity.y, dz))
}

/// Vanilla `OldMinecartBehavior.getPos`: snap a position onto the rail surface,
/// interpolating the height along ascending rails.
fn get_pos_on_rail(
    block_pos: BlockPos,
    shape: pumpkin_data::block_properties::RailShape,
    pos: Vector3<f64>,
) -> Vector3<f64> {
    let (exit0, exit1) = get_exits(shape);
    let bx = f64::from(block_pos.0.x);
    let by = f64::from(block_pos.0.y);
    let bz = f64::from(block_pos.0.z);

    let start = Vector3::new(
        bx + 0.5 + exit0.x * 0.5,
        by + RAIL_HEIGHT_OFFSET + exit0.y * 0.5,
        bz + 0.5 + exit0.z * 0.5,
    );
    let end = Vector3::new(
        bx + 0.5 + exit1.x * 0.5,
        by + RAIL_HEIGHT_OFFSET + exit1.y * 0.5,
        bz + 0.5 + exit1.z * 0.5,
    );

    let dx = end.x - start.x;
    let dy = (end.y - start.y) * 2.0;
    let dz = end.z - start.z;
    let progress = if dx == 0.0 {
        pos.z - bz
    } else if dz == 0.0 {
        pos.x - bx
    } else {
        ((pos.x - start.x) * dx + (pos.z - start.z) * dz) * 2.0
    };

    let mut y = start.y + dy * progress;
    if dy < 0.0 {
        y += 1.0;
    } else if dy > 0.0 {
        y += 0.5;
    }
    Vector3::new(start.x + dx * progress, y, start.z + dz * progress)
}

const GRAVITY: f64 = 0.04;
const RAIL_HEIGHT_OFFSET: f64 = 0.0625;
const STOPPED_RAIL_NUDGE: f64 = 0.02;

pub struct MinecartEntity {
    pub vehicle: VehicleEntity,
    kind: MinecartKind,
}

enum MinecartKind {
    Rideable(RideableMinecart),
    Chest(ChestMinecart),
    CommandBlock(CommandBlockMinecart),
    Furnace(FurnaceMinecart),
    Hopper(HopperMinecart),
    Spawner(SpawnerMinecart),
    Tnt(TntMinecart),
    Other,
}

impl MinecartEntity {
    pub fn new(mut entity: Entity) -> Self {
        entity.blocks_building = true;
        let kind = match entity.entity_type.id {
            id if id == EntityType::MINECART.id => MinecartKind::Rideable(RideableMinecart),
            id if id == EntityType::CHEST_MINECART.id => MinecartKind::Chest(ChestMinecart::new()),
            id if id == EntityType::COMMAND_BLOCK_MINECART.id => {
                MinecartKind::CommandBlock(CommandBlockMinecart::new())
            }
            id if id == EntityType::FURNACE_MINECART.id => {
                MinecartKind::Furnace(FurnaceMinecart::new())
            }
            id if id == EntityType::HOPPER_MINECART.id => {
                MinecartKind::Hopper(HopperMinecart::new())
            }
            id if id == EntityType::SPAWNER_MINECART.id => {
                MinecartKind::Spawner(SpawnerMinecart::new())
            }
            id if id == EntityType::TNT_MINECART.id => MinecartKind::Tnt(TntMinecart::new()),
            _ => MinecartKind::Other,
        };
        Self {
            vehicle: VehicleEntity::new(entity),
            kind,
        }
    }

    #[must_use]
    pub const fn command_block(&self) -> Option<&CommandBlockMinecart> {
        match &self.kind {
            MinecartKind::CommandBlock(minecart) => Some(minecart),
            _ => None,
        }
    }

    const fn container(&self) -> Option<&Arc<MinecartInventory>> {
        match &self.kind {
            MinecartKind::Chest(minecart) => Some(minecart.inventory()),
            MinecartKind::Hopper(minecart) => Some(minecart.inventory()),
            _ => None,
        }
    }

    const fn drop_item(&self) -> &'static Item {
        match &self.kind {
            MinecartKind::Chest(_) => &Item::CHEST_MINECART,
            MinecartKind::Furnace(_) => &Item::FURNACE_MINECART,
            MinecartKind::Hopper(_) => &Item::HOPPER_MINECART,
            MinecartKind::Tnt(_) => &Item::TNT_MINECART,
            MinecartKind::CommandBlock(_) => &Item::COMMAND_BLOCK_MINECART,
            MinecartKind::Rideable(_) | MinecartKind::Spawner(_) | MinecartKind::Other => {
                &Item::MINECART
            }
        }
    }
}

const LOWERED_PASSENGER_ATTACHMENT: Vector3<f64> = Vector3::new(0.0, 0.0, 0.0);

/// Vanilla `OldMinecartBehavior`: a powered rail adds 0.06 to the horizontal
/// speed along the current horizontal direction and leaves Y untouched.
fn powered_rail_boost(velocity: Vector3<f64>, max_speed: f64) -> Vector3<f64> {
    let speed = velocity.x.hypot(velocity.z);
    if speed <= 0.01 {
        return velocity;
    }
    let new_speed = (speed + 0.06).min(max_speed);
    Vector3::new(
        velocity.x / speed * new_speed,
        velocity.y,
        velocity.z / speed * new_speed,
    )
}

impl EntityBase for MinecartEntity {
    fn passenger_attachment_point(
        &self,
        passenger: &dyn EntityBase,
        _index: usize,
    ) -> Option<Vector3<f64>> {
        let passenger_type = passenger.get_entity().entity_type;
        (passenger_type == &EntityType::VILLAGER || passenger_type == &EntityType::WANDERING_TRADER)
            .then_some(LOWERED_PASSENGER_ATTACHMENT)
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        match &self.kind {
            MinecartKind::Chest(minecart) => minecart.write_nbt(nbt),
            MinecartKind::CommandBlock(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Furnace(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Hopper(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Spawner(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Tnt(minecart) => minecart.write_nbt(nbt),
            MinecartKind::Rideable(_) | MinecartKind::Other => {}
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        match &self.kind {
            MinecartKind::Chest(minecart) => minecart.read_nbt(nbt),
            MinecartKind::CommandBlock(minecart) => minecart.read_nbt(&self.vehicle.entity, nbt),
            MinecartKind::Furnace(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Hopper(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Spawner(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Tnt(minecart) => minecart.read_nbt(nbt),
            MinecartKind::Rideable(_) | MinecartKind::Other => {}
        }
    }

    #[allow(clippy::too_many_lines)]
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.vehicle.tick();
        self.vehicle.entity.check_out_of_world(caller);
        if self.vehicle.entity.is_removed() {
            return;
        }
        // Vanilla `AbstractMinecart.tick` calls `handlePortal` explicitly (it overrides tick and
        // never runs `Entity.baseTick`). This advances the portal timer and teleports the cart and
        // its passengers together once the delay elapses.
        self.vehicle.entity.tick_portal(caller);
        let from = self.vehicle.entity.pos.load();
        // Vanilla increments the entity tick counter (used by the command-block activation
        // cooldown) at the start of `Entity.tick`, before rail activation runs.
        if let MinecartKind::CommandBlock(minecart) = &self.kind {
            minecart.step_tick();
        }

        let world = self.vehicle.entity.world.load();
        // `max_minecart_speed` only exists with the `minecart_improvements` experiment;
        // without it vanilla keeps the fixed 0.4 blocks/tick cap (`OldMinecartBehavior`).
        // The gamerule counts blocks per second (default 8), so divide by 20 ticks.
        let max_speed = {
            let level_info = world.level_info.load();
            let improvements =
                world.server.upgrade().map_or_else(
                    || {
                        level_info.data_packs.enabled.iter().any(|p| {
                            p == "minecart_improvements" || p == "file/minecart_improvements"
                        })
                    },
                    |s| s.is_feature_enabled("minecraft:minecart_improvements"),
                );
            if improvements {
                level_info.game_rules.max_minecart_speed as f64 / 20.0
            } else {
                0.4
            }
        };
        // Vanilla `getMaxSpeed`: halved while the cart is in water.
        let max_speed = if self.vehicle.entity.touching_water.load(Ordering::Relaxed) {
            max_speed / 2.0
        } else {
            max_speed
        };
        let pos = self.vehicle.entity.pos.load();
        let mut block_pos = BlockPos(Vector3::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        ));

        let (mut block, mut state_id) = world.get_block_and_state_id(&block_pos);

        let mut is_powered_rail = block.id == Block::POWERED_RAIL.id;
        let mut is_activator_rail = block.id == Block::ACTIVATOR_RAIL.id;
        let mut is_on_rails = is_powered_rail
            || is_activator_rail
            || block.id == Block::RAIL.id
            || block.id == Block::DETECTOR_RAIL.id;

        // If not on rails at current Y level, check the block directly below
        if !is_on_rails {
            let below_block_pos = BlockPos(Vector3::new(
                block_pos.0.x,
                block_pos.0.y - 1,
                block_pos.0.z,
            ));
            let (below_block, below_state_id) = world.get_block_and_state_id(&below_block_pos);
            if below_block.id == Block::RAIL.id
                || below_block.id == Block::POWERED_RAIL.id
                || below_block.id == Block::DETECTOR_RAIL.id
                || below_block.id == Block::ACTIVATOR_RAIL.id
            {
                block_pos = below_block_pos;
                block = below_block;
                state_id = below_state_id;
                is_powered_rail = block.id == Block::POWERED_RAIL.id;
                is_activator_rail = block.id == Block::ACTIVATOR_RAIL.id;
                is_on_rails = true;
            }
        }

        if is_powered_rail || is_activator_rail {
            let props = PoweredRailLikeProperties::from_state_id(state_id);
            let powered = props.powered;

            if is_activator_rail && let MinecartKind::Hopper(minecart) = &self.kind {
                minecart.set_enabled(!powered);
            }
        }

        if is_powered_rail || is_activator_rail {
            let props = PoweredRailLikeProperties::from_state_id(state_id);
            let powered = props.powered;

            if powered {
                if is_powered_rail {
                    let current = self.vehicle.entity.velocity.load();
                    let horizontal_speed = current.x.hypot(current.z);
                    let velocity = if horizontal_speed > 0.01 {
                        powered_rail_boost(current, max_speed)
                    } else {
                        // Vanilla `OldMinecartBehavior`: a stopped cart nudges toward a conductor.
                        stopped_powered_rail_velocity(
                            current,
                            rail_shape_of(state_id, false),
                            &block_pos,
                            |pos| world.get_block_state(pos).is_full_cube(),
                        )
                        .unwrap_or(current)
                    };
                    self.vehicle.entity.velocity.store(velocity);
                    self.vehicle.entity.send_velocity();
                } else if is_activator_rail {
                    match &self.kind {
                        MinecartKind::CommandBlock(minecart) => {
                            minecart.activate(&self.vehicle.entity);
                        }
                        MinecartKind::Tnt(minecart) => {
                            minecart.prime(&self.vehicle.entity, 80);
                        }
                        MinecartKind::Rideable(_) => {
                            if let Ok(passengers) = self.vehicle.entity.passengers.try_lock() {
                                let p_ids: Vec<i32> = passengers
                                    .iter()
                                    .map(|p| p.get_entity().entity_id)
                                    .collect();
                                if !p_ids.is_empty() {
                                    let world = self.vehicle.entity.world.load();
                                    let vid = self.vehicle.entity.entity_id;
                                    if let Some(v) = world.get_entity_by_id(vid) {
                                        for pid in p_ids {
                                            v.get_entity().remove_passenger_sync(pid);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else if is_powered_rail {
                let velocity = unpowered_rail_halt(self.vehicle.entity.velocity.load());
                self.vehicle.entity.velocity.store(velocity);
                self.vehicle.entity.send_velocity();
            }
        }

        if let MinecartKind::Tnt(minecart) = &self.kind
            && minecart.tick(self)
        {
            return;
        }

        let mut velocity = self.vehicle.entity.velocity.load();

        let mut has_driver = false;
        let mut driver_input = 0;
        let mut driver_yaw = 0.0f32;

        if let Ok(passengers) = self.vehicle.entity.passengers.try_lock()
            && let Some(passenger) = passengers.first()
            && let Some(player) = passenger.get_player()
        {
            driver_input = player.last_input.load(Ordering::Relaxed);
            driver_yaw = player.get_entity().yaw.load();
            has_driver = true;
        }

        if has_driver && is_on_rails {
            let forward = driver_input & SPlayerInput::FORWARD != 0;
            let backward = driver_input & SPlayerInput::BACKWARD != 0;

            let mut force_dir = Vector3::new(0.0, 0.0, 0.0);
            if forward {
                let yaw_rad = f64::from(driver_yaw).to_radians();
                force_dir.x = -yaw_rad.sin();
                force_dir.z = yaw_rad.cos();
            } else if backward {
                let yaw_rad = f64::from(driver_yaw).to_radians();
                force_dir.x = yaw_rad.sin();
                force_dir.z = -yaw_rad.cos();
            }

            if forward || backward {
                velocity.x += force_dir.x * 0.02;
                velocity.z += force_dir.z * 0.02;

                let speed = velocity.x.hypot(velocity.z);
                if speed > 0.15 {
                    #[allow(clippy::suboptimal_flops)]
                    let old_speed = self
                        .vehicle
                        .entity
                        .velocity
                        .load()
                        .x
                        .hypot(self.vehicle.entity.velocity.load().z);

                    let driver_max_speed = old_speed.clamp(0.15, max_speed.max(0.15));
                    if speed > driver_max_speed {
                        velocity.x = (velocity.x / speed) * driver_max_speed;
                        velocity.z = (velocity.z / speed) * driver_max_speed;
                    }
                }
                self.vehicle.entity.velocity.store(velocity);
                self.vehicle.entity.send_velocity();
            }
        }

        let mut velocity = self.vehicle.entity.velocity.load();

        if is_on_rails {
            let shape = rail_shape_of(state_id, block.id == Block::RAIL.id);

            // Vanilla `OldMinecartBehavior.moveAlongTrack`: gravity along the slope.
            let slope_gravity = if self.vehicle.entity.touching_water.load(Ordering::Relaxed) {
                0.0078125 * 0.2
            } else {
                0.0078125
            };
            match shape {
                RailShape::AscendingEast => velocity.x -= slope_gravity,
                RailShape::AscendingWest => velocity.x += slope_gravity,
                RailShape::AscendingNorth => velocity.z += slope_gravity,
                RailShape::AscendingSouth => velocity.z -= slope_gravity,
                _ => {}
            }

            let pos = self.vehicle.entity.pos.load();
            let block_center_bottom = Vector3::new(
                f64::from(block_pos.0.x) + 0.5,
                f64::from(block_pos.0.y),
                f64::from(block_pos.0.z) + 0.5,
            );

            let (exit0, exit1) = get_exits(shape);
            let exit0 = exit0.multiply(0.5, 0.5, 0.5);
            let exit1 = exit1.multiply(0.5, 0.5, 0.5);

            let target_position = get_pos_on_rail(block_pos, shape, pos);
            self.vehicle.entity.set_pos(target_position);

            let horizontal_in_direction = Vector3::new(exit1.x, 0.0, exit1.z);
            let mut horizontal_out_direction = Vector3::new(exit0.x, 0.0, exit0.z);

            if velocity.dot(&horizontal_out_direction) < velocity.dot(&horizontal_in_direction) {
                horizontal_out_direction = horizontal_in_direction;
            }

            let out_position = block_center_bottom.add(&horizontal_out_direction).add(
                &horizontal_out_direction
                    .normalize()
                    .multiply(1e-5, 1e-5, 1e-5),
            );

            let mut towards_out = out_position - target_position;
            towards_out.y = 0.0;
            let towards_length = towards_out.length();
            if towards_length > 1e-5 {
                towards_out = towards_out.normalize();
                let speed = velocity.length();
                velocity = towards_out.multiply(speed, speed, speed);
            }

            velocity.y = 0.0;
            self.vehicle.entity.velocity.store(velocity);
        } else if !self.vehicle.entity.on_ground.load(Ordering::Relaxed) {
            velocity.y -= GRAVITY;
            self.vehicle.entity.velocity.store(velocity);
        }

        if velocity.length() > 0.001 {
            let move_velocity = if is_on_rails {
                let has_passengers = self
                    .vehicle
                    .entity
                    .passengers
                    .try_lock()
                    .is_ok_and(|p| !p.is_empty());
                on_rail_move_velocity(velocity, max_speed, has_passengers)
            } else {
                velocity
            };
            self.move_entity(caller, move_velocity);

            if let MinecartKind::Tnt(minecart) = &self.kind
                && self
                    .vehicle
                    .entity
                    .horizontal_collision
                    .load(Ordering::Relaxed)
                && velocity.x.mul_add(velocity.x, velocity.z * velocity.z) >= 0.01
            {
                minecart.explode(
                    self,
                    velocity.x.mul_add(velocity.x, velocity.z * velocity.z),
                    None,
                );
                return;
            }

            #[allow(clippy::useless_let_if_seq)]
            let mut friction = 0.95; // Vanilla minecart air drag

            if is_on_rails {
                let has_passengers = self
                    .vehicle
                    .entity
                    .passengers
                    .try_lock()
                    .is_ok_and(|p| !p.is_empty());
                // Vanilla `applyNaturalSlowdown`: water multiplies the factor by 0.95.
                let in_water = self.vehicle.entity.touching_water.load(Ordering::Relaxed);
                friction = rail_slowdown_factor(has_passengers) * if in_water { 0.95 } else { 1.0 };
            } else {
                let below_block_pos = BlockPos(Vector3::new(
                    block_pos.0.x,
                    block_pos.0.y - 1,
                    block_pos.0.z,
                ));
                let below_block = world.get_block(&below_block_pos);

                let is_on_ground = self.vehicle.entity.on_ground.load(Ordering::Relaxed)
                    || (below_block.id != Block::AIR.id
                        && below_block.id != Block::WATER.id
                        && below_block.id != Block::LAVA.id);
                let is_in_water = self.vehicle.entity.touching_water.load(Ordering::Relaxed)
                    || below_block.id == Block::WATER.id;

                if is_on_ground {
                    friction = 0.5;
                } else if is_in_water {
                    friction = 0.95;
                }
            }

            let mut next_vel = if is_on_rails && let MinecartKind::Furnace(minecart) = &self.kind {
                minecart.velocity(&self.vehicle.entity, velocity)
            } else if is_on_rails && let Some(inventory) = self.container() {
                container::velocity(&self.vehicle.entity, inventory, velocity)
            } else {
                velocity.multiply(friction, friction, friction)
            };
            if next_vel.length() < 0.005 {
                next_vel = Vector3::new(0.0, 0.0, 0.0);
            }
            self.vehicle.entity.velocity.store(next_vel);
            if next_vel.length_squared() == 0.0 {
                self.vehicle.entity.send_velocity();
            }
        }

        let attempted_to = self.vehicle.entity.pos.load();
        let move_result = self.vehicle.move_vehicle(from, attempted_to);
        let to = move_result.position;
        self.vehicle.entity.set_pos(to);
        if move_result.cancelled {
            self.vehicle
                .entity
                .velocity
                .store(Vector3::new(0.0, 0.0, 0.0));
            self.vehicle.entity.send_velocity();
        }
        if to != attempted_to {
            let world = self.vehicle.entity.world.load();
            self.vehicle.entity.teleport(to, None, None, &world);
        }
        if attempted_to != from {
            // Vanilla `Entity.positionRider` places passengers at their seat, which carries
            // the per-entity attachment offset, instead of at the cart's centre.
            crate::entity::position_riders(self);
        }
        // Vanilla `OldMinecartBehavior.tick`: `applyEffectsFromBlocks` after moving.
        self.vehicle.entity.tick_block_collisions(caller);

        // Vanilla `AbstractMinecart.tick` hurts minecarts in lava on top of the lava fluid effect.
        self.vehicle.entity.update_fluid_state(caller);
        if self.vehicle.entity.touching_lava.load(Ordering::SeqCst) {
            self.vehicle.entity.lava_ignite(caller);
            self.vehicle.entity.lava_hurt(caller);
        }
        self.vehicle
            .entity
            .first_tick
            .store(false, Ordering::Relaxed);

        match &self.kind {
            MinecartKind::Hopper(minecart) => minecart.tick(&self.vehicle.entity),
            MinecartKind::Spawner(minecart) => minecart.tick(&self.vehicle.entity),
            _ => {}
        }
        // Vanilla `MinecartFurnace.tick` decrements fuel and emits smoke after `super.tick()`
        // (movement), so the push is consumed by this tick's motion before being cleared.
        if let MinecartKind::Furnace(minecart) = &self.kind {
            minecart.tick(&self.vehicle.entity);
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
        self.container().is_some()
    }

    fn get_container_inventory(&self) -> Option<Arc<dyn Inventory>> {
        let container = self.container()?;
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

    fn is_pushable(&self) -> bool {
        true
    }

    #[allow(clippy::too_many_lines)]
    fn push(&self, entity: &dyn EntityBase) {
        let self_entity = self.get_entity();
        let other_entity = entity.get_entity();

        if self_entity.no_physics.load(Ordering::Relaxed)
            || other_entity.no_physics.load(Ordering::Relaxed)
        {
            return;
        }

        {
            let passengers = self_entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if passengers
                .iter()
                .any(|p| p.get_entity().entity_id == other_entity.entity_id)
            {
                return;
            }
        }
        {
            let passengers = other_entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if passengers
                .iter()
                .any(|p| p.get_entity().entity_id == self_entity.entity_id)
            {
                return;
            }
        }

        let mut xa = other_entity.pos.load().x - self_entity.pos.load().x;
        let mut za = other_entity.pos.load().z - self_entity.pos.load().z;
        let mut dd = xa * xa + za * za;
        if dd >= 1.0E-4 {
            dd = dd.sqrt();
            xa /= dd;
            za /= dd;
            let mut pow = 1.0 / dd;
            if pow > 1.0 {
                pow = 1.0;
            }
            xa *= pow;
            za *= pow;
            xa *= 0.1;
            za *= 0.1;
            xa *= 0.5;
            za *= 0.5;

            let is_other_minecart = other_entity.entity_type.id == EntityType::MINECART.id
                || other_entity.entity_type.id == EntityType::CHEST_MINECART.id
                || other_entity.entity_type.id == EntityType::COMMAND_BLOCK_MINECART.id
                || other_entity.entity_type.id == EntityType::FURNACE_MINECART.id
                || other_entity.entity_type.id == EntityType::HOPPER_MINECART.id
                || other_entity.entity_type.id == EntityType::SPAWNER_MINECART.id
                || other_entity.entity_type.id == EntityType::TNT_MINECART.id;

            if is_other_minecart {
                // Vanilla: the direction from this cart to the other one, not its motion.
                // A standing cart has no motion, so the old check never transferred velocity.
                let dir = Vector3::new(
                    other_entity.pos.load().x - self_entity.pos.load().x,
                    0.0,
                    other_entity.pos.load().z - self_entity.pos.load().z,
                )
                .normalize();
                let facing = Vector3::new(
                    f64::from(self_entity.yaw.load().to_radians().cos()),
                    0.0,
                    f64::from(self_entity.yaw.load().to_radians().sin()),
                )
                .normalize();

                let dot = dir.dot(&facing).abs();
                if dot >= 0.8 {
                    let vel = self_entity.velocity.load();
                    let ovel = other_entity.velocity.load();

                    let is_self_furnace =
                        self_entity.entity_type.id == EntityType::FURNACE_MINECART.id;
                    let is_other_furnace =
                        other_entity.entity_type.id == EntityType::FURNACE_MINECART.id;

                    if is_other_furnace && !is_self_furnace {
                        self_entity.velocity.store(vel.multiply(0.2, 1.0, 0.2));
                        let mut new_self_vel = self_entity.velocity.load();
                        new_self_vel.x += ovel.x - xa;
                        new_self_vel.z += ovel.z - za;
                        self_entity.velocity.store(new_self_vel);
                        self_entity.send_velocity();

                        other_entity.velocity.store(ovel.multiply(0.95, 1.0, 0.95));
                        other_entity.send_velocity();
                    } else if !is_other_furnace && is_self_furnace {
                        other_entity.velocity.store(ovel.multiply(0.2, 1.0, 0.2));
                        let mut new_other_vel = other_entity.velocity.load();
                        new_other_vel.x += vel.x + xa;
                        new_other_vel.z += vel.z + za;
                        other_entity.velocity.store(new_other_vel);
                        other_entity.send_velocity();

                        self_entity.velocity.store(vel.multiply(0.95, 1.0, 0.95));
                        self_entity.send_velocity();
                    } else {
                        #[allow(clippy::manual_midpoint)]
                        let xdd = (ovel.x + vel.x) / 2.0;
                        #[allow(clippy::manual_midpoint)]
                        let zdd = (ovel.z + vel.z) / 2.0;

                        self_entity.velocity.store(vel.multiply(0.2, 1.0, 0.2));
                        let mut new_self_vel = self_entity.velocity.load();
                        new_self_vel.x += xdd - xa;
                        new_self_vel.z += zdd - za;
                        self_entity.velocity.store(new_self_vel);
                        self_entity.send_velocity();

                        other_entity.velocity.store(ovel.multiply(0.2, 1.0, 0.2));
                        let mut new_other_vel = other_entity.velocity.load();
                        new_other_vel.x += xdd + xa;
                        new_other_vel.z += zdd + za;
                        other_entity.velocity.store(new_other_vel);
                        other_entity.send_velocity();
                    }
                }
            } else {
                // Vanilla `AbstractMinecart.push`
                self_entity.push_by(Vector3::new(-xa, 0.0, -za));
                other_entity.push_by(Vector3::new(xa / 4.0, 0.0, za / 4.0));
            }
        }
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn init_data_tracker(&self) {
        self.vehicle.send_wobble_metadata();
        match &self.kind {
            MinecartKind::CommandBlock(minecart) => {
                minecart.init_data_tracker(&self.vehicle.entity);
            }
            MinecartKind::Furnace(minecart) => {
                minecart.init_data_tracker(&self.vehicle.entity);
            }
            _ => {}
        }
    }

    fn can_hit(&self) -> bool {
        self.vehicle.entity.is_alive()
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
        let creative = source
            .and_then(EntityBase::get_player)
            .is_some_and(|player| player.gamemode.load() == GameMode::Creative);

        if let MinecartKind::Tnt(minecart) = &self.kind
            && let Some(projectile) = source
            && is_abstract_arrow(projectile.get_entity().entity_type)
            && projectile.get_entity().is_on_fire()
        {
            let world = self.vehicle.entity.world.load();
            let cause =
                cause.and_then(|cause| world.get_entity_by_id(cause.get_entity().entity_id));
            minecart.explode(
                self,
                projectile.get_entity().velocity.load().length_squared(),
                cause,
            );
            if self.vehicle.entity.is_removed() {
                return true;
            }
        }

        let will_break = self.vehicle.entity.is_alive()
            && (creative || self.vehicle.get_damage() + amount * 10.0 > 40.0);

        if let MinecartKind::Tnt(minecart) = &self.kind
            && self.vehicle.entity.is_alive()
        {
            let velocity = self.vehicle.entity.velocity.load();
            let speed_squared = velocity.x.mul_add(velocity.x, velocity.z * velocity.z);
            // Vanilla MinecartTNT.shouldSourceDestroy (= damageSourceIgnitesTnt) makes a
            // fire/explosion damage source trigger the TNT destroy path below 40 damage. The
            // burning-arrow case is handled above by the immediate explode.
            let ignites = damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FIRE)
                || damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_EXPLOSION);
            // Vanilla VehicleEntity.hurtServer calls destroy() when the cart would break OR the
            // source ignites TNT; a creative attacker skips the break path unless it ignites.
            let destroy = (!creative && will_break) || ignites;
            if destroy && (ignites || speed_squared >= 0.01) {
                self.vehicle.apply_damage_wobble(amount);
                // Vanilla MinecartTNT.destroy: nextInt(20) + nextInt(20) fuse, primed only while
                // the cart is unprimed and the tntExplodes rule is on.
                let fuse = rand::rng().random_range(0..20) + rand::rng().random_range(0..20);
                if self
                    .vehicle
                    .entity
                    .world
                    .load()
                    .level_info
                    .load()
                    .game_rules
                    .tnt_explodes
                {
                    minecart.prime(&self.vehicle.entity, fuse);
                }
                return true;
            }
        }

        let damaged = self.vehicle.damage_with_context(amount, source);

        if will_break && !creative && self.vehicle.entity.is_removed() {
            let world = self.vehicle.entity.world.load();
            // Vanilla AbstractMinecartContainer.remove scatters the stacks already inside on any
            // destroying removal, independent of doEntityDrops; only the cart item is gated. The
            // loot table is only unpacked when a player opens the container menu.
            if let Some(container) = self.container()
                && container.claim_drops()
            {
                let inventory: Arc<dyn Inventory> = container.clone();
                // Vanilla Containers.dropContents(level, entity, container) scatters around the
                // entity position, not the floored block origin.
                world.scatter_inventory_at(self.vehicle.entity.pos.load(), &inventory);
            }
            if world.level_info.load().game_rules.entity_drops {
                let mut stack = ItemStack::new(1, self.drop_item());
                if let Some(custom_name) = self.vehicle.entity.custom_name.load().as_ref() {
                    stack.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
                        name: custom_name.clone(),
                    });
                }
                // Vanilla VehicleEntity.destroy -> Entity.spawnAtLocation spawns the dropped
                // vehicle at the entity exact position, not the floored block origin.
                world.drop_stack_at(self.vehicle.entity.pos.load(), stack);
            }
        }

        damaged
    }

    fn interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        match &self.kind {
            MinecartKind::Chest(minecart) => {
                minecart.interact(&self.vehicle.entity, player);
                true
            }
            MinecartKind::CommandBlock(_) => CommandBlockMinecart::interact(player),
            MinecartKind::Furnace(minecart) => {
                minecart.interact(&self.vehicle.entity, player, item_stack)
            }
            MinecartKind::Hopper(minecart) => {
                minecart.interact(&self.vehicle.entity, player);
                true
            }
            MinecartKind::Rideable(_) => RideableMinecart::interact(&self.vehicle.entity, player),
            MinecartKind::Spawner(_) | MinecartKind::Tnt(_) | MinecartKind::Other => false,
        }
    }

    fn move_entity(&self, caller: &dyn EntityBase, motion: Vector3<f64>) {
        // Vanilla AbstractMinecart.move: capture the intended target before the first move,
        // then replay any leftover distance after pushAndPickupEntities.
        let to_position = self.vehicle.entity.pos.load().add(&motion);
        self.vehicle.entity.move_entity(caller, motion);
        let entity_id = self.vehicle.entity.entity_id;
        let world = self.vehicle.entity.world.load().clone();
        if let Some(dyn_self) = world.get_entity_by_id(entity_id) {
            let should_continue = dyn_self.push_entities(caller);
            if should_continue {
                let current_pos = dyn_self.get_entity().pos.load();
                let back_motion = to_position.sub(&current_pos);
                // Both moves use the base Entity::move_entity (vanilla super.move), never the
                // EntityBase override, so the residual cannot re-run pushAndPickupEntities.
                dyn_self.get_entity().move_entity(caller, back_motion);
            }
        }
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_util::text::TextComponent;

    #[test]
    fn powered_rail_boost_uses_horizontal_speed_and_preserves_y() {
        let boosted = powered_rail_boost(Vector3::new(3.0, 5.0, 4.0), 10.0);
        assert_eq!(boosted.y, 5.0);
        assert!((boosted.x.hypot(boosted.z) - 5.06).abs() < 1.0e-9);

        let capped = powered_rail_boost(Vector3::new(3.0, 5.0, 4.0), 5.02);
        assert_eq!(capped.y, 5.0);
        assert!((capped.x.hypot(capped.z) - 5.02).abs() < 1.0e-9);

        let still = powered_rail_boost(Vector3::new(0.0, 5.0, 0.0), 10.0);
        assert_eq!(still, Vector3::new(0.0, 5.0, 0.0));
    }

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn unpowered_rail_halt_stops_slow_and_halves_fast_carts() {
        let slow = unpowered_rail_halt(Vector3::new(0.02, 0.3, 0.0));
        assert_eq!((slow.x, slow.y, slow.z), (0.0, 0.0, 0.0));
        let fast = unpowered_rail_halt(Vector3::new(0.3, 0.3, 0.4));
        assert_eq!((fast.x, fast.y, fast.z), (0.15, 0.0, 0.2));
        assert!((rail_slowdown_factor(true) - 0.997).abs() < f64::EPSILON);
        assert!((rail_slowdown_factor(false) - 0.96).abs() < f64::EPSILON);
    }

    #[test]
    fn on_rail_move_clamps_to_cap_and_scales_with_rider() {
        let v = on_rail_move_velocity(Vector3::new(1.0, 0.5, -1.0), 0.4, false);
        assert_eq!((v.x, v.y, v.z), (0.4, 0.0, -0.4));
        let ridden = on_rail_move_velocity(Vector3::new(0.4, 0.0, 0.0), 0.4, true);
        assert!((ridden.x - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn stopped_powered_rail_nudges_toward_redstone_conductor_on_its_axis() {
        let rail = BlockPos(Vector3::new(0, 64, 0));
        let v = Vector3::new(0.0, 0.1, 0.0);
        let west = stopped_powered_rail_velocity(v, RailShape::EastWest, &rail, |p| p.0.x == -1);
        assert_eq!(west.map(|v| (v.x, v.y, v.z)), Some((0.02, 0.1, 0.0)));
        let east = stopped_powered_rail_velocity(v, RailShape::EastWest, &rail, |p| p.0.x == 1);
        assert_eq!(east.map(|v| (v.x, v.y, v.z)), Some((-0.02, 0.1, 0.0)));
        let north = stopped_powered_rail_velocity(v, RailShape::NorthSouth, &rail, |p| p.0.z == -1);
        assert_eq!(north.map(|v| (v.x, v.y, v.z)), Some((0.0, 0.1, 0.02)));
        let none = stopped_powered_rail_velocity(v, RailShape::EastWest, &rail, |_| false);
        assert_eq!(none.map(|v| (v.x, v.y, v.z)), Some((0.0, 0.1, 0.0)));
        let ascending = stopped_powered_rail_velocity(v, RailShape::AscendingEast, &rail, |_| true);
        assert!(ascending.is_none());
    }

    #[test]
    fn rail_position_follows_ascending_slope() {
        use pumpkin_data::block_properties::RailShape;

        let flat = get_pos_on_rail(
            BlockPos(Vector3::new(0, 64, 0)),
            RailShape::EastWest,
            Vector3::new(0.5, 64.0, 0.5),
        );
        assert_eq!(
            (flat.x, flat.y, flat.z),
            (0.5, 64.0 + RAIL_HEIGHT_OFFSET, 0.5)
        );

        // Vanilla getPos snaps the center of an ascending rail to half a block above
        // the flat-rail offset (y0 + yD*0.5 + 0.5 for the positive yD).
        let expected_slope_y = 64.0 + RAIL_HEIGHT_OFFSET + 0.5;
        let slope = get_pos_on_rail(
            BlockPos(Vector3::new(0, 64, 0)),
            RailShape::AscendingEast,
            Vector3::new(0.5, 64.0, 0.5),
        );
        assert_eq!((slope.x, slope.y, slope.z), (0.5, expected_slope_y, 0.5));
    }

    #[test]
    fn all_ascending_rails_snap_center_to_vanilla_height() {
        use pumpkin_data::block_properties::RailShape;

        let expected_y = 64.0 + RAIL_HEIGHT_OFFSET + 0.5;
        for shape in [
            RailShape::AscendingEast,
            RailShape::AscendingWest,
            RailShape::AscendingNorth,
            RailShape::AscendingSouth,
        ] {
            let snapped = get_pos_on_rail(
                BlockPos(Vector3::new(0, 64, 0)),
                shape,
                Vector3::new(0.5, 64.0, 0.5),
            );
            assert_eq!((snapped.x, snapped.y, snapped.z), (0.5, expected_y, 0.5));
        }
    }

    #[test]
    fn rail_position_snap_covers_north_south_and_curve() {
        use pumpkin_data::block_properties::RailShape;

        for (shape, pos, expected) in [
            (
                RailShape::NorthSouth,
                Vector3::new(0.5, 64.3, 0.25),
                Vector3::new(0.5, 64.0 + RAIL_HEIGHT_OFFSET, 0.25),
            ),
            (
                RailShape::NorthSouth,
                Vector3::new(0.5, 64.3, 0.75),
                Vector3::new(0.5, 64.0 + RAIL_HEIGHT_OFFSET, 0.75),
            ),
            (
                RailShape::SouthEast,
                Vector3::new(0.625, 64.3, 0.625),
                Vector3::new(0.75, 64.0 + RAIL_HEIGHT_OFFSET, 0.75),
            ),
        ] {
            let snapped = get_pos_on_rail(BlockPos(Vector3::new(0, 64, 0)), shape, pos);
            assert_eq!(
                (snapped.x, snapped.y, snapped.z),
                (expected.x, expected.y, expected.z)
            );
        }
    }

    #[tokio::test]
    async fn command_block_minecart_nbt_and_activation_cooldown() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::COMMAND_BLOCK_MINECART,
        ));

        let mut nbt = NbtCompound::new();
        nbt.put_string("Command", "Searge".to_string());
        nbt.put_bool("TrackOutput", true);
        nbt.put_long("LastExecution", 12);
        cart.read_nbt_non_mut(&nbt);

        let Some(cb) = cart.command_block() else {
            return Err("expected CommandBlockMinecart".into());
        };
        assert_eq!(cb.get_command(), "Searge");
        assert_eq!(cb.get_success_count(), 0);

        // Fewer than 4 ticks elapsed since `last_activated` (0): must not activate.
        for _ in 0..3 {
            cb.step_tick();
            cb.activate(&cart.vehicle.entity);
        }
        assert_eq!(cb.get_success_count(), 0);

        // Tick 4 reaches ACTIVATION_DELAY (4) and triggers the command.
        cb.step_tick();
        cb.activate(&cart.vehicle.entity);
        assert_eq!(cb.get_success_count(), 1);
        assert_eq!(
            cb.get_last_output_or_empty(),
            TextComponent::text("#itzlipofutzli")
        );

        let mut saved = NbtCompound::new();
        cart.write_nbt(&mut saved);
        assert_eq!(saved.get_string("Command"), Some("Searge"));
        assert_eq!(saved.get_int("SuccessCount"), Some(1));
        assert_eq!(saved.get_bool("TrackOutput"), Some(true));
        assert!(saved.get("LastOutput").is_some());

        // Setting TrackOutput=false clears LastOutput on load and omits it on save.
        let mut no_track = saved.clone();
        no_track.put_bool("TrackOutput", false);
        cart.read_nbt_non_mut(&no_track);
        let mut saved_no_track = NbtCompound::new();
        cart.write_nbt(&mut saved_no_track);
        assert!(saved_no_track.get("LastOutput").is_none());

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn command_block_minecart_perform_command_gamerule() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::COMMAND_BLOCK_MINECART,
        ));
        let Some(cb) = cart.command_block() else {
            return Err("expected CommandBlockMinecart".into());
        };

        // Decompiled vanilla `BaseCommandBlock#performCommand` resets SuccessCount before the
        // commandBlocksWork guard and updates lastExecution even when the gamerule is off.
        let mut level_info = (**world.level_info.load()).clone();
        level_info.game_rules.command_blocks_work = false;
        world.level_info.store(Arc::new(level_info));

        let mut nbt = NbtCompound::new();
        nbt.put_string("Command", "say hello".to_string());
        nbt.put_int("SuccessCount", 7);
        cart.read_nbt_non_mut(&nbt);
        assert!(cb.perform_command(&cart.vehicle.entity, &world));
        assert_eq!(cb.get_success_count(), 0);
        assert_eq!(cb.get_last_output_or_empty(), TextComponent::empty());

        // The Searge easter egg is checked before the gamerule guard with equalsIgnoreCase,
        // so lowercase "searge" still fires with command blocks disabled.
        let mut nbt = NbtCompound::new();
        nbt.put_string("Command", "searge".to_string());
        cart.read_nbt_non_mut(&nbt);
        assert!(cb.perform_command(&cart.vehicle.entity, &world));
        assert_eq!(cb.get_success_count(), 1);
        assert_eq!(
            cb.get_last_output_or_empty(),
            TextComponent::text("#itzlipofutzli")
        );

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn spawner_minecart_nbt_roundtrip() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cart = MinecartEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SPAWNER_MINECART,
        ));

        let mut spawn_entity = NbtCompound::new();
        spawn_entity.put_string("id", "minecraft:pig".to_string());
        let mut spawn_data = NbtCompound::new();
        spawn_data.put_compound("entity", spawn_entity);

        let mut nbt = NbtCompound::new();
        nbt.put_short("Delay", 45);
        nbt.put_short("MinSpawnDelay", 100);
        nbt.put_short("MaxSpawnDelay", 300);
        nbt.put_short("SpawnCount", 2);
        nbt.put_short("SpawnRange", 3);
        nbt.put_short("MaxNearbyEntities", 5);
        nbt.put_short("RequiredPlayerRange", 12);
        nbt.put_compound("SpawnData", spawn_data);

        cart.read_nbt_non_mut(&nbt);

        let mut saved = NbtCompound::new();
        cart.write_nbt(&mut saved);
        assert_eq!(saved.get_short("Delay"), Some(45));
        assert_eq!(saved.get_short("MinSpawnDelay"), Some(100));
        assert_eq!(saved.get_short("MaxSpawnDelay"), Some(300));
        assert_eq!(saved.get_short("SpawnCount"), Some(2));
        assert_eq!(saved.get_short("SpawnRange"), Some(3));
        assert_eq!(saved.get_short("MaxNearbyEntities"), Some(5));
        assert_eq!(saved.get_short("RequiredPlayerRange"), Some(12));
        assert_eq!(
            saved
                .get_compound("SpawnData")
                .and_then(|d| d.get_compound("entity"))
                .and_then(|e| e.get_string("id")),
            Some("minecraft:pig")
        );

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn minecart_drop_item_matches_vanilla() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        // Vanilla overrides `getDropItem` per minecart type; the command block minecart drops
        // its own item, not a plain minecart.
        for (entity_type, drop) in [
            (&EntityType::MINECART, &Item::MINECART),
            (&EntityType::CHEST_MINECART, &Item::CHEST_MINECART),
            (
                &EntityType::COMMAND_BLOCK_MINECART,
                &Item::COMMAND_BLOCK_MINECART,
            ),
            (&EntityType::FURNACE_MINECART, &Item::FURNACE_MINECART),
            (&EntityType::HOPPER_MINECART, &Item::HOPPER_MINECART),
            (&EntityType::TNT_MINECART, &Item::TNT_MINECART),
        ] {
            let cart = MinecartEntity::new(Entity::new(
                world.clone(),
                Vector3::new(0.5, 64.0, 0.5),
                entity_type,
            ));
            assert_eq!(cart.drop_item().id, drop.id);
        }

        world.level.shutdown().await;
        Ok(())
    }
}
