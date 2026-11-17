#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_util::version::JavaMinecraftVersion;

impl JavaClient {
    pub fn handle_use_item(&self, player: &Arc<Player>, use_item: &SUseItem, server: &Arc<Server>) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        let inventory = player.inventory();
        let Ok(hand) = Hand::from_packet_id(use_item.hand.0) else {
            self.try_kick(&TextComponent::text("InvalidHand"));
            return;
        };
        if self.version.load() >= JavaMinecraftVersion::V_1_21
            && (!use_item.yaw.is_finite() || !use_item.pitch.is_finite())
        {
            self.try_kick(&TextComponent::text("Invalid item use rotation"));
            return;
        }
        self.update_sequence(use_item.sequence.0);

        let mut item_in_hand = inventory.get_stack_in_hand(hand);

        let mut consume_event =
            crate::plugin::api::events::player::player_item_consume::PlayerItemConsumeEvent::new(
                player.clone(),
                item_in_hand.item.registry_key.to_string(),
            );
        server
            .plugin_manager
            .fire_blocking(server, &mut consume_event);
        if consume_event.cancelled {
            return;
        }

        // Vanilla `ServerGamePacketListenerImpl.handleUseItem` never awards ITEM_USED;
        // each item or component awards it once in its own use path (item behaviours,
        // `Consumable.onConsume` and `Equippable.swapAndEquip`). Awarding generically
        // here double-counts every item that already self-awards.

        let hit = player.world().clip(&pumpkin_world::clip::ClipContext::new(
            player.eye_position(),
            player.eye_position().add(
                &(Vector3::rotation_vector(f64::from(use_item.pitch), f64::from(use_item.yaw))
                    * 4.5),
            ),
            pumpkin_world::clip::ClipBlock::Outline,
            pumpkin_world::clip::ClipFluid::None,
        ));

        let event = if hit.is_miss {
            PlayerInteractEvent::new(player, InteractAction::RightClickAir, &Block::AIR, None)
        } else {
            let hit_pos = hit.block_pos;
            PlayerInteractEvent::new(
                player,
                InteractAction::RightClickBlock,
                player.world().get_block(&hit_pos),
                Some(hit_pos),
            )
        };
        let stack_for_use = item_in_hand.clone();
        let (use_yaw, use_pitch) = if self.version.load() >= JavaMinecraftVersion::V_1_21 {
            (use_item.yaw, use_item.pitch)
        } else {
            player.rotation()
        };
        Self::prepare_hand_item_for_use(player, hand, &mut item_in_hand);

        send_cancellable_blocking! {{
            server;
            event;
            'after: {
                server
                    .item_registry
                    .on_use_with_rotation(&stack_for_use, player, use_yaw, use_pitch, hand);
            }
        }}
    }

    fn prepare_hand_item_for_use(player: &Arc<Player>, hand: Hand, held: &mut ItemStack) {
        let inventory = player.inventory();

        let group = held
            .get_use_cooldown()
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| held.item.registry_key.to_string());
        if player.is_on_cooldown(&group) {
            return;
        }

        if held.get_data_component::<ConsumableImpl>().is_some()
            || held.get_data_component::<BlocksAttacksImpl>().is_some()
            || held
                .get_data_component::<pumpkin_data::data_component_impl::KineticWeaponImpl>()
                .is_some()
        {
            // If its food we want to make sure we can actually consume it
            if let Some(food) = held.get_data_component::<FoodImpl>() {
                if player.can_eat(food.can_always_eat) {
                    player.living_entity.set_active_hand(
                        hand,
                        held.clone(),
                        held.get_max_use_time(),
                    );
                }
            } else {
                player
                    .living_entity
                    .set_active_hand(hand, held.clone(), held.get_max_use_time());
            }
        }
        let equipment_slot = held
            .get_data_component::<EquippableImpl>()
            .filter(|equippable| {
                equippable.swappable
                    && (equippable.slot.slot_type()
                        == pumpkin_data::data_component_impl::EquipmentType::HumanoidArmor
                        || matches!(equippable.slot, EquipmentSlot::OffHand(_)))
                    && equippable.can_be_equipped_by(player.get_entity().entity_type)
            })
            .map(|equippable| equippable.slot.clone());
        if let Some(slot) = equipment_slot {
            // The equipment lock has to be released before touching the hand again:
            // the off hand lives in the same map, so holding it here would deadlock.
            let current_equipped = inventory
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&slot);
            let gamemode = player.gamemode.load();
            if (current_equipped.has_prevent_armor_change() && gamemode != GameMode::Creative)
                || current_equipped.are_items_and_components_equal(held)
            {
                return;
            }

            let equipped = if held.item_count <= 1 {
                let swapped_to_eq = if gamemode == GameMode::Creative {
                    held.clone()
                } else {
                    held.copy_with_count(1)
                };
                if gamemode != GameMode::Creative || !current_equipped.is_empty() {
                    *held = current_equipped;
                }
                swapped_to_eq
            } else {
                let swapped_to_eq = held.copy_with_count(1);
                held.decrement_unless_creative(gamemode, 1);
                if !current_equipped.is_empty() {
                    inventory.offer(current_equipped, false, player.as_ref());
                }
                swapped_to_eq
            };

            // Vanilla `Equippable.swapAndEquip` awards ITEM_USED to the newly equipped item.
            player.increment_stat(StatisticCategory::Used, i32::from(equipped.item.id), 1);
            player.enqueue_equipment_change(&slot, &equipped);
            inventory
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .put(&slot, equipped);
            inventory.set_stack_in_hand(hand, held.clone());
            player
                .player_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .send_content_updates();
        }
    }
}
