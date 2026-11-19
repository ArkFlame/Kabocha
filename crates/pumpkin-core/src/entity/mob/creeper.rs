use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{
    entity::EntityType,
    item::Item,
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::RngExt;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::{
    Entity, EntityBase,
    ai::behavior::fleeing::{self, Flee},
    ai::goal::{
        creeper_ignite::CreeperIgniteGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    ai::target_match::TargetMatch,
    mob::{Mob, MobEntity},
    player::Player,
};
use crate::world::game_event::GameEventContext;

const DEFAULT_FUSE_TIME: i32 = 30;
const DEFAULT_EXPLOSION_RADIUS: i32 = 3;

const CREEPER_FLEES: &[Flee] = &[
    Flee::new(3, TargetMatch::Type(&EntityType::OCELOT), 6.0, 1.0, 1.2),
    Flee::new(3, TargetMatch::Type(&EntityType::CAT), 6.0, 1.0, 1.2),
];

// Vanilla `Creeper`: NearestAttackableTargetGoal(Player) at priority 1, HurtByTargetGoal at 2.
const CREEPER_HUNTS: &[Hunt] = &[Hunt::new(1, TargetMatch::Type(&EntityType::PLAYER), true)];

impl crate::entity::custom_sound::CustomSound for CreeperEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityCreeperHurt)
    }

    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityCreeperDeath)
    }
}

pub struct CreeperEntity {
    pub mob_entity: MobEntity,
    pub fuse_speed: AtomicI32,
    pub current_fuse_time: AtomicI32,
    pub last_fuse_time: AtomicI32,
    pub fuse_time: AtomicI32,
    pub explosion_radius: AtomicI32,
    pub ignited: AtomicBool,
    pub charged: AtomicBool,
    pub dropped_skulls: AtomicBool,
}

impl CreeperEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let entity = Self {
            mob_entity,
            fuse_speed: AtomicI32::new(-1),
            current_fuse_time: AtomicI32::new(0),
            last_fuse_time: AtomicI32::new(0),
            fuse_time: AtomicI32::new(DEFAULT_FUSE_TIME),
            explosion_radius: AtomicI32::new(DEFAULT_EXPLOSION_RADIUS),
            ignited: AtomicBool::new(false),
            charged: AtomicBool::new(false),
            dropped_skulls: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(entity);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                2,
                Box::new(CreeperIgniteGoal::new(Arc::downgrade(&mob_arc))),
            );
            fleeing::apply(goal_selector, CREEPER_FLEES);
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::water_avoiding(0.8)));

            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
            hunting::apply(target_selector, &mob_arc.mob_entity, CREEPER_HUNTS);
        };

        mob_arc
    }

    pub fn set_fuse_speed(&self, speed: i32) {
        self.fuse_speed.store(speed, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(pumpkin_data::tracked_data::creeper::FUSE_ID, VarInt(speed));
        if let Some(meta) = self.mob_metadata() {
            entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
        }
    }

    pub fn explode(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let radius = self.explosion_radius.load(Ordering::Relaxed) as f32;
        let multiplier = if self.charged.load(Ordering::Relaxed) {
            2.0
        } else {
            1.0
        };
        let world = entity.world.load();
        let pos = entity.pos.load();
        // Vanilla `Creeper.explodeCreeper` lets plugins cancel or resize the prime event.
        let mut prime_event =
            crate::plugin::api::events::entity::explosion_prime::ExplosionPrimeEvent::new(
                entity.entity_id,
                radius * multiplier,
                false,
            );
        if let Some(server) = world.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut prime_event);
        }
        if prime_event.cancelled {
            // Vanilla resets the swell and clears the ignited flag, keeping the creeper alive.
            self.current_fuse_time.store(0, Ordering::Relaxed);
            self.set_ignited(false);
            return;
        }
        self.mob_entity
            .living_entity
            .dead
            .store(true, Ordering::Relaxed);
        world.explode_from(
            self,
            pos,
            prime_event.radius,
            crate::world::ExplosionInteraction::Mob,
        );

        let effects = self
            .mob_entity
            .living_entity
            .active_effects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .map(|effect| {
                (
                    effect.effect_type,
                    effect.duration,
                    effect.amplifier,
                    effect.ambient,
                    effect.show_particles,
                    effect.show_icon,
                )
            })
            .collect::<Vec<_>>();

        // Vanilla Creeper.explodeCreeper fires onMobRemoved(KILLED) for each active effect
        // (oozing spawns slimes, weaving cobwebs, wind_charged a gust) before discarding.
        let removed_effects: Vec<_> = effects
            .iter()
            .map(|&(effect_type, _, amplifier, _, _, _)| (effect_type, amplifier))
            .collect();

        if !effects.is_empty() {
            let cloud_entity = Entity::new(
                world.clone(),
                entity.pos.load(),
                &EntityType::AREA_EFFECT_CLOUD,
            );
            let cloud = crate::entity::area_effect_cloud::AreaEffectCloudEntity::create(
                cloud_entity,
                ItemStack::new(0, &Item::GLASS_BOTTLE),
                effects,
                // Vanilla Creeper.spawnLingeringCloud setDuration(300).
                300,
                2.5,
                20,
                10,
                -0.5,
                0,
            );
            world.spawn_entity(cloud);
        }

        for (effect_type, amplifier) in removed_effects {
            if let Some(mob_effect) = crate::entity::effect::get_mob_effect(effect_type) {
                mob_effect.on_mob_removed(&self.mob_entity.living_entity, amplifier);
            }
        }

        entity.remove();
    }
}

impl Mob for CreeperEntity {
    fn cause_fall_damage(
        &self,
        caller: &dyn EntityBase,
        fall_distance: f32,
        damage_modifier: f32,
    ) -> bool {
        let damaged =
            self.mob_entity
                .living_entity
                .cause_fall_damage(caller, fall_distance, damage_modifier);
        let max_swell = self.fuse_time.load(Ordering::Relaxed) - 5;
        let swell = self.current_fuse_time.load(Ordering::Relaxed) + (fall_distance * 1.5) as i32;
        self.current_fuse_time
            .store(swell.min(max_swell), Ordering::Relaxed);
        damaged
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("powered", self.charged.load(Ordering::Relaxed));
        nbt.put_short("Fuse", self.fuse_time.load(Ordering::Relaxed) as i16);
        nbt.put_byte(
            "ExplosionRadius",
            self.explosion_radius.load(Ordering::Relaxed) as i8,
        );
        nbt.put_bool("ignited", self.ignited.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(powered) = nbt.get_bool("powered") {
            self.charged.store(powered, Ordering::Relaxed);
            // Vanilla `Creeper.readAdditionalSaveData` sets the synced IS_POWERED data directly.
            self.mob_entity
                .living_entity
                .entity
                .set_synced_data(pumpkin_data::tracked_data::creeper::CHARGED, powered);
        }
        if let Some(fuse) = nbt.get_short("Fuse") {
            self.fuse_time.store(i32::from(fuse), Ordering::Relaxed);
        }
        if let Some(radius) = nbt.get_byte("ExplosionRadius") {
            self.explosion_radius
                .store(i32::from(radius), Ordering::Relaxed);
        }
        if let Some(ignited) = nbt.get_bool("ignited") {
            self.ignited.store(ignited, Ordering::Relaxed);
            // Vanilla `Creeper.readAdditionalSaveData` sets the synced IS_IGNITED data directly.
            if ignited {
                self.mob_entity
                    .living_entity
                    .entity
                    .set_synced_data(pumpkin_data::tracked_data::creeper::IS_IGNITED, true);
            }
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn apply_mob_target(&self, target: Option<Arc<dyn EntityBase>>) {
        // Vanilla `Creeper.setTarget` never targets a goat; keep the current target instead.
        if target
            .as_ref()
            .is_some_and(|t| t.get_entity().entity_type == &EntityType::GOAT)
        {
            return;
        }
        self.get_mob_entity().set_target(target);
    }

    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        let mut meta = crate::entity::mob::MobMetadata::new();
        if self.is_charged() {
            meta = meta.charged(true);
        }
        if self.is_ignited() {
            meta = meta.ignited(true);
        }
        meta = meta.fuse_speed(self.fuse_speed.load(Ordering::Relaxed));
        Some(meta)
    }

    fn mob_on_lightning_strike(
        &self,
        caller: &dyn EntityBase,
        lightning: &crate::entity::lightning::LightningBoltEntity,
    ) {
        // Vanilla `Creeper.thunderHit` runs `super.thunderHit` before the Paper CreeperPowerEvent,
        // so base lightning damage/fire applies even when the event is later cancelled.
        self.mob_entity
            .living_entity
            .on_lightning_strike(caller, lightning);

        let entity = &self.mob_entity.living_entity.entity;
        let mut event = crate::plugin::api::events::entity::creeper_power::CreeperPowerEvent::new(
            entity.entity_id,
            Some(lightning.get_entity().entity_id),
            "LIGHTNING".to_string(),
        );
        if let Some(server) = entity.world.load().server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if !event.cancelled {
            self.set_charged(true);
        }
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        self.last_fuse_time.store(
            self.current_fuse_time.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );

        if self.ignited.load(Ordering::Relaxed) {
            self.set_fuse_speed(1);
        }

        let fuse_speed = self.fuse_speed.load(Ordering::Relaxed);
        let current = self.current_fuse_time.load(Ordering::Relaxed);

        if fuse_speed > 0 && current == 0 {
            let world = entity.world.load();
            world.play_sound_fine(
                Sound::EntityCreeperPrimed,
                SoundCategory::Hostile,
                &entity.pos.load(),
                1.0,
                0.5,
            );
            world.emit_game_event_with_context(
                GameEvent::PrimeFuse.name(),
                entity.pos.load(),
                &GameEventContext {
                    source_entity: Some(caller),
                },
            );
        }

        let fuse_time = self.fuse_time.load(Ordering::Relaxed);
        let new_fuse = (current + fuse_speed).max(0);
        self.current_fuse_time.store(new_fuse, Ordering::Relaxed);

        if new_fuse >= fuse_time {
            self.current_fuse_time.store(fuse_time, Ordering::Relaxed);
            self.explode();
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if !item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_CREEPER_IGNITERS)
        {
            return self
                .mob_entity
                .mob_interact(player, item_stack, self.can_be_leashed());
        }

        let sound = if item_stack.item.id == Item::FIRE_CHARGE.id {
            Sound::ItemFirechargeUse
        } else {
            Sound::ItemFlintandsteelUse
        };
        let entity = &self.mob_entity.living_entity.entity;
        // The interacting client plays the sound itself.
        entity.world.load().play_sound_raw_expect(
            player,
            sound as u16,
            SoundCategory::Hostile,
            &entity.pos.load(),
            1.0,
            self.get_random().random::<f32>() * 0.4 + 0.8,
        );

        self.set_ignited(true);
        if item_stack.is_damageable() {
            let _ = player.damage_item_stack(item_stack, 1);
        } else {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        }

        true
    }

    // Creeper.killedEntityPreEvent: a charged creeper drops a single mob head.
    fn killed_entity(&self, killed: &dyn EntityBase) -> bool {
        // Vanilla guards with `!droppedSkulls` and checks the killer's own
        // `shouldDropLoot(level)` (a Monster's gamerule check), not the victim's.
        if self.charged.load(Ordering::Relaxed)
            && !self.dropped_skulls.load(Ordering::Relaxed)
            && Mob::should_drop_loot(self)
        {
            let world = killed.get_entity().world.load_full();
            if let Some(table) = world.get_loot_table("minecraft:charged_creeper/root") {
                let params = crate::world::loot::LootContextParameters {
                    world: Some(&world),
                    position: Some(killed.get_entity().pos.load()),
                    this_entity: Some(killed),
                    killer_entity: Some(self),
                    ..Default::default()
                };
                let pos = killed.get_entity().block_pos.load();
                let mut dropped = false;
                for stack in crate::world::loot::generate_loot_from_handle(
                    &table,
                    self.get_random().random(),
                    &params,
                ) {
                    world.drop_stack(&pos, stack);
                    dropped = true;
                }
                // Vanilla sets the flag inside the loot callback; a non-head victim does not consume it.
                if dropped {
                    self.dropped_skulls.store(true, Ordering::Relaxed);
                }
            }
        }
        true
    }

    // Creeper.doHurtTarget: a creeper never deals melee damage, it only swells.
    fn do_hurt_target(&self, _target: &dyn EntityBase) -> bool {
        true
    }
}

impl CreeperEntity {
    pub fn is_charged(&self) -> bool {
        self.charged.load(Ordering::Relaxed)
    }

    pub fn set_charged(&self, charged: bool) {
        self.charged.store(charged, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(pumpkin_data::tracked_data::creeper::CHARGED, charged);
        if let Some(meta) = self.mob_metadata() {
            entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
        }
    }

    pub fn is_ignited(&self) -> bool {
        self.ignited.load(Ordering::Relaxed)
    }

    pub fn set_ignited(&self, ignited: bool) {
        self.ignited.store(ignited, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_synced_data(pumpkin_data::tracked_data::creeper::IS_IGNITED, ignited);
        if let Some(meta) = self.mob_metadata() {
            entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
        }
    }

    pub fn get_fuse(&self) -> i32 {
        self.fuse_time.load(Ordering::Relaxed)
    }

    pub fn set_fuse(&self, fuse: i32) {
        self.fuse_time.store(fuse, Ordering::Relaxed);
    }

    pub fn get_explosion_radius(&self) -> i32 {
        self.explosion_radius.load(Ordering::Relaxed)
    }

    pub fn set_explosion_radius(&self, radius: i32) {
        self.explosion_radius.store(radius, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_fuse_time_matches_vanilla() {
        assert_eq!(DEFAULT_FUSE_TIME, 30);
    }

    #[test]
    fn default_explosion_radius_matches_vanilla() {
        assert_eq!(DEFAULT_EXPLOSION_RADIUS, 3);
    }
}
