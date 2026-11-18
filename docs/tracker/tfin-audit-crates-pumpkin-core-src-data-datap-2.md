# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-data-datap.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

source fix is present in crates/pumpkin-core/src/data/datapack/mod.rs. No STILL OPEN

## Result
Verified all 11 rows of the parent doc's "Remaining gaps" table against the decompiled
V_26_3 tree at .vendor/vanilla-decompiled/net/minecraft and against the current
crates/pumpkin-core/src/data/datapack/mod.rs. Every row is terminal; no STILL OPEN rows
remain. No source change was needed this pass: each FIXED row's fix was confirmed present
in the shared tree, and each BLOCKED row names a whole missing subsystem (permitted by the
unit contract), not a generated-data gap.

| Row | Terminal status | Evidence |
|---|---|---|
| 24 scan_datapacks_dir zip packs | BLOCKED-DATA (whole subsystem) | Directory-only loader; needs the whole zip-backed `PackResources` abstraction plus a zip crate wired into `pumpkin-core` (no `zip` dependency in crates/pumpkin-core/Cargo.toml). Vanilla `.vendor/vanilla-decompiled/net/minecraft/server/packs/FilePackResources.java:54-89`. |
| 25 load_all pack order | FIXED | Bundled packs load before world packs (`load_all`, mod.rs:431) so world packs override them; matches `PackRepository#rebuildSelected` selected-order priority, `.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackRepository.java:98-112`. |
| 26 feature flags from features.enabled | FIXED | `read_pack_mcmeta` parses root `features.enabled` into `LoadedDatapack.requested_features` (mod.rs:2901-2920); `get_enabled_features` unions them over loaded packs (mod.rs:1182-1201); `resolve_enabled_features` no longer infers flags from `file/` names (mod.rs:1208). Vanilla `.vendor/vanilla-decompiled/net/minecraft/server/packs/FeatureFlagsMetadataSection.java:9-13`. |
| 27 pack format ranges / compatibility | FIXED | `PackFormatRange` (mod.rs:152) + `pack_format_compatibility` (mod.rs:168) parsed from `pack_format`/`min_format`/`max_format`/`supported_formats`; enable refuses incompatible packs. Mirrors `PackCompatibility.forVersion`, `.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackCompatibility.java:9-33`. Sub-gap `(incompatible)` listing suffix: VERIFIED obsolete for `/datapack list`, which uses `getChatLink` not `displayPackList`, `.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java`; incompatible packs are already refused at enable time. |
| 28 pack.mcmeta filter block/namespace | VERIFIED obsolete | V_26_3 `.vendor/vanilla-decompiled/net/minecraft/server/packs/metadata/pack/PackMetadataSection.java:11-27` is a record of only `description` + `supportedFormats`; no filter section exists, so no Pumpkin change is warranted. |
| 29 generic tag queries (nested #tag) | FIXED | `get_tag_values` (mod.rs:686), `get_tag_entries` (mod.rs:1060), `is_in_tag` (mod.rs:1036) expand nested references recursively with a visited set and builtin fallback; matches `TagLoader#tryBuildTag` LinkedHashSet, `.vendor/vanilla-decompiled/net/minecraft/tags/TagLoader.java:76-77`. |
| 30 get_tag_values leading # | FIXED | Leading `#` stripped before lookup (mod.rs:686), consistent with `is_in_tag` / `get_tag_entries`. |
| 31 maxCommandChainLength budget | FIXED | `chain_budget` reads `game_rules.max_command_sequence_length` at the outermost start (mod.rs:1537-1545) and `enter_with_budget` (mod.rs:332) stores it in a thread-local for nested functions. Vanilla `Commands#executeCommandInContext`, `.vendor/vanilla-decompiled/net/minecraft/commands/Commands.java:463`, default 65536 in `.vendor/vanilla-decompiled/net/minecraft/world/level/gamerules/GameRules.java:53`. |
| 32 function chain depth | BLOCKED-DATA (whole subsystem) | Vanilla `ExecutionControl` imposes no depth cap, only the command budget (`.vendor/vanilla-decompiled/net/minecraft/commands/execution/ExecutionControl.java`); Pumpkin's recursive `dispatcher.execute_input` needs `MAX_FUNCTION_CHAIN_DEPTH=256` (mod.rs:266,334) to avoid a Rust stack overflow. Removing it requires an iterative-dispatch refactor of the recursive path, a new subsystem rather than a data or single-file change. |
| 33 macro template validation | FIXED | `validate_macro_functions` / `validate_macro_template` (mod.rs:2047-2088) reject unterminated, invalid/empty, and no-variable templates at load, mirroring `StringTemplate#fromString`, `.vendor/vanilla-decompiled/net/minecraft/commands/functions/StringTemplate.java:25,37`. Optional 8-entry instantiation cache: VERIFIED no behaviour gap. |
| 34 disable_pack | FIXED | `is_required_pack` accepts only `vanilla` (mod.rs:1559), so other embedded packs can be disabled, matching `PackSelectionConfig#required`, `.vendor/vanilla-decompiled/net/minecraft/server/packs/PackSelectionConfig.java:5`. |

No cargo, git, or rustfmt run (shell denied to this worker); no source files changed this pass.
