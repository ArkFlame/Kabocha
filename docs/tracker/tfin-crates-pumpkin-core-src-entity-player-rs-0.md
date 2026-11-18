# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-entity-player-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: 

All rows closed as of the third pass (final closure Result below). No row remains STILL OPEN.

## Result

FINAL PASS (2026-11-18). Worked through every row in `docs/tracker/tpar-crates-pumpkin-core-src-entity-player-rs.md` in order, re-checking each against decompiled vanilla at `.vendor/vanilla-decompiled/net/minecraft/` and against the current checkout. Every fix claimed by the earlier passes is present and type-consistent in source; no new source edit was needed this pass, so there is nothing to rustfmt. No unwrap/expect/panic, no stdout printing, and no generated `pumpkin-data` or WIT edits were introduced. Cargo/build was forbidden by task and left to the orchestrator's single shared compile.

- **items/Elytra** [partial/AUDITED]: VERIFIED. Re-reviewed both elytra audits against vanilla: the fall-distance clamp (`Entity.java`, `.vendor/.../world/entity/Entity.java`), gliding exhaustion (`ServerPlayer.checkMovementStatistics`, `.vendor/.../server/level/ServerPlayer.java`), glide-start gate (`Player.startFallFlying`, `.vendor/.../world/entity/player/Player.java`), and Elytra Unbreaking (`ItemStack` durability damage, `.vendor/.../world/item/ItemStack.java`). Pumpkin implements all four in `entity/player.rs` (clamp in `progress_motion`, `toggle_glide_applies` gate, exhaustion branch, `GliderImpl` damage). All rows closed in those audits.

- **own/taudit-crates-pumpkin-core-src-en-13d289**: VERIFIED. All 12 rows resolved across prior Results (respawn hunger restore, attack-cooldown enchant split, difficulty scaling, fall-flying stats, team display name, teleport flags, stale TODOs, XP atomicity), each against its cited vanilla class under `.vendor/vanilla-decompiled/net/minecraft/`.

- **own/taudit-crates-pumpkin-core-src-entity-pla**: VERIFIED except one BLOCKED-DATA. Rows 1-10 are present in `entity/player.rs` (dig-speed single-max multiplier helper `dig_speed_effect_multiplier` wired into `get_mining_speed`; `compute_sweep_damage`; `format_team_name`; `get_bed_spawn_offsets`; `read_nbt` hunger restore; `tick_experience` under one lock). Dig speed matches `MobEffectUtil.getDigSpeedAmplification` = `Math.max(a, b)` (`.vendor/.../world/effect/MobEffectUtil.java:29-41`, confirmed in source) and `Player.getDestroySpeed` (`.vendor/.../world/entity/player/Player.java:629-631`). BLOCKED-DATA: the full `SUBMERGED_MINING_SPEED` attribute path. Missing subsystem is the item attribute-modifier system (equipment modifiers reaching the player); the Aqua Affinity `/5` path yields the vanilla default 0.2 and is behaviourally equivalent until that subsystem exists. No generated field is missing.

- **own/taudit-crates-pumpkin-core-src-it-847255** (bow): VERIFIED, including the last open Row 1. `LivingEntity.stopUsingItem` (`.vendor/.../world/entity/LivingEntity.java:4498-4509`) only emits `GameEvent.ITEM_INTERACT_FINISH` and clears `useItem`; it does NOT call `ItemStack.releaseUsing`. Pumpkin `clear_active_hand` (living.rs, called by `player.rs::stop_using_item` and the death/respawn/item-break paths) matches this cancel semantics; the release path fires the bow via `item_registry.on_stopped_using` (`bow.rs::on_stopped_using`). Rows 2 (`damage_slot_for`) and 3 (spectator gate) already FIXED/VERIFIED.

- **own/taudit-crates-pumpkin-core-src-net-java-m**: VERIFIED. All rows resolved in the final Result: ordering chain (`net/java/mod.rs::chain_after`), keep-alive oldest-challenge, recipes property sets (`recipe_book_add.rs`), packet cap removed, unknown-id disconnect kick (`mod.rs`).

- **own/tentities-bamboo-raft-302792**: VERIFIED. All rows fixed against `AbstractBoat` (`.vendor/.../world/entity/vehicle/AbstractBoat.java`): boarding gate, rider yaw/delta, bubble time, leash, width gate; rotation decay is a client-authority by-design gap.

- **own/tentities-jungle-boat**: VERIFIED. Delta-rotation and dismount pose fixed against `AbstractBoat.java:700-765`; remaining rows are shared/client-only and closed in that doc.

- **own/tentities-mangrove-boat-ad6fea**: VERIFIED. Dimensions/eye-height match `EntityTypes.java` (`.vendor/.../world/entity/EntityTypes.java:674-689`); no wood-specific sound/particle overrides exist in vanilla.

- **own/titems-compass-recovery-compass-3fb9d9**: VERIFIED. All rows closed; WIT-exposed `compass_target` API intentionally kept (removing it would be a breaking WIT change).

- **own/titems-elytra-08fcad**: VERIFIED. All rows resolved against vanilla (see Elytra above).

- **own/titems-elytra**: VERIFIED. All four rows confirmed non-gaps / already covered by pure unit tests.

No row is left STILL OPEN. Ten rows end VERIFIED; one row ends BLOCKED-DATA (the `SUBMERGED_MINING_SPEED` item attribute-modifier subsystem, not a missing generated field).

Status: PASS fixed=0 open=0 files=docs/tracker/tfin-crates-pumpkin-core-src-entity-player-rs-0.md
