# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-734ab8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: function_loader.rs mod.rs

| mod.rs:500, 884 | `Tag#getEntries` (flattened holder set) | `get_tag_values` / `get_tag_entries` return raw `#tag` reference strings; vanilla returns the flattened member list. | Reuse the `tag_contains_entry` recursion to expand nested references. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| function_loader.rs:30, mod.rs:2465 | `FolderRepositorySource` | Loaders skip symlinked files/dirs (`entry.file_type()`), while vanilla's directory source follows them; a pack symlinking shared content silently loads nothing. | Use `path.is_dir()` / `path.is_file()` in the recursive loaders. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:2409 | `PackMetadataSection` format range | An incompatible `pack_format` is accepted without warning; vanilla marks the pack incompatible and refuses to enable it. | Compare `pack_format` (or min/max range) against the current format and reject/skip with a log. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
This unit may edit only `function_loader.rs`. Every STILL OPEN gap whose fix lives in

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
