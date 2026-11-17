# AUDIT: crates/pumpkin-core/src/data/datapack/damage_type_loader.rs

Tracker row: audit / damage_type_loader.rs (coverage AUDIT).
Audited against vanilla `DamageType` (CODEC in `world/damage/DamageType.java`),
`RegistryDataLoader` element/file id rules and Paper/Spigot (unmodified vanilla
codec path). No `TODO`/`FIXME`/`vanilla` markers remain in the file (the
tracker note of 4 markers is stale: the file has none at HEAD).

## Fixed in this pass

1. `to_registry_nbt_bytes` (damage_type_loader.rs:158) wrote `exhaustion` as a
   `Double` tag and always emitted it. Vanilla uses `Codec.FLOAT
   .optionalFieldOf("exhaustion", 0.0F)`, which serialises a `Float` tag and
   omits the field when it equals the default. Fixed: `put_float` + omit at 0.0.
   Commit 09913a138, test `to_registry_nbt_matches_vanilla_exhaustion_encoding`.
2. `load_damage_types_recursive` (damage_type_loader.rs:214) accepted any file
   name as an id and silently dropped unreadable / unparsable files. Vanilla
   builds a `ResourceLocation` from the datapack path (invalid characters are an
   error) and logs a parse failure per file; the sibling
   `dynamic_registry_loader` already does both. Fixed: `Identifier`
   namespace/path validation + `tracing::warn!` on every skip. Commit 9f59c6a75,
   test `load_damage_types_skips_invalid_resource_location`.

## Remaining gaps (not fixed)

1. damage_type_loader.rs:152 - `to_damage_type` `Box::leak`s `message_id` on
   every call, and `build_damage_type_registry` (mod.rs:1842) leaks a second
   copy of the whole `DamageType` per reload. Vanilla keeps the record in the
   registry with no leak. Suggested fix: leak the message id once per registry
   build (interning map) and store `&'static str`, or make
   `DamageType::message_id` a `Cow`/`String`. Size M.
2. mod.rs:1817-1838 - custom damage types get `next_custom_id` starting at 51
   with `saturating_add`, so more than 205 custom types collide on id 255 and
   overrides of vanilla ids depend on `from_name` matching the path. Vanilla
   allocates ids from the load-order registry (`RegistryDataLoader`), and the
   client accepts arbitrary ids. Suggested fix: allocate from a free-id set and
   log a warning when ids run out. Size S/M. Touches mod.rs, not this file.
3. damage_type_loader.rs:221 - `.JSON` is accepted case-insensitively, vanilla
   is case-sensitive and would ignore the file. Harmless superset; leave unless
   strict parity is wanted. Size XS.
4. mod.rs:2207 loads both `damage_type/` and `damage_types/` directories;
   vanilla only uses `damage_type/`. Intentional compatibility alias, documented
   here only. Size XS.
5. dynamic_registry_loader.rs:239 - `json_to_nbt_tag` maps every JSON float to a
   `Double` tag, so dynamic registries whose codec declares a `Codec.FLOAT`
   field drift from vanilla bytes (same class of bug as fix 1 but generic).
   Suggested fix: per-registry typed encoding, or a float hint per field. Size M.
6. damage_type_loader.rs:206 - a JSON file that overrides an existing id still
   counts in `load_damage_types_from_dir`'s return only when it is new; the
   count is used purely for admin statistics. Cosmetic.
