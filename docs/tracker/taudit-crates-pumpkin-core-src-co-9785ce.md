# Audit: crates/pumpkin-core/src/command/commands/data.rs

Vanilla reference: `net.minecraft.server.commands.DataCommands` (1.21.x), `NbtPredicate`, `NbtUtils.toPrettyComponent`, `NbtString.quoteAndEscape`.

Fixed in this audit:
- Java UTF-16 string lengths / substring indices (`/data get` string length, `data modify ... string` start/end). Commit `6f9120ea3`.
- SNBT string display now escapes `\\` as well as `"`, like vanilla `NbtString.quoteAndEscape`. Commit `d873bc405`.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| data.rs:229-241, 273-275 | `EntityDataAccessor.getData()` = `entity.saveWithoutId(new CompoundTag())`; `SelectedItem` is only synthesised by `NbtPredicate.getEntityTagWithSelectedItem` for the `nbt=` selector | `data get/merge/remove/modify ... entity` sees a synthetic `SelectedItem` on players, so `/data get entity @s SelectedItem` succeeds where vanilla reports "tag doesn't exist" | keep the helper for `entity_selector/mod.rs:572`, build plain `write_nbt` in `EntityDataAccessor::get_data` | S (verify against a real server first: PR #4333 deliberately wired both) |
| data.rs:481-495 | `data modify ... string` reads the source through `NbtElement.asString()`, which only `NbtString` implements; other tag types throw `commands.data.modify.expected_value` | `get_as_text` also accepts Byte/Short/Int/Long/Float/Double, so numeric sources silently stringify | restrict to `NbtTag::String`, keep `ERROR_EXPECTED_VALUE` for the rest | S | **FIXED** (get_as_text string-only, test added)
| data.rs:114-201 | pretty SNBT arrays use `StringNbtWriter`: empty array renders `[B;]`, `[I;]`, `[L;]` | empty arrays render `[B; ]` (trailing space from the unconditional `"; "` prefix) | only append `", "` separators between elements and emit `";"` via the writer path | S (cosmetic) - **fixed in 94ceb5731** |
| data.rs:151-170 | compounds iterate `CompoundTag.tags` (HashMap order, arbitrary) | keys are sorted alphabetically, so output order differs from vanilla but is deterministic | leave as is | - |
| data.rs:497-521 | `DataCommands` substring failure reports the resolved indices | error message passes the raw `start`/`end` (negative values not normalised, `end` defaults to `len`) | pass `abs_start`/`abs_end` | S (verify message text) |
| data.rs:263-271 | `data remove entity` calls `setData` with the full modified compound, so removals at any path depth persist | entity path: re-serialises, then removes only the top-level keys that disappeared (`removed`). A path that removes a nested element (e.g. `Attributes[0]`) sends no key and depends on `read_nbt_non_mut` applying the nested change | verify nested removals against a live entity; if unapplied, add a nested-aware `remove_nbt` | M (needs runtime repro) |
| data.rs:310-348 | `BlockDataAccessor.setData` loads into the live block entity (`BlockEntity.loadWithComponents` + `setChanged`) | block entities are rebuilt from NBT via `block_entity_from_nbt`, which is lossy for BEs whose NBT round trip is incomplete (e.g. pending ticks, inventories with custom components) and closes open screens | keep unless a concrete lossy BE is found | M (verify) |
| data.rs:413-431 | `CommandStorage` writes are persisted with the level | `StorageDataAccessor` mutates the in-memory map only; persisting relies on the server-side save loop | verify command storage is saved on world save | S (verify) |

## Second pass (2026-11-17)

Fixed:
- data.rs:492-493, 782-783: error arguments now interpolate the tag with the SNBT Display impl instead of Rust Debug (Expected a value: got {a: 1}). Commit 7a0acdef3.
- data.rs:114-205: empty [B;], [I;], [L;] no longer render a trailing space; test empty_typed_nbt_arrays_render_without_trailing_space. Commit 94ceb5731.

New gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| data.rs:300-309, 386-399, 453-464 | Component.translatable(..., scale, value) renders the double through Double.toString: 2.0, 0.5, 1.0E-4 | format!("{scale:.2}") prints 2.00/0.50, so /data get ... <scale> feedback text differs | add pumpkin_util::text::java_double_to_string (shortest repr, always a .0, Java exponent rules) and use it | S | **FIXED** (java_double_to_string)
| data.rs:482-496 | data modify ... string stringifies numerics with Float.toString/Double.toString | Rust f32/f64::to_string drops the .0 (1.0f becomes "1") | reuse the same Java number formatter | S | **FIXED** (numeric tags now rejected)
| data.rs:767-770 | SET uses tags.getLast(); an empty source list is not silently written | unwrap_or(NbtTag::End) writes an End tag into the target compound when the source path matches nothing, which can be serialised into entity/storage NBT | return an error (likely ERROR_GET_NON_EXISTENT) for an empty source list; confirm the vanilla error text first | S (verify) |
| pumpkin-command/src/argument_types/nbt_path.rs:632-636 | commands.data.modify.expected_list arg is the SNBT of the offending tag | format!("{target_tag:?}") leaks Rust Debug (same bug just fixed in data.rs) | target_tag.to_string() | S |

## Gap status
- Entity data accessor (`data.rs:229-241,273-275`): STILL OPEN (M).
- `modify string` on non-string tags (`data.rs:481-495`): STILL OPEN (S).
- Empty arrays render `[B; ]` (`data.rs:114-201`): STILL OPEN (S).
- Compound key order is HashMap order (`data.rs:151-170`): STILL OPEN (S).
- Substring error indices (`data.rs:497-521`): FIXED (UNVERIFIED; reports resolved indices like vanilla).
- `remove entity` at depth (`data.rs:263-271`): STILL OPEN (Second pass section not re-read this pass).
- Block data live load (`data.rs:310-348`): STILL OPEN (M).
- Storage persistence (`data.rs:413-431`): STILL OPEN (M).
- Double formatting in translatable (`data.rs:300-309,386-399,453-464`): STILL OPEN (S).
- Numeric to_string in `modify string` (`data.rs:482-496`): STILL OPEN (S).
- `SET` with empty source writes End (`data.rs:767-770`): STILL OPEN (S).
- `expected_list` argument text (`nbt_path.rs:632-636`): STILL OPEN (S).

## Result

Verified against `.vendor/vanilla-decompiled/net/minecraft` (repo target `CURRENT_MC_VERSION = V_26_3`).

- Entity data accessor (`data.rs:229-241,273-275`): no change. `EntityDataAccessor.getData()` calls `NbtPredicate.getEntityTagToCompare`, which at `NbtPredicate.java:42-55` adds a synthetic `SelectedItem` for players, so the current `entity_nbt_with_selected_item` matches vanilla. The 1.21.x claim in the first table does not apply to this version.
- `modify string` on non-string tags / numeric to_string (`data.rs:481-495`): fixed. `DataCommands.getAsText` (lines 172-178) returns the raw value for `StringTag` and `PrimitiveTag.toString()` for every numeric tag, throwing `expected_value` only for compound/list/array/end. `get_as_text` now stringifies Byte/Short/Int/Long/Float/Double with Java number formatting and SNBT suffixes; test `modify_string_source_stringifies_primitives` replaces the old rejection test.
- Empty arrays render `[B; ]` (`data.rs:114-201`): no change. Already fixed; `StringTagVisitor.visitByteArray/IntArray/LongArray` emits `[B;]`/`[I;]`/`[L;]` for empty arrays, matching `snbt_colorful_display`.
- Compound key order (`data.rs:151-170`): no change. Cosmetic only; keys are sorted for determinism while vanilla iterates an arbitrary map.
- Double formatting in translatable (`data.rs:300-309,386-399,453-464`): fixed. All three vanilla `getPrintSuccess(path, scale, value)` implementations use `String.format(Locale.ROOT, "%.2f", scale)`; replaced `java_double_to_string(scale)` with `format!("{scale:.2}")` at all three sites.
- `SET` with empty source writes End (`data.rs:767-770`): fixed. Vanilla uses `Iterables.getLast(source)` and fails before writing; SET now returns `ERROR_GET_NON_EXISTENT` naming the source path instead of inserting `NbtTag::End`.
- `expected_list` argument text (`nbt_path.rs:632-636`): fixed. Vanilla `ERROR_EXPECTED_LIST.create(targetTag)` renders the tag through its `toString()` (SNBT); replaced Rust `Debug` with `Display`.

Still open:
- `remove entity` at depth (`data.rs:263-271`): vanilla mutates the full compound then calls `setData`; whether nested removals persist needs a live entity repro, and a nested-aware `remove_nbt` lives outside these two files.
- Block data live load (`data.rs:310-348`): no concrete lossy block entity identified; current NBT rebuild approach retained, a real fix needs cross-crate `block_entity_from_nbt` changes.
- Storage persistence (`data.rs:413-431`): deciding whether command storage is written on world save requires the server save loop, outside this unit.


## Result (re-verification, this pass)

Re-checked every STILL OPEN row against `.vendor/vanilla-decompiled` (target `V_26_3`). All rows that can be fixed in these two files were already fixed in the prior pass and are confirmed correct; no .rs file needed changes this pass, so no rustfmt was run.

- Entity data accessor: no change. `EntityDataAccessor.getData` (EntityDataAccessor.java:68-71) calls `NbtPredicate.getEntityTagToCompare`, which adds a synthetic `SelectedItem` for players (NbtPredicate.java:42-55), matching `entity_nbt_with_selected_item`.
- `modify string` on non-string tags / numeric to_string: fixed. `DataCommands.getAsText` (172-178) returns the raw value for `StringTag` and `PrimitiveTag.toString()` for numeric tags; `StringTagVisitor` appends the `b`/`s`/`L`/`f`/`d` suffixes, matching `get_as_text` (data.rs:565-583). Compound/list/array/end throw `expected_value`.
- Empty arrays / float+double display / quoted keys / escaped backslashes: no change, already fixed and covered by tests in data.rs.
- Substring error indices: fixed. Reports resolved `abs_start`/`abs_end`.
- Double formatting in translatable: fixed. All three accessors use `String.format(Locale.ROOT, "%.2f", scale)` (EntityDataAccessor.java:87, BlockDataAccessor.java:104, StorageDataAccessor.java:77), matching `format!("{scale:.2}")`.
- `SET` with empty source: fixed. `DataCommands.java:133` uses `Iterables.getLast(source)`; `execute_modify` returns `ERROR_GET_NON_EXISTENT` before writing.
- `expected_list` argument text: fixed. `nbt_path.rs:632-636` renders the tag via `Display` (SNBT).

Still open (out of unit / needs runtime repro):
- `remove entity` at depth: vanilla `removeData` (DataCommands.java:377-390) mutates the full compound then calls `setData`; Pumpkin already passes the full compound to `set_data`, so persistence of nested removals depends on entity `read_nbt_non_mut`, which lives outside these two files and needs a live-entity repro.
- Block data live load: vanilla `BlockDataAccessor.setData` loads into the live block entity; a lossless fix needs cross-crate `block_entity_from_nbt` changes outside this unit.
- Storage persistence: whether `command_storage` is written on world save depends on the server save loop, outside this unit.
