# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-734ab8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

| mod.rs:2079 (scan loop), 1384 | `PackRepository` / `FolderRepositorySource` / `FilePackResources` | `.zip` datapacks are listed by `get_all_known_packs` but never loaded (`scan_datapacks_dir` only accepts directories), so `/datapack enable` on a zip appears to work but loads nothing. | Introduce a small read-only pack filesystem abstraction (dir or zip) and thread it through `load_pack_contents`, `read_pack_mcmeta`, structure/worldgen loaders. | L, STILL OPEN: size L, skipped per wave brief |
| mod.rs:122 | `CommandFunctionManager` (`maxCommandChainLength` game rule) | Command budget is the fixed constant 65536; vanilla reads the `maxCommandChainLength` game rule per execution, so lowering it in game rules is ignored. | Add `maxCommandChainLength` to the gamerule set, read it from the executing world before running a chain, and reset the thread-local budget accordingly. | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:2409-2421 | `PackMetadataSection` / `Component.Serializer` | `pack.mcmeta` `description` that is a JSON text component is stored as raw JSON (`d.to_string()`), so `/datapack list` shows JSON instead of text. | Parse the component and keep its plain text (fall back to `translate` key), or store a `TextComponent`. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:1459-1483, 2423 | `DataPackVersion` / `PackFormat` | `pack_format` for the vanilla/embedded/bundled packs and the mcmeta default are the literal `61`, unrelated to `CURRENT_MC_VERSION`; a version bump silently desyncs them. | Derive the current data pack format once from `CURRENT_MC_VERSION` (const next to `CURRENT_MC_VERSION`) and use it for all defaults. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
