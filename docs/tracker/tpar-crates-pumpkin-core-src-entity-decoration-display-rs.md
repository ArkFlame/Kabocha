# Parity rows for crates/pumpkin-core/src/entity/decoration/display.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **own/taudit-crates-pumpkin-core-src-en-cb5e22** [partial/OWN-OPEN] own audit: 4 open, 8 fixed, 2 verified, 0 blocked-data (docs/tracker/taudit-crates-pumpkin-core-src-en-cb5e22.md) docs/tracker/taudit-crates-pumpkin-core-src-en-cb5e22.md

## Result
Per-row results: row 1 (explosion immunity) FIXED; row 2 (TextDisplay flags reset on load) VERIFIED; row 3 (text NBT key) FIXED; row 4 (set_brightness pre-packed int) FIXED; NEW brightness packing FIXED + unit-tested; NEW NBT read defaults FIXED. No STILL OPEN rows remain (details below).


## Result (third pass, final verification against .vendor decompiled vanilla)

The single unit row (own/taudit-crates-pumpkin-core-src-en-cb5e22) is CLOSED. All four audit sub-rows plus the two new findings are already FIXED/VERIFIED in the live checkout and match decompiled vanilla at .vendor/vanilla-decompiled/net/minecraft/. No source change was needed this pass; no row needs a generated pumpkin-data field or a missing subsystem.

- Row 1 (explosion immunity): FIXED. Display.java has no ignoreExplosion override (0 matches); base Entity.ignoreExplosion is false. All three display is_immune_to_explosion() overrides return false (display.rs:816, 972, 1269) and damage_with_context returns false, so explosions push but do not damage displays.
- Row 2 (TextDisplay flags reset on load): VERIFIED, no change. Display.TextDisplay.readAdditionalSaveData (.vendor/.../world/entity/Display.java:760-777) rebuilds flags from (byte)0; an unknown alignment name makes input.read(...) return Optional.empty(), leaving alignment bits unset. Pumpkin display.rs matches.
- Row 3 (text NBT key): FIXED. Display.TextDisplay.addAdditionalSaveData (.vendor/.../world/entity/Display.java:800-802) stores text unconditionally; Pumpkin display.rs:1144-1155 always writes the key with an empty-JSON fallback on serialization failure.
- Row 4 (set_brightness pre-packed int): FIXED. Vanilla Display.setBrightnessOverride packs via Brightness.pack (Display.java:311). Additive WIT set-brightness-override is present in pumpkin-plugin-wit/v0.1/display.wit:99 and v0.2/display.wit:103; WASM hosts implement it (pumpkin-wasm-host-v0_1/src/display.rs:448, v0_2/src/display.rs:451); core clamps 0..=15 and packs (display.rs:278-279) and re-packs non -1 values (display.rs:261-274).
- NEW FINDING (brightness packing): FIXED + unit-tested. Verified exact against .vendor/.../util/LightCoordsUtil.java:9,13-22 (FULL_BRIGHT=15728880, pack=block<<4|sky<<20, block=(p>>4)&15, sky=(p>>20)&15) and Brightness.java (0..15 range). Helpers display.rs:57-67; test display.rs:1288-1298.
- NEW FINDING (NBT read defaults): FIXED. Display.readAdditionalSaveData (.vendor/.../world/entity/Display.java:211-224) reads each field with a default; Pumpkin read_display_nbt (display.rs:602-702) resets every field on missing keys and clears brightness on a missing/incomplete/out-of-range compound (Brightness.CODEC requires both 0..15).

No remaining STILL OPEN rows. Source files changed this pass: none (display.rs already carries every fix). No unit-test seam added (packing already unit-tested; NBT/EntityBase methods need live entity state). Compile-pitfall review: no unwrap/expect/panic added (existing unwrap_or / unwrap_or_else used), no import/helper/macro/struct-init changes, const fn and then() already clippy-clean. No cargo/git/shell build command run by this worker; orchestrator compiles once.
Status: PASS fixed=4 open=0 files=docs/tracker/tpar-crates-pumpkin-core-src-entity-decoration-display-rs.md

## Result (final pass, re-verified against .vendor decompiled vanilla)

Re-read display.rs and the vanilla sources; every row is closed, no source change needed this pass.

- Row 1 (explosion immunity): FIXED. `.vendor/vanilla-decompiled/net/minecraft/world/entity/Display.java` has no `ignoreExplosion` override (0 matches); base `Entity.ignoreExplosion` is false. All three display `is_immune_to_explosion()` overrides return false (display.rs:816, 972, 1269) and `damage_with_context` returns false, so explosions push but deal no damage.
- Row 2 (TextDisplay flags reset on load): VERIFIED. `Display.TextDisplay.readAdditionalSaveData` (.vendor/.../world/entity/Display.java:760-777) rebuilds flags from `(byte)0`; an unknown alignment name makes `input.read(...)` return empty, leaving alignment bits unset. Pumpkin display.rs:1195-1212 matches.
- Row 3 (text NBT key): FIXED. `Display.TextDisplay.addAdditionalSaveData` (.vendor/.../world/entity/Display.java:800-802) stores `text` unconditionally; Pumpkin display.rs:1144-1155 always writes the key with an empty-JSON fallback.
- Row 4 (set_brightness pre-packed int): FIXED. Vanilla `Display.setBrightnessOverride` packs via `Brightness.pack` (Display.java:311-312, Brightness.java:14-16). Additive WIT present in pumpkin-plugin-wit/v0.1/display.wit:99 and v0.2/display.wit:103; hosts implement it (wasm-host-v0_1/src/display.rs:448, v0_2/src/display.rs:451); core clamps 0..=15 and packs (display.rs:278-279), re-packs non -1 values (display.rs:261-274).
- NEW (brightness packing): FIXED + unit-tested. Exact match to `.vendor/vanilla-decompiled/net/minecraft/util/LightCoordsUtil.java:9,13-22` (FULL_BRIGHT=15728880, pack=block<<4|sky<<20, block=(p>>4)&15, sky=(p>>20)&15) and `Brightness.java:6-20` (0..15 range). Helpers display.rs:57-67; test display.rs:1288-1299.
- NEW (NBT read defaults): FIXED. `Display.readAdditionalSaveData` (.vendor/.../world/entity/Display.java:211-224) reads each field with a default; Pumpkin read_display_nbt (display.rs:602-702) resets every field on missing keys and clears brightness on a missing/incomplete/out-of-range compound (Brightness.CODEC requires both fields 0..15).

Compile-pitfall review: no unwrap/expect/panic/println/em dash added; helpers are `const fn`, `then` used instead of `if let ... else`; imports/struct-init/macro paths unchanged. No source edits this pass; only this doc updated. No cargo/git/shell build run by this worker.
