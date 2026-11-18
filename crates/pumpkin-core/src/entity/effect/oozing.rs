use uuid::Uuid;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::effect::MobEffect;
use crate::entity::living::LivingEntity;
use crate::entity::mob::slime::SlimeEntity;
use crate::entity::r#type::from_type;

pub struct OozingMobEffect;

impl MobEffect for OozingMobEffect {
    fn on_mob_removed(&self, living: &LivingEntity, _amplifier: u8) {
        let world = living.entity.world.load();
        let pos = living.entity.pos.load();
        let spawn_pos = Vector3::new(pos.x, pos.y + 0.5, pos.z);

        // Vanilla OOZING requests 2 slimes, capped by max_entity_cramming less nearby slimes.
        const REQUESTED_SLIMES: i64 = 2;
        let max_cramming = world.level_info.load().game_rules.max_entity_cramming;
        let spawn_count = if max_cramming < 1 {
            REQUESTED_SLIMES
        } else {
            let area = living.entity.bounding_box.load().expand(2.0, 2.0, 2.0);
            let nearby = world
                .get_entities_of_type_at_box(&area, |t| t.id == EntityType::SLIME.id)
                .into_iter()
                .filter(|e| e.get_entity().entity_id != living.entity.entity_id)
                .take(max_cramming as usize)
                .count();
            REQUESTED_SLIMES.min((max_cramming - nearby as i64).max(0))
        };

        for _ in 0..spawn_count {
            let entity_arc = from_type(&EntityType::SLIME, spawn_pos, &world, Uuid::new_v4());
            let entity = entity_arc.get_entity();
            entity.set_pos(spawn_pos);
            entity.yaw.store(rand::random::<f32>() * 360.0);
            entity.pitch.store(0.0);

            if let Some(slime) = entity_arc.cast_any().downcast_ref::<SlimeEntity>() {
                slime.set_size(2, true);
            }

            world.spawn_entity(entity_arc);
        }
    }
}
