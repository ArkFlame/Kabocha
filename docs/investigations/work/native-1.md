# Native agent 1 (24 open lines)
Each line is `[unit-note] text` from the latest Result of docs/tracker/<unit>.md (the note has the full table and context).

- [taudit-crates-pumpkin-core-src-co-122988] - 6. Bedrock `/time rate` message: LEFT OPEN. Vanilla has no rate command; Pumpkin falls back
- [taudit-crates-pumpkin-core-src-co-1acbb3] - 7. store score orphan objectives: STILL OPEN (M). The scoreboard cannot retain scores whose
- [taudit-crates-pumpkin-core-src-co-20e7ce] - Row 1 (M): OPEN - one-to-many slot-source iteration and `target.failed*` counting live in the `commands/item.rs` executors (outside this unit). In this unit `SlotsArgumentType`/`resolve_slot_range` already match vanilla `SlotRanges.read`/`SlotsArgument.parse` (`SlotRanges.java`, `SlotArgument.java
- [taudit-crates-pumpkin-core-src-co-20e7ce] - Row 2 (M): OPEN - `item set` subcommand executors and Set-success feedback are in `commands/item.rs` (`SuccessFeedback::Set`); outside this unit.
- [taudit-crates-pumpkin-core-src-co-20e7ce] - Row 3 (M): OPEN - entity slot 300-307/499/500-514 mapping needs inventory/crafting-grid APIs and lives in `commands/item.rs`; outside this unit. The slot-name table itself is verified correct.
- [taudit-crates-pumpkin-core-src-co-20e7ce] - Row 7 (M): OPEN - `BlockEntity::mark_dirty` lives in `block/entities/mod.rs` (outside this unit); `item.rs` cannot set the saved flag.
- [taudit-crates-pumpkin-core-src-co-20e7ce] - Row 9 (S): OPEN - maintenance dedup inside `commands/item.rs`, no behavioural difference; outside this unit.
- [taudit-crates-pumpkin-core-src-co-58043b] - Row 1 (custom name): IN-UNIT COMPLETE. `block/entities/command_block.rs` holds `custom_name: StdMutex<Option<TextComponent>>`, saves/loads `CustomName` via `CustomNameImpl` (String/Compound codec, matching `ComponentSerialization.CODEC`), and feeds it to `build_command_block_actor`; `net/java/play
- [taudit-crates-pumpkin-core-src-command-mo] - Final row status: 11 STILL OPEN (needs `command/mod.rs`, outside editable basenames); 12 FIXED (pre-existing); 13 FIXED (pre-existing edit); 14 FIXED (this pass); 15 VERIFIED FIXED (pre-existing); 16 no change. No remaining in-scope actionable rows.
- [taudit-crates-pumpkin-core-src-net-java-p] LEFT OPEN: Pumpkin's generated `Item` (`pumpkin-data/src/generated/item.rs:22`) has only
- [tentities-command-block-minecart-5e81b3] STILL OPEN (BLOCKED). One-line reason: threading the target protocol version through entity NBT
- [tentities-command-block-minecart] - Gap 5 (hardcoded NBT component version, doc lines 46-49): LEFT OPEN, one-line reason: threading a target
- [tentities-item-0024a5] - Age-based thrower gate (item.rs:826-838): STILL OPEN, out of unit. Removal of the
- [tentities-zombie-nautilus] - Row 4 (S) OPEN (unchanged): `AbstractNautilus.inventory` is a real `SimpleContainer` used by `openNautilusInventory`/`NautilusInventoryMenu` (`AbstractNautilus.java:82,494-522`); the dead 9-slot fields are already removed, and full `Items` NBT + mount-screen wiring needs `Player`/menu files outsid
- [tentities-cushion-2527f1] - `cushion.rs` placement/support tests: gametest cases LEFT OPEN - need
- [tentities-item-0024a5] - Fake item entity on the `added && copyToDrop.isEmpty()` branch: STILL OPEN, out of
- [tentities-jungle-boat] - Re-checked the only row ever marked STILL OPEN (delta-rotation accumulation, h6 status) against decompiled `.vendor/vanilla-decompiled/net/minecraft/world/entity/vehicle/boat/AbstractBoat.java`: `positionRider` adds `deltaRotation` to rider yaw/head yaw then `clampRotation` (+-105, Mob vs non-Mob)
- [tentities-oak-boat] - Rider-rotation clamp (M): LEFT OPEN. Needs a vehicle `positionRider`/`onPassengerTurned` hook across
- [tentities-poplar-chest-boat] - Row "c:boats missing poplar_chest_boat" (line 16): STILL OPEN, needs
- [tentities-sulfur-cube-b516b8] 7. Squish tracked data: LEFT OPEN. Client-only squash animation needs a tracked-data id from
- [titems-compass-recovery-compass-3fb9d9] - Row 2 STILL OPEN, out of scope: `isFoil` glint needs `ItemBehaviour::is_foil` (`item/mod.rs`) plus `ItemStack::has_foil` (cross-crate pumpkin-data). Cannot add from compass.rs. Verified vanilla `CompassItem.isFoil` (CompassItem.java:30-32) returns `has(LODESTONE_TRACKER) || super.isFoil`.
- [titems-compass-recovery-compass-3fb9d9] - Row 3 STILL OPEN, out of scope: dead `compass_target` field/methods live in `entity/player.rs`.
- [titems-elytra] - Row 3 (glide-veto gametest): LEFT OPEN (out of scope). Needs a pumpkin-gametest harness plus
- [titems-goat-horn-5fa4ed] - 2 TOOT_HORN use animation: OPEN, out of unit. Vanilla `InstrumentItem.getUseAnimation` returns
