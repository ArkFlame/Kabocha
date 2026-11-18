use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::tag::{RegistryKey, Taggable, get_tag_values};
use serde_json::Value;

pub type ItemEnchantment<'a> = (&'a str, u32);
pub type InventoryItemSnapshot<'a> = (&'a str, u32, &'a [ItemEnchantment<'a>]);

/// Checks if an expected identifier equals the actual identifier, ignoring "minecraft:" prefix.
#[must_use]
pub fn matches_id(expected: &str, actual: &str) -> bool {
    let exp_clean = expected.strip_prefix("minecraft:").unwrap_or(expected);
    let act_clean = actual.strip_prefix("minecraft:").unwrap_or(actual);
    exp_clean == act_clean
}

/// Matches an identifier or tag against a registry tag collection.
#[must_use]
pub fn matches_tag_or_id(registry_key: RegistryKey, expected: &str, actual: &str) -> bool {
    if let Some(tag_name) = expected.strip_prefix('#') {
        if let Some(values) = get_tag_values(registry_key, tag_name) {
            let act_clean = actual.strip_prefix("minecraft:").unwrap_or(actual);
            return values.iter().any(|v| {
                let v_clean = v.strip_prefix("minecraft:").unwrap_or(v);
                v_clean == act_clean
            });
        }
        if !tag_name.contains(':') {
            let full_tag = format!("minecraft:{tag_name}");
            if let Some(values) = get_tag_values(registry_key, &full_tag) {
                let act_clean = actual.strip_prefix("minecraft:").unwrap_or(actual);
                return values.iter().any(|v| {
                    let v_clean = v.strip_prefix("minecraft:").unwrap_or(v);
                    v_clean == act_clean
                });
            }
        }
        false
    } else {
        matches_id(expected, actual)
    }
}

/// Matches against a string, tag, or array of strings/tags in JSON.
#[must_use]
pub fn matches_any_tag_or_id(registry_key: RegistryKey, expected: &Value, actual: &str) -> bool {
    match expected {
        Value::String(s) => matches_tag_or_id(registry_key, s, actual),
        Value::Array(arr) => arr.iter().any(|item| {
            item.as_str()
                .is_some_and(|s| matches_tag_or_id(registry_key, s, actual))
        }),
        _ => false,
    }
}

/// Checks if integer value is within the specified bounds (single number or `{"min": ..., "max": ...}`).
#[must_use]
pub fn matches_int_bounds(bounds: &Value, val: i64) -> bool {
    IntBounds::from_value(bounds).is_none_or(|b| b.matches(val))
}

/// Checks if float value is within the specified bounds (single number or `{"min": ..., "max": ...}`).
#[must_use]
pub fn matches_float_bounds(bounds: &Value, val: f64) -> bool {
    FloatBounds::from_value(bounds).is_none_or(|b| b.matches(val))
}

/// Strongly-typed bounds for integer numbers.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct IntBounds {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl IntBounds {
    #[must_use]
    pub fn matches(&self, val: i64) -> bool {
        self.min.is_none_or(|min| val >= min) && self.max.is_none_or(|max| val <= max)
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Option<Self> {
        match val {
            Value::Number(n) => n.as_i64().map(|v| Self {
                min: Some(v),
                max: Some(v),
            }),
            Value::Object(map) => {
                let min = map.get("min").and_then(Value::as_i64);
                let max = map.get("max").and_then(Value::as_i64);
                Some(Self { min, max })
            }
            _ => None,
        }
    }
}

/// Strongly-typed bounds for floating point numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct FloatBounds {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl FloatBounds {
    #[must_use]
    pub fn matches(&self, val: f64) -> bool {
        self.min.is_none_or(|min| val >= min) && self.max.is_none_or(|max| val <= max)
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Option<Self> {
        match val {
            Value::Number(n) => n.as_f64().map(|v| Self {
                min: Some(v),
                max: Some(v),
            }),
            Value::Object(map) => {
                let min = map.get("min").and_then(Value::as_f64);
                let max = map.get("max").and_then(Value::as_f64);
                Some(Self { min, max })
            }
            _ => None,
        }
    }
}

/// `DistancePredicate`: per-axis and derived bounds over the origin-to-entity delta.
/// Vanilla truncates each delta component to f32 before comparing.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DistancePredicate {
    pub x: Option<FloatBounds>,
    pub y: Option<FloatBounds>,
    pub z: Option<FloatBounds>,
    pub horizontal: Option<FloatBounds>,
    pub absolute: Option<FloatBounds>,
}

impl DistancePredicate {
    /// `delta` is `entity - origin`, the vectors vanilla's `DistancePredicate::matches` receives.
    /// Each delta component is truncated to f32 there before the axis checks; the derived
    /// horizontal and absolute bounds compare the squared length, which is the same ordering.
    #[must_use]
    pub fn matches(&self, delta: (f64, f64, f64)) -> bool {
        let (dx, dy, dz) = delta;
        let xd = f64::from(dx as f32);
        let yd = f64::from(dy as f32);
        let zd = f64::from(dz as f32);
        self.x.as_ref().is_none_or(|b| b.matches(xd.abs()))
            && self.y.as_ref().is_none_or(|b| b.matches(yd.abs()))
            && self.z.as_ref().is_none_or(|b| b.matches(zd.abs()))
            && self
                .horizontal
                .as_ref()
                .is_none_or(|b| b.matches(xd.hypot(zd)))
            && self
                .absolute
                .as_ref()
                .is_none_or(|b| b.matches((xd * xd + yd * yd + zd * zd).sqrt()))
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Option<Self> {
        let Value::Object(map) = val else {
            return None;
        };
        Some(Self {
            x: map.get("x").and_then(FloatBounds::from_value),
            y: map.get("y").and_then(FloatBounds::from_value),
            z: map.get("z").and_then(FloatBounds::from_value),
            horizontal: map.get("horizontal").and_then(FloatBounds::from_value),
            absolute: map.get("absolute").and_then(FloatBounds::from_value),
        })
    }
}

/// Matches against one or more item/block/entity tags or IDs.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TagOrIdMatcher {
    pub patterns: Vec<String>,
}

impl TagOrIdMatcher {
    #[must_use]
    pub fn matches(&self, registry_key: RegistryKey, actual: &str) -> bool {
        if self.patterns.is_empty() {
            return true;
        }
        self.patterns
            .iter()
            .any(|pattern| matches_tag_or_id(registry_key, pattern, actual))
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Option<Self> {
        match val {
            Value::String(s) => Some(Self {
                patterns: vec![s.clone()],
            }),
            Value::Array(arr) => {
                let patterns = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect();
                Some(Self { patterns })
            }
            _ => None,
        }
    }
}

/// Enchantment requirement for an item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnchantmentPredicate {
    pub enchantment: String,
    pub levels: IntBounds,
}

/// Strongly-typed condition for item matching.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ItemPredicate {
    pub items: Option<TagOrIdMatcher>,
    pub count: Option<IntBounds>,
    pub enchantments: Vec<EnchantmentPredicate>,
}

impl ItemPredicate {
    #[must_use]
    pub fn matches(&self, item_key: &str, count: u32, enchantments: &[(&str, u32)]) -> bool {
        if let Some(items) = &self.items
            && !items.matches(RegistryKey::Item, item_key)
        {
            return false;
        }
        if let Some(count_bounds) = &self.count
            && !count_bounds.matches(i64::from(count))
        {
            return false;
        }
        for target_ench in &self.enchantments {
            let matched = enchantments.iter().any(|(name, level)| {
                matches_tag_or_id(RegistryKey::Enchantment, &target_ench.enchantment, name)
                    && target_ench.levels.matches(i64::from(*level))
            });
            if !matched {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(pred_val: &Value) -> Self {
        let Value::Object(pred_map) = pred_val else {
            return TagOrIdMatcher::from_value(pred_val).map_or_else(Self::default, |matcher| {
                Self {
                    items: Some(matcher),
                    count: None,
                    enchantments: Vec::new(),
                }
            });
        };

        let items = pred_map.get("items").and_then(TagOrIdMatcher::from_value);
        let count = pred_map.get("count").and_then(IntBounds::from_value);

        let mut enchantments = Vec::new();
        // 1.20.5+ serialises item component predicates under `components`; older packs use `predicates`.
        if let Some(predicates) = pred_map
            .get("components")
            .or_else(|| pred_map.get("predicates"))
            .and_then(Value::as_object)
            && let Some(enchants) = predicates
                .get("minecraft:enchantments")
                .and_then(Value::as_array)
        {
            for ench_cond in enchants {
                if let Some(target_ench) = ench_cond.get("enchantments").and_then(Value::as_str) {
                    let levels = ench_cond
                        .get("levels")
                        .and_then(IntBounds::from_value)
                        .unwrap_or_default();
                    enchantments.push(EnchantmentPredicate {
                        enchantment: target_ench.to_string(),
                        levels,
                    });
                }
            }
        }

        Self {
            items,
            count,
            enchantments,
        }
    }
}

/// Slot condition for `minecraft:inventory_changed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventorySlotsPredicate {
    pub empty: Option<IntBounds>,
    pub full: Option<IntBounds>,
    pub occupied: Option<IntBounds>,
}

/// Strongly-typed condition for `minecraft:inventory_changed`.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InventoryChangedPredicate {
    pub items: Vec<ItemPredicate>,
    pub slots: Option<InventorySlotsPredicate>,
}

impl InventoryChangedPredicate {
    #[must_use]
    pub fn matches(
        &self,
        inventory: &[InventoryItemSnapshot<'_>],
        slots_full: usize,
        slots_empty: usize,
        slots_occupied: usize,
    ) -> bool {
        if let Some(slots) = &self.slots {
            if let Some(empty) = &slots.empty
                && !empty.matches(slots_empty as i64)
            {
                return false;
            }
            if let Some(full) = &slots.full
                && !full.matches(slots_full as i64)
            {
                return false;
            }
            if let Some(occupied) = &slots.occupied
                && !occupied.matches(slots_occupied as i64)
            {
                return false;
            }
        }

        for item_pred in &self.items {
            let matched = inventory
                .iter()
                .any(|(key, count, enchants)| item_pred.matches(key, *count, enchants));
            if !matched {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };

        let slots = map
            .get("slots")
            .and_then(Value::as_object)
            .map(|slots_map| InventorySlotsPredicate {
                empty: slots_map.get("empty").and_then(IntBounds::from_value),
                full: slots_map.get("full").and_then(IntBounds::from_value),
                occupied: slots_map.get("occupied").and_then(IntBounds::from_value),
            });

        let mut items = Vec::new();
        if let Some(items_val) = map.get("items").and_then(Value::as_array) {
            for item_val in items_val {
                items.push(ItemPredicate::from_value(item_val));
            }
        }

        Self { items, slots }
    }
}

/// Strongly-typed location condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LocationPredicate {
    pub x: Option<FloatBounds>,
    pub y: Option<FloatBounds>,
    pub z: Option<FloatBounds>,
    pub biome: Option<TagOrIdMatcher>,
    pub dimension: Option<String>,
    pub block: Option<TagOrIdMatcher>,
    pub can_see_sky: Option<bool>,
    pub smokey: Option<bool>,
    pub structure: Option<TagOrIdMatcher>,
    pub stepping_on: Option<TagOrIdMatcher>,
    pub feet_item: Option<TagOrIdMatcher>,
    pub light: Option<IntBounds>,
    pub fluid: Option<TagOrIdMatcher>,
}

impl LocationPredicate {
    #[must_use]
    #[expect(clippy::too_many_arguments)]
    pub fn matches(
        &self,
        pos: (f64, f64, f64),
        biome: Option<&str>,
        dimension: Option<&str>,
        step_block: Option<&str>,
        can_see_sky: Option<bool>,
        smokey: Option<bool>,
        stepping_block: Option<&str>,
        feet_item: Option<&str>,
        structure: Option<&str>,
    ) -> bool {
        self.matches_at(
            pos,
            biome,
            dimension,
            step_block,
            can_see_sky,
            smokey,
            stepping_block,
            feet_item,
            structure,
            None,
            None,
        )
    }

    /// Like [`Self::matches`], with light and fluid values read at the position.
    #[must_use]
    #[expect(clippy::too_many_arguments)]
    pub fn matches_at(
        &self,
        pos: (f64, f64, f64),
        biome: Option<&str>,
        dimension: Option<&str>,
        step_block: Option<&str>,
        can_see_sky: Option<bool>,
        smokey: Option<bool>,
        stepping_block: Option<&str>,
        feet_item: Option<&str>,
        structure: Option<&str>,
        light: Option<i64>,
        fluid: Option<&str>,
    ) -> bool {
        if let Some(x) = &self.x
            && !x.matches(pos.0)
        {
            return false;
        }
        if let Some(y) = &self.y
            && !y.matches(pos.1)
        {
            return false;
        }
        if let Some(z) = &self.z
            && !z.matches(pos.2)
        {
            return false;
        }
        if let Some(expected_dim) = &self.dimension {
            let Some(dim) = dimension else {
                return false;
            };
            if !matches_id(expected_dim, dim) {
                return false;
            }
        }
        if let Some(biome_matcher) = &self.biome {
            let Some(b) = biome else {
                return false;
            };
            if !biome_matcher.matches(RegistryKey::WorldgenBiome, b) {
                return false;
            }
        }
        if let Some(block_matcher) = &self.block {
            let Some(blk) = step_block else {
                return false;
            };
            if !block_matcher.matches(RegistryKey::Block, blk) {
                return false;
            }
        }
        if let Some(stepping_matcher) = &self.stepping_on {
            let Some(blk) = stepping_block else {
                return false;
            };
            if !stepping_matcher.matches(RegistryKey::Block, blk) {
                return false;
            }
        }
        if let Some(feet_matcher) = &self.feet_item {
            let Some(f) = feet_item else {
                return false;
            };
            if !feet_matcher.matches(RegistryKey::Item, f) {
                return false;
            }
        }
        if let Some(expected_sky) = self.can_see_sky
            && can_see_sky != Some(expected_sky)
        {
            return false;
        }
        if let Some(expected_smokey) = self.smokey
            && smokey != Some(expected_smokey)
        {
            return false;
        }
        if let Some(structure_matcher) = &self.structure {
            let Some(current) = structure else {
                return false;
            };
            if !structure_matcher.matches(RegistryKey::WorldgenStructure, current) {
                return false;
            }
        }
        if let Some(light_bounds) = &self.light {
            let Some(current) = light else {
                return false;
            };
            if !light_bounds.matches(current) {
                return false;
            }
        }
        if let Some(fluid_matcher) = &self.fluid {
            let Some(current) = fluid else {
                return false;
            };
            if !fluid_matcher.matches(RegistryKey::Fluid, current) {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };

        let loc_val = map
            .get("player")
            .and_then(|p| p.get("predicate"))
            .and_then(|pred| {
                pred.get("minecraft:location")
                    .or_else(|| pred.get("location"))
            })
            .or_else(|| map.get("location"))
            .unwrap_or(val);
        let loc_map = loc_val.as_object().unwrap_or(map);

        let (x, y, z) = loc_map
            .get("position")
            .map_or((None, None, None), |pos_val| {
                (
                    pos_val.get("x").and_then(FloatBounds::from_value),
                    pos_val.get("y").and_then(FloatBounds::from_value),
                    pos_val.get("z").and_then(FloatBounds::from_value),
                )
            });

        let biome = loc_map
            .get("biomes")
            .or_else(|| loc_map.get("biome"))
            .and_then(TagOrIdMatcher::from_value);

        let dimension = loc_map
            .get("dimension")
            .and_then(Value::as_str)
            .map(ToString::to_string);

        let block = loc_map
            .get("block")
            .or_else(|| loc_map.get("blocks"))
            .and_then(|b_val| {
                let pred = b_val.get("blocks").unwrap_or(b_val);
                TagOrIdMatcher::from_value(pred)
            });

        let can_see_sky = loc_map.get("can_see_sky").and_then(Value::as_bool);
        let smokey = loc_map.get("smokey").and_then(Value::as_bool);
        // Vanilla wraps the integer bounds in a LightPredicate under "light".
        let light = loc_map.get("light").and_then(|v| {
            let bounds = v.get("light").unwrap_or(v);
            IntBounds::from_value(bounds)
        });
        let fluid = loc_map.get("fluid").and_then(|v| {
            let pred = v.get("fluids").unwrap_or(v);
            TagOrIdMatcher::from_value(pred)
        });
        let structure = loc_map
            .get("structures")
            .or_else(|| loc_map.get("structure"))
            .and_then(TagOrIdMatcher::from_value);

        let player_pred = map.get("player").and_then(|p| p.get("predicate"));
        let stepping_on = player_pred
            .and_then(|p| {
                p.get("minecraft:stepping_on")
                    .or_else(|| p.get("stepping_on"))
            })
            .and_then(|s| s.get("block").or_else(|| s.get("blocks")))
            .and_then(|b| b.get("blocks").or(Some(b)))
            .and_then(TagOrIdMatcher::from_value);
        let feet_item = player_pred
            .and_then(|p| p.get("minecraft:equipment").or_else(|| p.get("equipment")))
            .and_then(|eq| eq.get("feet"))
            .and_then(|f| f.get("items"))
            .and_then(TagOrIdMatcher::from_value);

        Self {
            x,
            y,
            z,
            biome,
            dimension,
            block,
            can_see_sky,
            smokey,
            structure,
            stepping_on,
            feet_item,
            light,
            fluid,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ItemUsedOnBlockCheck {
    pub blocks: Option<TagOrIdMatcher>,
    pub items: Option<TagOrIdMatcher>,
    pub state_key: Option<String>,
    pub state_value: Option<String>,
    pub smokey: Option<bool>,
    pub biome: Option<String>,
}

impl ItemUsedOnBlockCheck {
    #[must_use]
    fn matches(
        &self,
        block_id: &str,
        item_id: &str,
        block_state: &[(String, String)],
        smokey: Option<bool>,
        biome: Option<&str>,
    ) -> bool {
        if let Some(block_matcher) = &self.blocks
            && !block_matcher.matches(RegistryKey::Block, block_id)
        {
            return false;
        }
        if let Some(item_matcher) = &self.items
            && !item_matcher.matches(RegistryKey::Item, item_id)
        {
            return false;
        }
        if let Some(expected_key) = &self.state_key {
            let Some((_, actual_value)) = block_state.iter().find(|(key, _)| key == expected_key)
            else {
                return false;
            };
            if let Some(expected_val) = &self.state_value
                && expected_val != actual_value
            {
                return false;
            }
        }
        if let Some(expected_smokey) = self.smokey
            && smokey != Some(expected_smokey)
        {
            return false;
        }
        if let Some(expected_biome) = &self.biome {
            let Some(b) = biome else {
                return false;
            };
            if !matches_id(expected_biome, b) {
                return false;
            }
        }
        true
    }

    #[must_use]
    fn from_value(value: &Value) -> Self {
        let mut block_patterns = Vec::new();
        let mut item_patterns = Vec::new();
        let mut state_key = None;
        let mut state_value = None;
        let mut smokey = None;
        let mut biome = None;

        // Typed loot conditions keep their payload under the predicate key.
        let pred = value.get("predicate").unwrap_or(value);
        let blk = pred.get("block").unwrap_or(pred);

        if let Some(b_val) = blk.get("blocks").or_else(|| blk.get("block")) {
            if let Some(s) = b_val.as_str() {
                block_patterns.push(s.to_string());
            } else if let Some(arr) = b_val.as_array() {
                block_patterns.extend(
                    arr.iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string),
                );
            }
        }
        if let Some(st) = blk.get("state").and_then(Value::as_object) {
            for (k, v) in st {
                state_key = Some(k.clone());
                state_value = v.as_str().map(ToString::to_string);
            }
        }
        if let Some(sm) = pred.get("smokey").and_then(Value::as_bool) {
            smokey = Some(sm);
        }
        if let Some(b) = pred
            .get("biomes")
            .or_else(|| pred.get("biome"))
            .and_then(Value::as_str)
        {
            biome = Some(b.to_string());
        }
        if let Some(i_val) = pred.get("items").or_else(|| pred.get("item")) {
            push_item_patterns(i_val, &mut item_patterns);
        }
        // In the flat form the item predicate sits directly on the condition value.
        if let Some(i_val) = value.get("items").or_else(|| value.get("item")) {
            push_item_patterns(i_val, &mut item_patterns);
        }

        Self {
            blocks: (!block_patterns.is_empty()).then_some(TagOrIdMatcher {
                patterns: block_patterns,
            }),
            items: (!item_patterns.is_empty()).then_some(TagOrIdMatcher {
                patterns: item_patterns,
            }),
            state_key,
            state_value,
            smokey,
            biome,
        }
    }
}

/// One node of the loot-condition tree behind location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemUsedOnBlockTerm {
    AllOf(Vec<Self>),
    AnyOf(Vec<Self>),
    Inverted(Box<Self>),
    Check(ItemUsedOnBlockCheck),
}

impl ItemUsedOnBlockTerm {
    #[must_use]
    fn matches(
        &self,
        block_id: &str,
        item_id: &str,
        block_state: &[(String, String)],
        smokey: Option<bool>,
        biome: Option<&str>,
    ) -> bool {
        match self {
            Self::AllOf(terms) => terms
                .iter()
                .all(|term| term.matches(block_id, item_id, block_state, smokey, biome)),
            Self::AnyOf(terms) => terms
                .iter()
                .any(|term| term.matches(block_id, item_id, block_state, smokey, biome)),
            Self::Inverted(term) => !term.matches(block_id, item_id, block_state, smokey, biome),
            Self::Check(check) => check.matches(block_id, item_id, block_state, smokey, biome),
        }
    }

    #[must_use]
    fn from_value(value: &Value) -> Self {
        let kind = value
            .get("condition")
            .or_else(|| value.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("");
        match kind {
            "minecraft:all_of" | "all_of" => Self::AllOf(item_used_on_block_terms(value)),
            "minecraft:any_of" | "any_of" => Self::AnyOf(item_used_on_block_terms(value)),
            "minecraft:inverted" | "inverted" => value.get("term").map_or_else(
                || Self::Check(ItemUsedOnBlockCheck::from_value(value)),
                |term| Self::Inverted(Box::new(Self::from_value(term))),
            ),
            _ => Self::Check(ItemUsedOnBlockCheck::from_value(value)),
        }
    }
}

fn item_used_on_block_terms(value: &Value) -> Vec<ItemUsedOnBlockTerm> {
    value
        .get("terms")
        .and_then(Value::as_array)
        .map(|terms| terms.iter().map(ItemUsedOnBlockTerm::from_value).collect())
        .unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ItemUsedOnBlockPredicate {
    pub terms: Vec<ItemUsedOnBlockTerm>,
}

impl ItemUsedOnBlockPredicate {
    #[must_use]
    pub fn matches(
        &self,
        block_id: &str,
        item_id: &str,
        block_state: &[(String, String)],
        smokey: Option<bool>,
        biome: Option<&str>,
    ) -> bool {
        self.terms
            .iter()
            .all(|term| term.matches(block_id, item_id, block_state, smokey, biome))
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let loc = val.get("location").unwrap_or(val);
        let mut terms: Vec<ItemUsedOnBlockTerm> = Vec::new();
        if let Some(arr) = loc.as_array() {
            // 1.20.2+ packs list the loot conditions of location directly as an array.
            terms.extend(arr.iter().map(ItemUsedOnBlockTerm::from_value));
        } else if loc.get("terms").and_then(Value::as_array).is_some()
            && loc.get("condition").or_else(|| loc.get("type")).is_none()
        {
            // A bare object with only terms is an implicit all_of.
            terms.push(ItemUsedOnBlockTerm::AllOf(item_used_on_block_terms(loc)));
        } else {
            terms.push(ItemUsedOnBlockTerm::from_value(loc));
        }
        // A top-level item beside location is an extra all_of term (vanilla MatchTool).
        if val.get("location").is_some()
            && let Some(i_val) = val.get("item")
        {
            let mut patterns = Vec::new();
            push_item_patterns(i_val, &mut patterns);
            terms.push(ItemUsedOnBlockTerm::Check(ItemUsedOnBlockCheck {
                items: (!patterns.is_empty()).then_some(TagOrIdMatcher { patterns }),
                ..ItemUsedOnBlockCheck::default()
            }));
        }
        Self { terms }
    }
}

/// Mirrors vanilla's `MobEffectsPredicate.MobEffectInstancePredicate` together with its map key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobEffectInstancePredicate {
    pub effect: String,
    pub amplifier: Option<IntBounds>,
    pub duration: Option<IntBounds>,
    pub ambient: Option<bool>,
    pub visible: Option<bool>,
}

impl MobEffectInstancePredicate {
    #[must_use]
    pub fn matches(&self, instance: Option<&Effect>) -> bool {
        let Some(instance) = instance else {
            return false;
        };
        self.amplifier
            .as_ref()
            .is_none_or(|b| b.matches(i64::from(instance.amplifier)))
            && self
                .duration
                .as_ref()
                .is_none_or(|b| b.matches(i64::from(instance.duration)))
            && self.ambient.is_none_or(|a| a == instance.ambient)
            && self.visible.is_none_or(|v| v == instance.show_particles)
    }
}

/// Mirrors vanilla's `MobEffectsPredicate`: every listed effect must be active and match.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MobEffectsPredicate {
    pub effect_map: Vec<MobEffectInstancePredicate>,
}

impl MobEffectsPredicate {
    #[must_use]
    pub fn matches(&self, effects: &[Effect]) -> bool {
        self.effect_map.iter().all(|pred| {
            pred.matches(
                effects
                    .iter()
                    .find(|e| matches_id(&pred.effect, e.effect_type.minecraft_name)),
            )
        })
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        Self {
            effect_map: map
                .iter()
                .map(|(effect, pred)| MobEffectInstancePredicate {
                    effect: effect.clone(),
                    amplifier: pred.get("amplifier").and_then(IntBounds::from_value),
                    duration: pred.get("duration").and_then(IntBounds::from_value),
                    ambient: pred.get("ambient").and_then(Value::as_bool),
                    visible: pred.get("visible").and_then(Value::as_bool),
                })
                .collect(),
        }
    }
}

/// Distance between the predicate's origin and the checked entity, split the way vanilla's
/// `EntityPredicate.DistanceData` splits it: `horizontal` ignores Y, `absolute` is the 3D length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityDistance {
    pub horizontal: f64,
    pub absolute: f64,
}

impl EntityDistance {
    /// Builds the distance from the origin-to-entity delta (`entity - origin`).
    #[must_use]
    pub fn from_delta(dx: f64, dy: f64, dz: f64) -> Self {
        Self {
            horizontal: dx.hypot(dz),
            absolute: (dx * dx + dy * dy + dz * dz).sqrt(),
        }
    }

    /// Distance from `origin` to `target`, the positions vanilla passes to `EntityPredicate`.
    #[must_use]
    pub fn between(
        origin: pumpkin_util::math::vector3::Vector3<f64>,
        target: pumpkin_util::math::vector3::Vector3<f64>,
    ) -> Self {
        Self::from_delta(
            target.x - origin.x,
            target.y - origin.y,
            target.z - origin.z,
        )
    }
}

/// Snapshot of the target entity fields vanilla's entity sub-predicates read.
///
/// `distance` and `stepping_block` are `None` when the caller cannot supply them;
/// `stepping_block` is also `None` for an entity that is not on the ground, which is how
/// vanilla fails every `SteppingOnPredicate`.
#[derive(Clone, Default)]
pub struct EntityState {
    pub distance: Option<EntityDistance>,
    pub is_baby: Option<bool>,
    pub effects: Vec<Effect>,
    /// Slot name to item id, as the equipment predicate reads them.
    pub equipment: Vec<(String, String)>,
    /// Block under the entity, set only when it is on the ground;
    /// vanilla skips `SteppingOnPredicate` otherwise.
    pub stepping_block: Option<String>,
    /// Vehicle entity type, matching vanilla `VehiclePredicate` (`entity.getVehicle()`).
    pub vehicle_type: Option<String>,
}

impl EntityState {
    /// Reads the active effects and equipped items of a living entity.
    #[must_use]
    pub fn from_living(
        living: &crate::entity::living::LivingEntity,
        distance: Option<EntityDistance>,
    ) -> Self {
        let effects = living
            .active_effects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .collect::<Vec<Effect>>();
        let equipment = {
            let guard = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard
                .equipment
                .iter()
                .filter(|(_, stack)| !stack.is_empty())
                .map(|(slot, stack)| {
                    (
                        slot.to_name().to_string(),
                        stack.item.registry_key.to_string(),
                    )
                })
                .collect::<Vec<(String, String)>>()
        };
        let stepping_block = living
            .entity
            .on_ground
            .load(std::sync::atomic::Ordering::Relaxed)
            .then(|| {
                let pos = living.entity.get_on_pos();
                living.entity.world.load().get_block(&pos).name.to_string()
            });
        let vehicle_type = living.entity.get_vehicle().map(|vehicle| {
            format!(
                "minecraft:{}",
                vehicle.get_entity().entity_type.resource_name
            )
        });
        Self {
            distance,
            is_baby: Some(living.is_baby()),
            effects,
            equipment,
            stepping_block,
            vehicle_type,
        }
    }
}

impl std::fmt::Debug for EntityState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityState")
            .field("distance", &self.distance)
            .field("is_baby", &self.is_baby)
            .field("effects", &self.effects.len())
            .field("equipment", &self.equipment)
            .field("stepping_block", &self.stepping_block)
            .field("vehicle_type", &self.vehicle_type)
            .finish()
    }
}

/// Strongly-typed entity condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct EntityPredicate {
    pub entity_type: Option<TagOrIdMatcher>,
    pub distance_horizontal: Option<FloatBounds>,
    pub distance_absolute: Option<FloatBounds>,
    pub is_baby: Option<bool>,
    pub vehicle_type: Option<TagOrIdMatcher>,
    pub passenger_type: Option<TagOrIdMatcher>,
    pub stepping_on: Option<TagOrIdMatcher>,
    pub looking_at: Option<TagOrIdMatcher>,
    pub feet_item: Option<TagOrIdMatcher>,
    pub safe_armor_disallowed: Option<bool>,
    /// Vanilla `EntityEffectsPredicate`.
    pub effects: Option<MobEffectsPredicate>,
    /// Vanilla `EntityEquipmentPredicate`, one entry per checked slot.
    pub equipment: Vec<(String, ItemPredicate)>,
}

/// True when a sub-tree of loot conditions requires an item from the `piglin_safe_armor` tag.
fn subtree_requires_safe_armor(cond: &Value) -> bool {
    let Some(obj) = cond.as_object() else {
        return false;
    };
    let kind = obj.get("type").and_then(Value::as_str).unwrap_or("");
    if matches!(
        kind,
        "minecraft:any_of" | "any_of" | "minecraft:all_of" | "all_of"
    ) {
        return obj
            .get("terms")
            .and_then(Value::as_array)
            .is_some_and(|terms| terms.iter().any(subtree_requires_safe_armor));
    }

    let pred = cond.get("predicate").unwrap_or(cond);
    let Some(eq) = pred
        .get("minecraft:equipment")
        .or_else(|| pred.get("equipment"))
    else {
        return false;
    };
    eq.as_object().is_some_and(|slots| {
        slots.values().any(|slot| {
            slot.get("items").is_some_and(|items| match items {
                Value::String(s) => s.contains("piglin_safe_armor"),
                Value::Array(arr) => arr.iter().any(|item| {
                    item.as_str()
                        .is_some_and(|s| s.contains("piglin_safe_armor"))
                }),
                _ => false,
            })
        })
    })
}

/// Detects the `minecraft:inverted` loot condition vanilla uses for `piglin.safe_armor`.
fn safe_armor_predicate_is_inverted(cond: &Value) -> bool {
    let Some(obj) = cond.as_object() else {
        return false;
    };
    let kind = obj.get("type").and_then(Value::as_str).unwrap_or("");
    if matches!(kind, "minecraft:inverted" | "inverted") {
        let term = obj.get("term").or_else(|| obj.get("predicate"));
        return term.is_some_and(subtree_requires_safe_armor);
    }
    if matches!(
        kind,
        "minecraft:any_of" | "any_of" | "minecraft:all_of" | "all_of"
    ) {
        return obj
            .get("terms")
            .and_then(Value::as_array)
            .is_some_and(|terms| terms.iter().any(safe_armor_predicate_is_inverted));
    }
    false
}

impl EntityPredicate {
    #[must_use]
    pub fn matches(
        &self,
        entity_type: &str,
        distance: Option<EntityDistance>,
        is_baby: Option<bool>,
    ) -> bool {
        if let Some(types) = &self.entity_type
            && !types.matches(RegistryKey::EntityType, entity_type)
        {
            return false;
        }
        if let Some(dist_bounds) = &self.distance_horizontal {
            let Some(d) = distance else {
                return false;
            };
            if !dist_bounds.matches(d.horizontal) {
                return false;
            }
        }
        if let Some(dist_bounds) = &self.distance_absolute {
            let Some(d) = distance else {
                return false;
            };
            if !dist_bounds.matches(d.absolute) {
                return false;
            }
        }
        if let Some(expected_baby) = self.is_baby
            && is_baby != Some(expected_baby)
        {
            return false;
        }
        true
    }

    /// Evaluates the entity parts needing a target snapshot:
    /// vanilla's `EntityEffectsPredicate` and `EntityEquipmentPredicate`.
    #[must_use]
    pub fn matches_entity(&self, entity_type: &str, state: &EntityState) -> bool {
        if !self.matches(entity_type, state.distance, state.is_baby) {
            return false;
        }
        if let Some(effects) = &self.effects
            && !effects.matches(&state.effects)
        {
            return false;
        }
        if let Some(stepping) = &self.stepping_on {
            let Some(block) = state.stepping_block.as_deref() else {
                return false;
            };
            if !stepping.matches(RegistryKey::Block, block) {
                return false;
            }
        }
        // Vanilla VehiclePredicate.matches fails when the entity has no vehicle.
        if let Some(vehicle) = &self.vehicle_type {
            let Some(actual) = state.vehicle_type.as_deref() else {
                return false;
            };
            if !vehicle.matches(RegistryKey::EntityType, actual) {
                return false;
            }
        }
        self.equipment.iter().all(|(slot, predicate)| {
            state.equipment.iter().any(|(equipped_slot, item)| {
                equipped_slot == slot && predicate.matches(item, 1, &[])
            })
        })
    }

    #[must_use]
    fn equipment_from_value(val: &Value) -> Vec<(String, ItemPredicate)> {
        val.get("minecraft:equipment")
            .or_else(|| val.get("equipment"))
            .and_then(Value::as_object)
            .map(|slots| {
                slots
                    .iter()
                    .map(|(slot, pred)| (slot.clone(), ItemPredicate::from_value(pred)))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let pred_val = val.get("predicate").unwrap_or(val);
        let Value::Object(map) = pred_val else {
            return Self::default();
        };

        let entity_type = map
            .get("minecraft:entity_type")
            .or_else(|| {
                map.get("type").filter(|t| {
                    t.as_str().is_none_or(|s| {
                        !matches!(
                            s,
                            "minecraft:all_of"
                                | "minecraft:any_of"
                                | "minecraft:entity_properties"
                                | "minecraft:inverted"
                                | "minecraft:location_check"
                                | "minecraft:match_block"
                                | "minecraft:match_tool"
                        )
                    })
                })
            })
            .and_then(TagOrIdMatcher::from_value);

        let (distance_horizontal, distance_absolute) = map
            .get("minecraft:distance")
            .or_else(|| map.get("distance"))
            .map_or((None, None), |dist_val| {
                (
                    dist_val.get("horizontal").and_then(FloatBounds::from_value),
                    dist_val.get("absolute").and_then(FloatBounds::from_value),
                )
            });

        let is_baby = map
            .get("minecraft:flags")
            .or_else(|| map.get("flags"))
            .and_then(|f| f.get("is_baby"))
            .and_then(Value::as_bool);

        let (vehicle_type, passenger_type) = map
            .get("minecraft:vehicle")
            .or_else(|| map.get("vehicle"))
            .map_or((None, None), |v| {
                let vt = v
                    .get("minecraft:entity_type")
                    .or_else(|| v.get("type"))
                    .and_then(TagOrIdMatcher::from_value);
                let pt = v
                    .get("minecraft:passenger")
                    .or_else(|| v.get("passenger"))
                    .and_then(|p| p.get("minecraft:entity_type").or_else(|| p.get("type")))
                    .and_then(TagOrIdMatcher::from_value);
                (vt, pt)
            });

        let stepping_on = map
            .get("minecraft:stepping_on")
            .or_else(|| map.get("stepping_on"))
            .and_then(|s| {
                s.get("block")
                    .and_then(|b| b.get("blocks").or(Some(b)))
                    .or_else(|| s.get("blocks"))
            })
            .and_then(TagOrIdMatcher::from_value);

        let looking_at = map
            .get("minecraft:type_specific/player")
            .and_then(|p| p.get("looking_at"))
            .and_then(|l| l.get("minecraft:entity_type").or_else(|| l.get("type")))
            .and_then(TagOrIdMatcher::from_value);

        let feet_item = map
            .get("minecraft:equipment")
            .or_else(|| map.get("equipment"))
            .and_then(|eq| eq.get("feet"))
            .and_then(|f| f.get("items"))
            .and_then(TagOrIdMatcher::from_value);

        let effects = map
            .get("minecraft:effects")
            .or_else(|| map.get("effects"))
            .map(MobEffectsPredicate::from_value);

        let equipment = Self::equipment_from_value(pred_val);

        // Vanilla expresses "not wearing piglin safe armor" as an inverted loot condition
        // over an equipment predicate; detect that shape instead of scanning the whole JSON.
        let safe_armor_disallowed = (safe_armor_predicate_is_inverted(val)
            || safe_armor_predicate_is_inverted(pred_val))
        .then_some(true);

        Self {
            entity_type,
            distance_horizontal,
            distance_absolute,
            is_baby,
            vehicle_type,
            passenger_type,
            stepping_on,
            looking_at,
            feet_item,
            safe_armor_disallowed,
            effects,
            equipment,
        }
    }
}

/// Vanilla `TagPredicate<DamageType>` used by `DamageSourcePredicate.tags`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageTypeTagPredicate {
    pub tag: String,
    pub expected: bool,
}

impl DamageTypeTagPredicate {
    #[must_use]
    pub fn from_value(val: &Value) -> Option<Self> {
        let tag = val.get("id").and_then(Value::as_str)?;
        let expected = val.get("expected").and_then(Value::as_bool).unwrap_or(true);
        Some(Self {
            tag: tag.to_string(),
            expected,
        })
    }

    #[must_use]
    pub fn matches(&self, damage_type: &DamageType) -> bool {
        let member = self.tag.strip_prefix('#').map_or_else(
            || matches_id(&self.tag, damage_type.message_id),
            |name| damage_type.is_tagged_with(name).unwrap_or(false),
        );
        member == self.expected
    }
}

/// Original pre-mitigation and actually applied damage of a single hit.
#[derive(Clone, Copy, Debug, Default)]
pub struct DamagePair {
    pub original: f64,
    pub actual: f64,
}

/// Entity context used to evaluate the direct and source entity predicates of a hit.
#[derive(Clone, Copy, Debug, Default)]
pub struct DamageEntities<'a> {
    pub direct_entity_type: Option<&'a str>,
    pub source_entity_type: Option<&'a str>,
    pub source_distance: Option<EntityDistance>,
    pub source_is_baby: Option<bool>,
}

/// Strongly-typed damage condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DamagePredicate {
    pub blocked: Option<bool>,
    pub deflector: Option<bool>,
    pub is_mace_smash: Option<bool>,
    pub direct_entity: Option<String>,
    pub dealt: Option<FloatBounds>,
    pub taken: Option<FloatBounds>,
    pub type_tags: Vec<DamageTypeTagPredicate>,
    pub source_entity: Option<EntityPredicate>,
}

impl DamagePredicate {
    #[must_use]
    pub fn matches(&self, is_blocked: bool, is_deflector: bool, damage: Option<f64>) -> bool {
        self.matches_with_context(is_blocked, is_deflector, false, damage, None)
    }

    #[must_use]
    pub fn matches_with_context(
        &self,
        is_blocked: bool,
        is_deflector: bool,
        is_mace_smash: bool,
        damage: Option<f64>,
        direct_entity_type: Option<&str>,
    ) -> bool {
        self.matches_with_damage(
            is_blocked,
            is_deflector,
            is_mace_smash,
            damage,
            damage,
            direct_entity_type,
        )
    }

    /// Vanilla `DamagePredicate.matches` checks `dealtDamage` against the
    /// original pre-mitigation damage and `takenDamage` against the amount
    /// actually applied, so the two bounds see distinct values.
    fn matches_with_damage(
        &self,
        is_blocked: bool,
        is_deflector: bool,
        is_mace_smash: bool,
        original_damage: Option<f64>,
        actual_damage: Option<f64>,
        direct_entity_type: Option<&str>,
    ) -> bool {
        if let Some(blocked) = self.blocked
            && blocked != is_blocked
        {
            return false;
        }
        if let Some(deflector) = self.deflector
            && deflector != is_deflector
        {
            return false;
        }
        if let Some(expected_mace) = self.is_mace_smash
            && expected_mace != is_mace_smash
        {
            return false;
        }
        if let Some(expected_direct) = &self.direct_entity {
            let Some(actual_direct) = direct_entity_type else {
                return false;
            };
            if !matches_tag_or_id(RegistryKey::EntityType, expected_direct, actual_direct) {
                return false;
            }
        }
        if let Some(dealt) = &self.dealt {
            let Some(d) = original_damage else {
                return false;
            };
            if !dealt.matches(d) {
                return false;
            }
        }
        if let Some(taken) = &self.taken {
            let Some(d) = actual_damage else {
                return false;
            };
            if !taken.matches(d) {
                return false;
            }
        }
        true
    }

    /// Evaluates the full vanilla damage predicate, including
    /// `DamageSourcePredicate.tags` and the source entity predicate, using the
    /// actual damage type supplied by the caller.
    #[must_use]
    pub fn matches_with_damage_type(
        &self,
        is_blocked: bool,
        is_deflector: bool,
        is_mace_smash: bool,
        damage: DamagePair,
        entities: &DamageEntities<'_>,
        damage_type: &DamageType,
    ) -> bool {
        if !self.matches_with_damage(
            is_blocked,
            is_deflector,
            is_mace_smash,
            Some(damage.original),
            Some(damage.actual),
            entities.direct_entity_type,
        ) {
            return false;
        }
        if self.type_tags.iter().any(|tag| !tag.matches(damage_type)) {
            return false;
        }
        if let Some(source_predicate) = &self.source_entity {
            let Some(entity_type) = entities.source_entity_type else {
                return false;
            };
            if !source_predicate.matches(
                entity_type,
                entities.source_distance,
                entities.source_is_baby,
            ) {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };

        let damage_obj = map.get("damage").unwrap_or(val);
        let blocked = damage_obj.get("blocked").and_then(Value::as_bool);
        let type_tags = damage_obj
            .get("type")
            .and_then(|t| t.get("tags"))
            .and_then(Value::as_array);
        let deflector = damage_obj
            .get("type")
            .and_then(|t| t.get("is_projectile"))
            .and_then(Value::as_bool)
            .or_else(|| {
                type_tags.and_then(|tags| {
                    tags.iter()
                        .find(|entry| {
                            entry
                                .get("id")
                                .and_then(Value::as_str)
                                .is_some_and(|id| id.contains("projectile"))
                        })
                        .map(|entry| {
                            entry
                                .get("expected")
                                .and_then(Value::as_bool)
                                .unwrap_or(true)
                        })
                })
            });
        let is_mace_smash = type_tags.and_then(|tags| {
            tags.iter()
                .find(|entry| {
                    entry.get("id").and_then(Value::as_str) == Some("#minecraft:mace_smash")
                })
                .map(|entry| {
                    entry
                        .get("expected")
                        .and_then(Value::as_bool)
                        .unwrap_or(true)
                })
        });
        let direct_entity = damage_obj
            .get("type")
            .and_then(|t| t.get("direct_entity"))
            .map(|de| de.get("predicate").unwrap_or(de))
            .and_then(|de| de.get("minecraft:entity_type").or_else(|| de.get("type")))
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let type_tags = type_tags
            .map(|tags| {
                tags.iter()
                    .filter_map(DamageTypeTagPredicate::from_value)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let source_entity = damage_obj
            .get("source_entity")
            .or_else(|| damage_obj.get("type").and_then(|t| t.get("source_entity")))
            .map(EntityPredicate::from_value);
        let dealt = damage_obj.get("dealt").and_then(FloatBounds::from_value);
        let taken = damage_obj.get("taken").and_then(FloatBounds::from_value);

        Self {
            blocked,
            deflector,
            is_mace_smash,
            direct_entity,
            dealt,
            taken,
            type_tags,
            source_entity,
        }
    }
}

/// Strongly-typed dimension change condition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DimensionPredicate {
    pub from: Option<String>,
    pub to: Option<String>,
}

impl DimensionPredicate {
    #[must_use]
    pub fn matches(&self, from: Option<&str>, to: &str) -> bool {
        if let Some(expected_from) = &self.from {
            let Some(f) = from else {
                return false;
            };
            if !matches_id(expected_from, f) {
                return false;
            }
        }
        if let Some(expected_to) = &self.to
            && !matches_id(expected_to, to)
        {
            return false;
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let from = map
            .get("from")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let to = map
            .get("to")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        Self { from, to }
    }
}

/// Strongly-typed placed block condition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlacedBlockPredicate {
    pub block: Option<TagOrIdMatcher>,
    pub item: Option<ItemPredicate>,
}

impl PlacedBlockPredicate {
    #[must_use]
    pub fn matches(
        &self,
        block_id: &str,
        item_id: Option<&str>,
        item_count: u32,
        enchantments: &[(&str, u32)],
    ) -> bool {
        if let Some(b) = &self.block
            && !b.matches(RegistryKey::Block, block_id)
        {
            return false;
        }
        if let Some(item_pred) = &self.item {
            let Some(actual_item) = item_id else {
                return false;
            };
            if !item_pred.matches(actual_item, item_count, enchantments) {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let block = map
            .get("block")
            .or_else(|| map.get("blocks"))
            .and_then(TagOrIdMatcher::from_value);
        let item = map.get("item").map(ItemPredicate::from_value);
        Self { block, item }
    }
}

/// Strongly-typed target hit condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TargetHitPredicate {
    pub signal_strength: Option<IntBounds>,
    pub distance: Option<FloatBounds>,
}

impl TargetHitPredicate {
    #[must_use]
    pub fn matches(&self, signal: i32, distance: Option<f64>) -> bool {
        if let Some(sig) = &self.signal_strength
            && !sig.matches(i64::from(signal))
        {
            return false;
        }
        if let Some(dist) = &self.distance {
            let Some(d) = distance else {
                return false;
            };
            if !dist.matches(d) {
                return false;
            }
        }
        true
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let signal_strength = map.get("signal_strength").and_then(IntBounds::from_value);
        let mut distance = None;
        if let Some(proj_val) = map.get("projectile") {
            let pred = proj_val.get("predicate").unwrap_or(proj_val);
            if let Some(dist_val) = pred
                .get("minecraft:distance")
                .or_else(|| pred.get("distance"))
            {
                distance = dist_val.get("horizontal").and_then(FloatBounds::from_value);
            }
        }
        Self {
            signal_strength,
            distance,
        }
    }
}

/// Strongly-typed beacon construction condition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BeaconPredicate {
    pub level: Option<IntBounds>,
}

impl BeaconPredicate {
    #[must_use]
    pub fn matches(&self, level: i32) -> bool {
        self.level
            .as_ref()
            .is_none_or(|b| b.matches(i64::from(level)))
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        Self {
            level: map.get("level").and_then(IntBounds::from_value),
        }
    }
}

/// Strongly-typed animal breeding condition.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BredAnimalsPredicate {
    pub parent: Option<EntityPredicate>,
    pub partner: Option<EntityPredicate>,
    pub child: Option<EntityPredicate>,
}

impl BredAnimalsPredicate {
    fn matches_role(predicate: Option<&EntityPredicate>, entity_type: &str) -> bool {
        predicate.is_none_or(|p| p.matches(entity_type, None, None))
    }

    #[must_use]
    pub fn matches(&self, parent_type: &str, partner_type: &str, child_type: Option<&str>) -> bool {
        // Vanilla tests each role against its own entity. A present child predicate
        // requires a child, and parent/partner are symmetric (either ordering passes).
        let child_ok = self.child.as_ref().is_none_or(|c| {
            child_type.is_some_and(|entity_type| c.matches(entity_type, None, None))
        });
        let parents_ok = (Self::matches_role(self.parent.as_ref(), parent_type)
            && Self::matches_role(self.partner.as_ref(), partner_type))
            || (Self::matches_role(self.parent.as_ref(), partner_type)
                && Self::matches_role(self.partner.as_ref(), parent_type));
        child_ok && parents_ok
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let parent = map
            .get("parent")
            .map(|p| EntityPredicate::from_value(p.get("predicate").unwrap_or(p)));
        let partner = map
            .get("partner")
            .map(|p| EntityPredicate::from_value(p.get("predicate").unwrap_or(p)));
        let child = map
            .get("child")
            .map(|p| EntityPredicate::from_value(p.get("predicate").unwrap_or(p)));
        Self {
            parent,
            partner,
            child,
        }
    }
}

/// Strongly-typed recipe unlocked/crafted condition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RecipePredicate {
    pub recipes: Vec<String>,
}

impl RecipePredicate {
    #[must_use]
    pub fn matches(&self, recipe_id: &str) -> bool {
        if self.recipes.is_empty() {
            return true;
        }
        self.recipes.iter().any(|r| matches_id(r, recipe_id))
    }

    #[must_use]
    pub fn from_value(val: &Value) -> Self {
        let Value::Object(map) = val else {
            return Self::default();
        };
        let mut recipes = Vec::new();
        if let Some(r_val) = map
            .get("recipes")
            .or_else(|| map.get("recipe"))
            .or_else(|| map.get("recipe_id"))
        {
            match r_val {
                Value::String(s) => recipes.push(s.clone()),
                Value::Array(arr) => {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            recipes.push(s.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        Self { recipes }
    }
}

/// Generated advancement data stores only a lower y bound for vertical distance triggers.
fn vertical_distance(min: f64) -> DistancePredicate {
    DistancePredicate {
        y: Some(FloatBounds {
            min: Some(min),
            max: None,
        }),
        ..Default::default()
    }
}

/// Generated advancement data stores only a lower bound for horizontal distance triggers.
fn horizontal_distance(min: f64) -> DistancePredicate {
    DistancePredicate {
        horizontal: Some(FloatBounds {
            min: Some(min),
            max: None,
        }),
        ..Default::default()
    }
}

/// Strongly-typed criterion conditions for all advancement trigger types.
#[derive(Clone, Debug, PartialEq)]
pub enum CriterionConditions {
    InventoryChanged(InventoryChangedPredicate),
    Location(LocationPredicate),
    PlayerKilledEntity(EntityPredicate),
    KilledByArrow,
    ChangedDimension(DimensionPredicate),
    SleptInBed,
    ConsumeItem(ItemPredicate),
    PlacedBlock(PlacedBlockPredicate),
    EnterBlock(TagOrIdMatcher),
    SlideDownBlock(TagOrIdMatcher),
    FishingRodHooked(ItemPredicate),
    FilledBucket(ItemPredicate),
    UsingItem {
        item: ItemPredicate,
        looking_at: Option<TagOrIdMatcher>,
    },
    UsedTotem(ItemPredicate),
    EntityHurtPlayer(DamagePredicate),
    EntityKilledPlayer,
    PlayerHurtEntity(DamagePredicate),
    BredAnimals(BredAnimalsPredicate),
    UsedEnderEye,
    ItemUsedOnBlock(ItemUsedOnBlockPredicate),
    TargetHit(TargetHitPredicate),
    CuredZombieVillager,
    VillagerTrade,
    AllayDropItemOnBlock(ItemUsedOnBlockPredicate),
    ThrownItemPickedUpByPlayer(EntityPredicate),
    TameAnimal(EntityPredicate),
    ConstructBeacon(BeaconPredicate),
    RecipeCrafted(RecipePredicate),
    RecipeUnlocked(RecipePredicate),
    PlayerGeneratesContainerLoot(String),
    EnchantedItem,
    BrewedPotion,
    ItemDurabilityChanged {
        item: ItemPredicate,
        vehicle_type: Option<TagOrIdMatcher>,
    },
    Levitation {
        distance: Option<DistancePredicate>,
    },
    FallFromHeight {
        distance: Option<DistancePredicate>,
    },
    FallAfterExplosion {
        distance: Option<DistancePredicate>,
        cause_entity: Option<EntityPredicate>,
    },
    NetherTravel {
        distance: Option<DistancePredicate>,
    },
    RideEntityInLava {
        distance: Option<DistancePredicate>,
        dimension: Option<String>,
        vehicle_type: Option<TagOrIdMatcher>,
    },
    ShotCrossbow(ItemPredicate),
    AvoidVibration,
    BeeNestDestroyed {
        block: Option<String>,
        num_bees_inside: Option<i32>,
    },
    ChanneledLightning {
        victim_type: Option<String>,
    },
    LightningStrike {
        bystander: Option<EntityPredicate>,
        blocks_set_on_fire: Option<IntBounds>,
        entity_struck: Option<EntityPredicate>,
        distance_max: Option<f64>,
    },
    KillMobNearSculkCatalyst,
    HeroOfTheVillage,
    PlayerInteractedWithEntity {
        entity: Option<EntityPredicate>,
        item: Option<ItemPredicate>,
        player: Option<EntityPredicate>,
    },
    ThrownItemPickedUpByEntity {
        entity: Option<EntityPredicate>,
        item: Option<ItemPredicate>,
        player: Option<EntityPredicate>,
    },
    PlayerShearedEquipment {
        entity: Option<EntityPredicate>,
        item: Option<ItemPredicate>,
    },
    StartedRiding(Option<EntityPredicate>),
    SummonedEntity(EntityPredicate),
    CrafterRecipeCrafted(String),
    EffectsChanged {
        player: Option<EntityPredicate>,
        effects: Option<MobEffectsPredicate>,
        source: Option<EntityPredicate>,
    },
    SpearMobs {
        count: Option<i32>,
    },
    Impossible,
    Tick,
    Custom {
        trigger: String,
        conditions: Value,
    },
}

#[expect(clippy::too_many_lines, reason = "converting from data enum")]
impl From<pumpkin_data::advancement_data::AdvancementCondition> for CriterionConditions {
    fn from(cond: pumpkin_data::advancement_data::AdvancementCondition) -> Self {
        use pumpkin_data::advancement_data::AdvancementCondition as AC;
        match cond {
            AC::InventoryChanged(ic) => {
                let items =
                    ic.items
                        .iter()
                        .map(|item| ItemPredicate {
                            items: if item.items.is_empty() && item.tag.is_none() {
                                None
                            } else {
                                let mut patterns = Vec::new();
                                if let Some(t) = item.tag {
                                    patterns.push(format!("#{t}"));
                                }
                                for i in item.items {
                                    patterns.push((*i).to_string());
                                }
                                Some(TagOrIdMatcher { patterns })
                            },
                            count: (item.count_min.is_some() || item.count_max.is_some())
                                .then_some(IntBounds {
                                    min: item.count_min,
                                    max: item.count_max,
                                }),
                            enchantments: item
                                .enchantments
                                .iter()
                                .map(|e| EnchantmentPredicate {
                                    enchantment: e.enchantment.to_string(),
                                    levels: IntBounds {
                                        min: Some(e.min_level),
                                        max: None,
                                    },
                                })
                                .collect(),
                        })
                        .collect();
                let slots = ic.slots.map(|s| InventorySlotsPredicate {
                    empty: (s.empty_min.is_some() || s.empty_max.is_some()).then_some(IntBounds {
                        min: s.empty_min,
                        max: s.empty_max,
                    }),
                    full: (s.full_min.is_some() || s.full_max.is_some()).then_some(IntBounds {
                        min: s.full_min,
                        max: s.full_max,
                    }),
                    occupied: (s.occupied_min.is_some() || s.occupied_max.is_some()).then_some(
                        IntBounds {
                            min: s.occupied_min,
                            max: s.occupied_max,
                        },
                    ),
                });
                Self::InventoryChanged(InventoryChangedPredicate { items, slots })
            }
            AC::RecipeUnlocked(r) => Self::RecipeUnlocked(RecipePredicate {
                recipes: vec![r.to_string()],
            }),
            AC::RecipeCrafted(r) => Self::RecipeCrafted(RecipePredicate {
                recipes: vec![r.to_string()],
            }),
            AC::PlayerGeneratesContainerLoot(lt) => {
                Self::PlayerGeneratesContainerLoot(lt.to_string())
            }
            AC::ChangedDimension { from, to } => Self::ChangedDimension(DimensionPredicate {
                from: from.map(ToString::to_string),
                to: to.map(ToString::to_string),
            }),
            AC::ConsumeItem(item) => Self::ConsumeItem(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: item.items.iter().map(ToString::to_string).collect(),
                }),
                count: None,
                enchantments: Vec::new(),
            }),
            AC::PlacedBlock { block, item } => Self::PlacedBlock(PlacedBlockPredicate {
                block: block.map(|b| TagOrIdMatcher {
                    patterns: vec![b.to_string()],
                }),
                item: item.map(|i| ItemPredicate {
                    items: Some(TagOrIdMatcher {
                        patterns: vec![i.to_string()],
                    }),
                    count: None,
                    enchantments: Vec::new(),
                }),
            }),
            AC::EnterBlock(b) => Self::EnterBlock(TagOrIdMatcher {
                patterns: vec![b.to_string()],
            }),
            AC::SlideDownBlock(b) => Self::SlideDownBlock(TagOrIdMatcher {
                patterns: vec![b.to_string()],
            }),
            AC::PlayerKilledEntity(e) => Self::PlayerKilledEntity(from_entity_condition(&e)),
            AC::BredAnimals(b) => Self::BredAnimals(BredAnimalsPredicate {
                parent: b.parent.as_ref().map(from_entity_condition),
                partner: b.partner.as_ref().map(from_entity_condition),
                child: b.child.as_ref().map(from_entity_condition),
            }),
            AC::TameAnimal(e) => Self::TameAnimal(from_entity_condition(&e)),
            AC::FilledBucket(item) => Self::FilledBucket(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: vec![item.to_string()],
                }),
                count: None,
                enchantments: Vec::new(),
            }),
            AC::FishingRodHooked(item) => Self::FishingRodHooked(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: item.items.iter().map(ToString::to_string).collect(),
                }),
                count: None,
                enchantments: Vec::new(),
            }),
            AC::UsingItem { item, looking_at } => Self::UsingItem {
                item: ItemPredicate {
                    items: (!item.items.is_empty() || item.tag.is_some()).then_some(
                        TagOrIdMatcher {
                            patterns: item
                                .items
                                .iter()
                                .map(ToString::to_string)
                                .chain(item.tag.map(|t| {
                                    if t.starts_with('#') {
                                        t.to_string()
                                    } else {
                                        format!("#{t}")
                                    }
                                }))
                                .collect(),
                        },
                    ),
                    count: None,
                    enchantments: Vec::new(),
                },
                looking_at: looking_at.map(|l| TagOrIdMatcher {
                    patterns: vec![l.to_string()],
                }),
            },
            AC::UsedTotem(item) => Self::UsedTotem(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: item.items.iter().map(ToString::to_string).collect(),
                }),
                count: None,
                enchantments: Vec::new(),
            }),
            AC::EntityHurtPlayer(d) => Self::EntityHurtPlayer(DamagePredicate {
                blocked: d.blocked,
                deflector: d.is_projectile,
                is_mace_smash: d.is_mace_smash,
                direct_entity: d.direct_entity.map(ToString::to_string),
                dealt: d.dealt_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
                taken: d.taken_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
                type_tags: Vec::new(),
                source_entity: None,
            }),
            AC::PlayerHurtEntity(d) => Self::PlayerHurtEntity(DamagePredicate {
                blocked: d.blocked,
                deflector: d.is_projectile,
                is_mace_smash: d.is_mace_smash,
                direct_entity: d.direct_entity.map(ToString::to_string),
                dealt: d.dealt_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
                taken: d.taken_min.map(|min| FloatBounds {
                    min: Some(min),
                    max: None,
                }),
                type_tags: Vec::new(),
                source_entity: None,
            }),
            AC::TargetHit(th) => Self::TargetHit(TargetHitPredicate {
                signal_strength: th.signal_strength.map(|sig| IntBounds {
                    min: Some(i64::from(sig)),
                    max: None,
                }),
                distance: th.distance_min.map(|dist| FloatBounds {
                    min: Some(dist),
                    max: None,
                }),
            }),
            AC::ConstructBeacon(b) => Self::ConstructBeacon(BeaconPredicate {
                level: b.level_min.map(|lvl| IntBounds {
                    min: Some(i64::from(lvl)),
                    max: None,
                }),
            }),
            AC::AllayDropItemOnBlock(c) => {
                Self::AllayDropItemOnBlock(from_item_used_on_block_condition(c))
            }
            AC::ThrownItemPickedUpByPlayer(e) => {
                Self::ThrownItemPickedUpByPlayer(from_entity_condition(&e))
            }
            AC::ItemUsedOnBlock(c) => Self::ItemUsedOnBlock(from_item_used_on_block_condition(c)),
            AC::Location(l) => Self::Location(LocationPredicate {
                x: None,
                y: None,
                z: None,
                biome: l.biome.map(|b| TagOrIdMatcher {
                    patterns: vec![b.to_string()],
                }),
                dimension: l.dimension.map(ToString::to_string),
                block: l.block.map(|b| TagOrIdMatcher {
                    patterns: vec![b.to_string()],
                }),
                can_see_sky: l.can_see_sky,
                smokey: l.smokey,
                structure: l.structure.map(|s| TagOrIdMatcher {
                    patterns: vec![s.to_string()],
                }),
                stepping_on: l.stepping_on.map(|b| TagOrIdMatcher {
                    patterns: vec![b.to_string()],
                }),
                feet_item: l.feet_item.map(|f| TagOrIdMatcher {
                    patterns: vec![f.to_string()],
                }),
                light: None,
                fluid: None,
            }),
            AC::KilledByArrow => Self::KilledByArrow,
            AC::SleptInBed => Self::SleptInBed,
            AC::EntityKilledPlayer => Self::EntityKilledPlayer,
            AC::UsedEnderEye => Self::UsedEnderEye,
            AC::CuredZombieVillager => Self::CuredZombieVillager,
            AC::VillagerTrade => Self::VillagerTrade,
            AC::EnchantedItem => Self::EnchantedItem,
            AC::BrewedPotion => Self::BrewedPotion,
            AC::ItemDurabilityChanged { item, vehicle_type } => Self::ItemDurabilityChanged {
                item: ItemPredicate {
                    items: (!item.items.is_empty() || item.tag.is_some()).then_some(
                        TagOrIdMatcher {
                            patterns: item
                                .items
                                .iter()
                                .map(ToString::to_string)
                                .chain(item.tag.map(|t| {
                                    if t.starts_with('#') {
                                        t.to_string()
                                    } else {
                                        format!("#{t}")
                                    }
                                }))
                                .collect(),
                        },
                    ),
                    count: None,
                    enchantments: Vec::new(),
                },
                vehicle_type: vehicle_type.map(|v| TagOrIdMatcher {
                    patterns: vec![v.to_string()],
                }),
            },
            AC::Levitation { distance_y_min } => Self::Levitation {
                distance: distance_y_min.map(vertical_distance),
            },
            AC::FallFromHeight { distance_y_min } => Self::FallFromHeight {
                distance: distance_y_min.map(vertical_distance),
            },
            AC::FallAfterExplosion {
                distance_y_min,
                cause_entity,
            } => Self::FallAfterExplosion {
                distance: distance_y_min.map(vertical_distance),
                cause_entity: cause_entity.map(|c| EntityPredicate {
                    entity_type: Some(TagOrIdMatcher {
                        patterns: vec![c.to_string()],
                    }),
                    ..Default::default()
                }),
            },
            AC::NetherTravel {
                distance_horizontal_min,
            } => Self::NetherTravel {
                distance: distance_horizontal_min.map(horizontal_distance),
            },
            AC::RideEntityInLava {
                distance_horizontal_min,
                dimension,
                vehicle_type,
            } => Self::RideEntityInLava {
                distance: distance_horizontal_min.map(horizontal_distance),
                dimension: dimension.map(ToString::to_string),
                vehicle_type: vehicle_type.map(|v| TagOrIdMatcher {
                    patterns: vec![v.to_string()],
                }),
            },
            AC::ShotCrossbow(item) => Self::ShotCrossbow(ItemPredicate {
                items: Some(TagOrIdMatcher {
                    patterns: item.items.iter().map(ToString::to_string).collect(),
                }),
                count: None,
                enchantments: Vec::new(),
            }),
            AC::AvoidVibration => Self::AvoidVibration,
            AC::BeeNestDestroyed {
                block,
                num_bees_inside,
            } => Self::BeeNestDestroyed {
                block: block.map(ToString::to_string),
                num_bees_inside,
            },
            AC::ChanneledLightning { victim_type } => Self::ChanneledLightning {
                victim_type: victim_type.map(ToString::to_string),
            },
            AC::LightningStrike {
                bystander_type,
                blocks_set_on_fire,
                distance_max,
            } => Self::LightningStrike {
                bystander: bystander_type.map(|entity_type| EntityPredicate {
                    entity_type: Some(TagOrIdMatcher {
                        patterns: vec![entity_type.to_string()],
                    }),
                    ..Default::default()
                }),
                blocks_set_on_fire: blocks_set_on_fire.map(|v| IntBounds {
                    min: Some(i64::from(v)),
                    max: Some(i64::from(v)),
                }),
                entity_struck: None,
                distance_max,
            },
            AC::KillMobNearSculkCatalyst => Self::KillMobNearSculkCatalyst,
            AC::HeroOfTheVillage => Self::HeroOfTheVillage,
            AC::PlayerInteractedWithEntity {
                entity,
                item,
                player,
            } => Self::PlayerInteractedWithEntity {
                entity: entity.as_ref().map(from_entity_condition),
                item: item.as_ref().map(from_item_condition),
                player: player.as_ref().map(from_entity_condition),
            },
            AC::ThrownItemPickedUpByEntity {
                entity,
                item,
                player,
            } => Self::ThrownItemPickedUpByEntity {
                entity: entity.as_ref().map(from_entity_condition),
                item: item.as_ref().map(from_item_condition),
                player: player.as_ref().map(from_entity_condition),
            },
            AC::PlayerShearedEquipment { entity, item } => Self::PlayerShearedEquipment {
                entity: entity.as_ref().map(from_entity_condition),
                item: item.as_ref().map(from_item_condition),
            },
            AC::StartedRiding(p) => Self::StartedRiding(p.as_ref().map(from_entity_condition)),
            AC::SummonedEntity(e) => Self::SummonedEntity(from_entity_condition(&e)),
            AC::CrafterRecipeCrafted(r) => Self::CrafterRecipeCrafted(r.to_string()),
            AC::EffectsChanged {
                player,
                effects,
                source,
            } => Self::EffectsChanged {
                player: player.as_ref().map(from_entity_condition),
                effects: effects.map(|effects| MobEffectsPredicate {
                    effect_map: effects
                        .iter()
                        .map(|e| MobEffectInstancePredicate {
                            effect: e.effect.to_string(),
                            amplifier: (e.amplifier_min.is_some() || e.amplifier_max.is_some())
                                .then_some(IntBounds {
                                    min: e.amplifier_min,
                                    max: e.amplifier_max,
                                }),
                            duration: (e.duration_min.is_some() || e.duration_max.is_some())
                                .then_some(IntBounds {
                                    min: e.duration_min,
                                    max: e.duration_max,
                                }),
                            ambient: e.ambient,
                            visible: e.visible,
                        })
                        .collect(),
                }),
                source: source.as_ref().map(from_entity_condition),
            },
            AC::SpearMobs { count } => Self::SpearMobs { count },
            AC::Impossible => Self::Impossible,
            AC::Empty | AC::Tick => Self::Tick,
            AC::Custom(raw) => Self::parse("", raw),
        }
    }
}

fn from_entity_condition(e: &pumpkin_data::advancement_data::EntityCondition) -> EntityPredicate {
    let mut patterns = Vec::new();
    if let Some(t) = e.entity_type {
        patterns.push(t.to_string());
    }
    if let Some(t) = e.tag {
        let tag_str = if t.starts_with('#') {
            t.to_string()
        } else {
            format!("#{t}")
        };
        patterns.push(tag_str);
    }
    let mut vpatterns = Vec::new();
    if let Some(vt) = e.vehicle_type {
        vpatterns.push(vt.to_string());
    }
    if let Some(vt) = e.vehicle_tag {
        let tag_str = if vt.starts_with('#') {
            vt.to_string()
        } else {
            format!("#{vt}")
        };
        vpatterns.push(tag_str);
    }
    let passenger_type = e.passenger_type.map(|pt| TagOrIdMatcher {
        patterns: vec![pt.to_string()],
    });
    let looking_at = e.looking_at.map(|l| TagOrIdMatcher {
        patterns: vec![l.to_string()],
    });
    let feet_item = e.feet_item.map(|f| TagOrIdMatcher {
        patterns: vec![f.to_string()],
    });

    EntityPredicate {
        entity_type: (!patterns.is_empty()).then_some(TagOrIdMatcher { patterns }),
        distance_horizontal: e.distance_horizontal_min.map(|min| FloatBounds {
            min: Some(min),
            max: None,
        }),
        distance_absolute: e.distance_absolute_min.map(|min| FloatBounds {
            min: Some(min),
            max: None,
        }),
        is_baby: e.is_baby,
        vehicle_type: (!vpatterns.is_empty()).then_some(TagOrIdMatcher {
            patterns: vpatterns,
        }),
        passenger_type,
        stepping_on: e.stepping_on.map(|b| TagOrIdMatcher {
            patterns: vec![b.to_string()],
        }),
        looking_at,
        feet_item,
        safe_armor_disallowed: e.safe_armor_disallowed,
        effects: None,
        equipment: Vec::new(),
    }
}

fn from_item_used_on_block_condition(
    c: &pumpkin_data::advancement_data::ItemUsedOnBlockCondition,
) -> ItemUsedOnBlockPredicate {
    let mut block_patterns: Vec<String> = c.blocks.iter().map(ToString::to_string).collect();
    if let Some(tag) = c.block_tag {
        let tag_str = if tag.starts_with('#') {
            tag.to_string()
        } else {
            format!("#{tag}")
        };
        block_patterns.push(tag_str);
    }
    let mut item_patterns: Vec<String> = c.items.iter().map(ToString::to_string).collect();
    if let Some(tag) = c.item_tag {
        let tag_str = if tag.starts_with('#') {
            tag.to_string()
        } else {
            format!("#{tag}")
        };
        item_patterns.push(tag_str);
    }
    ItemUsedOnBlockPredicate {
        terms: vec![ItemUsedOnBlockTerm::Check(ItemUsedOnBlockCheck {
            blocks: (!block_patterns.is_empty()).then_some(TagOrIdMatcher {
                patterns: block_patterns,
            }),
            items: (!item_patterns.is_empty()).then_some(TagOrIdMatcher {
                patterns: item_patterns,
            }),
            state_key: c.state_key.map(ToString::to_string),
            state_value: c.state_value.map(ToString::to_string),
            smokey: c.smokey,
            biome: c.biome.map(ToString::to_string),
        })],
    }
}

fn from_item_condition(i: &pumpkin_data::advancement_data::ItemCondition) -> ItemPredicate {
    let mut patterns: Vec<String> = i.items.iter().map(ToString::to_string).collect();
    if let Some(t) = i.tag {
        let tag_str = if t.starts_with('#') {
            t.to_string()
        } else {
            format!("#{t}")
        };
        patterns.push(tag_str);
    }
    ItemPredicate {
        items: if patterns.is_empty() {
            None
        } else {
            Some(TagOrIdMatcher { patterns })
        },
        count: (i.count_min.is_some() || i.count_max.is_some()).then_some(IntBounds {
            min: i.count_min,
            max: i.count_max,
        }),
        enchantments: Vec::new(),
    }
}

fn parse_entity_pred(val: &Value) -> EntityPredicate {
    val.get("predicate").map_or_else(
        || {
            val.get("entity").map_or_else(
                || EntityPredicate::from_value(val),
                |p| {
                    if p.is_object() {
                        EntityPredicate::from_value(p.get("predicate").unwrap_or(p))
                    } else {
                        EntityPredicate::from_value(val)
                    }
                },
            )
        },
        EntityPredicate::from_value,
    )
}

/// Collects the id strings of an item predicate, either bare or wrapped as `{"items": ...}`.
fn push_item_patterns(i_val: &Value, out: &mut Vec<String>) {
    let i_val = i_val.get("items").unwrap_or(i_val);
    if let Some(s) = i_val.as_str() {
        out.push(s.to_string());
    } else if let Some(arr) = i_val.as_array() {
        out.extend(
            arr.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string),
        );
    }
}

fn parse_item_pred(val: &Value) -> ItemPredicate {
    val.get("item")
        .map_or_else(|| ItemPredicate::from_value(val), ItemPredicate::from_value)
}

impl CriterionConditions {
    #[must_use]
    #[expect(clippy::too_many_lines)]
    pub fn parse(trigger: &str, json_str: &str) -> Self {
        let val: Value = serde_json::from_str(json_str).unwrap_or(Value::Null);
        match trigger {
            "minecraft:inventory_changed" => {
                Self::InventoryChanged(InventoryChangedPredicate::from_value(&val))
            }
            "minecraft:location" => Self::Location(LocationPredicate::from_value(&val)),
            "minecraft:player_killed_entity" => Self::PlayerKilledEntity(parse_entity_pred(&val)),
            "minecraft:killed_by_arrow" => Self::KilledByArrow,
            "minecraft:changed_dimension" => {
                Self::ChangedDimension(DimensionPredicate::from_value(&val))
            }
            "minecraft:slept_in_bed" => Self::SleptInBed,
            "minecraft:consume_item" => Self::ConsumeItem(parse_item_pred(&val)),
            "minecraft:placed_block" => Self::PlacedBlock(PlacedBlockPredicate::from_value(&val)),
            "minecraft:enter_block" => {
                let block = val
                    .get("blocks")
                    .or_else(|| val.get("block"))
                    .and_then(TagOrIdMatcher::from_value)
                    .unwrap_or_default();
                Self::EnterBlock(block)
            }
            "minecraft:slide_down_block" => {
                let block = val
                    .get("blocks")
                    .or_else(|| val.get("block"))
                    .and_then(TagOrIdMatcher::from_value)
                    .unwrap_or_default();
                Self::SlideDownBlock(block)
            }
            "minecraft:fishing_rod_hooked" => Self::FishingRodHooked(parse_item_pred(&val)),
            "minecraft:filled_bucket" => Self::FilledBucket(parse_item_pred(&val)),
            "minecraft:using_item" => {
                let looking_at = val
                    .get("player")
                    .and_then(|p| p.get("predicate"))
                    .and_then(|pred| pred.get("minecraft:type_specific/player"))
                    .and_then(|p| p.get("looking_at"))
                    .and_then(|l| l.get("minecraft:entity_type").or_else(|| l.get("type")))
                    .and_then(TagOrIdMatcher::from_value);
                Self::UsingItem {
                    item: parse_item_pred(&val),
                    looking_at,
                }
            }
            "minecraft:used_totem" => Self::UsedTotem(parse_item_pred(&val)),
            "minecraft:entity_hurt_player" => {
                Self::EntityHurtPlayer(DamagePredicate::from_value(&val))
            }
            "minecraft:entity_killed_player" => Self::EntityKilledPlayer,
            "minecraft:player_hurt_entity" => {
                Self::PlayerHurtEntity(DamagePredicate::from_value(&val))
            }
            "minecraft:bred_animals" => Self::BredAnimals(BredAnimalsPredicate::from_value(&val)),
            "minecraft:used_ender_eye" => Self::UsedEnderEye,
            "minecraft:item_used_on_block" => {
                Self::ItemUsedOnBlock(ItemUsedOnBlockPredicate::from_value(&val))
            }
            "minecraft:target_hit" => Self::TargetHit(TargetHitPredicate::from_value(&val)),
            "minecraft:cured_zombie_villager" => Self::CuredZombieVillager,
            "minecraft:villager_trade" => Self::VillagerTrade,
            "minecraft:allay_drop_item_on_block" => {
                Self::AllayDropItemOnBlock(ItemUsedOnBlockPredicate::from_value(&val))
            }
            "minecraft:thrown_item_picked_up_by_player" => {
                Self::ThrownItemPickedUpByPlayer(parse_entity_pred(&val))
            }
            "minecraft:tame_animal" => Self::TameAnimal(parse_entity_pred(&val)),
            "minecraft:construct_beacon" => {
                Self::ConstructBeacon(BeaconPredicate::from_value(&val))
            }
            "minecraft:recipe_crafted" => Self::RecipeCrafted(RecipePredicate::from_value(&val)),
            "minecraft:recipe_unlocked" => Self::RecipeUnlocked(RecipePredicate::from_value(&val)),
            "minecraft:player_generates_container_loot" => Self::PlayerGeneratesContainerLoot(
                val.get("loot_table")
                    .or_else(|| val.get("loot_tables"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            "minecraft:enchanted_item" => Self::EnchantedItem,
            "minecraft:brewed_potion" => Self::BrewedPotion,
            "minecraft:item_durability_changed" => {
                let vehicle_type = val
                    .get("player")
                    .and_then(|p| p.get("predicate"))
                    .and_then(|pred| {
                        pred.get("minecraft:vehicle")
                            .or_else(|| pred.get("vehicle"))
                    })
                    .and_then(|v| v.get("minecraft:entity_type").or_else(|| v.get("type")))
                    .and_then(TagOrIdMatcher::from_value);
                Self::ItemDurabilityChanged {
                    item: parse_item_pred(&val),
                    vehicle_type,
                }
            }
            "minecraft:levitation" => Self::Levitation {
                distance: val.get("distance").and_then(DistancePredicate::from_value),
            },
            "minecraft:fall_from_height" => Self::FallFromHeight {
                distance: val.get("distance").and_then(DistancePredicate::from_value),
            },
            "minecraft:fall_after_explosion" => Self::FallAfterExplosion {
                distance: val.get("distance").and_then(DistancePredicate::from_value),
                cause_entity: val.get("cause").map(parse_entity_pred),
            },
            "minecraft:nether_travel" => Self::NetherTravel {
                distance: val.get("distance").and_then(DistancePredicate::from_value),
            },
            "minecraft:ride_entity_in_lava" => {
                let distance = val.get("distance").and_then(DistancePredicate::from_value);
                let dimension = val
                    .get("player")
                    .and_then(|p| p.get("predicate"))
                    .and_then(|pred| {
                        pred.get("minecraft:location")
                            .or_else(|| pred.get("location"))
                    })
                    .and_then(|l| l.get("dimension"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let vehicle_type = val
                    .get("player")
                    .and_then(|p| p.get("predicate"))
                    .and_then(|pred| {
                        pred.get("minecraft:vehicle")
                            .or_else(|| pred.get("vehicle"))
                    })
                    .and_then(|v| v.get("minecraft:entity_type").or_else(|| v.get("type")))
                    .and_then(TagOrIdMatcher::from_value);
                Self::RideEntityInLava {
                    distance,
                    dimension,
                    vehicle_type,
                }
            }
            "minecraft:shot_crossbow" => Self::ShotCrossbow(parse_item_pred(&val)),
            "minecraft:avoid_vibration" => Self::AvoidVibration,
            "minecraft:bee_nest_destroyed" => Self::BeeNestDestroyed {
                block: val
                    .get("blocks")
                    .or_else(|| val.get("block"))
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
                num_bees_inside: val
                    .get("num_bees_inside")
                    .and_then(Value::as_i64)
                    .map(|v| v as i32),
            },
            "minecraft:channeled_lightning" => Self::ChanneledLightning {
                victim_type: val
                    .get("victims")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|e| e.get("predicate"))
                    .and_then(|p| {
                        p.get("minecraft:entity_type")
                            .or_else(|| p.get("entity_type"))
                    })
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string),
            },
            "minecraft:lightning_strike" => {
                let lightning_pred = val.get("lightning").and_then(|l| l.get("predicate"));
                let type_specific =
                    lightning_pred.and_then(|p| p.get("minecraft:type_specific/lightning"));
                let bystander = val.get("bystander").map(parse_entity_pred);
                let blocks_set_on_fire = type_specific
                    .and_then(|t| t.get("blocks_set_on_fire"))
                    .and_then(IntBounds::from_value);
                let entity_struck = type_specific
                    .and_then(|t| t.get("entity_struck"))
                    .map(parse_entity_pred);
                let distance_max = lightning_pred
                    .and_then(|p| p.get("minecraft:distance"))
                    .and_then(|d| d.get("absolute"))
                    .and_then(|a| a.get("max"))
                    .and_then(Value::as_f64);
                Self::LightningStrike {
                    bystander,
                    blocks_set_on_fire,
                    entity_struck,
                    distance_max,
                }
            }
            "minecraft:kill_mob_near_sculk_catalyst" => Self::KillMobNearSculkCatalyst,
            "minecraft:hero_of_the_village" => Self::HeroOfTheVillage,
            "minecraft:player_interacted_with_entity" => Self::PlayerInteractedWithEntity {
                entity: val.get("entity").map(parse_entity_pred),
                item: val.get("item").map(parse_item_pred),
                player: val.get("player").map(parse_entity_pred),
            },
            "minecraft:thrown_item_picked_up_by_entity" => Self::ThrownItemPickedUpByEntity {
                entity: val.get("entity").map(parse_entity_pred),
                item: val.get("item").map(parse_item_pred),
                player: val.get("player").map(parse_entity_pred),
            },
            "minecraft:player_sheared_equipment" => Self::PlayerShearedEquipment {
                entity: val.get("entity").map(parse_entity_pred),
                item: val.get("item").map(parse_item_pred),
            },
            "minecraft:started_riding" => {
                let player = val.get("player").map(parse_entity_pred);
                Self::StartedRiding(player)
            }
            "minecraft:summoned_entity" => Self::SummonedEntity(parse_entity_pred(&val)),
            "minecraft:crafter_recipe_crafted" => Self::CrafterRecipeCrafted(
                val.get("recipe_id")
                    .or_else(|| val.get("recipes"))
                    .or_else(|| val.get("recipe"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            "minecraft:effects_changed" => Self::EffectsChanged {
                player: val.get("player").map(parse_entity_pred),
                effects: val.get("effects").map(MobEffectsPredicate::from_value),
                source: val.get("source").map(parse_entity_pred),
            },
            "minecraft:spear_mobs" => Self::SpearMobs {
                count: val.get("count").and_then(Value::as_i64).map(|v| v as i32),
            },
            "minecraft:impossible" => Self::Impossible,
            "minecraft:tick" => Self::Tick,
            _ => Self::Custom {
                trigger: trigger.to_string(),
                conditions: val,
            },
        }
    }
}

// Backward-compatible standalone matching functions

/// Matches an individual item predicate.
#[must_use]
pub fn matches_item(
    item_pred: &Value,
    item_key: &str,
    count: u32,
    enchantments: &[(&str, u32)],
) -> bool {
    ItemPredicate::from_value(item_pred).matches(item_key, count, enchantments)
}

/// Evaluates conditions for `minecraft:inventory_changed`.
#[must_use]
pub fn matches_inventory_condition(
    cond: &Value,
    inventory: &[InventoryItemSnapshot<'_>],
    slots_full: usize,
    slots_empty: usize,
    slots_occupied: usize,
) -> bool {
    InventoryChangedPredicate::from_value(cond).matches(
        inventory,
        slots_full,
        slots_empty,
        slots_occupied,
    )
}

/// Evaluates conditions for `minecraft:location`.
#[expect(
    clippy::too_many_arguments,
    reason = "location condition has many properties"
)]
#[must_use]
pub fn matches_location_condition(
    cond: &Value,
    pos: (f64, f64, f64),
    biome: Option<&str>,
    dimension: Option<&str>,
    _features: &[&str],
    can_see_sky: Option<bool>,
    step_block: Option<&str>,
    feet_item: Option<&str>,
    structure: Option<&str>,
    _is_sneaking: bool,
) -> bool {
    LocationPredicate::from_value(cond).matches(
        pos,
        biome,
        dimension,
        step_block,
        can_see_sky,
        None,
        step_block,
        feet_item,
        structure,
    )
}

/// Evaluates conditions for `minecraft:player_killed_entity`.
#[must_use]
pub fn matches_entity_condition(
    cond: &Value,
    entity_key: &str,
    target_type: &str,
    distance: Option<f64>,
    _location: Option<(&str, (f64, f64, f64))>,
    is_baby: Option<bool>,
    _variant: Option<&str>,
) -> bool {
    let pred_val = cond.get(entity_key).unwrap_or(cond);
    let distance = distance.map(|d| EntityDistance {
        horizontal: d,
        absolute: d,
    });
    EntityPredicate::from_value(pred_val).matches(target_type, distance, is_baby)
}

/// Evaluates conditions for `minecraft:entity_hurt_player` and `minecraft:player_hurt_entity`.
#[must_use]
pub fn matches_damage_condition(
    cond: &Value,
    is_blocked: bool,
    is_deflector: bool,
    damage: Option<f64>,
) -> bool {
    DamagePredicate::from_value(cond).matches(is_blocked, is_deflector, damage)
}

/// Evaluates conditions for `minecraft:placed_block` and `minecraft:item_used_on_block`.
#[must_use]
pub fn matches_placed_block_condition(
    cond: &Value,
    block_id: &str,
    item_id: Option<&str>,
    item_count: u32,
    enchantments: &[(&str, u32)],
) -> bool {
    PlacedBlockPredicate::from_value(cond).matches(block_id, item_id, item_count, enchantments)
}

/// Evaluates conditions for `minecraft:changed_dimension`.
#[must_use]
pub fn matches_dimension_condition(cond: &Value, from: Option<&str>, to: &str) -> bool {
    DimensionPredicate::from_value(cond).matches(from, to)
}

/// Evaluates conditions for `minecraft:recipe_unlocked` and `minecraft:recipe_crafted`.
#[must_use]
pub fn matches_recipe_condition(cond: &Value, recipe_id: &str) -> bool {
    RecipePredicate::from_value(cond).matches(recipe_id)
}

/// Evaluates conditions for `minecraft:consume_item`, `minecraft:filled_bucket`, `minecraft:fishing_rod_hooked`, `minecraft:used_totem`.
#[must_use]
pub fn matches_item_condition(cond: &Value, item_key: &str, count: u32) -> bool {
    ItemPredicate::from_value(cond).matches(item_key, count, &[])
}

/// Evaluates conditions for `minecraft:target_hit`.
#[must_use]
pub fn matches_target_hit_condition(cond: &Value, signal: i32, distance: Option<f64>) -> bool {
    TargetHitPredicate::from_value(cond).matches(signal, distance)
}

/// Evaluates conditions for `minecraft:construct_beacon`.
#[must_use]
pub fn matches_beacon_condition(cond: &Value, level: i32) -> bool {
    BeaconPredicate::from_value(cond).matches(level)
}

/// Evaluates conditions for `minecraft:enter_block` and `minecraft:slide_down_block`.
#[must_use]
pub fn matches_enter_block_condition(cond: &Value, block_id: &str) -> bool {
    let block = cond
        .get("blocks")
        .or_else(|| cond.get("block"))
        .unwrap_or(cond);
    TagOrIdMatcher::from_value(block).is_none_or(|m| m.matches(RegistryKey::Block, block_id))
}

/// `EffectsChangedTrigger.TriggerInstance.matches`, plus the `player` check that vanilla runs in
/// `SimpleCriterionTrigger.trigger`. A required `source` never matches when there is no source.
#[must_use]
pub fn matches_effects_changed_condition(
    cond: &CriterionConditions,
    effects: &[Effect],
    source_type: Option<&str>,
    source_distance: Option<EntityDistance>,
) -> bool {
    let CriterionConditions::EffectsChanged {
        player,
        effects: effects_pred,
        source,
    } = cond
    else {
        return false;
    };
    effects_pred.as_ref().is_none_or(|e| e.matches(effects))
        && source.as_ref().is_none_or(|s| {
            source_type.is_some_and(|source_type| s.matches(source_type, source_distance, None))
        })
        && player
            .as_ref()
            .is_none_or(|p| p.matches(EntityType::PLAYER.resource_name, None, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Advancement;
    use pumpkin_data::effect::StatusEffect;
    use serde_json::json;

    #[test]
    fn item_enchantment_levels_uses_full_bounds() {
        let exact = ItemPredicate::from_value(&json!({
            "predicates": {"minecraft:enchantments": [
                {"enchantments": "minecraft:sharpness", "levels": 3}
            ]}
        }));
        assert!(exact.matches("minecraft:diamond_sword", 1, &[("minecraft:sharpness", 3)]));
        assert!(!exact.matches("minecraft:diamond_sword", 1, &[("minecraft:sharpness", 2)]));
        assert!(!exact.matches("minecraft:diamond_sword", 1, &[("minecraft:sharpness", 4)]));

        let ranged = ItemPredicate::from_value(&json!({
            "predicates": {"minecraft:enchantments": [
                {"enchantments": "minecraft:sharpness", "levels": {"max": 2}}
            ]}
        }));
        assert!(ranged.matches("minecraft:diamond_sword", 1, &[("minecraft:sharpness", 2)]));
        assert!(!ranged.matches("minecraft:diamond_sword", 1, &[("minecraft:sharpness", 3)]));
    }

    #[test]
    fn item_enchantment_predicate_reads_components_key() {
        let pred = ItemPredicate::from_value(&json!({
            "items": "minecraft:diamond_sword",
            "components": {"minecraft:enchantments": [
                {"enchantments": "minecraft:sharpness", "levels": {"min": 2}}
            ]}
        }));
        assert!(pred.matches("minecraft:diamond_sword", 1, &[("minecraft:sharpness", 2)]));
        assert!(!pred.matches("minecraft:diamond_sword", 1, &[("minecraft:sharpness", 1)]));
        assert!(!pred.matches("minecraft:iron_sword", 1, &[("minecraft:sharpness", 2)]));
    }

    #[test]
    fn distance_trigger_keeps_upper_bound() {
        let cond = CriterionConditions::parse(
            "minecraft:fall_from_height",
            r#"{"distance": {"y": {"max": 5}}}"#,
        );
        match &cond {
            CriterionConditions::FallFromHeight {
                distance: Some(bounds),
            } => {
                assert!(bounds.y.as_ref().is_some_and(|b| b.matches(3.0)));
                assert!(bounds.y.as_ref().is_none_or(|b| !b.matches(6.0)));
            }
            _ => panic!("expected FallFromHeight"),
        }

        let cond = CriterionConditions::parse("minecraft:levitation", r#"{"distance": {"y": 7}}"#);
        match &cond {
            CriterionConditions::Levitation {
                distance: Some(bounds),
            } => {
                assert!(bounds.y.as_ref().is_some_and(|b| b.matches(7.0)));
                assert!(bounds.y.as_ref().is_none_or(|b| !b.matches(8.0)));
            }
            _ => panic!("expected Levitation"),
        }
    }

    #[test]
    fn distance_trigger_checks_x_z_and_absolute_axes() {
        let cond = CriterionConditions::parse(
            "minecraft:nether_travel",
            r#"{"distance": {"x": {"min": 2}, "z": {"max": 1}, "absolute": {"min": 5}}}"#,
        );
        let CriterionConditions::NetherTravel {
            distance: Some(bounds),
        } = &cond
        else {
            panic!("expected NetherTravel");
        };
        // x 3, z 0, y 4: every axis matches (absolute 5).
        assert!(bounds.matches((3.0, 4.0, 0.0)));
        // x below the minimum fails even when the absolute distance is enough.
        assert!(!bounds.matches((1.0, 10.0, 0.0)));
        // z above the maximum fails.
        assert!(!bounds.matches((3.0, 4.0, 2.0)));
        // absolute below the minimum fails even when x and z pass.
        assert!(!bounds.matches((3.0, 0.5, 0.0)));
    }

    #[test]
    fn lightning_strike_blocks_set_on_fire_uses_full_bounds() {
        let cond = CriterionConditions::parse(
            "minecraft:lightning_strike",
            r#"{"lightning": {"predicate": {"minecraft:type_specific/lightning": {"blocks_set_on_fire": {"min": 2}}}}}"#,
        );
        let bounds = match &cond {
            CriterionConditions::LightningStrike {
                blocks_set_on_fire, ..
            } => blocks_set_on_fire.as_ref(),
            _ => None,
        };
        assert!(bounds.is_some_and(|b| b.matches(2)));
        assert!(bounds.is_some_and(|b| b.matches(5)));
        assert!(bounds.is_none_or(|b| !b.matches(1)));

        let exact = CriterionConditions::parse(
            "minecraft:lightning_strike",
            r#"{"lightning": {"predicate": {"minecraft:type_specific/lightning": {"blocks_set_on_fire": 0}}}}"#,
        );
        let exact_bounds = match &exact {
            CriterionConditions::LightningStrike {
                blocks_set_on_fire, ..
            } => blocks_set_on_fire.as_ref(),
            _ => None,
        };
        assert!(exact_bounds.is_some_and(|b| b.matches(0)));
        assert!(exact_bounds.is_none_or(|b| !b.matches(1)));
    }

    #[test]
    fn lightning_strike_entity_struck_is_parsed_and_matched() {
        let cond = CriterionConditions::parse(
            "minecraft:lightning_strike",
            r#"{"lightning": {"predicate": {"minecraft:type_specific/lightning": {"entity_struck": {"type": "minecraft:villager"}}}}}"#,
        );
        let struck = match &cond {
            CriterionConditions::LightningStrike { entity_struck, .. } => entity_struck.as_ref(),
            _ => None,
        };
        assert!(struck.is_some_and(|p| p.matches("minecraft:villager", None, None)));
        assert!(struck.is_none_or(|p| !p.matches("minecraft:zombie", None, None)));
    }

    #[test]
    fn lightning_strike_bystander_uses_entity_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:lightning_strike",
            r#"{"bystander": {"type": "minecraft:player"}}"#,
        );
        let bystander = match &cond {
            CriterionConditions::LightningStrike { bystander, .. } => bystander.as_ref(),
            _ => None,
        };
        assert!(bystander.is_some_and(|p| p.matches("minecraft:player", None, None)));
        assert!(bystander.is_none_or(|p| !p.matches("minecraft:zombie", None, None)));
    }

    #[test]
    fn item_used_on_block_reads_location_array_and_item_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:item_used_on_block",
            r#"{
                "item": {"items": "minecraft:wheat_seeds"},
                "location": [
                    {
                        "condition": "minecraft:location_check",
                        "predicate": {"block": {"blocks": "minecraft:farmland"}}
                    }
                ]
            }"#,
        );
        match &cond {
            CriterionConditions::ItemUsedOnBlock(pred) => {
                assert!(pred.matches(
                    "minecraft:farmland",
                    "minecraft:wheat_seeds",
                    &[],
                    None,
                    None
                ));
                assert!(!pred.matches("minecraft:dirt", "minecraft:wheat_seeds", &[], None, None));
                assert!(!pred.matches(
                    "minecraft:farmland",
                    "minecraft:stone_hoe",
                    &[],
                    None,
                    None
                ));
            }
            _ => panic!("expected ItemUsedOnBlock"),
        }
    }

    #[test]
    fn matches_id_works() {
        assert!(matches_id("minecraft:stone", "stone"));
        assert!(matches_id("stone", "minecraft:stone"));
        assert!(matches_id("minecraft:stone", "minecraft:stone"));
        assert!(!matches_id("minecraft:stone", "minecraft:dirt"));
    }

    #[test]
    fn matches_int_bounds_works() {
        assert!(matches_int_bounds(&json!(5), 5));
        assert!(!matches_int_bounds(&json!(5), 4));
        assert!(matches_int_bounds(&json!({"min": 1, "max": 10}), 5));
        assert!(matches_int_bounds(&json!({"min": 5}), 5));
        assert!(matches_int_bounds(&json!({"min": 5}), 10));
        assert!(!matches_int_bounds(&json!({"min": 5}), 4));
        assert!(matches_int_bounds(&json!({"max": 5}), 5));
        assert!(!matches_int_bounds(&json!({"max": 5}), 6));
    }

    #[test]
    fn item_predicate_ignores_non_vanilla_durability_field() {
        // Vanilla 1.21 ItemPredicate has no `durability` field; unknown fields are ignored.
        let pred = ItemPredicate::from_value(&json!({
            "items": "minecraft:stone",
            "durability": {"min": 100}
        }));
        assert!(pred.matches("minecraft:stone", 1, &[]));
    }

    #[test]
    fn matches_item_predicate_works() {
        let pred = json!({
            "items": "minecraft:stone_pickaxe",
            "count": {"min": 1}
        });
        assert!(matches_item(&pred, "stone_pickaxe", 1, &[]));
        assert!(matches_item(&pred, "minecraft:stone_pickaxe", 2, &[]));
        assert!(!matches_item(&pred, "iron_pickaxe", 1, &[]));

        // Array of items
        let pred_arr = json!({
            "items": ["minecraft:stone", "minecraft:cobblestone"]
        });
        assert!(matches_item(&pred_arr, "stone", 1, &[]));
        assert!(matches_item(&pred_arr, "cobblestone", 1, &[]));
        assert!(!matches_item(&pred_arr, "dirt", 1, &[]));
    }

    #[test]
    fn matches_inventory_condition_works() {
        let cond = json!({
            "items": [
                {"items": "minecraft:crafting_table"}
            ]
        });
        let inv = [("crafting_table", 1, &[][..])];
        assert!(matches_inventory_condition(&cond, &inv, 0, 35, 1));

        let empty_inv = [];
        assert!(!matches_inventory_condition(&cond, &empty_inv, 0, 36, 0));
    }

    #[test]
    fn location_light_and_fluid_predicates() {
        let pred = LocationPredicate::from_value(&json!({
            "light": { "light": { "min": 8, "max": 12 } },
            "fluid": { "fluids": "minecraft:water" }
        }));
        let at = |light: Option<i64>, fluid: Option<&str>| {
            pred.matches_at(
                (0.0, 64.0, 0.0),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                light,
                fluid,
            )
        };
        assert!(at(Some(10), Some("water")));
        assert!(!at(Some(6), Some("water")));
        assert!(!at(Some(10), Some("lava")));
        assert!(!at(None, Some("water")));

        let shorthand = LocationPredicate::from_value(&json!({
            "light": 7,
            "fluid": { "fluids": "minecraft:water" }
        }));
        assert!(shorthand.matches_at(
            (0.0, 64.0, 0.0),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(7),
            Some("water"),
        ));
    }

    #[test]
    fn matches_location_condition_works() {
        let cond = json!({
            "player": {
                "type": "minecraft:entity_properties",
                "entity": "this",
                "predicate": {
                    "minecraft:location": {
                        "biomes": "minecraft:badlands",
                        "dimension": "minecraft:overworld"
                    }
                }
            }
        });
        assert!(matches_location_condition(
            &cond,
            (0.0, 64.0, 0.0),
            Some("badlands"),
            Some("minecraft:overworld"),
            &[],
            None,
            None,
            None,
            None,
            false,
        ));
        assert!(!matches_location_condition(
            &cond,
            (0.0, 64.0, 0.0),
            Some("desert"),
            Some("minecraft:overworld"),
            &[],
            None,
            None,
            None,
            None,
            false,
        ));
    }

    #[test]
    fn matches_dimension_condition_works() {
        let cond = json!({
            "to": "minecraft:the_nether"
        });
        assert!(matches_dimension_condition(
            &cond,
            None,
            "minecraft:the_nether"
        ));
        assert!(matches_dimension_condition(&cond, None, "the_nether"));
        assert!(!matches_dimension_condition(
            &cond,
            None,
            "minecraft:the_end"
        ));
    }

    #[test]
    fn damage_type_tag_predicate_honours_expected() {
        let projectile = DamageTypeTagPredicate {
            tag: "#minecraft:is_projectile".to_string(),
            expected: true,
        };
        let not_projectile = DamageTypeTagPredicate {
            tag: "#minecraft:is_projectile".to_string(),
            expected: false,
        };
        assert!(projectile.matches(&DamageType::ARROW));
        assert!(!not_projectile.matches(&DamageType::ARROW));
    }

    #[test]
    fn damage_predicate_checks_dealt_and_taken_separately() {
        let predicate = DamagePredicate {
            dealt: Some(FloatBounds {
                min: Some(8.0),
                max: Some(10.0),
            }),
            taken: Some(FloatBounds {
                min: Some(4.0),
                max: Some(5.0),
            }),
            ..DamagePredicate::default()
        };
        let pair = DamagePair {
            original: 9.0,
            actual: 4.5,
        };
        assert!(predicate.matches_with_damage_type(
            false,
            false,
            false,
            pair,
            &DamageEntities::default(),
            &DamageType::ARROW
        ));
        assert!(!predicate.matches_with_damage_type(
            false,
            false,
            false,
            DamagePair {
                original: 9.0,
                actual: 9.0,
            },
            &DamageEntities::default(),
            &DamageType::ARROW
        ));
        assert!(!predicate.matches_with_damage_type(
            false,
            false,
            false,
            DamagePair {
                original: 4.5,
                actual: 4.5,
            },
            &DamageEntities::default(),
            &DamageType::ARROW
        ));
    }

    #[test]
    fn damage_source_entity_predicate_uses_distance_and_baby() {
        let predicate = DamagePredicate {
            source_entity: Some(EntityPredicate::from_value(&json!({
                "minecraft:entity_type": "minecraft:zombie",
                "minecraft:distance": { "horizontal": { "max": 4.0 } },
                "minecraft:flags": { "is_baby": false }
            }))),
            ..DamagePredicate::default()
        };
        let pair = DamagePair {
            original: 10.0,
            actual: 10.0,
        };
        let near = DamageEntities {
            source_entity_type: Some("minecraft:zombie"),
            source_distance: Some(EntityDistance::from_delta(3.0, 0.0, 0.0)),
            source_is_baby: Some(false),
            ..DamageEntities::default()
        };
        assert!(predicate.matches_with_damage_type(
            false,
            false,
            false,
            pair,
            &near,
            &DamageType::ARROW
        ));
        let far = DamageEntities {
            source_distance: Some(EntityDistance::from_delta(9.0, 0.0, 0.0)),
            ..near
        };
        assert!(!predicate.matches_with_damage_type(
            false,
            false,
            false,
            pair,
            &far,
            &DamageType::ARROW
        ));
    }

    #[test]
    fn matches_damage_condition_works() {
        let cond = json!({
            "damage": {
                "blocked": true,
                "type": {
                    "tags": [
                        {"id": "#minecraft:is_projectile", "expected": true}
                    ]
                }
            }
        });
        assert!(matches_damage_condition(&cond, true, true, None));
        assert!(!matches_damage_condition(&cond, false, true, None));
        assert!(!matches_damage_condition(&cond, true, false, None));
    }

    #[test]
    fn location_structure_condition_does_not_match_everywhere() {
        for structures in [json!("minecraft:fortress"), json!(["minecraft:fortress"])] {
            let predicate = LocationPredicate::from_value(&json!({
                "player": {
                    "type": "minecraft:entity_properties",
                    "entity": "this",
                    "predicate": { "minecraft:location": { "structures": structures } }
                }
            }));
            assert!(predicate.structure.is_some());
            assert!(!predicate.matches(
                (0.0, 64.0, 0.0),
                Some("plains"),
                Some("minecraft:overworld"),
                None,
                None,
                None,
                None,
                None,
                None,
            ));
        }
    }

    #[test]
    fn location_structure_condition_matches_inside_that_structure() {
        for structures in [json!("minecraft:fortress"), json!(["minecraft:fortress"])] {
            let predicate = LocationPredicate::from_value(&json!({
                "player": {
                    "type": "minecraft:entity_properties",
                    "entity": "this",
                    "predicate": { "minecraft:location": { "structures": structures } }
                }
            }));
            assert!(predicate.matches(
                (0.0, 64.0, 0.0),
                Some("nether_wastes"),
                Some("minecraft:the_nether"),
                None,
                None,
                None,
                None,
                None,
                Some("fortress"),
            ));
            assert!(!predicate.matches(
                (0.0, 64.0, 0.0),
                Some("nether_wastes"),
                Some("minecraft:the_nether"),
                None,
                None,
                None,
                None,
                None,
                Some("bastion_remnant"),
            ));
        }
    }

    #[test]
    fn typed_criterion_conditions_parsing() {
        let cond = CriterionConditions::parse(
            "minecraft:inventory_changed",
            r#"{"items":[{"items":"minecraft:diamond","count":{"min":1}}]}"#,
        );
        match &cond {
            CriterionConditions::InventoryChanged(pred) => {
                let inv = [("diamond", 2, &[][..])];
                assert!(pred.matches(&inv, 0, 35, 1));
                let inv_wood = [("oak_planks", 2, &[][..])];
                assert!(!pred.matches(&inv_wood, 0, 35, 1));
            }
            _ => panic!("Expected InventoryChanged variant"),
        }
    }

    #[test]
    fn item_used_on_block_complex_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:item_used_on_block",
            r#"{
                "location": {
                    "type": "minecraft:all_of",
                    "terms": [
                        {
                            "type": "minecraft:location_check",
                            "predicate": {
                                "block": { "blocks": "minecraft:copper_bulb", "state": { "lit": "true" } }
                            }
                        },
                        {
                            "type": "minecraft:match_tool",
                            "predicate": { "items": "minecraft:redstone_torch" }
                        }
                    ]
                }
            }"#,
        );
        match &cond {
            CriterionConditions::ItemUsedOnBlock(pred) => {
                assert!(pred.matches(
                    "minecraft:copper_bulb",
                    "minecraft:redstone_torch",
                    &[("lit".to_string(), "true".to_string())],
                    None,
                    None
                ));
                assert!(!pred.matches(
                    "minecraft:copper_bulb",
                    "minecraft:redstone_torch",
                    &[("lit".to_string(), "false".to_string())],
                    None,
                    None
                ));
                assert!(!pred.matches(
                    "minecraft:tuff",
                    "minecraft:redstone_torch",
                    &[("lit".to_string(), "true".to_string())],
                    None,
                    None
                ));
            }
            _ => panic!("Expected ItemUsedOnBlock variant"),
        }
    }

    #[test]
    fn item_used_on_block_matches_any_state_property() {
        let cond = CriterionConditions::parse(
            "minecraft:item_used_on_block",
            r#"{
                "location": {
                    "condition": "minecraft:location_check",
                    "predicate": {
                        "block": {
                            "blocks": "minecraft:farmland",
                            "state": { "moisture": "7" }
                        }
                    }
                }
            }"#,
        );
        match &cond {
            CriterionConditions::ItemUsedOnBlock(pred) => {
                let state = [
                    ("moisture".to_string(), "7".to_string()),
                    ("waterlogged".to_string(), "false".to_string()),
                ];
                assert!(pred.matches(
                    "minecraft:farmland",
                    "minecraft:wheat_seeds",
                    &state,
                    None,
                    None
                ));
                let wrong_value = [("moisture".to_string(), "1".to_string())];
                assert!(!pred.matches(
                    "minecraft:farmland",
                    "minecraft:wheat_seeds",
                    &wrong_value,
                    None,
                    None
                ));
                let absent_key = [("facing".to_string(), "north".to_string())];
                assert!(!pred.matches(
                    "minecraft:farmland",
                    "minecraft:wheat_seeds",
                    &absent_key,
                    None,
                    None
                ));
            }
            _ => panic!("expected ItemUsedOnBlock"),
        }
    }

    #[test]
    fn item_used_on_block_all_of_any_of_inverted() {
        let cond = CriterionConditions::parse(
            "minecraft:item_used_on_block",
            r#"{
                "location": {
                    "condition": "minecraft:all_of",
                    "terms": [
                        {
                            "condition": "minecraft:location_check",
                            "predicate": {
                                "block": {"blocks": "minecraft:farmland"},
                                "biome": "minecraft:plains"
                            }
                        },
                        {
                            "condition": "minecraft:any_of",
                            "terms": [
                                {
                                    "condition": "minecraft:location_check",
                                    "predicate": {"smokey": true}
                                },
                                {
                                    "condition": "minecraft:inverted",
                                    "term": {
                                        "condition": "minecraft:location_check",
                                        "predicate": {"biome": "minecraft:desert"}
                                    }
                                }
                            ]
                        }
                    ]
                }
            }"#,
        );
        match &cond {
            CriterionConditions::ItemUsedOnBlock(pred) => {
                assert!(pred.matches(
                    "minecraft:farmland",
                    "minecraft:wheat_seeds",
                    &[],
                    Some(true),
                    Some("minecraft:plains")
                ));
                assert!(pred.matches(
                    "minecraft:farmland",
                    "minecraft:wheat_seeds",
                    &[],
                    Some(false),
                    Some("minecraft:plains")
                ));
                assert!(!pred.matches(
                    "minecraft:dirt",
                    "minecraft:wheat_seeds",
                    &[],
                    Some(true),
                    Some("minecraft:plains")
                ));
                assert!(!pred.matches(
                    "minecraft:farmland",
                    "minecraft:wheat_seeds",
                    &[],
                    Some(false),
                    Some("minecraft:desert")
                ));
            }
            _ => panic!("expected ItemUsedOnBlock"),
        }
    }

    #[test]
    fn walk_on_powder_snow_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:location",
            r#"{
                "player": {
                    "type": "minecraft:entity_properties",
                    "predicate": {
                        "minecraft:equipment": { "feet": { "items": "minecraft:leather_boots" } },
                        "minecraft:stepping_on": { "block": { "blocks": "minecraft:powder_snow" } }
                    }
                }
            }"#,
        );
        match &cond {
            CriterionConditions::Location(pred) => {
                assert!(pred.matches(
                    (0.0, 64.0, 0.0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("powder_snow"),
                    Some("leather_boots"),
                    None
                ));
                assert!(!pred.matches(
                    (0.0, 64.0, 0.0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("stone"),
                    Some("leather_boots"),
                    None
                ));
                assert!(!pred.matches(
                    (0.0, 64.0, 0.0),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some("powder_snow"),
                    Some("diamond_boots"),
                    None
                ));
            }
            _ => panic!("Expected Location variant"),
        }
    }

    #[test]
    fn ride_boat_with_goat_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:started_riding",
            r##"{
                "player": {
                    "type": "minecraft:entity_properties",
                    "predicate": {
                        "vehicle": {
                            "type": "#minecraft:boat",
                            "passenger": { "type": "minecraft:goat" }
                        }
                    }
                }
            }"##,
        );
        match &cond {
            CriterionConditions::StartedRiding(Some(pred)) => {
                assert!(
                    pred.vehicle_type
                        .as_ref()
                        .unwrap()
                        .matches(RegistryKey::EntityType, "minecraft:oak_boat")
                );
                assert!(
                    pred.passenger_type
                        .as_ref()
                        .unwrap()
                        .matches(RegistryKey::EntityType, "minecraft:goat")
                );
                assert!(
                    !pred
                        .passenger_type
                        .as_ref()
                        .unwrap()
                        .matches(RegistryKey::EntityType, "minecraft:pig")
                );
            }
            _ => panic!("Expected StartedRiding variant"),
        }
    }

    #[test]
    fn safe_armor_not_inferred_from_unrelated_conditions() {
        let pred = EntityPredicate::from_value(&json!({
            "type": "minecraft:inverted",
            "term": {
                "type": "minecraft:entity_properties",
                "predicate": {"type": "minecraft:zombie"}
            },
            "comment": "piglin_safe_armor mentioned but not required"
        }));
        assert_eq!(pred.safe_armor_disallowed, None);
    }

    #[test]
    fn distract_piglin_safe_armor_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:player_interacted_with_entity",
            r##"{
                "entity": {
                    "predicate": {
                        "type": "minecraft:piglin",
                        "flags": { "is_baby": false }
                    }
                },
                "item": { "items": "minecraft:gold_ingot" },
                "player": {
                    "type": "minecraft:inverted",
                    "term": {
                        "type": "minecraft:any_of",
                        "terms": [
                            {
                                "type": "minecraft:entity_properties",
                                "predicate": {
                                    "equipment": { "head": { "items": "#minecraft:piglin_safe_armor" } }
                                }
                            }
                        ]
                    }
                }
            }"##,
        );
        match &cond {
            CriterionConditions::PlayerInteractedWithEntity {
                entity,
                item,
                player,
            } => {
                assert!(
                    entity
                        .as_ref()
                        .unwrap()
                        .matches("minecraft:piglin", None, Some(false))
                );
                assert!(
                    !entity
                        .as_ref()
                        .unwrap()
                        .matches("minecraft:piglin", None, Some(true))
                );
                assert!(
                    item.as_ref()
                        .unwrap()
                        .matches("minecraft:gold_ingot", 1, &[])
                );
                assert_eq!(player.as_ref().unwrap().safe_armor_disallowed, Some(true));
            }
            _ => panic!("Expected PlayerInteractedWithEntity variant"),
        }
    }

    fn effects_changed_condition(advancement: &Advancement) -> CriterionConditions {
        advancement.criteria_definitions[0].conditions.into()
    }

    fn active(effect_type: &'static StatusEffect) -> Effect {
        Effect {
            effect_type,
            duration: 200,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }
    }

    #[test]
    fn effects_changed_requires_every_listed_effect() {
        let cond = effects_changed_condition(Advancement::NETHER_ALL_EFFECTS);
        let CriterionConditions::EffectsChanged {
            effects: Some(effects_pred),
            ..
        } = &cond
        else {
            panic!("Expected EffectsChanged with effects");
        };
        let mut effects: Vec<Effect> = effects_pred
            .effect_map
            .iter()
            .map(|p| active(StatusEffect::from_minecraft_name(&p.effect).unwrap()))
            .collect();

        assert!(!matches_effects_changed_condition(
            &cond,
            &[active(&StatusEffect::SPEED)],
            None,
            None
        ));
        assert!(matches_effects_changed_condition(
            &cond, &effects, None, None
        ));
        effects.pop();
        assert!(!matches_effects_changed_condition(
            &cond, &effects, None, None
        ));
    }

    #[test]
    fn enter_block_condition_reads_block_field() {
        assert!(matches_enter_block_condition(
            &json!({"block": "minecraft:water"}),
            "minecraft:water"
        ));
        assert!(!matches_enter_block_condition(
            &json!({"block": "minecraft:water"}),
            "minecraft:stone"
        ));
    }

    #[test]
    fn recipe_crafted_reads_recipe_id() {
        let cond = CriterionConditions::parse(
            "minecraft:recipe_crafted",
            r#"{"recipe_id": "minecraft:bread"}"#,
        );
        match &cond {
            CriterionConditions::RecipeCrafted(pred) => {
                assert!(pred.matches("minecraft:bread"));
                assert!(!pred.matches("minecraft:cake"));
            }
            _ => panic!("Expected RecipeCrafted variant"),
        }
    }

    #[test]
    fn container_loot_reads_loot_table() {
        let cond = CriterionConditions::parse(
            "minecraft:player_generates_container_loot",
            r#"{"loot_table": "minecraft:chests/simple_dungeon"}"#,
        );
        match &cond {
            CriterionConditions::PlayerGeneratesContainerLoot(lt) => {
                assert_eq!(lt, "minecraft:chests/simple_dungeon");
            }
            _ => panic!("Expected PlayerGeneratesContainerLoot variant"),
        }
    }

    #[test]
    fn effects_changed_source_requires_a_source() {
        let cond = effects_changed_condition(Advancement::HUSBANDRY_KILL_AXOLOTL_TARGET);
        let effects = [active(&StatusEffect::REGENERATION)];

        assert!(!matches_effects_changed_condition(
            &cond, &effects, None, None
        ));
        assert!(!matches_effects_changed_condition(
            &cond,
            &effects,
            Some("minecraft:zombie"),
            None
        ));
        assert!(matches_effects_changed_condition(
            &cond,
            &effects,
            Some("minecraft:axolotl"),
            None
        ));
    }

    #[test]
    fn effects_changed_source_distance_is_checked() {
        let cond = CriterionConditions::parse(
            "minecraft:effects_changed",
            r#"{"source": {"distance": {"absolute": {"max": 5.0}}}}"#,
        );
        let effects = [active(&StatusEffect::REGENERATION)];

        assert!(!matches_effects_changed_condition(
            &cond, &effects, None, None
        ));
        assert!(matches_effects_changed_condition(
            &cond,
            &effects,
            Some("minecraft:zombie"),
            Some(EntityDistance::from_delta(3.0, 0.0, 0.0))
        ));
        assert!(!matches_effects_changed_condition(
            &cond,
            &effects,
            Some("minecraft:zombie"),
            Some(EntityDistance::from_delta(6.0, 0.0, 0.0))
        ));
    }

    #[test]
    fn entity_distance_checks_horizontal_and_absolute_separately() {
        let horizontal = EntityPredicate::from_value(&serde_json::json!({
            "distance": {"horizontal": {"max": 5.0}}
        }));
        let absolute = EntityPredicate::from_value(&serde_json::json!({
            "distance": {"absolute": {"max": 5.0}}
        }));
        // 3 blocks east and 10 blocks up: horizontal 3 (matches), absolute ~10.44 (fails).
        let delta = EntityDistance::from_delta(3.0, 10.0, 0.0);
        assert!(horizontal.matches("minecraft:zombie", Some(delta), None));
        assert!(!absolute.matches("minecraft:zombie", Some(delta), None));
    }

    #[test]
    fn sniper_duel_absolute_min_50_boundary() {
        let pred = EntityPredicate::from_value(&serde_json::json!({
            "distance": {"absolute": {"min": 50.0}}
        }));
        // Exactly 50 blocks straight up passes; 49 blocks horizontally fails the absolute bound.
        assert!(pred.matches(
            "minecraft:skeleton",
            Some(EntityDistance::from_delta(0.0, 50.0, 0.0)),
            None
        ));
        assert!(!pred.matches(
            "minecraft:skeleton",
            Some(EntityDistance::from_delta(49.0, 0.0, 0.0)),
            None
        ));
        assert!(!pred.matches("minecraft:skeleton", None, None));
    }

    #[test]
    fn item_count_bounds_use_actual_stack_count() {
        let cond = json!({"item": {"items": "minecraft:stone", "count": {"min": 2}}});
        assert!(!matches_placed_block_condition(
            &cond,
            "minecraft:stone",
            Some("minecraft:stone"),
            1,
            &[]
        ));
        assert!(matches_placed_block_condition(
            &cond,
            "minecraft:stone",
            Some("minecraft:stone"),
            2,
            &[]
        ));
        let consume = json!({"items": "minecraft:stone", "count": {"min": 2}});
        assert!(!matches_item_condition(&consume, "minecraft:stone", 1));
        assert!(matches_item_condition(&consume, "minecraft:stone", 2));
    }

    #[test]
    fn bred_animals_matches_each_role_against_its_own_entity() {
        let pred = BredAnimalsPredicate::from_value(&json!({
            "parent": {"minecraft:entity_type": "minecraft:cow"},
            "partner": {"minecraft:entity_type": "minecraft:sheep"},
            "child": {"minecraft:entity_type": "minecraft:cow"}
        }));

        // Each role is checked against its own entity.
        assert!(pred.matches("minecraft:cow", "minecraft:sheep", Some("minecraft:cow")));
        // Parent and partner predicates are symmetric.
        assert!(pred.matches("minecraft:sheep", "minecraft:cow", Some("minecraft:cow")));
        // A wrong child type fails even when both parents match.
        assert!(!pred.matches("minecraft:cow", "minecraft:sheep", Some("minecraft:sheep")));
        // A present child predicate requires a child to exist.
        assert!(!pred.matches("minecraft:cow", "minecraft:sheep", None));
        // Both parents must match; the old OR-based check returned true here.
        assert!(!pred.matches("minecraft:cow", "minecraft:pig", Some("minecraft:cow")));

        // No role predicates matches anything.
        assert!(BredAnimalsPredicate::default().matches("minecraft:cow", "minecraft:pig", None));
    }

    #[test]
    fn fall_after_explosion_keeps_full_cause_predicate() {
        let cond = CriterionConditions::parse(
            "minecraft:fall_after_explosion",
            r#"{"distance":{"y":{"min":2}},
                "cause":{"condition":"minecraft:entity_properties","entity":"this",
                  "predicate":{"type":"minecraft:wind_charge",
                    "flags":{"is_baby":true},
                    "distance":{"horizontal":{"min":3}}}}}"#,
        );
        let CriterionConditions::FallAfterExplosion {
            distance,
            cause_entity: Some(cause),
        } = &cond
        else {
            panic!("expected FallAfterExplosion with a cause predicate");
        };
        assert!(
            distance
                .as_ref()
                .is_some_and(|d| d.matches((0.0, 3.0, 0.0)))
        );
        // The whole cause predicate is kept, not just its entity type string.
        assert_eq!(cause.is_baby, Some(true));
        let delta = EntityDistance::from_delta(4.0, 0.0, 0.0);
        assert!(cause.matches("minecraft:wind_charge", Some(delta), Some(true)));
        // The baby flag and horizontal distance are both enforced.
        assert!(!cause.matches("minecraft:wind_charge", Some(delta), Some(false)));
        assert!(!cause.matches(
            "minecraft:wind_charge",
            Some(EntityDistance::from_delta(2.0, 0.0, 0.0)),
            Some(true)
        ));
        assert!(!cause.matches("minecraft:creeper", Some(delta), Some(true)));
    }

    #[test]
    fn entity_effects_and_equipment_are_enforced() {
        let pred = EntityPredicate::from_value(&json!({
            "effects": {"minecraft:speed": {"amplifier": {"min": 1}}},
            "equipment": {"head": {"items": "minecraft:leather_helmet"}}
        }));
        let state = EntityState {
            distance: None,
            is_baby: None,
            effects: vec![active(&StatusEffect::SPEED)],
            equipment: vec![("head".to_string(), "minecraft:leather_helmet".to_string())],
            stepping_block: None,
            vehicle_type: None,
        };
        assert!(!pred.matches_entity("minecraft:cow", &state));
        let mut amplified = state;
        if let Some(effect) = amplified.effects.first_mut() {
            effect.amplifier = 1;
        }
        assert!(pred.matches_entity("minecraft:cow", &amplified));
        let mut wrong_head = amplified.clone();
        wrong_head.equipment = vec![("head".to_string(), "minecraft:diamond_helmet".to_string())];
        assert!(!pred.matches_entity("minecraft:cow", &wrong_head));
    }

    #[test]
    fn entity_vehicle_predicate_is_enforced() {
        let pred = EntityPredicate::from_value(&json!({
            "vehicle": {"type": "minecraft:boat"}
        }));
        let mut state = EntityState::default();
        // Vanilla VehiclePredicate.matches returns false when the entity has no vehicle.
        assert!(!pred.matches_entity("minecraft:player", &state));
        state.vehicle_type = Some("minecraft:boat".to_string());
        assert!(pred.matches_entity("minecraft:player", &state));
        state.vehicle_type = Some("minecraft:minecart".to_string());
        assert!(!pred.matches_entity("minecraft:player", &state));
    }
}
