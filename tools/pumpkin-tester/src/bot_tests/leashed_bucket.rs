use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_core::{
    entity::{
        Entity, EntityBase,
        decoration::leash_knot::LeashKnotEntity,
        item::ItemEntity,
        passive::{axolotl::AxolotlEntity, cow::CowEntity},
        player::Player,
    },
    net::ClientPlatform,
    plugin::{
        BoxFuture, EventHandler,
        api::events::{
            EventPriority,
            entity::{
                entity_dismount::EntityDismountEvent, entity_unleash::EntityUnleashEvent,
                item_spawn::ItemSpawnEvent,
            },
            vehicle::vehicle_exit::VehicleExitEvent,
        },
    },
    server::Server,
    world::World,
};
use pumpkin_data::{entity::EntityType, item::Item, item_stack::ItemStack};
use pumpkin_protocol::{
    codec::var_int::VarInt,
    java::server::play::{ActionType, SInteract},
};
use pumpkin_util::{GameMode, Hand};

struct UnleashEvents {
    entity_id: i32,
    cancel: AtomicBool,
}

impl EventHandler<EntityUnleashEvent> for UnleashEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityUnleashEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                event.cancelled = self.cancel.load(Ordering::Relaxed);
            }
        })
    }
}

struct DismountEvents {
    entity_id: i32,
    cancel: AtomicBool,
}

impl EventHandler<EntityDismountEvent> for DismountEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityDismountEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                event.cancelled = self.cancel.load(Ordering::Relaxed);
            }
        })
    }
}

impl EventHandler<VehicleExitEvent> for DismountEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut VehicleExitEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.exited_id == self.entity_id {
                event.cancelled = self.cancel.load(Ordering::Relaxed);
            }
        })
    }
}

struct SpawnEvents {
    cancel: AtomicBool,
}

impl EventHandler<ItemSpawnEvent> for SpawnEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut ItemSpawnEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            event.cancelled = self.cancel.load(Ordering::Relaxed);
        })
    }
}

pub fn run_leashed_bucket_pickup_test(
    server: &Arc<Server>,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let player = server
        .get_player_by_name("TestBot1")
        .ok_or("Missing TestBot1")?;
    let world = player.world();
    let original_mode = player.gamemode.load();
    let original_main = player.inventory.get_stack_in_hand(Hand::Right);
    player.set_gamemode(GameMode::Survival);
    let result = check_leashed_pickup(server, &player, &world)
        .and_then(|()| check_expired_holder_pickup(server, &player, &world))
        .and_then(|()| check_cancelled_dismount_blocks_leash(server, &player, &world))
        .and_then(|()| check_item_spawn_cancellation(server, &player, &world));
    player
        .inventory
        .set_stack_in_hand(Hand::Right, original_main);
    player.set_gamemode(original_mode);
    result?;
    summary_lines.push("[PASS] bot:leashed_bucket_pickup_respects_unleash".into());
    Ok(())
}

fn spawn_axolotl(player: &Player, world: &Arc<World>) -> Result<Arc<AxolotlEntity>, String> {
    let axolotl = AxolotlEntity::new(Entity::new(
        world.clone(),
        player.position().add_raw(2.0, 0.0, 0.0),
        &EntityType::AXOLOTL,
    ));
    axolotl.mob_entity.set_no_ai(true);
    if world.spawn_entity(axolotl.clone()) {
        Ok(axolotl)
    } else {
        Err("Could not spawn the axolotl".into())
    }
}

fn scoop(
    server: &Arc<Server>,
    player: &Arc<Player>,
    axolotl: &AxolotlEntity,
) -> Result<(), String> {
    let ClientPlatform::Java(client) = player.client.as_ref() else {
        return Err("Expected a Java player for the bucket test".into());
    };
    player
        .inventory
        .set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::WATER_BUCKET));
    let interact = SInteract {
        entity_id: VarInt(axolotl.get_entity().entity_id),
        r#type: VarInt(ActionType::Interact as i32),
        target_position: None,
        hand: Some(VarInt(0)),
        sneaking: false,
    };
    client.handle_interact(player, &interact, server);
    Ok(())
}

fn check_leashed_pickup(
    server: &Arc<Server>,
    player: &Arc<Player>,
    world: &Arc<World>,
) -> Result<(), String> {
    let axolotl = spawn_axolotl(player, world)?;
    let holder = CowEntity::new(Entity::new(
        world.clone(),
        player.position().add_raw(3.0, 0.0, 0.0),
        &EntityType::COW,
    ));
    holder.mob_entity.set_no_ai(true);
    if !world.spawn_entity(holder.clone()) {
        axolotl.get_entity().remove();
        return Err("Could not spawn the leash holder".into());
    }
    let holder: Arc<dyn EntityBase> = holder;
    axolotl.get_entity().attach_leash(&holder);
    let hooks = Arc::new(UnleashEvents {
        entity_id: axolotl.get_entity().entity_id,
        cancel: AtomicBool::new(true),
    });
    server.plugin_manager.register::<EntityUnleashEvent, _>(
        hooks.clone(),
        EventPriority::Normal,
        true,
    );

    let leads_before = lead_drops(world);
    let result = scoop(server, player, &axolotl).and_then(|()| {
        if axolotl.get_entity().is_removed()
            || axolotl.get_entity().get_leash_holder().is_none()
            || player.inventory.held_item().item != &Item::WATER_BUCKET
            || lead_drops(world) != leads_before
        {
            return Err("A cancelled unleash still bucketed the mob or changed the leash".into());
        }
        hooks.cancel.store(false, Ordering::Relaxed);
        scoop(server, player, &axolotl)?;
        if !axolotl.get_entity().is_removed()
            || player.inventory.held_item().item != &Item::AXOLOTL_BUCKET
            || lead_drops(world) != leads_before + 1
        {
            return Err("An allowed unleash did not bucket the mob and drop one lead".into());
        }
        Ok(())
    });
    axolotl.get_entity().remove();
    holder.get_entity().remove();
    result
}

fn check_expired_holder_pickup(
    server: &Arc<Server>,
    player: &Arc<Player>,
    world: &Arc<World>,
) -> Result<(), String> {
    let axolotl = spawn_axolotl(player, world)?;
    let knot: Arc<dyn EntityBase> = Arc::new(LeashKnotEntity::new(Entity::new(
        world.clone(),
        player.position(),
        &EntityType::LEASH_KNOT,
    )));
    axolotl.get_entity().attach_leash(&knot);
    drop(knot);

    let leads_before = lead_drops(world);
    let result = scoop(server, player, &axolotl).and_then(|()| {
        if !axolotl.get_entity().is_removed()
            || player.inventory.held_item().item != &Item::AXOLOTL_BUCKET
            || lead_drops(world) != leads_before + 1
        {
            return Err("A leash whose holder was gone blocked the pickup or lost its lead".into());
        }
        Ok(())
    });
    axolotl.get_entity().remove();
    result
}

/// A cancelled dismount must not leave a mob both leashed and riding.
fn check_cancelled_dismount_blocks_leash(
    server: &Arc<Server>,
    player: &Arc<Player>,
    world: &Arc<World>,
) -> Result<(), String> {
    let cow = CowEntity::new(Entity::new(
        world.clone(),
        player.position().add_raw(4.0, 0.0, 0.0),
        &EntityType::COW,
    ));
    cow.mob_entity.set_no_ai(true);
    let cow: Arc<dyn EntityBase> = cow;
    if !world.spawn_entity(cow.clone()) {
        return Err("Could not spawn the dismount test mob".into());
    }
    let vehicle = CowEntity::new(Entity::new(
        world.clone(),
        player.position().add_raw(5.0, 0.0, 0.0),
        &EntityType::COW,
    ));
    vehicle.mob_entity.set_no_ai(true);
    let vehicle: Arc<dyn EntityBase> = vehicle;
    if !world.spawn_entity(vehicle.clone()) {
        cow.get_entity().remove();
        return Err("Could not spawn the dismount test vehicle".into());
    }
    vehicle
        .get_entity()
        .add_passenger(vehicle.clone(), cow.clone());
    let hooks = Arc::new(DismountEvents {
        entity_id: cow.get_entity().entity_id,
        cancel: AtomicBool::new(true),
    });
    server.plugin_manager.register::<EntityDismountEvent, _>(
        hooks.clone(),
        EventPriority::Normal,
        true,
    );
    server
        .plugin_manager
        .register::<VehicleExitEvent, _>(hooks, EventPriority::Normal, true);

    let holder: Arc<dyn EntityBase> = CowEntity::new(Entity::new(
        world.clone(),
        player.position().add_raw(6.0, 0.0, 0.0),
        &EntityType::COW,
    ));
    cow.get_entity().attach_leash(&holder);
    let result = if cow.get_entity().is_leashed() || cow.get_entity().get_vehicle().is_none() {
        Err("A cancelled dismount left the mob leashed or dismounted".into())
    } else {
        Ok(())
    };
    cow.get_entity().remove();
    vehicle.get_entity().remove();
    holder.get_entity().remove();
    result
}

/// Every item drop goes through the shared helper and honours `ItemSpawnEvent` cancellation.
fn check_item_spawn_cancellation(
    server: &Arc<Server>,
    player: &Arc<Player>,
    world: &Arc<World>,
) -> Result<(), String> {
    let cow = CowEntity::new(Entity::new(
        world.clone(),
        player.position().add_raw(7.0, 0.0, 0.0),
        &EntityType::COW,
    ));
    let cow: Arc<dyn EntityBase> = cow;
    let hooks = Arc::new(SpawnEvents {
        cancel: AtomicBool::new(true),
    });
    server
        .plugin_manager
        .register::<ItemSpawnEvent, _>(hooks.clone(), EventPriority::Normal, true);

    let before = lead_drops(world);
    cow.get_entity()
        .spawn_at_location(ItemStack::new(1, &Item::LEAD));
    let cancelled = lead_drops(world) == before;
    hooks.cancel.store(false, Ordering::Relaxed);
    cow.get_entity()
        .spawn_at_location(ItemStack::new(1, &Item::LEAD));
    let allowed = lead_drops(world) == before + 1;
    let result = if cancelled && allowed {
        Ok(())
    } else {
        Err("ItemSpawnEvent cancellation was not honoured by the shared helper".into())
    };
    // Remove the dropped lead so later tests start from a clean world.
    for entity in world.entities.load().iter() {
        if entity.get_entity().get_item_entity().is_some_and(|item| {
            item.get_item_stack()
                .lock()
                .is_ok_and(|s| s.item == &Item::LEAD)
        }) {
            entity.get_entity().remove();
        }
    }
    result
}

fn lead_drops(world: &World) -> usize {
    world
        .entities
        .load()
        .iter()
        .filter_map(|entity| entity.cast_any().downcast_ref::<ItemEntity>())
        .map(|entity| {
            let stack = entity
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if stack.item == &Item::LEAD {
                usize::from(stack.item_count)
            } else {
                0
            }
        })
        .sum()
}
