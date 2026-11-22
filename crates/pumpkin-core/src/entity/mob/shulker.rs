use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockDirection;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CEntityPositionSync;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use rand::RngExt;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::look_around::RandomLookAroundGoal;
use crate::entity::ai::goal::look_at_entity::LookAtEntityGoal;
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::attributes::{Modifier, ModifierOperation};
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::projectile::shulker_bullet::ShulkerBulletEntity;
use crate::entity::{Entity, EntityBase};

const DEFAULT_ATTACH_FACE: BlockDirection = BlockDirection::Down;
const NO_COLOR: u8 = 16;
const PEEK_PER_TICK: f32 = 0.05;
const MAX_TELEPORT_DISTANCE: i32 = 8;
const OTHER_SHULKER_SCAN_RADIUS: f64 = 8.0;
const OTHER_SHULKER_LIMIT: f32 = 5.0;
/// Vanilla `Shulker.MODIFIER_COVERED_ARMOR`: +20 armor while fully closed.
const COVERED_ARMOR_MODIFIER_ID: &str = "covered";
const COVERED_ARMOR: f64 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

trait BlockDirectionExt {
    fn axis_of(self) -> Axis;
}

impl BlockDirectionExt for BlockDirection {
    fn axis_of(self) -> Axis {
        match self {
            Self::East | Self::West => Axis::X,
            Self::Up | Self::Down => Axis::Y,
            Self::North | Self::South => Axis::Z,
        }
    }
}

const SHULKER_HUNTS: &[Hunt] = &[Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true)];

pub struct ShulkerEntity {
    pub mob_entity: MobEntity,
    attach_face: AtomicU8,
    peek_amount: AtomicU8,
    color: AtomicU8,
    current_peek_amount: AtomicCell<f32>,
    prev_peek_amount: AtomicCell<f32>,
}

impl ShulkerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        entity.set_pos(Self::snap_position(entity.pos.load()));
        let mob_entity = MobEntity::new(entity);
        let shulker = Self {
            mob_entity,
            attach_face: AtomicU8::new(DEFAULT_ATTACH_FACE as u8),
            peek_amount: AtomicU8::new(0),
            color: AtomicU8::new(NO_COLOR),
            current_peek_amount: AtomicCell::new(0.0),
            prev_peek_amount: AtomicCell::new(0.0),
        };
        let mob_arc = Arc::new(shulker);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            // Vanilla Shulker passes onlyHorizontal=true so it looks at the player's
            // horizontal position only (own eye Y), not up/down at the player's eye.
            goal_selector.add_goal(
                1,
                Box::new(LookAtEntityGoal::new(
                    mob_weak,
                    &EntityType::PLAYER,
                    8.0,
                    0.02,
                    true,
                )),
            );
            goal_selector.add_goal(
                4,
                Box::new(ShulkerAttackGoal::new(Arc::downgrade(&mob_arc))),
            );
            goal_selector.add_goal(7, Box::new(ShulkerPeekGoal::new(Arc::downgrade(&mob_arc))));
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                1,
                Box::new(
                    RevengeGoal::new(true)
                        .ignoring(|entity_type| entity_type == &EntityType::SHULKER)
                        .alerting_others(),
                ),
            );
            hunting::apply(target_selector, &mob_arc.mob_entity, SHULKER_HUNTS);
            // TODO: search box flattened to 4 along the attach face; ShulkerDefenseAttackGoal at 3.
        };

        mob_arc
    }

    pub fn get_attach_face(&self) -> BlockDirection {
        BlockDirection::try_from(i32::from(self.attach_face.load(Ordering::Relaxed)))
            .unwrap_or(DEFAULT_ATTACH_FACE)
    }

    pub fn set_attach_face(&self, face: BlockDirection) {
        self.attach_face.store(face as u8, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(
            pumpkin_data::tracked_data::shulker::ATTACH_FACE_ID,
            VarInt(face as i32),
        );
    }

    pub fn get_color(&self) -> Option<u8> {
        let c = self.color.load(Ordering::Relaxed);
        if c == NO_COLOR { None } else { Some(c) }
    }

    pub fn set_color(&self, color: Option<u8>) {
        let val = color.unwrap_or(NO_COLOR);
        self.color.store(val, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(pumpkin_data::tracked_data::shulker::COLOR, val as i8);
    }

    pub fn get_raw_peek(&self) -> u8 {
        self.peek_amount.load(Ordering::Relaxed)
    }

    pub fn set_raw_peek(&self, amount: u8) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        if amount == 0 {
            world.play_sound_fine(
                Sound::EntityShulkerClose,
                SoundCategory::Hostile,
                &pos,
                1.0,
                1.0,
            );
        } else if self.peek_amount.load(Ordering::Relaxed) == 0 {
            world.play_sound_fine(
                Sound::EntityShulkerOpen,
                SoundCategory::Hostile,
                &pos,
                1.0,
                1.0,
            );
        }

        self.peek_amount.store(amount, Ordering::Relaxed);

        // Vanilla `setRawPeekAmount`: closed shulkers gain a permanent +20 armor modifier.
        let living = &self.mob_entity.living_entity;
        if amount == 0 {
            living.update_attribute(&Attributes::ARMOR, |armor| {
                armor.add_or_replace_permanent_modifier(Modifier {
                    id: COVERED_ARMOR_MODIFIER_ID.to_string(),
                    amount: COVERED_ARMOR,
                    operation: ModifierOperation::Add,
                });
            });
        } else {
            living.update_attribute(&Attributes::ARMOR, |armor| {
                armor.remove_modifier(COVERED_ARMOR_MODIFIER_ID);
            });
        }

        entity.set_synced_data(pumpkin_data::tracked_data::shulker::PEEK_ID, amount);
    }

    pub fn is_closed(&self) -> bool {
        self.peek_amount.load(Ordering::Relaxed) == 0
    }

    fn update_peek_amount(&self) -> bool {
        let current = self.current_peek_amount.load();
        self.prev_peek_amount.store(current);
        let target = f32::from(self.get_raw_peek()) * 0.01;

        if (current - target).abs() < 1e-6 {
            return false;
        }

        let new = if current > target {
            (current - PEEK_PER_TICK).max(target)
        } else {
            (current + PEEK_PER_TICK).min(target)
        };

        self.current_peek_amount.store(new);
        true
    }

    pub(crate) fn snap_position(pos: Vector3<f64>) -> Vector3<f64> {
        Vector3::new(
            pos.x.floor() + 0.5,
            (pos.y + 0.5).floor(),
            pos.z.floor() + 0.5,
        )
    }

    fn physical_peek(amount: f32) -> f32 {
        0.5 - pumpkin_util::math::sin((0.5 + amount) * std::f32::consts::PI) * 0.5
    }

    fn progress_aabb(
        size: f32,
        direction: BlockDirection,
        from: f32,
        to: f32,
        pos: Vector3<f64>,
    ) -> BoundingBox {
        let size = f64::from(size);
        let offset = direction.to_offset();
        let movement = Vector3::new(
            f64::from(offset.x),
            f64::from(offset.y),
            f64::from(offset.z),
        );
        let mut bounds = BoundingBox {
            min: Vector3::new(-size * 0.5, 0.0, -size * 0.5),
            max: Vector3::new(size * 0.5, size, size * 0.5),
        }
        .expand_towards(
            movement.x * f64::from(from.max(to)) * size,
            movement.y * f64::from(from.max(to)) * size,
            movement.z * f64::from(from.max(to)) * size,
        );
        let inset = (1.0 + f64::from(from.min(to))) * size;
        match direction {
            BlockDirection::Down => bounds.max.y -= inset,
            BlockDirection::Up => bounds.min.y += inset,
            BlockDirection::North => bounds.max.z -= inset,
            BlockDirection::South => bounds.min.z += inset,
            BlockDirection::West => bounds.max.x -= inset,
            BlockDirection::East => bounds.min.x += inset,
        }
        bounds.shift(pos)
    }

    fn on_peek_amount_change(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        entity.refresh_dimensions();
        let old = Self::physical_peek(self.prev_peek_amount.load());
        let current = Self::physical_peek(self.current_peek_amount.load());
        let push = (current - old) * entity.scale.load();
        if push <= 0.0 {
            return;
        }
        let direction = self.get_attach_face().opposite();
        let area = Self::progress_aabb(
            entity.scale.load(),
            direction,
            old,
            current,
            entity.pos.load(),
        );
        let offset = direction.to_offset();
        let motion = Vector3::new(
            f64::from(offset.x) * f64::from(push),
            f64::from(offset.y) * f64::from(push),
            f64::from(offset.z) * f64::from(push),
        );
        for other in entity.world.load().get_all_at_box(&area) {
            if !other.is_spectator()
                && other.get_entity().entity_type != &EntityType::SHULKER
                && !other.get_entity().no_physics.load(Ordering::Relaxed)
                && !entity.is_passenger_of_same_vehicle(other.get_entity())
            {
                other.move_by_shulker(other.as_ref(), motion);
            }
        }
    }

    fn hit_by_shulker_bullet(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let old_pos = entity.pos.load();
        let old_box = entity.bounding_box.load();
        if self.is_closed() || !self.teleport_somewhere() {
            return;
        }
        let world = entity.world.load();
        let count = world
            .get_entities_of_type_at_box(&old_box.expand_all(OTHER_SHULKER_SCAN_RADIUS), |kind| {
                kind == &EntityType::SHULKER
            })
            .iter()
            .filter(|other| {
                other
                    .get_living_entity()
                    .is_some_and(crate::entity::living::LivingEntity::is_alive)
            })
            .count();
        if rand::rng().random::<f32>() < (count as f32 - 1.0) / OTHER_SHULKER_LIMIT {
            return;
        }
        let clone = Self::new(Entity::from_uuid(
            uuid::Uuid::new_v4(),
            (*world).clone(),
            old_pos,
            &EntityType::SHULKER,
        ));
        clone.set_color(self.get_color());
        world.spawn_entity(clone);
    }

    fn find_attachable_face(&self, pos: &BlockPos) -> Option<BlockDirection> {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        for dir in BlockDirection::all() {
            let neighbour = pos.offset_direction(dir);
            let state = world.get_block_state(&neighbour);
            if state.is_solid() {
                return Some(dir);
            }
        }
        None
    }

    fn can_stay_at(&self, pos: &BlockPos, face: BlockDirection) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();

        let own_state = world.get_block_state(pos);
        if !own_state.is_air() {
            return false;
        }

        let neighbour = pos.offset_direction(face);
        let state = world.get_block_state(&neighbour);
        state.is_solid()
    }

    fn find_new_attachment(&self) {
        let pos = self.mob_entity.living_entity.entity.block_pos.load();
        if let Some(dir) = self.find_attachable_face(&pos) {
            self.set_attach_face(dir);
        } else {
            self.teleport_somewhere();
        }
    }

    pub fn teleport_somewhere(&self) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let base_pos = entity.block_pos.load();
        let world = entity.world.load();

        let candidates: Vec<(i32, i32, i32)> = {
            let mut rng = rand::rng();
            (0..20)
                .map(|_| {
                    (
                        rng.random_range(-MAX_TELEPORT_DISTANCE..=MAX_TELEPORT_DISTANCE),
                        rng.random_range(-MAX_TELEPORT_DISTANCE..=MAX_TELEPORT_DISTANCE),
                        rng.random_range(-MAX_TELEPORT_DISTANCE..=MAX_TELEPORT_DISTANCE),
                    )
                })
                .collect()
        };

        for (dx, dy, dz) in candidates {
            let candidate = BlockPos::new(base_pos.0.x + dx, base_pos.0.y + dy, base_pos.0.z + dz);

            let candidate_state = world.get_block_state(&candidate);
            if !candidate_state.is_air() {
                continue;
            }

            if let Some(dir) = self.find_attachable_face(&candidate) {
                let new_pos = Vector3::new(
                    f64::from(candidate.0.x) + 0.5,
                    f64::from(candidate.0.y),
                    f64::from(candidate.0.z) + 0.5,
                );

                self.set_attach_face(dir);
                entity.set_pos(new_pos);

                let chunk_pos = entity.chunk_pos.load();
                world.broadcast_to_chunk(
                    chunk_pos,
                    &CEntityPositionSync::new(
                        entity.entity_id.into(),
                        new_pos,
                        Vector3::new(0.0, 0.0, 0.0),
                        entity.yaw.load(),
                        entity.pitch.load(),
                        entity.on_ground.load(Ordering::Relaxed),
                    ),
                );

                entity.last_sent_pos.store(new_pos);

                world.play_sound_fine(
                    Sound::EntityShulkerTeleport,
                    SoundCategory::Hostile,
                    &new_pos,
                    1.0,
                    1.0,
                );

                self.set_raw_peek(0);
                self.mob_entity.set_target(None);

                return true;
            }
        }
        false
    }

    fn on_shulker_damage(&self) {
        self.teleport_somewhere();
    }
}

impl Mob for ShulkerEntity {
    fn on_start_riding(&self) {
        self.set_attach_face(BlockDirection::Down);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_byte("AttachFace", self.attach_face.load(Ordering::Relaxed) as i8);
        nbt.put_byte("PeekAmount", self.peek_amount.load(Ordering::Relaxed) as i8);
        nbt.put_byte("Color", self.color.load(Ordering::Relaxed) as i8);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(face) = nbt.get_byte("AttachFace") {
            self.attach_face.store(face as u8, Ordering::Relaxed);
        }
        if let Some(peek) = nbt.get_byte("PeekAmount") {
            self.peek_amount.store(peek as u8, Ordering::Relaxed);
        }
        if let Some(color) = nbt.get_byte("Color") {
            self.color.store(color as u8, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;

        if !entity.is_alive() {
            return;
        }

        entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));

        if self.update_peek_amount() {
            self.on_peek_amount_change();
        }

        let pos = entity.block_pos.load();
        let face = self.get_attach_face();
        if !entity.has_vehicle() && !self.can_stay_at(&pos, face) {
            self.find_new_attachment();
        }
    }

    fn on_damage(&self, damage_type: DamageType, source: Option<&dyn EntityBase>) {
        let living = &self.mob_entity.living_entity;
        if living.health.load() < living.get_max_health() * 0.5
            && rand::rng().random_range(0..4) == 0
        {
            self.on_shulker_damage();
        } else if damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_PROJECTILE)
            && source.is_some_and(|source| {
                source.get_entity().entity_type == &EntityType::SHULKER_BULLET
            })
        {
            self.hit_by_shulker_bullet();
        }
    }

    fn pre_damage(&self, damage_type: DamageType, _source: Option<&dyn EntityBase>) -> bool {
        !(self.is_closed() && damage_type == DamageType::ARROW)
    }
}

struct ShulkerAttackGoal {
    shulker: Weak<ShulkerEntity>,
    attack_cooldown: AtomicI32,
}

impl ShulkerAttackGoal {
    const fn new(shulker: Weak<ShulkerEntity>) -> Self {
        Self {
            shulker,
            attack_cooldown: AtomicI32::new(20),
        }
    }
}

impl Goal for ShulkerAttackGoal {
    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target();
        target
            .as_ref()
            .is_some_and(|t| t.get_living_entity().is_some_and(|l| l.entity.is_alive()))
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target();
        target
            .as_ref()
            .is_some_and(|t| t.get_living_entity().is_some_and(|l| l.entity.is_alive()))
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.attack_cooldown.store(20, Ordering::Relaxed);
        if let Some(shulker) = self.shulker.upgrade() {
            shulker.set_raw_peek(100);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(shulker) = self.shulker.upgrade() {
            shulker.set_raw_peek(0);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(shulker) = self.shulker.upgrade() else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let target_arc = mob_entity.get_target();

        let Some(target) = target_arc else {
            return;
        };

        if !target.get_entity().is_alive() {
            return;
        }

        let entity = &mob_entity.living_entity.entity;
        let shulker_pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = shulker_pos.squared_distance_to_vec(&target_pos);

        if dist_sq > 400.0 {
            mob_entity.set_target(None);
            return;
        }

        let cooldown = self.attack_cooldown.fetch_sub(1, Ordering::Relaxed) - 1;
        if cooldown <= 0 {
            let new_cd = 20 + mob.get_random().random_range(0..5) * 10;
            self.attack_cooldown.store(new_cd, Ordering::Relaxed);

            let world = entity.world.load();
            let target_pos = target.get_entity().pos.load();
            let bullet = ShulkerBulletEntity::new(
                entity,
                target.get_entity().entity_id,
                target_pos,
                shulker.get_attach_face().axis_of(),
            );
            world.spawn_entity_non_save(Arc::new(bullet));

            let pitch =
                1.0 + (mob.get_random().random::<f32>() - mob.get_random().random::<f32>()) * 0.2;
            world.play_sound_fine(
                Sound::EntityShulkerShoot,
                SoundCategory::Hostile,
                &shulker_pos,
                2.0,
                pitch,
            );
        }
    }
}

struct ShulkerPeekGoal {
    shulker: Weak<ShulkerEntity>,
    peek_time: AtomicI32,
}

impl ShulkerPeekGoal {
    const fn new(shulker: Weak<ShulkerEntity>) -> Self {
        Self {
            shulker,
            peek_time: AtomicI32::new(0),
        }
    }
}

impl Goal for ShulkerPeekGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(shulker) = self.shulker.upgrade() else {
            return false;
        };
        let has_target = mob.get_mob_entity().get_target().is_some();
        if has_target {
            return false;
        }
        if mob.get_random().random_range(0..40) != 0 {
            return false;
        }
        let pos = mob.get_mob_entity().living_entity.entity.block_pos.load();
        let face = shulker.get_attach_face();
        shulker.can_stay_at(&pos, face)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let has_target = mob.get_mob_entity().get_target().is_some();
        !has_target && self.peek_time.load(Ordering::Relaxed) > 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        let duration = 20 * (1 + mob.get_random().random_range(0..3));
        self.peek_time.store(duration, Ordering::Relaxed);
        if let Some(shulker) = self.shulker.upgrade() {
            shulker.set_raw_peek(30);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        let Some(shulker) = self.shulker.upgrade() else {
            return;
        };
        let has_target = shulker.mob_entity.get_target().is_some();
        if !has_target {
            shulker.set_raw_peek(0);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.peek_time.fetch_sub(1, Ordering::Relaxed);
    }
}

trait BlockPosExt {
    fn offset_direction(&self, dir: BlockDirection) -> Self;
}

impl BlockPosExt for BlockPos {
    fn offset_direction(&self, dir: BlockDirection) -> Self {
        let offset = dir.to_offset();
        Self::new(
            self.0.x + offset.x,
            self.0.y + offset.y,
            self.0.z + offset.z,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_world::chunk::ChunkData;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn opening_lid_pushes_nearby_entity_upward() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let shulker = ShulkerEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::SHULKER,
        ));
        world.spawn_entity(shulker.clone());
        shulker.set_attach_face(BlockDirection::Down);
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 65.2, 0.5),
            &EntityType::COW,
        ));
        world.spawn_entity(cow.clone());
        shulker.prev_peek_amount.store(0.0);
        shulker.current_peek_amount.store(0.5);
        shulker.on_peek_amount_change();
        assert!(cow.get_entity().pos.load().y > 65.2);
        world.level.shutdown().await;
        Ok(())
    }
}
