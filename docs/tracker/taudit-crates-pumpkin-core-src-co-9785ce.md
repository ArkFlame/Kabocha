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
