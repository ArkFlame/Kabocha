# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-734ab8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: function_loader.rs mod.rs

| mod.rs:500, 884 | `Tag#getEntries` (flattened holder set) | `get_tag_values` / `get_tag_entries` return raw `#tag` reference strings; vanilla returns the flattened member list. | Reuse the `tag_contains_entry` recursion to expand nested references. | S, CLOSED (see Result): needs its own pass |
| function_loader.rs:30, mod.rs:2465 | `FolderRepositorySource` | Loaders skip symlinked files/dirs (`entry.file_type()`), while vanilla's directory source follows them; a pack symlinking shared content silently loads nothing. | Use `path.is_dir()` / `path.is_file()` in the recursive loaders. | S, CLOSED (see Result): needs its own pass |
| mod.rs:2409 | `PackMetadataSection` format range | An incompatible `pack_format` is accepted without warning; vanilla marks the pack incompatible and refuses to enable it. | Compare `pack_format` (or min/max range) against the current format and reject/skip with a log. | S, CLOSED (see Result): needs its own pass |
This unit may edit only `function_loader.rs`. Every open gap whose fix lives in

## Result

All three rows are closed; code was implemented in earlier passes and this pass verified each against decompiled vanilla.

- Row 1 `Tag#getEntries` flattened holder set (mod.rs `get_tag_values` / `get_tag_entries`): FIXED. `collect_generic_tag_values` expands nested `#tag` references in first-seen order with a `visited` set for cycles; builtin tags from `pumpkin_data::tag::get_tag_values` are already flattened (generated tag data contains no `#` entries). Verified against `.vendor/vanilla-decompiled/net/minecraft/tags/TagLoader.java` (`tryBuildTag` populates a `LinkedHashSet` via `TagEntry.build`, which expands tag references). Unit test `generic_tag_queries_expand_nested_references_and_strip_hash`.
- Row 2 `FolderRepositorySource` symlinks (function_loader.rs recursive loaders): FIXED. The loaders use `path.is_dir()` / `path.is_file()` (which follow symlinks) instead of `entry.file_type()`; symlinked directories and files are covered by tests. Verified against `.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/FolderRepositorySource.java` (`discoverPacks` iterates `Files.newDirectoryStream` and `PackDetector.detectPackResources` follows symlinked entries).
- Row 3 `PackMetadataSection` format range (mod.rs `pack_format_range` / `pack_format_compatibility` / `enable_pack_checked`): FIXED. `min_format` / `max_format` / `supported_formats` / `pack_format` are parsed into an inclusive range and compared using `PackCompatibility.forVersion` semantics; incompatible packs are refused with a warning and `DatapackError::Incompatible`. Verified against `.vendor/vanilla-decompiled/net/minecraft/server/packs/metadata/pack/PackMetadataSection.java` and `.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackCompatibility.java` (`forVersion`). Unit test `pack_format_compatibility_matches_vanilla`.
