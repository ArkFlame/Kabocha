use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use pumpkin_data::registry::RegistryEntryData;
use pumpkin_nbt::Nbt;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::identifier::Identifier;
use serde_json::Value;
use tracing::warn;

pub const DYNAMIC_REGISTRIES: &[(&str, &[&str])] = &[
    ("minecraft:wolf_variant", &["wolf_variant", "wolf_variants"]),
    ("minecraft:cat_variant", &["cat_variant", "cat_variants"]),
    ("minecraft:frog_variant", &["frog_variant", "frog_variants"]),
    ("minecraft:instrument", &["instrument", "instruments"]),
    (
        "minecraft:trim_material",
        &["trim_material", "trim_materials"],
    ),
    ("minecraft:trim_pattern", &["trim_pattern", "trim_patterns"]),
    (
        "minecraft:banner_pattern",
        &["banner_pattern", "banner_patterns"],
    ),
    (
        "minecraft:decorated_pot_pattern",
        &["decorated_pot_pattern", "decorated_pot_patterns"],
    ),
    ("minecraft:chat_type", &["chat_type", "chat_types"]),
    ("minecraft:enchantment", &["enchantment", "enchantments"]),
    (
        "minecraft:enchantment_provider",
        &["enchantment_provider", "enchantment_providers"],
    ),
    ("minecraft:cow_variant", &["cow_variant", "cow_variants"]),
    (
        "minecraft:cow_sound_variant",
        &["cow_sound_variant", "cow_sound_variants"],
    ),
    ("minecraft:pig_variant", &["pig_variant", "pig_variants"]),
    (
        "minecraft:pig_sound_variant",
        &["pig_sound_variant", "pig_sound_variants"],
    ),
    (
        "minecraft:chicken_variant",
        &["chicken_variant", "chicken_variants"],
    ),
    (
        "minecraft:chicken_sound_variant",
        &["chicken_sound_variant", "chicken_sound_variants"],
    ),
    (
        "minecraft:cat_sound_variant",
        &["cat_sound_variant", "cat_sound_variants"],
    ),
    (
        "minecraft:wolf_sound_variant",
        &["wolf_sound_variant", "wolf_sound_variants"],
    ),
    (
        "minecraft:zombie_nautilus_variant",
        &["zombie_nautilus_variant", "zombie_nautilus_variants"],
    ),
    ("minecraft:trade_set", &["trade_set", "trade_sets"]),
    (
        "minecraft:villager_trade",
        &["villager_trade", "villager_trades"],
    ),
    (
        "minecraft:painting_variant",
        &["painting_variant", "painting_variants"],
    ),
    ("minecraft:dimension", &["dimension", "dimensions"]),
    (
        "minecraft:dimension_type",
        &["dimension_type", "dimension_types"],
    ),
    (
        "minecraft:item_modifier",
        &["item_modifier", "item_modifiers"],
    ),
    ("minecraft:slot_source", &["slot_source", "slot_sources"]),
    ("minecraft:jukebox_song", &["jukebox_song", "jukebox_songs"]),
    ("minecraft:dialog", &["dialog", "dialogs"]),
    ("minecraft:timeline", &["timeline", "timelines"]),
    ("minecraft:world_clock", &["world_clock", "world_clocks"]),
    (
        "minecraft:test_environment",
        &["test_environment", "test_environments"],
    ),
    (
        "minecraft:test_instance",
        &["test_instance", "test_instances"],
    ),
    (
        "minecraft:sulfur_cube_archetype",
        &["sulfur_cube_archetype", "sulfur_cube_archetypes"],
    ),
    (
        "minecraft:block_transformer",
        &["block_transformer", "block_transformers"],
    ),
    (
        "minecraft:block_sound_set",
        &["block_sound_set", "block_sound_sets"],
    ),
    (
        "minecraft:trial_spawner",
        &["trial_spawner", "trial_spawners"],
    ),
    (
        "minecraft:worldgen/biome",
        &["worldgen/biome", "worldgen/biomes"],
    ),
    (
        "minecraft:worldgen/block_state_provider",
        &[
            "worldgen/block_state_provider",
            "worldgen/block_state_providers",
        ],
    ),
    (
        "minecraft:worldgen/carver",
        &[
            "worldgen/carver",
            "worldgen/carvers",
            "worldgen/configured_carver",
            "worldgen/configured_carvers",
        ],
    ),
    (
        "minecraft:worldgen/configured_carver",
        &[
            "worldgen/configured_carver",
            "worldgen/configured_carvers",
            "worldgen/carver",
            "worldgen/carvers",
        ],
    ),
    (
        "minecraft:worldgen/density_function",
        &["worldgen/density_function", "worldgen/density_functions"],
    ),
    (
        "minecraft:worldgen/feature",
        &[
            "worldgen/feature",
            "worldgen/features",
            "worldgen/configured_feature",
            "worldgen/configured_features",
        ],
    ),
    (
        "minecraft:worldgen/configured_feature",
        &[
            "worldgen/configured_feature",
            "worldgen/configured_features",
            "worldgen/feature",
            "worldgen/features",
        ],
    ),
    (
        "minecraft:worldgen/flat_level_generator_preset",
        &[
            "worldgen/flat_level_generator_preset",
            "worldgen/flat_level_generator_presets",
        ],
    ),
    (
        "minecraft:worldgen/material_condition",
        &[
            "worldgen/material_condition",
            "worldgen/material_conditions",
        ],
    ),
    (
        "minecraft:worldgen/material_rule",
        &["worldgen/material_rule", "worldgen/material_rules"],
    ),
    (
        "minecraft:worldgen/multi_noise_biome_source_parameter_list",
        &[
            "worldgen/multi_noise_biome_source_parameter_list",
            "worldgen/multi_noise_biome_source_parameter_lists",
        ],
    ),
    (
        "minecraft:worldgen/noise",
        &["worldgen/noise", "worldgen/noises"],
    ),
    (
        "minecraft:worldgen/noise_settings",
        &["worldgen/noise_settings"],
    ),
    (
        "minecraft:worldgen/placed_feature",
        &["worldgen/placed_feature", "worldgen/placed_features"],
    ),
    (
        "minecraft:worldgen/processor_list",
        &["worldgen/processor_list", "worldgen/processor_lists"],
    ),
    (
        "minecraft:worldgen/structure",
        &[
            "worldgen/structure",
            "worldgen/structures",
            "worldgen/configured_structure",
            "worldgen/configured_structures",
        ],
    ),
    (
        "minecraft:worldgen/structure_set",
        &["worldgen/structure_set", "worldgen/structure_sets"],
    ),
    (
        "minecraft:worldgen/template_pool",
        &["worldgen/template_pool", "worldgen/template_pools"],
    ),
    (
        "minecraft:worldgen/world_preset",
        &["worldgen/world_preset", "worldgen/world_presets"],
    ),
];

/// Widens all numeric elements of a list to the widest numeric type present.
///
/// NBT lists are homogeneous. Without this, a JSON array mixing integers and
/// longs (or integers and doubles) would be encoded as a list of `{"": value}`
/// wrapper compounds by the NBT serializer instead of plain numbers.
fn normalize_numeric_list(list: Vec<NbtTag>) -> Vec<NbtTag> {
    const INT: u8 = 1;
    const LONG: u8 = 2;
    const DOUBLE: u8 = 3;

    let mut widest = INT;
    let mut numeric = true;
    for tag in &list {
        let rank = match tag {
            NbtTag::Int(_) => INT,
            NbtTag::Long(_) => LONG,
            NbtTag::Double(_) => DOUBLE,
            _ => {
                numeric = false;
                break;
            }
        };
        widest = widest.max(rank);
    }

    // Leave non-numeric and already-`Int` lists untouched.
    if !numeric || widest == INT {
        return list;
    }

    list.into_iter()
        .map(|tag| match (widest, tag) {
            (LONG, NbtTag::Int(v)) => NbtTag::Long(i64::from(v)),
            (DOUBLE, NbtTag::Int(v)) => NbtTag::Double(f64::from(v)),
            (DOUBLE, NbtTag::Long(v)) => NbtTag::Double(v as f64),
            (_, other) => other,
        })
        .collect()
}

/// Converts a JSON `serde_json::Value` into an `NbtTag`.
///
/// Returns `None` for JSON `null`, which has no NBT equivalent. Writing
/// [`NbtTag::End`] into a compound or list would truncate the encoded stream,
/// so object keys and list elements holding `null` are dropped instead.
#[must_use]
pub fn json_to_nbt_tag(v: &Value) -> Option<NbtTag> {
    match v {
        Value::Null => None,
        Value::Bool(b) => Some(NbtTag::Byte(i8::from(*b))),
        Value::Number(num) => Some(num.as_i64().map_or_else(
            || num.as_f64().map_or(NbtTag::Int(0), NbtTag::Double),
            |i| {
                if i32::try_from(i).is_ok() {
                    NbtTag::Int(i as i32)
                } else {
                    NbtTag::Long(i)
                }
            },
        )),
        Value::String(s) => Some(NbtTag::String(s.clone().into())),
        Value::Array(arr) => {
            let list: Vec<NbtTag> = arr.iter().filter_map(json_to_nbt_tag).collect();
            Some(NbtTag::List(normalize_numeric_list(list)))
        }
        Value::Object(obj) => {
            let mut compound = NbtCompound::new();
            for (k, val) in obj {
                if let Some(tag) = json_to_nbt_tag(val) {
                    compound.put(k, tag);
                }
            }
            Some(NbtTag::Compound(compound))
        }
    }
}

/// Encodes a JSON definition into binary unnamed NBT bytes.
///
/// The dynamic registry loader rejects a non-object root before calling this,
/// matching vanilla's registry codecs.
#[must_use]
pub fn json_to_registry_nbt_bytes(val: &Value) -> Box<[u8]> {
    let nbt_tag = json_to_nbt_tag(val).unwrap_or_else(|| NbtTag::Compound(NbtCompound::new()));
    match nbt_tag {
        NbtTag::Compound(compound) => Nbt::from(compound)
            .write_unnamed()
            .to_vec()
            .into_boxed_slice(),
        other => {
            let mut bytes = Vec::new();
            let mut writer = pumpkin_nbt::serializer::NbtWriteHelperJava::new(&mut bytes);
            let _ = other.serialize(&mut writer);
            bytes.into_boxed_slice()
        }
    }
}

/// Recursively loads JSON files from `current_dir` and inserts them into `registry_map`.
fn load_registry_dir_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry_map: &mut HashMap<String, RegistryEntryData, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Vanilla's pack listing never follows symbolic links, and following a
            // directory symlink here would recurse forever on a loop.
            if entry.file_type().is_ok_and(|ft| ft.is_symlink()) {
                continue;
            }
            load_registry_dir_recursive(namespace, base_dir, &path, registry_map);
        } else if path
            .extension()
            .is_some_and(|ext| ext.to_str() == Some("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();
            if let Some(stem) = stem_path.strip_suffix(".json") {
                stem_path = stem.to_string();
            }
            let stem_path = stem_path.replace('\\', "/");
            // Vanilla rejects entry ids that are not valid resource locations.
            if !Identifier::is_valid_namespace(namespace) || !Identifier::is_valid_path(&stem_path) {
                warn!(
                    "Skipping dynamic registry file {}: invalid resource location",
                    path.display()
                );
                continue;
            }
            let entry_id = format!("{namespace}:{stem_path}");

            let Ok(content) = fs::read_to_string(&path) else {
                warn!("Skipping dynamic registry file {}: unreadable", path.display());
                continue;
            };
            let val = match serde_json::from_str::<Value>(&content) {
                Ok(val) => val,
                Err(err) => {
                    warn!("Skipping dynamic registry file {}: {err}", path.display());
                    continue;
                }
            };
            if !val.is_object() {
                warn!(
                    "Skipping dynamic registry file {}: root is not a JSON object",
                    path.display()
                );
                continue;
            }

            let nbt_bytes = json_to_registry_nbt_bytes(&val);
            registry_map.insert(
                entry_id.clone(),
                RegistryEntryData {
                    entry_id,
                    data: Some(nbt_bytes),
                },
            );
        }
    }
}

const EXCLUDED_REGISTRY_DIRS: &[&str] = &[
    "recipe",
    "recipes",
    "function",
    "functions",
    "tags",
    "structure",
    "structures",
    "advancement",
    "advancements",
    "loot_table",
    "loot_tables",
    "predicate",
    "predicates",
    "item_modifier",
    "item_modifiers",
    "slot_source",
    "slot_sources",
    "context_int_provider",
    "context_int_providers",
    "context_float_provider",
    "context_float_providers",
    "damage_type",
    "damage_types",
    "datapacks",
];

/// Scans a namespace directory for any dynamic registry subfolders and loads their JSON definitions.
pub fn load_dynamic_registries_from_ns<
    S1: std::hash::BuildHasher + Default,
    S2: std::hash::BuildHasher,
>(
    namespace: &str,
    ns_path: &Path,
    registries: &mut HashMap<String, HashMap<String, RegistryEntryData, S1>, S2>,
) -> usize {
    let mut total_loaded = 0;
    let mut visited_paths = HashSet::new();

    for &(registry_id, dir_names) in DYNAMIC_REGISTRIES {
        let registry_map = registries.entry(registry_id.to_string()).or_default();
        for &dir_name in dir_names {
            let reg_dir = ns_path.join(dir_name);
            if reg_dir.is_dir() {
                visited_paths.insert(reg_dir.clone());
                let before = registry_map.len();
                load_registry_dir_recursive(namespace, &reg_dir, &reg_dir, registry_map);
                total_loaded += registry_map.len() - before;
            }
        }
    }

    // Auto-discover any additional registry folders in the namespace (including custom/modded ones)
    if let Ok(entries) = fs::read_dir(ns_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if !path.is_dir()
                || EXCLUDED_REGISTRY_DIRS.contains(&name.as_str())
                || visited_paths.contains(&path)
            {
                continue;
            }

            if name == "worldgen" {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                        if sub_path.is_dir() && !visited_paths.contains(&sub_path) {
                            let reg_id = format!("{namespace}:worldgen/{sub_name}");
                            let registry_map = registries.entry(reg_id).or_default();
                            let before = registry_map.len();
                            load_registry_dir_recursive(
                                namespace,
                                &sub_path,
                                &sub_path,
                                registry_map,
                            );
                            total_loaded += registry_map.len() - before;
                        }
                    }
                }
            } else {
                let reg_id = format!("{namespace}:{name}");
                let registry_map = registries.entry(reg_id).or_default();
                let before = registry_map.len();
                load_registry_dir_recursive(namespace, &path, &path, registry_map);
                total_loaded += registry_map.len() - before;
            }
        }
    }

    total_loaded
}

/// Merges vanilla registry entries with custom datapack entries for a registry.
///
/// Vanilla entries are preserved in their original order, unless overridden by a custom entry
/// with matching entry ID. Custom entries not matching any vanilla entry are sorted
/// lexicographically and appended to the end.
#[must_use]
pub fn merge_dynamic_registry_entries<S: std::hash::BuildHasher>(
    vanilla_entries: &[RegistryEntryData],
    custom_entries: &HashMap<String, RegistryEntryData, S>,
) -> Vec<RegistryEntryData> {
    if custom_entries.is_empty() {
        return vanilla_entries
            .iter()
            .map(|entry| RegistryEntryData {
                entry_id: entry.entry_id.clone(),
                data: entry.data.clone(),
            })
            .collect();
    }

    let mut merged = Vec::with_capacity(vanilla_entries.len() + custom_entries.len());
    let mut overridden = HashSet::new();

    for entry in vanilla_entries {
        let full_id = if entry.entry_id.contains(':') {
            entry.entry_id.clone()
        } else {
            format!("minecraft:{}", entry.entry_id)
        };

        if let Some(custom) = custom_entries
            .get(&full_id)
            .or_else(|| custom_entries.get(&entry.entry_id))
        {
            let mut reg_entry = RegistryEntryData {
                entry_id: entry.entry_id.clone(),
                data: custom.data.clone(),
            };
            merged.push(reg_entry);
            overridden.insert(full_id);
            overridden.insert(entry.entry_id.clone());
        } else {
            merged.push(RegistryEntryData {
                entry_id: entry.entry_id.clone(),
                data: entry.data.clone(),
            });
        }
    }

    let mut sorted_custom: Vec<&RegistryEntryData> = custom_entries
        .values()
        .filter(|e| !overridden.contains(&e.entry_id))
        .collect();
    sorted_custom.sort_by(|a, b| a.entry_id.cmp(&b.entry_id));

    for custom in sorted_custom {
        merged.push(RegistryEntryData {
            entry_id: custom.entry_id.clone(),
            data: custom.data.clone(),
        });
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_to_nbt_roundtrip() {
        let json: Value = serde_json::json!({
            "asset_id": "custom:entity/cat/fluffy",
            "number": 42,
            "float": 3.25,
            "flag": true,
            "nested": {
                "tag": "value"
            },
            "list": ["a", "b"]
        });

        let bytes = json_to_registry_nbt_bytes(&json);
        assert!(!bytes.is_empty());

        let mut cursor = std::io::Cursor::new(&bytes[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = Nbt::read_unnamed(&mut reader).unwrap().root_tag;
        assert_eq!(nbt.get_string("asset_id"), Some("custom:entity/cat/fluffy"));
        assert_eq!(nbt.get_int("number"), Some(42));
        assert_eq!(nbt.get_byte("flag"), Some(1));
    }

    #[test]
    fn json_null_fields_are_dropped_without_truncating() {
        let json: Value = serde_json::json!({
            "asset_id": "custom:entity/cat/fluffy",
            "optional": null,
            "list": ["a", null, "b"],
            "trailing": 7
        });

        let bytes = json_to_registry_nbt_bytes(&json);
        let mut cursor = std::io::Cursor::new(&bytes[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = Nbt::read_unnamed(&mut reader).unwrap().root_tag;

        // A JSON null must not write an End tag, which would end the compound early.
        assert_eq!(nbt.get_int("trailing"), Some(7));
        assert_eq!(nbt.get_string("asset_id"), Some("custom:entity/cat/fluffy"));
        assert!(nbt.get_string("optional").is_none());
        assert_eq!(nbt.get_list("list").map(<[NbtTag]>::len), Some(2));
    }

    #[test]
    fn only_lowercase_json_files_are_loaded() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_test_dr_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("lower.json"), r#"{"asset_id":"custom:lower"}"#).unwrap();
        fs::write(temp_dir.join("upper.JSON"), r#"{"asset_id":"custom:upper"}"#).unwrap();

        let mut registry: HashMap<String, RegistryEntryData> = HashMap::new();
        load_registry_dir_recursive("custom", &temp_dir, &temp_dir, &mut registry);

        // Vanilla only reads the exact `.json` suffix, so `.JSON` files are ignored.
        assert_eq!(registry.len(), 1);
        assert!(registry.contains_key("custom:lower"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn invalid_registry_files_are_skipped() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_test_dr_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(
            temp_dir.join("Upper Case.json"),
            r#"{"asset_id":"custom:uppercase"}"#,
        )
        .unwrap();
        fs::write(temp_dir.join("array_root.json"), "[1,2]").unwrap();
        fs::write(temp_dir.join("broken.json"), "{not json").unwrap();
        fs::write(
            temp_dir.join("good.json"),
            r#"{"asset_id":"custom:good"}"#,
        )
        .unwrap();

        let mut registry: HashMap<String, RegistryEntryData> = HashMap::new();
        load_registry_dir_recursive("custom", &temp_dir, &temp_dir, &mut registry);

        // Vanilla rejects ids that are not valid resource locations and files whose root
        // is not a JSON object; unparsable files are skipped as well.
        assert_eq!(registry.len(), 1);
        assert!(registry.contains_key("custom:good"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn merge_preserves_and_appends() {
        let vanilla = vec![
            RegistryEntryData {
                entry_id: "minecraft:all_black".to_string(),
                data: Some(vec![1, 2, 3].into_boxed_slice()),
            },
            RegistryEntryData {
                entry_id: "minecraft:black".to_string(),
                data: Some(vec![4, 5, 6].into_boxed_slice()),
            },
        ];

        let mut custom = HashMap::new();
        custom.insert(
            "custom:fluffy".to_string(),
            RegistryEntryData {
                entry_id: "custom:fluffy".to_string(),
                data: Some(vec![7, 8, 9].into_boxed_slice()),
            },
        );
        custom.insert(
            "minecraft:black".to_string(),
            RegistryEntryData {
                entry_id: "minecraft:black".to_string(),
                data: Some(vec![99].into_boxed_slice()),
            },
        );

        let merged = merge_dynamic_registry_entries(&vanilla, &custom);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].entry_id, "minecraft:all_black");
        assert_eq!(merged[0].data.as_deref(), Some(&[1, 2, 3][..]));
        assert_eq!(merged[1].entry_id, "minecraft:black");
        assert_eq!(merged[1].data.as_deref(), Some(&[99][..]));
        assert_eq!(merged[2].entry_id, "custom:fluffy");
        assert_eq!(merged[2].data.as_deref(), Some(&[7, 8, 9][..]));
    }

    #[test]
    fn json_numeric_lists_use_one_numeric_type() {
        let json: Value = serde_json::json!({
            "ints": [1, 2],
            "longs": [1, 3_000_000_000i64],
            "floats": [1, 2.5],
        });

        let bytes = json_to_registry_nbt_bytes(&json);
        let mut cursor = std::io::Cursor::new(&bytes[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = Nbt::read_unnamed(&mut reader).unwrap().root_tag;

        // A homogeneous list decodes as Int/Long/Double; widening keeps that shape
        // instead of falling back to `{"": value}` wrapper compounds.
        let ints = nbt.get_list("ints").unwrap();
        assert_eq!(ints.len(), 2);
        assert!(ints.iter().all(|t| matches!(t, NbtTag::Int(_))));
        assert!(
            nbt.get_list("longs")
                .unwrap()
                .iter()
                .all(|t| matches!(t, NbtTag::Long(_)))
        );
        assert!(
            nbt.get_list("floats")
                .unwrap()
                .iter()
                .all(|t| matches!(t, NbtTag::Double(_)))
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_registry_dirs_are_not_followed() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_test_dr_{}", uuid::Uuid::new_v4()));
        let reg_dir = temp_dir.join("reg");
        fs::create_dir_all(&reg_dir).unwrap();
        fs::write(reg_dir.join("good.json"), r#"{"asset_id":"custom:good"}"#).unwrap();
        std::os::unix::fs::symlink(&temp_dir, reg_dir.join("loop")).unwrap();

        let mut registry: HashMap<String, RegistryEntryData> = HashMap::new();
        load_registry_dir_recursive("custom", &reg_dir, &reg_dir, &mut registry);

        // Vanilla never follows pack symlinks, so `loop` is not descended into.
        assert_eq!(registry.len(), 1);
        assert!(registry.contains_key("custom:good"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
