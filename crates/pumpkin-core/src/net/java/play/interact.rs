#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_interact(
        &self,
        player: &Arc<Player>,
        interact: &SInteract,
        server: &Arc<Server>,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();
        let entity_id = interact.entity_id;

        let sneaking = interact.sneaking;
        let player_entity = &player.get_entity();
        if player_entity.is_sneaking() != sneaking {
            player_entity.set_sneaking(sneaking);
        }
        let Ok(action) = ActionType::try_from(interact.r#type.0) else {
            self.try_kick(&TextComponent::text("Invalid action type"));
            return;
        };

        // Resolve the target entity for the event
        let world = player_entity.world.load_full();
        let player_target = world.get_player_by_id(entity_id.0);
        let target: Option<Arc<dyn EntityBase>> = player_target
            .as_ref()
            .map(|p| Arc::clone(p) as Arc<dyn EntityBase>)
            .or_else(|| world.get_entity_by_id(entity_id.0));

        if let Some(target) = target {
            if !player.can_interact_with_entity(target.as_ref(), 1.0) {
                return;
            }
            if player.gamemode.load() == GameMode::Spectator {
                player.camera_target_id.store(Some(entity_id.0));
                player.try_send_client_packet(&CSetCamera::new(entity_id));
                return;
            }
            let event = PlayerInteractEntityEvent::new(
                player,
                target,
                action,
                interact.target_position,
                sneaking,
            );
            self.handle_entity_interaction(player, event, interact.hand, server);
        } else {
            // Entity not found: vanilla only warns, a client can legitimately attack an
            // entity the server already removed (or not track yet).
            send_cancellable_blocking! {{
                server;
                PlayerInteractUnknownEntityEvent::new(player, entity_id.0, action);

                if event.action == ActionType::Attack {
                    warn!(
                        "Player id {} interacted with entity id {}, which was not found.",
                        player.entity_id(),
                        event.entity_id
                    );
                }
            }}
        }
    }

    fn attack_entity(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        target: &Arc<dyn crate::entity::EntityBase>,
    ) {
        // Paper's `pvp` setting only gates player-versus-player combat; attacks on
        // non-player entities (mobs, boats, minecarts) always go through.
        let config = &server.advanced_config.pvp;

        if target.get_entity().entity_id == player.entity_id() {
            self.try_kick(&TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                [],
            ));
            return;
        }

        if let Some(player_victim) = target.get_player() {
            if !config.enabled {
                return;
            }
            if player_victim.living_entity.health.load() <= 0.0 {
                return;
            }
            if config.protect_creative && player_victim.gamemode.load() == GameMode::Creative {
                player.world().play_sound(
                    Sound::EntityPlayerAttackNodamage,
                    SoundCategory::Players,
                    &player_victim.position(),
                );
                return;
            }
        }
        player.attack(target);
    }

    pub(super) fn handle_entity_interaction(
        &self,
        player: &Arc<Player>,
        mut event: PlayerInteractEntityEvent,
        hand: Option<VarInt>,
        server: &Arc<Server>,
    ) {
        server.plugin_manager.fire_blocking(server, &mut event);
        if event.cancelled {
            return;
        }

        match event.action {
            ActionType::Attack => self.attack_entity(player, server, &event.target),
            ActionType::Interact | ActionType::InteractAt => {
                if event.action == ActionType::InteractAt
                    && let Some(pos) = event.target_position
                {
                    let mut at_event = crate::plugin::api::events::player::player_interact_at_entity::PlayerInteractAtEntityEvent::new(
                        player.clone(),
                        event.target.get_entity().entity_id,
                        pos.x,
                        pos.y,
                        pos.z,
                        u8::from(hand.map_or(0, |h| h.0) != 0),
                    );
                    server.plugin_manager.fire_blocking(server, &mut at_event);
                    if at_event.cancelled {
                        return;
                    }
                }
                let mut stack = player.inventory().held_item();

                let item_id = stack.item.id;
                let before = stack.clone();
                let mut interacted = match event.target_position {
                    Some(position) if event.action == ActionType::InteractAt => {
                        event.target.interact_at(player, &mut stack, position)
                    }
                    _ => event.target.interact(player, &mut stack),
                };
                if !interacted {
                    let result = server.item_registry.use_on_entity(
                        &mut stack,
                        player,
                        event.target.clone(),
                    );
                    interacted = result.consumes_action();
                }
                if interacted {
                    player.trigger_datapack_advancements(
                        &crate::data::datapack::advancement::TriggerEvent::InteractedWithEntity {
                            item: &before,
                            target: event.target.as_ref(),
                        },
                    );
                    let item_id = format!("minecraft:{}", before.item.registry_key);
                    player.trigger_advancement(
                        crate::entity::player::advancement::trigger::AdvancementTrigger::PlayerInteractedWithEntity {
                            entity_type: format!(
                                "minecraft:{}",
                                event.target.get_entity().entity_type.resource_name
                            ),
                            item_id,
                            is_baby: event
                                .target
                                .get_living_entity()
                                .is_some_and(crate::entity::living::LivingEntity::is_baby),
                            player_wearing_safe_armor:
                                crate::entity::mob::piglin_ai::PiglinAi::is_wearing_safe_armor(
                                    &player.living_entity,
                                ),
                            distance: crate::entity::player::advancement::predicate::EntityDistance::between(
                                player.get_entity().pos.load(),
                                event.target.get_entity().pos.load(),
                            ),
                        },
                    );
                    // Vanilla swings the used hand whenever the entity interaction consumes the action.
                    let swing_hand = if hand.is_some_and(|h| h.0 != 0) {
                        pumpkin_util::Hand::Left
                    } else {
                        pumpkin_util::Hand::Right
                    };
                    player.swing_hand(swing_hand, true);
                }
                if !stack.are_equal(&before) {
                    player.increment_stat(StatisticCategory::Used, item_id as i32, 1);
                    if before.is_damageable() && stack.is_empty() {
                        player.increment_stat(StatisticCategory::Broken, item_id as i32, 1);
                        player.world().send_entity_status(
                            player.get_entity(),
                            equipment_break_status(&EquipmentSlot::MAIN_HAND),
                            pumpkin_protocol::bedrock::server::actor_event::ActorEventID::None,
                        );
                    }
                }
                player.inventory().set_held_item(stack);
            }
        }
    }
}
