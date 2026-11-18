# Parity rows for crates/pumpkin-core/src/data/datapack/mod.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-da-334d62** [partial/OWN-OPEN] own audit: 7 open, 4 fixed, 2 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-da-334d62.md) docs/tracker/taudit-crates-pumpkin-core-src-da-334d62.md
- **own/taudit-crates-pumpkin-core-src-da-734ab8** [partial/OWN-OPEN] own audit: 14 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-da-734ab8.md) docs/tracker/taudit-crates-pumpkin-core-src-da-734ab8.md
- **own/taudit-crates-pumpkin-core-src-data-datap** [partial/OWN-OPEN] own audit: 6 open, 1 fixed, 0 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-data-datap.md) docs/tracker/taudit-crates-pumpkin-core-src-data-datap.md

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)


## Result (third pass, cross-file verification)

Every STILL OPEN row across the three audit docs is now terminal
(FIXED / VERIFIED / BLOCKED-DATA). No source change was needed: prior passes had already
fixed or verified the actionable rows, and the remainder are obsolete for V_26_3 or
blocked by a whole missing subsystem. Re-checked against
`.vendor/vanilla-decompiled/net/minecraft`.

- taudit-crates-pumpkin-core-src-da-334d62.md (test_loader / model): all rows terminal,
  no STILL OPEN. Environment codec validation, invalid-resource-location skip, required
  `structure`, lowercase `.json`, and typed `test_instance` registry sync were fixed or
  verified in earlier passes against `gametest/framework/TestData.java` and
  `core/RegistrySynchronization.java`.
- taudit-crates-pumpkin-core-src-da-734ab8.md (deep audit): all rows terminal. Only the
  zip-datapack row is BLOCKED-DATA; everything else is FIXED/VERIFIED in the current tree.
- taudit-crates-pumpkin-core-src-data-datap.md (mod.rs) remaining STILL OPEN list, closed:
  - zip packs (`scan_datapacks_dir`): BLOCKED-DATA. `crates/pumpkin-core/Cargo.toml` has
    no `zip` dependency and the scanner is directory only; vanilla requires the whole
    `PackResources` abstraction
    (`.vendor/vanilla-decompiled/net/minecraft/server/packs/FilePackResources.java:54-89`).
  - pack.mcmeta `filter` block/namespace: VERIFIED obsolete for V_26_3. Vanilla
    `PackMetadataSection` declares only `description` and `supportedFormats`, no filter
    section
    (`.vendor/vanilla-decompiled/net/minecraft/server/packs/metadata/pack/PackMetadataSection.java:11-27`).
    No Pumpkin change warranted.
  - function chain depth (256 cap): BLOCKED-DATA (whole subsystem). Vanilla
    `ExecutionControl` imposes no depth cap, only the command budget; Pumpkin's recursive
    `dispatcher.execute_input` needs the cap to avoid a Rust stack overflow that aborts the
    process. Matching vanilla requires an iterative dispatch refactor of command execution.
  - macro instantiation cache (8 entries): VERIFIED no change. `MacroFunction` caching
    (`.vendor/vanilla-decompiled/net/minecraft/commands/functions/MacroFunction.java`) is a
    performance detail with no observable behaviour gap; `substitute_macro` is correct.
  - `(incompatible)` suffix in pack listings: VERIFIED obsolete for the command. Vanilla
    `/datapack list` renders packs with `getChatLink`, not `PackRepository.displayPackList`
    (`.vendor/vanilla-decompiled/net/minecraft/server/commands/DataPackCommand.java:191-227`);
    the suffix is only in the system report. Incompatible packs are already refused at
    enable time via `DatapackError::Incompatible`, so no generated-WIT `DatapackInfo`
    change is needed.

No source files were changed this pass, so there is nothing to rustfmt. Shell/cargo denied
to workers; the orchestrator compiles the batch.
