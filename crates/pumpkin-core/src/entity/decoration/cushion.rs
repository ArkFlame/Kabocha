use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU32, Ordering},
};

use pumpkin_data::damage::DamageType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::server::Server;
use crate::world::World;

pub struct CushionEntity {
    pub entity: Entity,
    color: AtomicU8,
    ticks_since_last_check: AtomicU32,
}

impl CushionEntity {
    const ANCHOR_DEPTH: f64 = 0.015_625;

    /// Vanilla `Cushion.canBePlacedAt`.
    pub fn can_be_placed_at(world: &World, bounding_box: &BoundingBox) -> bool {
        Self::would_survive_at(world, bounding_box) && !Self::is_anchor_buried(world, bounding_box)
    }

    /// Vanilla `Cushion.wouldSurviveAt`.
    pub fn would_survive_at(world: &World, bounding_box: &BoundingBox) -> bool {
        Self::has_anchor_below(world, bounding_box)
            && !Self::is_covered_by_suffocating_blocks(world, bounding_box)
    }

    fn has_anchor_below(world: &World, bounding_box: &BoundingBox) -> bool {
        let anchor_box = BoundingBox::new(
            Vector3::new(
                bounding_box.min.x,
                bounding_box.min.y - Self::ANCHOR_DEPTH,
                bounding_box.min.z,
            ),
            Vector3::new(
                bounding_box.max.x.next_down(),
                bounding_box.min.y,
                bounding_box.max.z.next_down(),
            ),
        );
        let mut search_box = anchor_box;
        search_box.min.y -= 0.125;

        BlockPos::iterate(search_box.min_block_pos(), search_box.max_block_pos()).any(|pos| {
            let state = world.get_block_state(&pos);
            state
                .get_block_outline_shapes_at(&pos)
                .reduce(|bounds, shape| {
                    BoundingBox::new(
                        Vector3::new(
                            bounds.min.x.min(shape.min.x),
                            bounds.min.y.min(shape.min.y),
                            bounds.min.z.min(shape.min.z),
                        ),
                        Vector3::new(
                            bounds.max.x.max(shape.max.x),
                            bounds.max.y.max(shape.max.y),
                            bounds.max.z.max(shape.max.z),
                        ),
                    )
                })
                .is_some_and(|bounds| bounds.at_pos(pos).intersects(&anchor_box))
        })
    }

    fn is_anchor_buried(world: &World, bounding_box: &BoundingBox) -> bool {
        let mut resting_slice = *bounding_box;
        resting_slice.max.y = resting_slice.min.y + Self::ANCHOR_DEPTH;
        let resting_slice = resting_slice.contract_all(1.0e-7);

        let mut exposed_surface = vec![resting_slice];
        for pos in BlockPos::iterate(resting_slice.min_block_pos(), resting_slice.max_block_pos()) {
            let state = world.get_block_state(&pos);
            for collider in state.get_block_collision_shapes_at(&pos) {
                let collider = collider.at_pos(pos);
                exposed_surface = exposed_surface
                    .into_iter()
                    .flat_map(|part| subtract_box(part, &collider))
                    .collect();
                if exposed_surface.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    fn is_covered_by_suffocating_blocks(world: &World, bounding_box: &BoundingBox) -> bool {
        let inner = bounding_box.contract_all(1.0e-7);
        BlockPos::iterate(inner.min_block_pos(), inner.max_block_pos())
            .all(|pos| world.get_block_state(&pos).is_suffocating())
    }

    pub const fn new(entity: Entity, color: u8) -> Self {
        Self {
            entity,
            color: AtomicU8::new(color),
            ticks_since_last_check: AtomicU32::new(0),
        }
    }

    pub fn color(&self) -> u8 {
        self.color.load(Ordering::Relaxed)
    }

    pub fn set_color(&self, color: u8) {
        self.color.store(color, Ordering::Relaxed);
        self.sync_color();
    }

    fn sync_color(&self) {
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::cushion::COLOR,
            VarInt(i32::from(self.color())),
        );
    }

    #[must_use]
    pub const fn item_for_color(color: u8) -> &'static Item {
        match color {
            0 => &Item::WHITE_CUSHION,
            1 => &Item::ORANGE_CUSHION,
            2 => &Item::MAGENTA_CUSHION,
            3 => &Item::LIGHT_BLUE_CUSHION,
            4 => &Item::YELLOW_CUSHION,
            5 => &Item::LIME_CUSHION,
            6 => &Item::PINK_CUSHION,
            7 => &Item::GRAY_CUSHION,
            8 => &Item::LIGHT_GRAY_CUSHION,
            9 => &Item::CYAN_CUSHION,
            10 => &Item::PURPLE_CUSHION,
            11 => &Item::BLUE_CUSHION,
            12 => &Item::BROWN_CUSHION,
            13 => &Item::GREEN_CUSHION,
            14 => &Item::RED_CUSHION,
            _ => &Item::BLACK_CUSHION,
        }
    }

    #[must_use]
    pub fn color_from_item(item: &Item) -> u8 {
        for (id, comp) in item.components {
            if *id == pumpkin_data::data_component::DataComponent::CushionColor {
                return pumpkin_data::data_component_impl::get::<
                    pumpkin_data::data_component_impl::CushionColorImpl,
                >(*comp)
                .color
                .id();
            }
        }
        0
    }

    #[must_use]
    pub fn color_from_stack(stack: &ItemStack) -> u8 {
        stack
            .get_data_component::<pumpkin_data::data_component_impl::CushionColorImpl>()
            .map_or_else(|| Self::color_from_item(stack.get_item()), |c| c.color.id())
    }

    fn drop_and_remove(&self, caused_by: Option<&dyn EntityBase>) {
        let entity = &self.entity;
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for passenger in passengers {
            entity.remove_passenger(passenger.get_entity().entity_id);
        }
        // Vanilla `BlockAttachedEntity.hurtServer` removes and drops the cushion
        // unconditionally; a cancelled dismount leaves the rider attached but the
        // cushion still breaks, so a surviving rider must not block the break.

        let world = entity.world.load();
        // Vanilla `BlockAttachedEntity` drop path: break sound and drop only
        // happen under `doEntityDrops`, and a creative attacker gets no item.
        if world.level_info.load().game_rules.entity_drops {
            world.play_sound(
                Sound::EntityCushionBreak,
                SoundCategory::Blocks,
                &entity.pos.load(),
            );

            let is_creative = caused_by.is_some_and(|c| {
                c.cast_any()
                    .downcast_ref::<Player>()
                    .is_some_and(Player::is_creative)
            });

            if !is_creative {
                world.drop_stack(
                    &entity.block_pos.load(),
                    ItemStack::new(1, Self::item_for_color(self.color())),
                );
            }
        }
        entity.remove();
    }
}

/// The parts of `part` that `other` doesn't cover.
fn subtract_box(part: BoundingBox, other: &BoundingBox) -> Vec<BoundingBox> {
    if !part.intersects(other) {
        return vec![part];
    }
    let mut pieces = Vec::new();
    let mut rest = part;
    if rest.min.x < other.min.x {
        pieces.push(BoundingBox::new(
            rest.min,
            Vector3::new(other.min.x, rest.max.y, rest.max.z),
        ));
        rest.min.x = other.min.x;
    }
    if rest.max.x > other.max.x {
        pieces.push(BoundingBox::new(
            Vector3::new(other.max.x, rest.min.y, rest.min.z),
            rest.max,
        ));
        rest.max.x = other.max.x;
    }
    if rest.min.y < other.min.y {
        pieces.push(BoundingBox::new(
            rest.min,
            Vector3::new(rest.max.x, other.min.y, rest.max.z),
        ));
        rest.min.y = other.min.y;
    }
    if rest.max.y > other.max.y {
        pieces.push(BoundingBox::new(
            Vector3::new(rest.min.x, other.max.y, rest.min.z),
            rest.max,
        ));
        rest.max.y = other.max.y;
    }
    if rest.min.z < other.min.z {
        pieces.push(BoundingBox::new(
            rest.min,
            Vector3::new(rest.max.x, rest.max.y, other.min.z),
        ));
        rest.min.z = other.min.z;
    }
    if rest.max.z > other.max.z {
        pieces.push(BoundingBox::new(
            Vector3::new(rest.min.x, rest.min.y, other.max.z),
            rest.max,
        ));
    }
    pieces
}

impl EntityBase for CushionEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn init_data_tracker(&self) {
        self.sync_color();
    }

    /// The colour has to ride along with the spawn packet; a client seeing the
    /// cushion for the first time never got the tracked data update.
    fn java_spawn_metadata(&self, version: JavaMinecraftVersion) -> Option<Box<[u8]>> {
        let mut metadata = Vec::new();
        Metadata::new(
            pumpkin_data::tracked_data::cushion::COLOR,
            VarInt(i32::from(self.color())),
        )
        .write(&mut metadata, &version)
        .ok()?;
        metadata.push(255);
        Some(metadata.into_boxed_slice())
    }

    /// Vanilla BlockAttachedEntity.tick: drops the cushion once nothing holds it up.
    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        if self.ticks_since_last_check.fetch_add(1, Ordering::Relaxed) < 100 {
            return;
        }
        self.ticks_since_last_check.store(0, Ordering::Relaxed);
        let world = self.entity.world.load();
        if self.entity.is_alive()
            && !Self::would_survive_at(&world, &self.entity.bounding_box.load())
        {
            self.drop_and_remove(None);
        }
    }

    fn can_hit(&self) -> bool {
        self.entity.is_alive()
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_byte("Color", self.color() as i8);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        if let Some(color) = nbt.get_byte("Color") {
            // Vanilla resolves the stored byte through `DyeColor.byId`, which
            // falls back to white instead of syncing an out-of-range dye id.
            let color = pumpkin_data::dye_color::DyeColor::by_id_or_white(i32::from(color as u8));
            self.set_color(color.id());
        }
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        if player.get_entity().is_sneaking() {
            return false;
        }
        // Do not replace an unacknowledged teleport with another seat switch.
        if player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
        {
            return false;
        }
        if !self
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
        {
            return false;
        }

        let world = self.entity.world.load();
        let Some(vehicle) = world.get_entity_by_id(self.entity.entity_id) else {
            return false;
        };
        let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
            return false;
        };

        // Vanilla `startRiding` leaves the current vehicle first, so this switches seats.
        let current_vehicle = player
            .get_entity()
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        // A dead weak ref means the old vehicle is already gone, so there is nothing to leave.
        if let Some(current_vehicle) = current_vehicle.and_then(|weak| weak.upgrade()) {
            current_vehicle
                .get_entity()
                .remove_passenger(player.entity_id());
            if player.get_entity().has_vehicle() {
                return false;
            }
        }

        self.entity
            .add_passenger(vehicle.clone(), passenger.clone() as Arc<dyn EntityBase>);
        if player
            .get_entity()
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .is_none_or(|vehicle| vehicle.get_entity().entity_id != self.entity.entity_id)
        {
            return false;
        }

        // Vanilla `startRiding` seats the rider through `positionRider`, which
        // subtracts the rider's `vehicle` attachment from the vehicle's `passenger` point.
        crate::entity::position_rider(vehicle.as_ref(), passenger.as_ref());
        let position = player.get_entity().pos.load();
        let mut awaiting_teleport = player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // The dismount teleport precedes the mount packet. Its acknowledgement
        // must preserve the new riding position, not restore the old seat.
        if let Some((_, target)) = awaiting_teleport.as_mut() {
            *target = position;
        }
        drop(awaiting_teleport);

        world.play_sound(
            Sound::EntityCushionSit,
            SoundCategory::Blocks,
            &self.entity.pos.load(),
        );
        true
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        // Vanilla `HangingEntity#hurt` ignores a hit on an entity that is already
        // removed, so one cushion cannot drop twice in the same tick.
        if !self.entity.is_alive() {
            return true;
        }
        self.drop_and_remove(source);
        true
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arc_swap::ArcSwap;
    use pumpkin_config::world::LevelConfig;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::{Block, BlockStateId};
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_world::chunk::ChunkData;
    use pumpkin_world::level::Level;
    use pumpkin_world::world_info::LevelData;
    use std::sync::Weak;

    #[test]
    fn subtract_box_underpins_anchor_buried_detection() {
        let part = BoundingBox::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));
        let overlap = BoundingBox::new(Vector3::new(0.5, 0.0, 0.0), Vector3::new(1.5, 1.0, 1.0));
        let pieces = subtract_box(part, &overlap);
        assert_eq!(pieces.len(), 1, "only the uncovered slab should remain");
        assert_eq!(pieces[0].min, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(pieces[0].max, Vector3::new(0.5, 1.0, 1.0));

        let covering =
            BoundingBox::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(2.0, 2.0, 2.0));
        assert!(
            subtract_box(part, &covering).is_empty(),
            "a fully covered resting slice counts as buried"
        );
    }

    /// A `World` with one loaded, empty chunk so placement rules can be exercised
    /// synchronously without force-loading or ticking a full server.
    struct PlacementWorld {
        world: Arc<World>,
        level: Arc<Level>,
        _directory: tempfile::TempDir,
    }

    impl PlacementWorld {
        fn new() -> Self {
            let directory = tempfile::tempdir().unwrap();
            let level = Level::from_root_folder(
                &LevelConfig::default(),
                directory.path().to_path_buf(),
                0,
                Dimension::OVERWORLD,
            );
            level
                .loaded_chunks
                .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
            let world = Arc::new(World::load(
                level.clone(),
                Arc::new(ArcSwap::from_pointee(LevelData::default(
                    pumpkin_util::world_seed::Seed(0),
                ))),
                Dimension::OVERWORLD,
                crate::block::registry::default_registry(),
                crate::world::weather::WeatherSetup::default(),
                Weak::new(),
            ));
            Self {
                world,
                level,
                _directory: directory,
            }
        }

        fn set_block(&self, position: BlockPos, state: BlockStateId) {
            let _ = self
                .level
                .read_chunk_sync(&position.chunk_position(), |chunk| {
                    chunk.set_blocks_batch([(
                        (position.0.x & 15) as usize,
                        position.0.y,
                        (position.0.z & 15) as usize,
                        state,
                    )]);
                });
        }

        async fn shutdown(self) {
            self.level.shutdown().await;
        }
    }

    /// Bounding box of a cushion resting on top of the block at (0, 64, 0).
    const fn resting_cushion_box() -> BoundingBox {
        BoundingBox::new(Vector3::new(0.0, 65.0, 0.0), Vector3::new(1.0, 65.25, 1.0))
    }

    #[tokio::test]
    async fn placement_needs_an_anchor_below() {
        let fixture = PlacementWorld::new();
        let bb = resting_cushion_box();

        assert!(
            !CushionEntity::can_be_placed_at(&fixture.world, &bb),
            "air cannot anchor a cushion"
        );

        fixture.set_block(BlockPos::new(0, 64, 0), Block::STONE.default_state.id);
        assert!(
            CushionEntity::would_survive_at(&fixture.world, &bb),
            "a full stone block supports the cushion"
        );
        assert!(
            CushionEntity::can_be_placed_at(&fixture.world, &bb),
            "a full stone block is a valid placement"
        );

        fixture.shutdown().await;
    }

    #[tokio::test]
    async fn placement_rejects_a_suffocating_cover() {
        let fixture = PlacementWorld::new();
        let bb = resting_cushion_box();
        fixture.set_block(BlockPos::new(0, 64, 0), Block::STONE.default_state.id);
        fixture.set_block(BlockPos::new(0, 65, 0), Block::STONE.default_state.id);

        assert!(
            CushionEntity::is_covered_by_suffocating_blocks(&fixture.world, &bb),
            "stone occupying the cushion volume is suffocating"
        );
        assert!(
            !CushionEntity::would_survive_at(&fixture.world, &bb),
            "a suffocating cover prevents the cushion from surviving"
        );

        fixture.shutdown().await;
    }

    #[tokio::test]
    async fn anchor_is_buried_when_a_block_fills_the_resting_slice() {
        let fixture = PlacementWorld::new();
        let bb = resting_cushion_box();

        assert!(
            !CushionEntity::is_anchor_buried(&fixture.world, &bb),
            "an empty resting slice is not buried"
        );

        fixture.set_block(BlockPos::new(0, 65, 0), Block::STONE.default_state.id);
        assert!(
            CushionEntity::is_anchor_buried(&fixture.world, &bb),
            "a block filling the resting slice buries the anchor"
        );

        fixture.shutdown().await;
    }

    #[tokio::test]
    async fn breaking_an_occupied_cushion_detaches_its_passenger() {
        let directory = tempfile::tempdir().unwrap();
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            directory.path().to_path_buf(),
            0,
            Dimension::OVERWORLD,
        );
        let world = Arc::new(World::load(
            level.clone(),
            Arc::new(ArcSwap::from_pointee(LevelData::default(
                pumpkin_util::world_seed::Seed(0),
            ))),
            Dimension::OVERWORLD,
            crate::block::registry::default_registry(),
            crate::world::weather::WeatherSetup::default(),
            Weak::new(),
        ));
        let position = Vector3::new(0.5, 64.0, 0.5);
        let cushion = Arc::new(CushionEntity::new(
            Entity::new(world.clone(), position, &EntityType::CUSHION),
            0,
        ));
        let passenger = Arc::new(Entity::new(world.clone(), position, &EntityType::PIG));
        world.spawn_entity(cushion.clone());
        cushion
            .entity
            .add_passenger(cushion.clone(), passenger.clone());

        cushion.drop_and_remove(None);
        let passenger_detached = !passenger.has_vehicle();
        let cushion_empty = !cushion.entity.has_passengers();
        let cushion_removed = world.get_entity_by_id(cushion.entity.entity_id).is_none();
        level.shutdown().await;

        assert!(
            passenger_detached,
            "passenger still references the removed cushion"
        );
        assert!(cushion_empty, "removed cushion still owns its passenger");
        assert!(cushion_removed);
    }

    #[tokio::test]
    async fn hit_on_already_removed_cushion_reports_handled() {
        let directory = tempfile::tempdir().unwrap();
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            directory.path().to_path_buf(),
            0,
            Dimension::OVERWORLD,
        );
        let world = Arc::new(World::load(
            level.clone(),
            Arc::new(ArcSwap::from_pointee(LevelData::default(
                pumpkin_util::world_seed::Seed(0),
            ))),
            Dimension::OVERWORLD,
            crate::block::registry::default_registry(),
            crate::world::weather::WeatherSetup::default(),
            Weak::new(),
        ));
        let position = Vector3::new(0.5, 64.0, 0.5);
        let cushion = Arc::new(CushionEntity::new(
            Entity::new(world.clone(), position, &EntityType::CUSHION),
            0,
        ));
        world.spawn_entity(cushion.clone());

        cushion.drop_and_remove(None);
        let handled =
            cushion.damage_with_context(&*cushion, 1.0, DamageType::GENERIC, None, None, None);
        level.shutdown().await;

        assert!(
            handled,
            "a second hit on a removed cushion must count as handled"
        );
    }
}
