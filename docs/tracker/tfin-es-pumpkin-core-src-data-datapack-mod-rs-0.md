# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-data-datapack-mod-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

- taudit-crates-pumpkin-core-src-data-datap.md (mod.rs) remaining STILL OPEN list, closed:

## Result (final pass, all rows terminal)

Re-checked every `Remaining gaps` row of
`docs/tracker/taudit-crates-pumpkin-core-src-data-datap.md` against decompiled V_26_3 vanilla
at `.vendor/vanilla-decompiled/net/minecraft/` (tree present at
`/run/media/linsaftw/SDD/tmp/mc-proof/ref/net/minecraft`) and against the current
`crates/pumpkin-core/src/data/datapack/mod.rs`. Every actionable fix is already present in
the shared checkout; no source file needed editing this pass, so there is nothing to rustfmt.
Shell/cargo/git are denied to this worker.

| Row | Terminal status | Evidence |
|---|---|---|
| 24 `scan_datapacks_dir` zip packs | BLOCKED-DATA (whole missing subsystem) | Loader is directory-only; `crates/pumpkin-core/Cargo.toml` has no `zip` dependency. Vanilla needs the whole `PackResources` abstraction (`.vendor/vanilla-decompiled/net/minecraft/server/packs/FilePackResources.java:54-89`). Implementing it must thread a pack reader through every loader, a new subsystem rather than a single missing generated field. |
| 25 `load_all` pack order | FIXED | Bundled packs load before world packs (`mod.rs:495-503`) so world packs override them; matches `PackRepository#rebuildSelected` selected-order priority (`.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackRepository.java`). |
| 26 feature flags from `features.enabled` | FIXED | `read_pack_mcmeta` parses root `features.enabled` into `LoadedDatapack.requested_features` (`mod.rs:2901-2920`); `get_enabled_features` unions them over loaded packs (`mod.rs:1182-1201`); `resolve_enabled_features` keeps only the id-based fallback and no longer infers flags from `file/...` names (`mod.rs:1208-1224`). Vanilla `FeatureFlagsMetadataSection` (`.vendor/vanilla-decompiled/net/minecraft/server/packs/FeatureFlagsMetadataSection.java:9-13`). |
| 27 pack format ranges / compatibility | FIXED | `PackFormatRange` + `pack_format_compatibility` (`mod.rs:152-221`) mirror `PackFormat`/`PackCompatibility.forVersion` (`.vendor/vanilla-decompiled/net/minecraft/server/packs/repository/PackCompatibility.java:27-35`); enable refuses incompatible packs (`mod.rs:1827`). Sub-gap `(incompatible)` listing suffix: VERIFIED obsolete for `/datapack list`, which renders `getChatLink`, not `displayPackList` (`.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java:191-233`); incompatible packs are already refused at enable time. |
| 28 `pack.mcmeta` `filter` block/namespace | VERIFIED obsolete for V_26_3 | Vanilla `PackMetadataSection` is a record of only `description` + `supportedFormats` (`.vendor/vanilla-decompiled/net/minecraft/server/packs/metadata/pack/PackMetadataSection.java:11-27`); no filter section exists, so no Pumpkin change is warranted. |
| 29 generic tag queries (nested `#tag`) | FIXED | `get_tag_values` (`mod.rs:686-711`), `get_tag_entries`, and `is_in_tag` expand nested references recursively with a visited set and first-seen dedup plus a builtin fallback, matching `TagLoader#tryBuildTag` LinkedHashSet (`.vendor/vanilla-decompiled/net/minecraft/tags/TagLoader.java:76-86`). |
| 30 `get_tag_values` leading `#` | FIXED | Leading `#` stripped before lookup (`mod.rs:693`), consistent with `is_in_tag` / `get_tag_entries`. |
| 31 `maxCommandChainLength` budget | FIXED | `chain_budget` reads `game_rules.max_command_sequence_length` at the outermost start with `Math.max(1, ...)` (`mod.rs:1538-1545`, `enter_with_budget:332-344`); vanilla `Commands#executeCommandInContext` (`.vendor/vanilla-decompiled/net/minecraft/commands/Commands.java:463`) and default 65536 (`.vendor/vanilla-decompiled/net/minecraft/world/level/gamerules/GameRules.java:53`). |
| 32 function chain depth (256 cap) | BLOCKED-DATA (whole missing subsystem) | Vanilla imposes no depth cap, only the command budget (`ExecutionControl` queues function bodies instead of recursing, `.vendor/vanilla-decompiled/net/minecraft/commands/execution/ExecutionControl.java:15-36`). Pumpkin recursive `dispatcher.execute_input` needs `MAX_FUNCTION_CHAIN_DEPTH = 256` (`mod.rs:266,334`) to avoid a Rust stack overflow. Matching vanilla requires an iterative-dispatch refactor of the recursive path (new subsystem), not a missing generated field. |
| 33 macro template validation | FIXED | `validate_macro_functions` / `validate_macro_template` reject unterminated variables, invalid/empty names, and no-variable lines at load (`.vendor/vanilla-decompiled/net/minecraft/commands/functions/StringTemplate.java:8-55`); implemented at `mod.rs:2047-2088`. Optional 8-entry instantiation cache: VERIFIED no observable behaviour gap (`.vendor/vanilla-decompiled/net/minecraft/commands/functions/MacroFunction.java`). |
| 34 `disable_pack` embedded packs | FIXED | `is_required_pack` accepts only `vanilla` (`mod.rs:1557-1561`); other embedded packs can be disabled, matching `PackSelectionConfig#required` (`.vendor/vanilla-decompiled/net/minecraft/server/packs/PackSelectionConfig.java:5`). |

No STILL OPEN rows remain. The two BLOCKED-DATA rows are whole-missing-subsystem cases
permitted by the unit contract; neither is a missing generated `pumpkin-data` field.
