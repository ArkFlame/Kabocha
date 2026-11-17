use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::world::World;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
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
        let center = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + Self::OFFSET_Y,
            f64::from(pos.0.z) + 0.5,
        );

        let search_dim = EntityDimensions {
            width: 2.0,
            height: 2.0,
            eye_height: 1.0,
        };

        let search_box = BoundingBox::new_from_pos(center.x, center.y, center.z, &search_dim);
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

            let drop_lead = world.level_info.load().game_rules.entity_drops;
            let mut retained_leash = false;
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
                retained_leash |= ent
                    .get_leash_holder()
                    .is_some_and(|holder| holder.get_entity().entity_id == knot_id);
            }

            if retained_leash {
                return;
            }
            world.play_sound(Sound::ItemLeadUntied, SoundCategory::Neutral, &pos);
            self.entity.remove();
        }
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        let world = player.world();
        let knot_id = self.entity.entity_id;
        let player_id = player.entity_id();

        // Vanilla searches a 7 block cube around the knot.
        const SEARCH_RADIUS: f64 = 7.0;
        let pos = self.entity.pos.load();
        let search_box = BoundingBox::new(
            Vector3::new(
                pos.x - SEARCH_RADIUS,
                pos.y - SEARCH_RADIUS,
                pos.z - SEARCH_RADIUS,
            ),
            Vector3::new(
                pos.x + SEARCH_RADIUS,
                pos.y + SEARCH_RADIUS,
                pos.z + SEARCH_RADIUS,
            ),
        );
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

        let mut any_dropped = false;
        if !attached_mob {
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
        // Vanilla `BlockAttachedEntity.hurtServer` checks invulnerability first.
        if self.entity.is_invulnerable_to(&damage_type, cause) {
            return false;
        }
        // `BlockAttachedEntity.hurtServer`: no damage from a mob while `mob_griefing` is off.
        let world = self.entity.world.load();
        if !world.level_info.load().game_rules.mob_griefing
            && cause.or(source).is_some_and(|atk| atk.get_mob().is_some())
        {
            return false;
        }
        // Hitting the knot destroys it. Leashed mobs release on their next tick (see `tick_leash`).
        world.play_sound(
            Sound::ItemLeadUntied,
            SoundCategory::Neutral,
            &self.entity.pos.load(),
        );
        self.entity.remove();
        true
    }
}
