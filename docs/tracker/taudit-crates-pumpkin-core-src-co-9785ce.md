# Audit: crates/pumpkin-core/src/command/commands/data.rs

Vanilla reference: `net.minecraft.server.commands.DataCommands` (1.21.x), `NbtPredicate`, `NbtUtils.toPrettyComponent`, `NbtString.quoteAndEscape`.

Fixed in this audit:
- Java UTF-16 string lengths / substring indices (`/data get` string length, `data modify ... string` start/end). Commit `6f9120ea3`.
- SNBT string display now escapes `\\` as well as `"`, like vanilla `NbtString.quoteAndEscape`. Commit `d873bc405`.

Remaining gaps:

| Location | Vanilla | Gap | Fix | Size |
|---|---|---|---|---|
| data.rs:229-241, 273-275 | `EntityDataAccessor.getData()` = `entity.saveWithoutId(new CompoundTag())`; `SelectedItem` is only synthesised by `NbtPredicate.getEntityTagWithSelectedItem` for the `nbt=` selector | `data get/merge/remove/modify ... entity` sees a synthetic `SelectedItem` on players, so `/data get entity @s SelectedItem` succeeds where vanilla reports "tag doesn't exist" | keep the helper for `entity_selector/mod.rs:572`, build plain `write_nbt` in `EntityDataAccessor::get_data` | S (verify against a real server first: PR #4333 deliberately wired both) |
| data.rs:481-495 | `data modify ... string` reads the source through `NbtElement.asString()`, which only `NbtString` implements; other tag types throw `commands.data.modify.expected_value` | `get_as_text` also accepts Byte/Short/Int/Long/Float/Double, so numeric sources silently stringify | restrict to `NbtTag::String`, keep `ERROR_EXPECTED_VALUE` for the rest | S |
| data.rs:114-201 | pretty SNBT arrays use `StringNbtWriter`: empty array renders `[B;]`, `[I;]`, `[L;]` | empty arrays render `[B; ]` (trailing space from the unconditional `"; "` prefix) | only append `", "` separators between elements and emit `";"` via the writer path | S (cosmetic) |
| data.rs:151-170 | compounds iterate `CompoundTag.tags` (HashMap order, arbitrary) | keys are sorted alphabetically, so output order differs from vanilla but is deterministic | leave as is | - |
| data.rs:497-521 | `DataCommands` substring failure reports the resolved indices | error message passes the raw `start`/`end` (negative values not normalised, `end` defaults to `len`) | pass `abs_start`/`abs_end` | S (verify message text) |
| data.rs:263-271 | `data remove entity` calls `setData` with the full modified compound, so removals at any path depth persist | entity path: re-serialises, then removes only the top-level keys that disappeared (`removed`). A path that removes a nested element (e.g. `Attributes[0]`) sends no key and depends on `read_nbt_non_mut` applying the nested change | verify nested removals against a live entity; if unapplied, add a nested-aware `remove_nbt` | M (needs runtime repro) |
| data.rs:310-348 | `BlockDataAccessor.setData` loads into the live block entity (`BlockEntity.loadWithComponents` + `setChanged`) | block entities are rebuilt from NBT via `block_entity_from_nbt`, which is lossy for BEs whose NBT round trip is incomplete (e.g. pending ticks, inventories with custom components) and closes open screens | keep unless a concrete lossy BE is found | M (verify) |
| data.rs:413-431 | `CommandStorage` writes are persisted with the level | `StorageDataAccessor` mutates the in-memory map only; persisting relies on the server-side save loop | verify command storage is saved on world save | S (verify) |
