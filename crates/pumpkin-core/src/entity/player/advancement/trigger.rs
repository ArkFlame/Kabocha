use crate::entity::player::Player;
use crate::entity::player::advancement::PlayerAdvancement;
use crate::entity::player::advancement::predicate::{
    self, CriterionConditions, DamageEntities, DamagePair, EntityDistance, EntityState,
};
use pumpkin_util::math::position::BlockPos;

/// One entity hit by the bolt, used to test the lightning sub-predicate.
#[derive(Debug, Clone)]
pub struct LightningStruckEntity {
    pub entity_type: String,
    pub distance: Option<EntityDistance>,
    pub is_baby: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum AdvancementTrigger {
    InventoryChanged,
    PlayerKilledEntity {
        entity_type_resource: String,
        /// Victim snapshot: distance to the player, effects, equipment and baby flag.
        state: EntityState,
    },
    SniperDuel {
        /// Killer-to-skeleton distance, checked against the skeleton's `distance` predicate.
        distance: EntityDistance,
    },
    TwoBirdsOneArrow,
    Arbalistic,
    EnterDimension {
        dimension: String,
    },
    SleptInBed,
    ConsumeItem {
        item_id: String,
        count: u32,
    },
    PlacedBlock {
        block_id: String,
        item_id: Option<String>,
        count: u32,
        enchantments: Vec<(String, u32)>,
    },
    EnterBlock {
        block_id: String,
    },
    FishedItem {
        items: Vec<(String, u32)>,
    },
    DeflectedDamage,
    PlayerKilled,
    BredAnimal {
        parent_type: String,
        partner_type: String,
        child_type: Option<String>,
    },
    LaunchedEyeOfEnder,
    GlowedSign,
    DealtOverkillDamage,
    PlayerHurtEntity {
        is_blocked: bool,
        is_projectile: bool,
        is_mace_smash: bool,
        original_damage: f64,
        actual_damage: f64,
        direct_entity_type: Option<String>,
        source_entity_type: Option<String>,
        source_distance: Option<EntityDistance>,
        source_is_baby: Option<bool>,
        damage_type: pumpkin_data::damage::DamageType,
    },
    Bullseye,
    CuredZombieVillager,
    TradedWithVillager,
    AllayDropItemOnBlock {
        block_id: String,
        item_id: String,
    },
    ThrownItemPickedUpByPlayer {
        thrower_type: String,
    },
    Location,
    ItemUsedOnBlock {
        block_id: String,
        item_id: String,
        state: Option<Vec<(String, String)>>,
        smokey: Option<bool>,
        biome: Option<String>,
    },
    TameAnimal {
        entity_type: String,
        /// Player-to-animal distance, checked against the entity's `distance` predicate.
        distance: Option<EntityDistance>,
        /// Data component values of the tamed entity (e.g. its wolf/cat variant).
        components: Vec<(String, String)>,
    },
    FilledBucket {
        item_id: String,
        count: u32,
    },
    UsingItem {
        item_id: String,
        looking_at: Option<String>,
    },
    ConstructBeacon {
        level: i32,
    },
    SlideDownBlock {
        block_id: String,
    },
    RecipeCrafted {
        recipe_id: String,
    },
    RecipeUnlocked {
        recipe_id: String,
    },
    PlayerGeneratesContainerLoot {
        loot_table_id: String,
    },
    EnchantedItem {
        levels: i32,
    },
    BrewedPotion,
    ItemDurabilityChanged {
        item_id: String,
        vehicle_type: Option<String>,
    },
    Levitation {
        /// `entity - origin` delta the `distance` predicate is evaluated against.
        delta: (f64, f64, f64),
    },
    FallFromHeight {
        delta: (f64, f64, f64),
        from_y: f64,
        to_y: f64,
    },
    FallAfterExplosion {
        delta: (f64, f64, f64),
        cause: String,
    },
    NetherTravel {
        delta: (f64, f64, f64),
    },
    RideEntityInLava {
        delta: (f64, f64, f64),
        dimension: String,
        vehicle_type: Option<String>,
    },
    ShotCrossbow {
        item_id: String,
        count: u32,
    },
    AvoidVibration,
    BeeNestDestroyed {
        block_id: String,
        num_bees_inside: i32,
    },
    ChanneledLightning {
        victim_type: String,
    },
    LightningStrike {
        bystander_types: Vec<String>,
        struck_entities: Vec<LightningStruckEntity>,
        blocks_set_on_fire: i32,
        distance: f64,
    },
    KillMobNearSculkCatalyst,
    HeroOfTheVillage,
    PlayerInteractedWithEntity {
        entity_type: String,
        item_id: String,
        is_baby: bool,
        player_wearing_safe_armor: bool,
        distance: EntityDistance,
    },
    ThrownItemPickedUpByEntity {
        entity_type: String,
        item_id: String,
        is_baby: bool,
        player_wearing_safe_armor: bool,
    },
    PlayerShearedEquipment {
        entity_type: String,
        item_id: String,
        distance: EntityDistance,
    },
    StartedRiding {
        vehicle_type: Option<String>,
        passenger_type: Option<String>,
    },
    SummonedEntity {
        entity_type: String,
        /// Player-to-summoned-entity distance, checked against the entity's `distance` predicate.
        distance: Option<EntityDistance>,
    },
    CrafterRecipeCrafted {
        recipe_id: String,
    },
    EffectsChanged {
        /// Entity type of the effect's source; vanilla passes `null` for removals and most sources.
        source_type: Option<String>,
        /// Distance from the source to the player, for the source's `distance` predicate.
        source_distance: Option<EntityDistance>,
    },
    SpearMobs {
        count: i32,
    },
    UsedTotem {
        item_id: String,
        count: u32,
    },
    Tick,
}

impl Player {
    /// Evaluates any active criteria listeners for the given trigger type against the provided predicate.
    pub fn trigger_criterion_event(
        &self,
        trigger_type: &str,
        predicate_matcher: impl Fn(&CriterionConditions) -> bool,
    ) {
        let to_finish = {
            let Ok(mut adv) = self.advancements.try_lock() else {
                return;
            };

            let Some(listeners) = adv.active_listeners.get(trigger_type) else {
                return;
            };

            if listeners.is_empty() {
                return;
            }

            let mut matched = Vec::new();
            for listener in listeners {
                if predicate_matcher(&listener.conditions) {
                    matched.push((listener.advancement, listener.criterion));
                }
            }

            if matched.is_empty() {
                return;
            }

            let Some(player) = adv.player.upgrade() else {
                return;
            };

            let mut awards = Vec::with_capacity(matched.len());
            for (advancement, criterion) in matched {
                let award = adv.award(advancement, criterion);
                if award.completed {
                    awards.push((player.clone(), advancement, award));
                }
            }
            awards
        };

        for (player, advancement, award) in to_finish {
            PlayerAdvancement::finish_award(&player, advancement, award);
        }
    }

    /// Evaluates data-driven advancement triggers for the player.
    #[expect(
        clippy::too_many_lines,
        reason = "dispatching all advancement triggers"
    )]
    pub fn trigger_advancement(&self, trigger: AdvancementTrigger) {
        match trigger {
            AdvancementTrigger::InventoryChanged => {
                let mut inventory_items: Vec<predicate::InventoryItemSnapshot<'_>> = Vec::new();
                let mut slots_full = 0;
                let mut slots_empty = 0;
                let mut slots_occupied = 0;

                if let Ok(main_inv) = self.inventory.main_inventory.try_read() {
                    for stack in main_inv.iter() {
                        if stack.is_empty() {
                            slots_empty += 1;
                        } else {
                            slots_occupied += 1;
                            if stack.item_count >= 64 {
                                slots_full += 1;
                            }
                            inventory_items.push((
                                stack.item.registry_key,
                                u32::from(stack.item_count),
                                &[],
                            ));
                        }
                    }
                }
                if let Ok(equipment) = self.inventory.entity_equipment.try_lock() {
                    for stack in equipment.equipment.values() {
                        if !stack.is_empty() {
                            inventory_items.push((
                                stack.item.registry_key,
                                u32::from(stack.item_count),
                                &[],
                            ));
                        }
                    }
                }

                self.trigger_criterion_event("minecraft:inventory_changed", |cond| match cond {
                    CriterionConditions::InventoryChanged(pred) => {
                        pred.matches(&inventory_items, slots_full, slots_empty, slots_occupied)
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::Location => {
                let pos = self.position();
                let block_pos = BlockPos::new(
                    pos.x.floor() as i32,
                    pos.y.floor() as i32,
                    pos.z.floor() as i32,
                );
                let world = self.world();
                let biome = world.get_biome(&block_pos).registry_id;
                let dimension = world.dimension.minecraft_name;
                let step_pos = BlockPos::new(block_pos.0.x, block_pos.0.y - 1, block_pos.0.z);
                let step_block = Some(
                    pumpkin_data::Block::from_state_id(world.get_block_state(&step_pos).id).name,
                );
                let current_block =
                    pumpkin_data::Block::from_state_id(world.get_block_state(&block_pos).id).name;
                let stepping_block = if current_block == "powder_snow" {
                    Some("powder_snow")
                } else {
                    step_block
                };
                let feet_item = self
                    .living_entity
                    .entity_equipment
                    .try_lock()
                    .ok()
                    .and_then(|guard| {
                        guard
                            .equipment
                            .get(&pumpkin_data::data_component_impl::EquipmentSlot::FEET)
                            .filter(|stack| !stack.is_empty())
                            .map(|stack| stack.item.registry_key.to_string())
                    });

                self.trigger_criterion_event("minecraft:location", |cond| match cond {
                    CriterionConditions::Location(pred) => {
                        let structure = pred
                            .structure
                            .as_ref()
                            .and_then(|_| world.structure_at(block_pos));
                        let light = pred
                            .light
                            .as_ref()
                            .map(|_| i64::from(world.get_max_local_raw_brightness(&block_pos)));
                        let fluid = pred
                            .fluid
                            .as_ref()
                            .map(|_| world.get_fluid(&block_pos).name);
                        pred.matches_at(
                            (pos.x, pos.y, pos.z),
                            Some(biome),
                            Some(dimension),
                            step_block,
                            None,
                            None,
                            stepping_block,
                            feet_item.as_deref(),
                            structure,
                            light,
                            fluid,
                        )
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::PlayerKilledEntity {
                entity_type_resource,
                state,
            } => {
                self.trigger_criterion_event("minecraft:player_killed_entity", |cond| match cond {
                    CriterionConditions::PlayerKilledEntity(pred) => {
                        pred.matches_entity(&entity_type_resource, &state)
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::SniperDuel { distance } => {
                self.trigger_criterion_event("minecraft:player_killed_entity", |cond| match cond {
                    CriterionConditions::PlayerKilledEntity(pred) => {
                        pred.matches("minecraft:skeleton", Some(distance), None)
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::TwoBirdsOneArrow | AdvancementTrigger::Arbalistic => {
                self.trigger_criterion_event("minecraft:killed_by_arrow", |_cond| true);
            }

            AdvancementTrigger::EnterDimension { dimension } => {
                self.trigger_criterion_event("minecraft:changed_dimension", |cond| match cond {
                    CriterionConditions::ChangedDimension(pred) => pred.matches(None, &dimension),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::SleptInBed => {
                self.trigger_criterion_event("minecraft:slept_in_bed", |_cond| true);
            }

            AdvancementTrigger::ConsumeItem { item_id, count } => {
                self.trigger_criterion_event("minecraft:consume_item", |cond| match cond {
                    CriterionConditions::ConsumeItem(pred) => pred.matches(&item_id, count, &[]),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::PlacedBlock {
                block_id,
                item_id,
                count,
                enchantments,
            } => {
                let ench_refs: Vec<(&str, u32)> = enchantments
                    .iter()
                    .map(|(name, level)| (name.as_str(), *level))
                    .collect();
                self.trigger_criterion_event("minecraft:placed_block", |cond| match cond {
                    CriterionConditions::PlacedBlock(pred) => {
                        pred.matches(&block_id, item_id.as_deref(), count, &ench_refs)
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::EnterBlock { block_id } => {
                self.trigger_criterion_event("minecraft:enter_block", |cond| match cond {
                    CriterionConditions::EnterBlock(pred) => {
                        pred.matches(pumpkin_data::tag::RegistryKey::Block, &block_id)
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::FishedItem { items } => {
                self.trigger_criterion_event("minecraft:fishing_rod_hooked", |cond| match cond {
                    CriterionConditions::FishingRodHooked(pred) => items
                        .iter()
                        .any(|(item_id, count)| pred.matches(item_id, *count, &[])),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::DeflectedDamage => {
                self.trigger_criterion_event("minecraft:entity_hurt_player", |cond| match cond {
                    CriterionConditions::EntityHurtPlayer(pred) => pred.matches(true, true, None),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::PlayerKilled => {
                self.trigger_criterion_event("minecraft:entity_killed_player", |_cond| true);
            }

            AdvancementTrigger::BredAnimal {
                parent_type,
                partner_type,
                child_type,
            } => {
                self.trigger_criterion_event("minecraft:bred_animals", |cond| match cond {
                    CriterionConditions::BredAnimals(pred) => {
                        pred.matches(&parent_type, &partner_type, child_type.as_deref())
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::LaunchedEyeOfEnder => {
                self.trigger_criterion_event("minecraft:used_ender_eye", |_cond| true);
            }

            AdvancementTrigger::GlowedSign => {
                self.trigger_criterion_event("minecraft:item_used_on_block", |_cond| true);
            }

            AdvancementTrigger::DealtOverkillDamage => {
                self.trigger_criterion_event("minecraft:player_hurt_entity", |cond| match cond {
                    CriterionConditions::PlayerHurtEntity(pred) => pred.matches_with_context(
                        false,
                        false,
                        true,
                        Some(100.0),
                        Some("minecraft:player"),
                    ),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::PlayerHurtEntity {
                is_blocked,
                is_projectile,
                is_mace_smash,
                original_damage,
                actual_damage,
                direct_entity_type,
                source_entity_type,
                source_distance,
                source_is_baby,
                damage_type,
            } => {
                self.trigger_criterion_event("minecraft:player_hurt_entity", |cond| match cond {
                    CriterionConditions::PlayerHurtEntity(pred) => pred.matches_with_damage_type(
                        is_blocked,
                        is_projectile,
                        is_mace_smash,
                        DamagePair {
                            original: original_damage,
                            actual: actual_damage,
                        },
                        &DamageEntities {
                            direct_entity_type: direct_entity_type.as_deref(),
                            source_entity_type: source_entity_type.as_deref(),
                            source_distance,
                            source_is_baby,
                        },
                        &damage_type,
                    ),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::Bullseye => {
                self.trigger_criterion_event("minecraft:target_hit", |cond| match cond {
                    CriterionConditions::TargetHit(pred) => pred.matches(15, Some(30.0)),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::CuredZombieVillager => {
                self.trigger_criterion_event("minecraft:cured_zombie_villager", |_cond| true);
            }

            AdvancementTrigger::TradedWithVillager => {
                self.trigger_criterion_event("minecraft:villager_trade", |_cond| true);
            }

            AdvancementTrigger::AllayDropItemOnBlock { block_id, item_id } => {
                self.trigger_criterion_event(
                    "minecraft:allay_drop_item_on_block",
                    |cond| match cond {
                        CriterionConditions::AllayDropItemOnBlock(pred) => {
                            pred.matches(&block_id, &item_id, &[], None, None)
                        }
                        CriterionConditions::Custom { .. } => true,
                        _ => false,
                    },
                );
            }

            AdvancementTrigger::ThrownItemPickedUpByPlayer { thrower_type } => {
                self.trigger_criterion_event("minecraft:thrown_item_picked_up_by_player", |cond| {
                    match cond {
                        CriterionConditions::ThrownItemPickedUpByPlayer(pred) => {
                            pred.matches(&thrower_type, None, None)
                        }
                        CriterionConditions::Custom { .. } => true,
                        _ => false,
                    }
                });
            }

            AdvancementTrigger::ItemUsedOnBlock {
                block_id,
                item_id,
                state,
                smokey,
                biome,
            } => {
                self.trigger_criterion_event("minecraft:item_used_on_block", |cond| match cond {
                    CriterionConditions::ItemUsedOnBlock(pred) => pred.matches(
                        &block_id,
                        &item_id,
                        state.as_deref().unwrap_or(&[]),
                        smokey,
                        biome.as_deref(),
                    ),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::TameAnimal {
                entity_type,
                distance,
                components,
            } => {
                self.trigger_criterion_event("minecraft:tame_animal", |cond| match cond {
                    CriterionConditions::TameAnimal(pred) => {
                        pred.matches(&entity_type, distance, None)
                            && pred.matches_components(&components)
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::FilledBucket { item_id, count } => {
                self.trigger_criterion_event("minecraft:filled_bucket", |cond| match cond {
                    CriterionConditions::FilledBucket(pred) => pred.matches(&item_id, count, &[]),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::UsingItem {
                item_id,
                looking_at,
            } => {
                self.trigger_criterion_event("minecraft:using_item", |cond| match cond {
                    CriterionConditions::UsingItem {
                        item,
                        looking_at: exp_look,
                    } => {
                        if !item.matches(&item_id, 1, &[]) {
                            return false;
                        }
                        if let Some(expected) = exp_look {
                            let Some(actual) = looking_at.as_deref() else {
                                return false;
                            };
                            if !expected.matches(pumpkin_data::tag::RegistryKey::EntityType, actual)
                            {
                                return false;
                            }
                        }
                        true
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::ConstructBeacon { level } => {
                self.trigger_criterion_event("minecraft:construct_beacon", |cond| match cond {
                    CriterionConditions::ConstructBeacon(pred) => pred.matches(level),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::SlideDownBlock { block_id } => {
                self.trigger_criterion_event("minecraft:slide_down_block", |cond| match cond {
                    CriterionConditions::SlideDownBlock(pred) => {
                        pred.matches(pumpkin_data::tag::RegistryKey::Block, &block_id)
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::RecipeCrafted { recipe_id } => {
                self.trigger_criterion_event("minecraft:recipe_crafted", |cond| match cond {
                    CriterionConditions::RecipeCrafted(pred) => pred.matches(&recipe_id),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::RecipeUnlocked { recipe_id } => {
                self.trigger_criterion_event("minecraft:recipe_unlocked", |cond| match cond {
                    CriterionConditions::RecipeUnlocked(pred) => pred.matches(&recipe_id),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::PlayerGeneratesContainerLoot { loot_table_id } => {
                self.trigger_criterion_event("minecraft:player_generates_container_loot", |cond| {
                    match cond {
                        CriterionConditions::PlayerGeneratesContainerLoot(lt) => {
                            predicate::matches_id(lt, &loot_table_id)
                        }
                        CriterionConditions::Custom { .. } => true,
                        _ => false,
                    }
                });
            }

            AdvancementTrigger::EnchantedItem { .. } => {
                self.trigger_criterion_event("minecraft:enchanted_item", |_cond| true);
            }

            AdvancementTrigger::BrewedPotion => {
                self.trigger_criterion_event("minecraft:brewed_potion", |_cond| true);
            }

            AdvancementTrigger::ItemDurabilityChanged {
                item_id,
                vehicle_type,
            } => {
                self.trigger_criterion_event(
                    "minecraft:item_durability_changed",
                    |cond| match cond {
                        CriterionConditions::ItemDurabilityChanged {
                            item,
                            vehicle_type: exp_vt,
                        } => {
                            if !item.matches(&item_id, 1, &[]) {
                                return false;
                            }
                            if let Some(expected) = exp_vt {
                                let Some(actual) = vehicle_type.as_deref() else {
                                    return false;
                                };
                                if !expected
                                    .matches(pumpkin_data::tag::RegistryKey::EntityType, actual)
                                {
                                    return false;
                                }
                            }
                            true
                        }
                        CriterionConditions::Custom { .. } => true,
                        _ => false,
                    },
                );
            }

            AdvancementTrigger::Levitation { delta } => {
                self.trigger_criterion_event("minecraft:levitation", |cond| match cond {
                    CriterionConditions::Levitation {
                        distance: Some(bounds),
                    } => bounds.matches(delta),
                    CriterionConditions::Levitation { distance: None }
                    | CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::FallFromHeight {
                delta,
                from_y,
                to_y,
            } => {
                self.trigger_criterion_event("minecraft:fall_from_height", |cond| match cond {
                    CriterionConditions::FallFromHeight {
                        distance: Some(bounds),
                    } => bounds.matches(delta) && from_y >= 319.0 && to_y <= -59.0,
                    CriterionConditions::FallFromHeight { distance: None } => {
                        from_y >= 319.0 && to_y <= -59.0
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::FallAfterExplosion { delta, cause } => {
                self.trigger_criterion_event("minecraft:fall_after_explosion", |cond| match cond {
                    CriterionConditions::FallAfterExplosion {
                        distance: Some(bounds),
                        cause_entity,
                    } => {
                        bounds.matches(delta)
                            && cause_entity
                                .as_ref()
                                .is_none_or(|c| c.matches(&cause, None, None))
                    }
                    CriterionConditions::FallAfterExplosion {
                        distance: None,
                        cause_entity,
                    } => cause_entity
                        .as_ref()
                        .is_none_or(|c| c.matches(&cause, None, None)),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::NetherTravel { delta } => {
                self.trigger_criterion_event("minecraft:nether_travel", |cond| match cond {
                    CriterionConditions::NetherTravel {
                        distance: Some(bounds),
                    } => bounds.matches(delta),
                    CriterionConditions::NetherTravel { distance: None }
                    | CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::RideEntityInLava {
                delta,
                dimension,
                vehicle_type,
            } => {
                self.trigger_criterion_event("minecraft:ride_entity_in_lava", |cond| match cond {
                    CriterionConditions::RideEntityInLava {
                        distance: exp_d,
                        dimension: exp_dim,
                        vehicle_type: exp_vt,
                    } => {
                        if let Some(d) = exp_d
                            && !d.matches(delta)
                        {
                            return false;
                        }
                        if let Some(dim) = exp_dim
                            && !predicate::matches_id(dim, &dimension)
                        {
                            return false;
                        }
                        if let Some(vt) = exp_vt {
                            let Some(actual) = vehicle_type.as_deref() else {
                                return false;
                            };
                            if !vt.matches(pumpkin_data::tag::RegistryKey::EntityType, actual) {
                                return false;
                            }
                        }
                        true
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::ShotCrossbow { item_id, count } => {
                self.trigger_criterion_event("minecraft:shot_crossbow", |cond| match cond {
                    CriterionConditions::ShotCrossbow(pred) => pred.matches(&item_id, count, &[]),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::AvoidVibration => {
                self.trigger_criterion_event("minecraft:avoid_vibration", |_cond| true);
            }

            AdvancementTrigger::BeeNestDestroyed {
                block_id,
                num_bees_inside,
            } => {
                self.trigger_criterion_event("minecraft:bee_nest_destroyed", |cond| match cond {
                    CriterionConditions::BeeNestDestroyed {
                        block,
                        num_bees_inside: req_bees,
                    } => {
                        block
                            .as_ref()
                            .is_none_or(|b| predicate::matches_id(b, &block_id))
                            && req_bees.is_none_or(|r| num_bees_inside >= r)
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::ChanneledLightning { victim_type } => {
                self.trigger_criterion_event("minecraft:channeled_lightning", |cond| match cond {
                    CriterionConditions::ChanneledLightning { victim_type: req } => req
                        .as_ref()
                        .is_none_or(|r| predicate::matches_id(r, &victim_type)),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::LightningStrike {
                bystander_types,
                struck_entities,
                blocks_set_on_fire,
                distance,
            } => {
                self.trigger_criterion_event("minecraft:lightning_strike", |cond| match cond {
                    CriterionConditions::LightningStrike {
                        bystander: req_b,
                        blocks_set_on_fire: req_fire,
                        entity_struck: req_struck,
                        distance_max,
                    } => {
                        if let Some(r) = req_b
                            && !bystander_types
                                .iter()
                                .any(|b| r.matches(b.as_str(), None, None))
                        {
                            return false;
                        }
                        if let Some(f) = req_fire
                            && !f.matches(i64::from(blocks_set_on_fire))
                        {
                            return false;
                        }
                        if let Some(struck) = req_struck
                            && !struck_entities
                                .iter()
                                .any(|e| struck.matches(&e.entity_type, e.distance, e.is_baby))
                        {
                            return false;
                        }
                        if let Some(max_d) = distance_max
                            && distance > *max_d
                        {
                            return false;
                        }
                        true
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::KillMobNearSculkCatalyst => {
                self.trigger_criterion_event("minecraft:kill_mob_near_sculk_catalyst", |_cond| {
                    let pos = self.position().to_block_pos();
                    let world = self.world();
                    for dx in -8..=8 {
                        for dy in -8..=8 {
                            for dz in -8..=8 {
                                let check = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                                if world.get_block(&check) == &pumpkin_data::Block::SCULK_CATALYST {
                                    return true;
                                }
                            }
                        }
                    }
                    false
                });
            }

            AdvancementTrigger::HeroOfTheVillage => {
                self.trigger_criterion_event("minecraft:hero_of_the_village", |_cond| true);
            }

            AdvancementTrigger::PlayerInteractedWithEntity {
                entity_type,
                item_id,
                is_baby,
                player_wearing_safe_armor,
                distance,
            } => {
                self.trigger_criterion_event("minecraft:player_interacted_with_entity", |cond| {
                    match cond {
                        CriterionConditions::PlayerInteractedWithEntity {
                            entity,
                            item,
                            player,
                        } => {
                            if let Some(e) = entity
                                && !e.matches(&entity_type, Some(distance), Some(is_baby))
                            {
                                return false;
                            }
                            if let Some(i) = item
                                && !i.matches(&item_id, 1, &[])
                            {
                                return false;
                            }
                            if let Some(p) = player
                                && p.safe_armor_disallowed == Some(true)
                                && player_wearing_safe_armor
                            {
                                return false;
                            }
                            true
                        }
                        CriterionConditions::Custom { .. } => true,
                        _ => false,
                    }
                });
            }

            AdvancementTrigger::ThrownItemPickedUpByEntity {
                entity_type,
                item_id,
                is_baby,
                player_wearing_safe_armor,
            } => {
                self.trigger_criterion_event("minecraft:thrown_item_picked_up_by_entity", |cond| {
                    match cond {
                        CriterionConditions::ThrownItemPickedUpByEntity {
                            entity,
                            item,
                            player,
                        } => {
                            if let Some(e) = entity
                                && !e.matches(&entity_type, None, Some(is_baby))
                            {
                                return false;
                            }
                            if let Some(i) = item
                                && !i.matches(&item_id, 1, &[])
                            {
                                return false;
                            }
                            if let Some(p) = player
                                && p.safe_armor_disallowed == Some(true)
                                && player_wearing_safe_armor
                            {
                                return false;
                            }
                            true
                        }
                        CriterionConditions::Custom { .. } => true,
                        _ => false,
                    }
                });
            }

            AdvancementTrigger::PlayerShearedEquipment {
                entity_type,
                item_id,
                distance,
            } => {
                self.trigger_criterion_event(
                    "minecraft:player_sheared_equipment",
                    |cond| match cond {
                        CriterionConditions::PlayerShearedEquipment { entity, item } => {
                            entity
                                .as_ref()
                                .is_none_or(|e| e.matches(&entity_type, Some(distance), None))
                                && item.as_ref().is_none_or(|i| i.matches(&item_id, 1, &[]))
                        }
                        CriterionConditions::Custom { .. } => true,
                        _ => false,
                    },
                );
            }

            AdvancementTrigger::StartedRiding {
                vehicle_type,
                passenger_type,
            } => {
                self.trigger_criterion_event("minecraft:started_riding", |cond| match cond {
                    CriterionConditions::StartedRiding(Some(pred)) => {
                        if let Some(vt) = &pred.vehicle_type {
                            let Some(actual_v) = vehicle_type.as_deref() else {
                                return false;
                            };
                            if !vt.matches(pumpkin_data::tag::RegistryKey::EntityType, actual_v) {
                                return false;
                            }
                        }
                        if let Some(pt) = &pred.passenger_type {
                            let Some(actual_p) = passenger_type.as_deref() else {
                                return false;
                            };
                            if !pt.matches(pumpkin_data::tag::RegistryKey::EntityType, actual_p) {
                                return false;
                            }
                        }
                        true
                    }
                    CriterionConditions::StartedRiding(None)
                    | CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::SummonedEntity {
                entity_type,
                distance,
            } => {
                self.trigger_criterion_event("minecraft:summoned_entity", |cond| match cond {
                    CriterionConditions::SummonedEntity(pred) => {
                        pred.matches(&entity_type, distance, None)
                    }
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::CrafterRecipeCrafted { recipe_id } => {
                self.trigger_criterion_event(
                    "minecraft:crafter_recipe_crafted",
                    |cond| match cond {
                        CriterionConditions::CrafterRecipeCrafted(r) => {
                            predicate::matches_id(r, &recipe_id)
                        }
                        CriterionConditions::Custom { .. } => true,
                        _ => false,
                    },
                );
            }

            AdvancementTrigger::EffectsChanged {
                source_type,
                source_distance,
            } => {
                let effects = self.get_active_effects();
                self.trigger_criterion_event("minecraft:effects_changed", |cond| {
                    predicate::matches_effects_changed_condition(
                        cond,
                        &effects,
                        source_type.as_deref(),
                        source_distance,
                    )
                });
            }

            AdvancementTrigger::SpearMobs { count } => {
                self.trigger_criterion_event("minecraft:spear_mobs", |cond| match cond {
                    CriterionConditions::SpearMobs { count: req } => req.is_none_or(|r| count >= r),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::UsedTotem { item_id, count } => {
                self.trigger_criterion_event("minecraft:used_totem", |cond| match cond {
                    CriterionConditions::UsedTotem(pred) => pred.matches(&item_id, count, &[]),
                    CriterionConditions::Custom { .. } => true,
                    _ => false,
                });
            }

            AdvancementTrigger::Tick => {
                self.trigger_criterion_event("minecraft:tick", |_cond| true);
            }
        }
    }
}
