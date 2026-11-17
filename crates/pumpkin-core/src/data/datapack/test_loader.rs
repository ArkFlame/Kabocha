use std::collections::HashMap;
use std::fs;
use std::path::Path;

use pumpkin_data::registry::RegistryEntryData;
use pumpkin_nbt::{Nbt, NbtCompound, tag::NbtTag};
use serde_json::Value;

pub use pumpkin_gametest::model::{GameTestDefinition as TestInstance, GameTestRotation, TestType};
use tracing::warn;

pub type TestInstanceRegistry = HashMap<String, TestInstance>;

/// Loads test environments into the dynamic registry before on-disk packs.
pub fn load_test_environments<S: std::hash::BuildHasher>(
    registry: &mut HashMap<String, RegistryEntryData, S>,
) -> usize {
    let before = registry.len();
    for &id in pumpkin_data::test_environment::all_test_environment_names() {
        let Some(content) = pumpkin_data::test_environment::get_test_environment_json(id) else {
            warn!("Test environment '{id}' has no JSON payload");
            continue;
        };
        match serde_json::from_str::<Value>(content) {
            Ok(value) => {
                registry.insert(
                    id.to_owned(),
                    RegistryEntryData {
                        entry_id: id.to_owned(),
                        data: Some(super::dynamic_registry_loader::json_to_registry_nbt_bytes(
                            &value,
                        )),
                    },
                );
            }
            Err(error) => warn!("Failed to parse test environment '{id}': {error}"),
        }
    }
    registry.len() - before
}

/// Loads all test instances embedded in the binary at compile time.
///
/// Ids are fully qualified, so entries here can be overridden by later
/// [`load_test_instances_from_dir`] calls for the same id.
pub fn load_embedded_test_instances(registry: &mut TestInstanceRegistry) -> usize {
    let before = registry.len();

    crate::server::builtin_test_functions::ensure_builtin_test_functions_registered();
    for &id in pumpkin_world::test_instance::all_names() {
        let Some(content) = pumpkin_world::test_instance::json(id) else {
            warn!("Embedded test instance '{id}' has no JSON payload");
            continue;
        };

        match serde_json::from_str::<TestInstance>(content) {
            Ok(instance) if instance.is_valid() => {
                registry.insert(id.to_owned(), instance);
            }
            Ok(_) => warn!("Embedded test instance '{id}' failed validation"),
            Err(e) => warn!("Failed to parse embedded test instance '{id}': {e}"),
        }
    }

    registry.len() - before
}

pub fn load_test_instances_from_dir(
    namespace: &str,
    test_instance_dir: &Path,
    registry: &mut TestInstanceRegistry,
) -> usize {
    if !test_instance_dir.is_dir() {
        return 0;
    }

    let before = registry.len();
    load_test_instances_recursive(namespace, test_instance_dir, test_instance_dir, registry);
    registry.len() - before
}

fn load_test_instances_recursive(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry: &mut TestInstanceRegistry,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();

        if file_type.is_dir() {
            load_test_instances_recursive(namespace, base_dir, &path, registry);
        } else if file_type.is_file()
            // Vanilla matches the lowercase .json suffix only, so Foo.JSON is ignored
            // rather than registered as ns:Foo.JSON.
            && path.extension().is_some_and(|ext| ext.to_str() == Some("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();

            if let Some(stem) = stem_path.strip_suffix(".json") {
                stem_path = stem.to_string();
            }

            let stem_path = stem_path.replace('\\', "/");
            let test_instance_id = format!("{namespace}:{stem_path}");

            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<TestInstance>(&content) {
                    Ok(instance) if instance.is_valid() => {
                        registry.insert(test_instance_id, instance);
                    }
                    Ok(_) => warn!("Test instance '{}' failed validation", path.display()),
                    Err(error) => warn!(
                        "Failed to parse test instance '{}': {error}",
                        path.display()
                    ),
                },
                Err(error) => warn!("Failed to read test instance '{}': {error}", path.display()),
            }
        }
    }
}

/// Encodes the datapack test definition using the same map shape consumed by
/// `GameTestInstance.CODEC` / `TestData.CODEC` in vanilla.
///
/// `RegistryData` expects an unnamed NBT compound payload for each registry entry.
#[must_use]
pub fn to_registry_entry(entry_id: String, instance: &TestInstance) -> RegistryEntryData {
    let mut nbt = NbtCompound::new();
    nbt.put_string("type", instance.instance_type.serialized_name().to_string());
    nbt.put("environment", json_value_to_nbt(&instance.environment));
    if let Some(function) = &instance.function {
        nbt.put_string("function", function.clone());
    }
    nbt.put_string("structure", instance.structure.clone());
    nbt.put_int("max_ticks", instance.max_ticks);
    nbt.put_int("setup_ticks", instance.setup_ticks);
    nbt.put_bool("required", instance.required);
    nbt.put_string("rotation", instance.rotation.serialized_name().to_string());
    nbt.put_bool("manual_only", instance.manual_only);
    nbt.put_int("max_attempts", instance.max_attempts);
    nbt.put_int("required_successes", instance.required_successes);
    nbt.put_bool("sky_access", instance.sky_access);
    nbt.put_int("padding", instance.padding);

    RegistryEntryData {
        entry_id,
        data: Some(Nbt::from(nbt).write_unnamed().to_vec().into_boxed_slice()),
    }
}

fn json_value_to_nbt(value: &Value) -> NbtTag {
    match value {
        Value::Null => NbtTag::String(String::new().into()),
        Value::Bool(value) => NbtTag::Byte(i8::from(*value)),
        Value::Number(value) => value.as_i64().map_or_else(
            || {
                value.as_u64().map_or_else(
                    || NbtTag::Double(value.as_f64().unwrap_or_default()),
                    |value| {
                        i32::try_from(value).map_or_else(
                            |_| {
                                i64::try_from(value)
                                    .map_or_else(|_| NbtTag::Double(value as f64), NbtTag::Long)
                            },
                            NbtTag::Int,
                        )
                    },
                )
            },
            |value| i32::try_from(value).map_or(NbtTag::Long(value), NbtTag::Int),
        ),
        Value::String(value) => NbtTag::String(value.clone().into()),
        Value::Array(values) => NbtTag::List(values.iter().map(json_value_to_nbt).collect()),
        Value::Object(values) => {
            let mut compound = NbtCompound::new();
            for (name, value) in values {
                compound.put(name, json_value_to_nbt(value));
            }
            NbtTag::Compound(compound)
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn loads_block_based_test_instance_with_vanilla_defaults() {
        let dir = tempfile::tempdir().expect("temp dir");
        let nested = dir.path().join("mob_ai");

        fs::create_dir_all(&nested).expect("create nested test_instance dir");

        fs::write(
            nested.join("some_ai_test.json"),
            r#"{
                "type": "minecraft:block_based",
                "environment": "minecraft:default",
                "structure": "pumpkin:some_ai_test",
                "max_ticks": 200
            }"#,
        )
        .expect("write test instance");

        let mut registry = TestInstanceRegistry::new();
        let loaded = load_test_instances_from_dir("pumpkin", dir.path(), &mut registry);

        assert_eq!(loaded, 1);

        let instance = registry
            .get("pumpkin:mob_ai/some_ai_test")
            .expect("test instance should be registered");

        assert_eq!(instance.instance_type, TestType::BlockBased);
        assert_eq!(
            instance.environment,
            Value::String("minecraft:default".into())
        );
        assert_eq!(instance.structure, "pumpkin:some_ai_test");
        assert_eq!(instance.max_ticks, 200);
        assert_eq!(instance.setup_ticks, 0);
        assert!(instance.required);
        assert_eq!(instance.rotation, GameTestRotation::None);
        assert!(!instance.manual_only);
        assert_eq!(instance.max_attempts, 1);
        assert_eq!(instance.required_successes, 1);
        assert!(!instance.sky_access);
        assert_eq!(instance.padding, 0);
    }

    #[test]
    fn loads_function_test_instance_with_vanilla_defaults() {
        let dir = tempfile::tempdir().expect("temp dir");

        fs::write(
            dir.path().join("always_pass.json"),
            r#"{
                "type": "minecraft:function",
                "environment": "minecraft:default",
                "function": "minecraft:always_pass",
                "max_ticks": 1,
                "required": false,
                "setup_ticks": 1,
                "structure": "minecraft:empty"
            }"#,
        )
        .expect("write test instance");

        let mut registry = TestInstanceRegistry::new();
        let loaded = load_test_instances_from_dir("minecraft", dir.path(), &mut registry);

        assert_eq!(loaded, 1);

        let instance = registry
            .get("minecraft:always_pass")
            .expect("test instance should be registered");

        assert_eq!(instance.instance_type, TestType::Function);
        assert_eq!(
            instance.environment,
            Value::String("minecraft:default".into())
        );
        assert_eq!(instance.function.as_deref(), Some("minecraft:always_pass"));
        assert_eq!(instance.structure, "minecraft:empty");
        assert_eq!(instance.max_ticks, 1);
        assert_eq!(instance.setup_ticks, 1);
        assert!(!instance.required);
        assert_eq!(instance.rotation, GameTestRotation::None);
        assert!(!instance.manual_only);
        assert_eq!(instance.max_attempts, 1);
        assert_eq!(instance.required_successes, 1);
        assert!(!instance.sky_access);
        assert_eq!(instance.padding, 0);
    }

    #[test]
    fn loads_embedded_test_instances_including_always_pass() {
        let mut registry = TestInstanceRegistry::new();
        let count = load_embedded_test_instances(&mut registry);
        assert!(count > 0);

        let always_pass = registry
            .get("minecraft:always_pass")
            .expect("minecraft:always_pass should be successfully parsed and registered");
        assert_eq!(always_pass.instance_type, TestType::Function);
        assert_eq!(
            always_pass.function.as_deref(),
            Some("minecraft:always_pass")
        );
        assert_eq!(always_pass.structure, "minecraft:empty");
    }

    #[test]
    fn to_registry_entry_writes_unnamed_compound_root() {
        let instance: TestInstance = serde_json::from_str(
            r#"{
                "type": "minecraft:function",
                "environment": "minecraft:default",
                "function": "minecraft:always_pass",
                "structure": "minecraft:empty",
                "max_ticks": 1,
                "max_attempts": 3,
                "required_successes": 2,
                "sky_access": true
            }"#,
        )
        .expect("parse test instance");

        let entry = to_registry_entry("minecraft:always_pass".to_string(), &instance);
        let data = entry.data.expect("registry payload");

        let mut cursor = std::io::Cursor::new(&data[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = Nbt::read_unnamed(&mut reader)
            .expect("vanilla sends registry entry payloads as unnamed NBT")
            .root_tag;

        assert_eq!(nbt.get_string("type"), Some("minecraft:function"));
        assert_eq!(nbt.get_string("environment"), Some("minecraft:default"));
        assert_eq!(nbt.get_string("function"), Some("minecraft:always_pass"));
        assert_eq!(nbt.get_string("structure"), Some("minecraft:empty"));
        assert_eq!(nbt.get_string("rotation"), Some("none"));
        assert_eq!(nbt.get_int("max_ticks"), Some(1));
        assert_eq!(nbt.get_int("max_attempts"), Some(3));
        assert_eq!(nbt.get_int("required_successes"), Some(2));
        assert_eq!(nbt.get_bool("required"), Some(true));
        assert_eq!(nbt.get_bool("sky_access"), Some(true));
        assert_eq!(nbt.get_int("padding"), Some(0));
        // A named root would leave two extra bytes explaining the empty name, so a
        // full read proves the payload is exactly one unnamed compound.
        assert_eq!(cursor.position() as usize, data.len());
    }

    #[test]
    fn load_test_instances_ignores_uppercase_json_extension() {
        let dir = tempfile::tempdir().expect("temp dir");
        let body = r#"{
            "type": "minecraft:block_based",
            "environment": "minecraft:default",
            "structure": "pumpkin:some_ai_test",
            "max_ticks": 20
        }"#;
        fs::write(dir.path().join("lower.json"), body).expect("write lower case instance");
        fs::write(dir.path().join("upper.JSON"), body).expect("write upper case instance");

        let mut registry = TestInstanceRegistry::new();
        let loaded = load_test_instances_from_dir("pumpkin", dir.path(), &mut registry);

        assert_eq!(loaded, 1);
        assert!(registry.contains_key("pumpkin:lower"));
        assert!(!registry.contains_key("pumpkin:upper.JSON"));
    }

    fn try_create_dir_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link)
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_dir(target, link)
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "symlinks not supported",
            ))
        }
    }

    fn try_create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link)
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(target, link)
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "symlinks not supported",
            ))
        }
    }

    #[test]
    fn load_test_instances_ignores_symlinks_and_loads_regular_files() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        let test_dir = root.join("test_instance");
        let sub_dir = test_dir.join("sub");
        fs::create_dir_all(&sub_dir).expect("create sub dir");

        fs::write(
            sub_dir.join("valid.json"),
            r#"{
                "type": "minecraft:block_based",
                "environment": "minecraft:default",
                "structure": "pumpkin:valid",
                "max_ticks": 100
            }"#,
        )
        .expect("write valid test instance");

        let external_dir = root.join("external");
        fs::create_dir_all(&external_dir).expect("create external dir");
        fs::write(
            external_dir.join("sym.json"),
            r#"{
                "type": "minecraft:block_based",
                "environment": "minecraft:default",
                "structure": "pumpkin:sym",
                "max_ticks": 100
            }"#,
        )
        .expect("write sym test instance");

        let sym_dir = test_dir.join("sym_dir");
        let sym_dir_created = try_create_dir_symlink(&external_dir, &sym_dir).is_ok();

        let sym_file = test_dir.join("sym_file.json");
        let sym_file_created =
            try_create_file_symlink(&external_dir.join("sym.json"), &sym_file).is_ok();

        let mut registry = TestInstanceRegistry::new();
        let count = load_test_instances_from_dir("pumpkin", &test_dir, &mut registry);

        assert_eq!(count, 1);
        assert!(registry.contains_key("pumpkin:sub/valid"));

        if sym_dir_created {
            assert!(!registry.contains_key("pumpkin:sym_dir/sym"));
        }
        if sym_file_created {
            assert!(!registry.contains_key("pumpkin:sym_file"));
        }
    }
}
