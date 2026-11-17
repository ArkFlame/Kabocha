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
    ("minecraft:wolf_variant", &["wolf_variant"]),
    ("minecraft:cat_variant", &["cat_variant"]),
    ("minecraft:frog_variant", &["frog_variant"]),
    ("minecraft:instrument", &["instrument"]),
    ("minecraft:trim_material", &["trim_material"]),
    ("minecraft:trim_pattern", &["trim_pattern"]),
    ("minecraft:banner_pattern", &["banner_pattern"]),
    (
        "minecraft:decorated_pot_pattern",
        &["decorated_pot_pattern"],
    ),
    ("minecraft:chat_type", &["chat_type"]),
    ("minecraft:enchantment", &["enchantment"]),
    ("minecraft:enchantment_provider", &["enchantment_provider"]),
    ("minecraft:cow_variant", &["cow_variant"]),
    ("minecraft:cow_sound_variant", &["cow_sound_variant"]),
    ("minecraft:pig_variant", &["pig_variant"]),
    ("minecraft:pig_sound_variant", &["pig_sound_variant"]),
    ("minecraft:chicken_variant", &["chicken_variant"]),
    (
        "minecraft:chicken_sound_variant",
        &["chicken_sound_variant"],
    ),
    ("minecraft:cat_sound_variant", &["cat_sound_variant"]),
    ("minecraft:wolf_sound_variant", &["wolf_sound_variant"]),
    (
        "minecraft:zombie_nautilus_variant",
        &["zombie_nautilus_variant"],
    ),
    ("minecraft:trade_set", &["trade_set"]),
    ("minecraft:villager_trade", &["villager_trade"]),
    ("minecraft:painting_variant", &["painting_variant"]),
    ("minecraft:dimension", &["dimension"]),
    ("minecraft:dimension_type", &["dimension_type"]),
    ("minecraft:item_modifier", &["item_modifier"]),
    ("minecraft:slot_source", &["slot_source"]),
    ("minecraft:jukebox_song", &["jukebox_song"]),
    ("minecraft:dialog", &["dialog"]),
    ("minecraft:timeline", &["timeline"]),
    ("minecraft:world_clock", &["world_clock"]),
    ("minecraft:test_environment", &["test_environment"]),
    ("minecraft:test_instance", &["test_instance"]),
    (
        "minecraft:sulfur_cube_archetype",
        &["sulfur_cube_archetype"],
    ),
    ("minecraft:block_transformer", &["block_transformer"]),
    ("minecraft:block_sound_set", &["block_sound_set"]),
    ("minecraft:trial_spawner", &["trial_spawner"]),
    ("minecraft:worldgen/biome", &["worldgen/biome"]),
    (
        "minecraft:worldgen/block_state_provider",
        &["worldgen/block_state_provider"],
    ),
    (
        "minecraft:worldgen/carver",
        &["worldgen/carver", "worldgen/configured_carver"],
    ),
    (
        "minecraft:worldgen/density_function",
        &["worldgen/density_function"],
    ),
    (
        "minecraft:worldgen/feature",
        &["worldgen/feature", "worldgen/configured_feature"],
    ),
    (
        "minecraft:worldgen/flat_level_generator_preset",
        &["worldgen/flat_level_generator_preset"],
    ),
    (
        "minecraft:worldgen/material_condition",
        &["worldgen/material_condition"],
    ),
    (
        "minecraft:worldgen/material_rule",
        &["worldgen/material_rule"],
    ),
    (
        "minecraft:worldgen/multi_noise_biome_source_parameter_list",
        &["worldgen/multi_noise_biome_source_parameter_list"],
    ),
    ("minecraft:worldgen/noise", &["worldgen/noise"]),
    (
        "minecraft:worldgen/noise_settings",
        &["worldgen/noise_settings"],
    ),
    (
        "minecraft:worldgen/placed_feature",
        &["worldgen/placed_feature"],
    ),
    (
        "minecraft:worldgen/processor_list",
        &["worldgen/processor_list"],
    ),
    (
        "minecraft:worldgen/structure",
        &["worldgen/structure", "worldgen/configured_structure"],
    ),
    (
        "minecraft:worldgen/structure_set",
        &["worldgen/structure_set"],
    ),
    (
        "minecraft:worldgen/template_pool",
        &["worldgen/template_pool"],
    ),
    (
        "minecraft:worldgen/world_preset",
        &["worldgen/world_preset"],
    ),
];

/// Field names that vanilla levelgen/datapack codecs declare as `Codec.LONG`.
///
/// NBT integers are not self-describing, so a JSON `5` may become either an `IntTag` or a
/// `LongTag`. Vanilla normalizes through the element codec; without it, small values of a
/// known long field would be sent as `IntTag` where vanilla sends `LongTag`.
const LONG_FIELDS: &[&str] = &["seed"];

fn json_number_to_nbt_tag(num: &serde_json::Number, field: Option<&str>) -> NbtTag {
    if field.is_some_and(|name| LONG_FIELDS.contains(&name)) {
        return num.as_i64().map_or_else(
            || {
                num.as_f64()
                    .map_or(NbtTag::Long(0), |value| NbtTag::Long(value as i64))
            },
            NbtTag::Long,
        );
    }
    num.as_i64().map_or_else(
        || num.as_f64().map_or(NbtTag::Int(0), NbtTag::Double),
        |i| {
            if i32::try_from(i).is_ok() {
                NbtTag::Int(i as i32)
            } else {
                NbtTag::Long(i)
            }
        },
    )
}

/// Converts a JSON `serde_json::Value` into an `NbtTag`.
///
/// Returns `None` for JSON `null`, which has no NBT equivalent. Writing
/// [`NbtTag::End`] into a compound or list would truncate the encoded stream,
/// so object keys and list elements holding `null` are dropped instead.
#[must_use]
pub fn json_to_nbt_tag(v: &Value) -> Option<NbtTag> {
    json_to_nbt_tag_field(v, None)
}

fn json_to_nbt_tag_field(v: &Value, field: Option<&str>) -> Option<NbtTag> {
    match v {
        Value::Null => None,
        Value::Bool(b) => Some(NbtTag::Byte(i8::from(*b))),
        Value::Number(num) => Some(json_number_to_nbt_tag(num, field)),
        Value::String(s) => Some(NbtTag::String(s.clone().into())),
        Value::Array(arr) => {
            let list = arr
                .iter()
                .filter_map(|value| json_to_nbt_tag_field(value, None))
                .collect();
            Some(NbtTag::List(list))
        }
        Value::Object(obj) => {
            let mut compound = NbtCompound::new();
            for (k, val) in obj {
                if let Some(tag) = json_to_nbt_tag_field(val, Some(k)) {
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
            if !Identifier::is_valid_namespace(namespace) || !Identifier::is_valid_path(&stem_path)
            {
                warn!(
                    "Skipping dynamic registry file {}: invalid resource location",
                    path.display()
                );
                continue;
            }
            let entry_id = format!("{namespace}:{stem_path}");

            let Ok(content) = fs::read_to_string(&path) else {
                warn!(
                    "Skipping dynamic registry file {}: unreadable",
                    path.display()
                );
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

    // Vanilla ignores unknown data folders entirely, so only worldgen subfolders are
    // auto-discovered (for custom/modded worldgen registries). Other unrecognized folders
    // under `data/<ns>/` must not become registries.
    if let Ok(entries) = fs::read_dir(ns_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if !path.is_dir() || name != "worldgen" {
                continue;
            }

            if let Ok(sub_entries) = fs::read_dir(&path) {
                for sub_entry in sub_entries.flatten() {
                    let sub_path = sub_entry.path();
                    let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                    if sub_path.is_dir() && !visited_paths.contains(&sub_path) {
                        // Vanilla worldgen registry keys are fixed `minecraft:` ids
                        // (`Registries` constants), never derived from the pack namespace.
                        let reg_id = format!("minecraft:worldgen/{sub_name}");
                        let registry_map = registries.entry(reg_id).or_default();
                        let before = registry_map.len();
                        load_registry_dir_recursive(namespace, &sub_path, &sub_path, registry_map);
                        total_loaded += registry_map.len() - before;
                    }
                }
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
            let reg_entry = RegistryEntryData {
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
    fn long_typed_seed_fields_encode_as_long_tags() {
        let json: Value = serde_json::json!({
            "seed": 5,
            "count": 5,
            "nested": { "seed": 7, "count": 7 }
        });

        let bytes = json_to_registry_nbt_bytes(&json);
        let mut cursor = std::io::Cursor::new(&bytes[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = Nbt::read_unnamed(&mut reader).unwrap().root_tag;

        // Vanilla levelgen codecs declare `seed` as Codec.LONG; a small seed must still be
        // a LongTag, while an ordinary numeric field stays an IntTag.
        assert_eq!(nbt.get_long("seed"), Some(5));
        assert_eq!(nbt.get_int("seed"), None);
        assert_eq!(nbt.get_int("count"), Some(5));
        let nested = nbt.get_compound("nested").unwrap();
        assert_eq!(nested.get_long("seed"), Some(7));
        assert_eq!(nested.get_int("count"), Some(7));
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
        fs::write(
            temp_dir.join("lower.json"),
            r#"{"asset_id":"custom:lower"}"#,
        )
        .unwrap();
        fs::write(
            temp_dir.join("upper.JSON"),
            r#"{"asset_id":"custom:upper"}"#,
        )
        .unwrap();

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
        fs::write(temp_dir.join("good.json"), r#"{"asset_id":"custom:good"}"#).unwrap();

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
    fn unknown_worldgen_dirs_use_minecraft_registry_key() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_test_dr_wg_{}", uuid::Uuid::new_v4()));
        let ns_dir = temp_dir.join("custom");
        let reg_dir = ns_dir.join("worldgen").join("sulfur_pool");
        fs::create_dir_all(&reg_dir).unwrap();
        fs::write(
            reg_dir.join("sample.json"),
            r#"{"asset_id":"custom:sample"}"#,
        )
        .unwrap();

        let mut registries: HashMap<String, HashMap<String, RegistryEntryData>> = HashMap::new();
        let loaded = load_dynamic_registries_from_ns("custom", &ns_dir, &mut registries);

        // Vanilla registry keys are fixed `minecraft:` ids regardless of the datapack namespace.
        assert_eq!(loaded, 1);
        assert!(registries.contains_key("minecraft:worldgen/sulfur_pool"));
        assert!(!registries.contains_key("custom:worldgen/sulfur_pool"));
        assert!(
            registries["minecraft:worldgen/sulfur_pool"].contains_key("custom:sample"),
            "entry id must keep the datapack namespace"
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_instance_registry_uses_singular_directory_only() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_test_ti_{}", uuid::Uuid::new_v4()));
        let ns_dir = temp_dir.join("custom");
        let singular = ns_dir.join("test_instance");
        let plural = ns_dir.join("test_instances");
        fs::create_dir_all(&singular).unwrap();
        fs::create_dir_all(&plural).unwrap();
        fs::write(singular.join("a.json"), r#"{"type":"minecraft:function"}"#).unwrap();
        fs::write(plural.join("b.json"), r#"{"type":"minecraft:function"}"#).unwrap();

        let mut registries: HashMap<String, HashMap<String, RegistryEntryData>> = HashMap::new();
        load_dynamic_registries_from_ns("custom", &ns_dir, &mut registries);

        // Vanilla loads registry entries from the singular test_instance directory only.
        let entries = &registries["minecraft:test_instance"];
        assert!(entries.contains_key("custom:a"));
        assert!(!entries.contains_key("custom:b"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn unknown_non_worldgen_folders_are_ignored() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_test_dr_uf_{}", uuid::Uuid::new_v4()));
        let ns_dir = temp_dir.join("custom");
        let config_dir = ns_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("options.json"), r#"{"value":1}"#).unwrap();

        let mut registries: HashMap<String, HashMap<String, RegistryEntryData>> = HashMap::new();
        let loaded = load_dynamic_registries_from_ns("custom", &ns_dir, &mut registries);

        // Vanilla ignores unknown data folders; they must not become registries.
        assert_eq!(loaded, 0);
        assert!(!registries.contains_key("custom:config"));
        assert!(registries.is_empty());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn plural_directory_aliases_are_not_loaded() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_test_plural_{}", uuid::Uuid::new_v4()));
        let ns_dir = temp_dir.join("custom");
        let singular = ns_dir.join("enchantment");
        let plural = ns_dir.join("enchantments");
        fs::create_dir_all(&singular).unwrap();
        fs::create_dir_all(&plural).unwrap();
        fs::write(singular.join("a.json"), r#"{}"#).unwrap();
        fs::write(plural.join("b.json"), r#"{}"#).unwrap();

        let mut registries: HashMap<String, HashMap<String, RegistryEntryData>> = HashMap::new();
        load_dynamic_registries_from_ns("custom", &ns_dir, &mut registries);

        // Vanilla reads only the singular registry path; plural aliases are ignored.
        let entries = &registries["minecraft:enchantment"];
        assert!(entries.contains_key("custom:a"));
        assert!(!entries.contains_key("custom:b"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn configured_carver_entries_load_into_canonical_registry() {
        let temp_dir =
            std::env::temp_dir().join(format!("pumpkin_test_dr_cc_{}", uuid::Uuid::new_v4()));
        let ns_dir = temp_dir.join("custom");
        let legacy_dir = ns_dir.join("worldgen").join("configured_carver");
        fs::create_dir_all(&legacy_dir).unwrap();
        fs::write(
            legacy_dir.join("canyon.json"),
            r#"{"type":"minecraft:canyon"}"#,
        )
        .unwrap();

        let mut registries: HashMap<String, HashMap<String, RegistryEntryData>> = HashMap::new();
        load_dynamic_registries_from_ns("custom", &ns_dir, &mut registries);

        // The legacy folder loads into the real `worldgen/carver` registry; there is no
        // separate `worldgen/configured_carver` registry in this version.
        assert!(
            registries
                .get("minecraft:worldgen/carver")
                .is_some_and(|entries| entries.contains_key("custom:canyon"))
        );
        assert!(!registries.contains_key("minecraft:worldgen/configured_carver"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
