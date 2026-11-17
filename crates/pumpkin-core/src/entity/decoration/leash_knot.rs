use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::world::World;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

pub struct LeashKnotEntity {
    entity: Entity,
}

impl LeashKnotEntity {
    pub const OFFSET_Y: f64 = 0.375;

    pub const fn new(entity: Entity) -> Self {
        Self { entity }
    }

    pub fn get_or_create(world: &Arc<World>, pos: BlockPos) -> Arc<dyn EntityBase> {
        Self::get_knot(world, pos).unwrap_or_else(|| Self::create_knot(world, pos))
    }

    pub fn get_knot(world: &Arc<World>, pos: BlockPos) -> Option<Arc<dyn EntityBase>> {
        let search_box = knot_search_box(pos);
        world
            .get_entities_at_box(&search_box)
            .into_iter()
            .find(|entity| {
                let entity = entity.get_entity();
                entity.entity_type == &EntityType::LEASH_KNOT && entity.block_pos.load() == pos
            })
    }

    pub fn create_knot(world: &Arc<World>, pos: BlockPos) -> Arc<dyn EntityBase> {
        let raw_pos = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + Self::OFFSET_Y,
            f64::from(pos.0.z) + 0.5,
        );

        let entity = Entity::new(world.clone(), raw_pos, &EntityType::LEASH_KNOT);
        let knot: Arc<dyn EntityBase> = Arc::new(Self::new(entity));
        world.spawn_entity(knot.clone());

        knot
    }

    pub fn play_placement_sound(&self, world: &World) {
        let pos = self.entity.pos.load();
        world.play_sound(Sound::ItemLeadTied, SoundCategory::Neutral, &pos);
    }
}

use crate::server::Server;

impl EntityBase for LeashKnotEntity {
    fn can_hit(&self) -> bool {
        true
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn notify_leashee_removed(&self) {
        if self.entity.leashable_leashed_to().is_empty() {
            self.entity.remove();
        }
    }

    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        let world = self.entity.world.load();
        let block = world.get_block(&self.entity.block_pos.load());
        if !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_FENCES) {
            let knot_id = self.entity.entity_id;
            let pos = self.entity.pos.load();
            // Vanilla `Leashable.leashableLeashedTo` scans a 32 block box centred on the knot, so
            // mobs below the knot must be found too.
            let search_box = BoundingBox::of_size(pos, 32.0, 32.0, 32.0);
            let entities = world.get_entities_at_box(&search_box);

            // Vanilla BlockAttachedEntity.tick discards the knot as soon as survives() is
            // false; leashed mobs release themselves on their next tick_leash. Here we also
            // eagerly release nearby mobs so their leads drop before the knot Arc is freed
            // (a Rust Weak cannot be upgraded once the world drops it), but the knot is
            // always removed regardless of any still-attached mob, as in vanilla.
            let drop_lead = world.level_info.load().game_rules.entity_drops;
            for entity_base in entities {
                let ent = entity_base.get_entity();
                let is_attached_to_knot = ent
                    .leashed_to
                    .try_lock()
                    .ok()
                    .and_then(|guard| {
                        guard
                            .as_ref()
                            .and_then(std::sync::Weak::upgrade)
                            .map(|holder| holder.get_entity().entity_id == knot_id)
                    })
                    .unwrap_or(false);

                let unleashed = is_attached_to_knot && ent.unleash();
                if unleashed && drop_lead {
                    let lead_item = pumpkin_data::item_stack::ItemStack::new(
                        1,
                        &pumpkin_data::item::Item::LEAD,
                    );
                    world.drop_stack(&ent.block_pos.load(), lead_item);
                }
            }

            world.play_sound(Sound::ItemLeadUntied, SoundCategory::Neutral, &pos);
            self.entity.remove();
        }
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        let world = player.world();
        let knot_id = self.entity.entity_id;
        let player_id = player.entity_id();

        let pos = self.entity.pos.load();
        let search_box = transfer_search_box(pos);
        let entities = world.get_entities_at_box(&search_box);

        let mut attached_mob = false;
        let mut player_leashed_mobs = Vec::new();

        for entity_base in &entities {
            let ent = entity_base.get_entity();
            if let Ok(guard) = ent.leashed_to.try_lock()
                && let Some(holder) = guard.as_ref().and_then(std::sync::Weak::upgrade)
                && holder.get_entity().entity_id == player_id
            {
                player_leashed_mobs.push(ent);
            }
        }

        if let Some(self_knot) = Self::get_knot(&world, self.entity.block_pos.load()) {
            for mob in player_leashed_mobs {
                attached_mob |= mob.leash_to(self_knot.clone());
            }
        }

        // Vanilla LeashFenceKnotEntity.interact skips re-homing knot mobs onto the player
        // while secondary use (shift) is active.
        let mut any_dropped = false;
        if !attached_mob && !player.get_entity().is_sneaking() {
            let player_holder = player.clone() as Arc<dyn EntityBase>;
            for entity_base in &entities {
                if entity_base
                    .get_entity()
                    .leash_to_if_held_by(knot_id, &player_holder)
                {
                    any_dropped = true;
                }
            }
        }

        if attached_mob || any_dropped {
            self.play_placement_sound(&world);
            true
        } else {
            false
        }
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        if self.entity.is_invulnerable_to(&damage_type, cause) {
            return false;
        }
        // Vanilla `BlockAttachedEntity.hurtServer` rejects mob-caused damage while `mob_griefing`
        // is off; the causing entity (not the direct projectile) is what counts.
        if !self
            .entity
            .world
            .load()
            .level_info
            .load()
            .game_rules
            .mob_griefing
            && cause
                .or(source)
                .is_some_and(|attacker| attacker.get_mob().is_some())
        {
            return false;
        }
        // Hitting the knot destroys it. Leashed mobs release on their next tick (see `tick_leash`).
        let world = self.entity.world.load();
        world.play_sound(
            Sound::ItemLeadUntied,
            SoundCategory::Neutral,
            &self.entity.pos.load(),
        );
        self.entity.remove();
        // Vanilla hurtServer calls markHurt() after kill so the removal syncs to clients.
        self.entity.mark_hurt();
        true
    }
}

/// Vanilla `LeashFenceKnotEntity.getKnot` searches `AABB(x-1, y-1, z-1, x+1, y+1, z+1)`
/// measured from the integer block coordinates.
fn knot_search_box(pos: BlockPos) -> BoundingBox {
    let x = f64::from(pos.0.x);
    let y = f64::from(pos.0.y);
    let z = f64::from(pos.0.z);
    BoundingBox::new(
        Vector3::new(x - 1.0, y - 1.0, z - 1.0),
        Vector3::new(x + 1.0, y + 1.0, z + 1.0),
    )
}

/// Vanilla `LeashFenceKnotEntity.interact` searches `AABB(blockPos).inflate(7)`,
/// measured from the integer block coordinates rather than the entity position.
fn transfer_search_box(pos: Vector3<f64>) -> BoundingBox {
    const SEARCH_RADIUS: f64 = 7.0;
    let block_min = Vector3::new(pos.x.floor(), pos.y.floor(), pos.z.floor());
    BoundingBox::new(
        Vector3::new(
            block_min.x - SEARCH_RADIUS,
            block_min.y - SEARCH_RADIUS,
            block_min.z - SEARCH_RADIUS,
        ),
        Vector3::new(
            block_min.x + 1.0 + SEARCH_RADIUS,
            block_min.y + 1.0 + SEARCH_RADIUS,
            block_min.z + 1.0 + SEARCH_RADIUS,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{knot_search_box, transfer_search_box};
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn transfer_box_spans_block_coordinates_plus_seven() {
        // Knot centred in block (10, 64, 10).
        let bb = transfer_search_box(Vector3::new(10.5, 64.375, 10.5));
        assert_eq!(bb.min.x, 3.0);
        assert_eq!(bb.max.x, 18.0);
        assert_eq!(bb.min.y, 57.0);
        assert_eq!(bb.max.y, 72.0);
    }

    #[test]
    fn knot_search_box_spans_block_plus_minus_one() {
        let bb = knot_search_box(BlockPos::new(10, 64, 10));
        assert_eq!(bb.min.x, 9.0);
        assert_eq!(bb.min.y, 63.0);
        assert_eq!(bb.min.z, 9.0);
        assert_eq!(bb.max.x, 11.0);
        assert_eq!(bb.max.y, 65.0);
        assert_eq!(bb.max.z, 11.0);
    }
}
