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
        if self.entity.is_removed() {
            return;
        }
        let world = self.entity.world.load();
        let block = world.get_block(&self.entity.block_pos.load());
        if !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_FENCES) {
            // Vanilla `BlockAttachedEntity.tick` discards the knot as soon as `survives()` is
            // false and calls `dropItem` (the untied sound for a knot). Leashed mobs release
            // themselves on their next `tick_leash`, which refunds the lead when the holder
            // is gone; no eager scan is needed.
            world.play_sound(
                Sound::ItemLeadUntied,
                SoundCategory::Neutral,
                &self.entity.pos.load(),
            );
            self.entity.remove();
        }
    }
    fn interact_with_hand(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        hand: pumpkin_util::Hand,
    ) -> Option<bool> {
        // Vanilla falls through to the leash logic when shears cut nothing, so only a cut is final.
        crate::entity::interact_with_shears(self, player, item_stack, hand)
            .filter(|sheared| *sheared)
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        let world = player.world();
        let knot_id = self.entity.entity_id;

        // Vanilla `LeashFenceKnotEntity.interact` first moves mobs leashed to the player onto
        // the knot. `leashable_leashed_to` scans a 32 block cube around the player's bounding
        // box centre, matching `Leashable.leashableInArea`.
        let mut attached_mob = false;
        if let Some(self_knot) = Self::get_knot(&world, self.entity.block_pos.load()) {
            for mob in player.get_entity().leashable_leashed_to() {
                attached_mob |= mob.get_entity().leash_to(self_knot.clone());
            }
        }

        // If none moved and secondary use (shift) is inactive, re-home mobs leashed to the knot
        // onto the player (`Leashable.leashableLeashedTo(this)`).
        let mut any_dropped = false;
        if !attached_mob && !player.get_entity().is_sneaking() {
            let player_holder = player.clone() as Arc<dyn EntityBase>;
            for mob in self.entity.leashable_leashed_to() {
                any_dropped |= mob
                    .get_entity()
                    .leash_to_if_held_by(knot_id, &player_holder);
            }
        }

        if attached_mob || any_dropped {
            // Vanilla `gameEvent(BLOCK_ATTACH, player)`: at the knot, sourced from the player.
            world.emit_game_event_with_context(
                pumpkin_data::game_event::GameEvent::BlockAttach.name(),
                self.entity.pos.load(),
                &crate::world::game_event::GameEventContext {
                    source_entity: Some(player.as_ref() as &dyn EntityBase),
                },
            );
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
        _source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        if self.entity.is_invulnerable_to(&damage_type, cause) {
            return false;
        }
        let world = self.entity.world.load();
        // Vanilla `BlockAttachedEntity.hurtServer` skips mob-caused damage while `mob_griefing`
        // is off. The check uses the causing entity (`DamageSource.getEntity`), not the direct one.
        let mob_griefing = world.level_info.load().game_rules.mob_griefing;
        if !mob_griefing && cause.is_some_and(|attacker| attacker.get_mob().is_some()) {
            return false;
        }
        // Hitting the knot destroys it. Leashed mobs release on their next tick (see `tick_leash`).
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

#[cfg(test)]
mod tests {
    use super::knot_search_box;
    use pumpkin_util::math::position::BlockPos;

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
