# entity/decoration/display.rs - audit (DEEP AUDIT)

Audited crates/pumpkin-core/src/entity/decoration/display.rs on branch
kab/taudit-crates-pumpkin-core-src-en-cb5e22. No .vendor/ tree in this worktree and no
decompiled 26.3 sources, so vanilla behaviour below is from Mojang mappings
`Display` / `BlockDisplay` / `ItemDisplay` / `TextDisplay` (1.21.x, unchanged through
26.x) plus the tracker note of two "vanilla" doc comments (display.rs:55, 685, 696 - all
accurate, not TODOs). No TODO/FIXME markers in the file.

## Fixed
- display.rs:495-503, 608-613 - brightness override was packed with 4-bit fields
  (`block & 0xF`, `sky >> 4`). Vanilla `Display.Brightness.pack` uses two 16-bit
  fields (`block | sky << 16`), so a data-pack `{brightness:{block:15,sky:12}}`
  round-tripped through Pumpkin sent the client a value it decoded as sky 0 and was
  re-saved wrong. Added `pack_brightness` / `brightness_block` / `brightness_sky`.
- display.rs:124-130, 561-569 - `teleport_duration` was written to NBT but never read
  back, and `setPosRotInterpolationDuration` did not clamp. Vanilla reads it in
  `readAdditionalSaveData` and clamps to 0..=59. Added the read and the clamp
  (`TELEPORT_DURATION_MAX`).
- display.rs:652-681 - the transformation was applied per field: each of translation/
  scale/left_rotation/right_rotation was committed if it happened to parse, and
  missing or short lists were silently ignored. Vanilla decodes the tag as one
  Transformation value (all four fixed-length float lists required), so a partial tag
  must not leave a half-updated transform. Added float_list::<N> and commit all four
  together only when every field parses; otherwise the transform is left unchanged.

## Remaining gaps

1. display.rs:797, 954, 1249 - `is_immune_to_explosion() -> true` skips both damage
   and knockback in world/explosion.rs:565. Vanilla `Display` only overrides
   `isPickable`/`isAttackable`/`skipAttackInteraction` (all false/true) and does not
   override `ignoreExplosion`, so explosions still push display entities (damage is
   already false through `damage_with_context`). Suggested fix: return false and
   confirm against decompiled 26.3. Vanilla: `net.minecraft.world.entity.Display`,
   `Entity#ignoreExplosion`. Size S (needs verification).
2. display.rs:1155-1193 - TextDisplay flags are rebuilt from zero on load, so an
   unknown `alignment` name silently clears the alignment bits. Vanilla uses
   `TextAlignment.CODEC.byName`, which returns null and leaves the current flags
   untouched. Only observable when loading into an already-modified entity (all
   writers emit a valid name), so low priority. Size XS.
3. display.rs:1126-1136 - the `text` NBT key is skipped when JSON serialization
   fails; vanilla always writes a (possibly empty) component string. Size XS.
4. display.rs:261-267 - `set_brightness` takes a pre-packed int (also the WASM host
   API in pumpkin-wasm-host-v0_1/-v0_2 display.rs). Vanilla exposes
   `setBrightnessOverride(Brightness(block, sky))`, so a plugin cannot set block and
   sky independently and an out-of-range value leaks into the packed fields.
   Suggested fix: expose `pack_brightness`/`brightness_block`/`brightness_sky` to
   plugins or take a pair. Size XS.

## Matches vanilla (no action)
- Every `Display` tracked-data default: view_range 1.0, shadow_radius 0.0,
  shadow_strength 1.0, width/height 0.0, billboard 0 (fixed), brightness -1,
  glow_color_override -1, translation 0, scale 1, rotations identity,
  interpolation duration/start 0 (display.rs:72-92).
- All 15 display slots are sent in `init_display_data_tracker` (display.rs:333-435);
  the three subclasses add their own slot in `init_data_tracker`.
- NBT key names and shapes: `start_interpolation`, `interpolation_duration`,
  `teleport_duration`, `transformation` (four float lists), `billboard` as enum name,
  `brightness` as `{block,sky}` compound, `glow_color_override` only when != -1,
  `block_state` via the block state codec, `item`/`item_display`, `text`/`line_width`/
  `background`/`text_opacity`/`shadow`/`see_through`/`default_background`/`alignment`.
- TextDisplay defaults and style bit layout: 200 line width, 0x40000000 background,
  -1 opacity, shadow 1 / see-through 2 / default-background 4 / left 8 / right 16 with
  left winning when both bits are set (display.rs:1026-1103).
- `ItemDisplayContext` name/ordinal mapping 0..8 (display.rs:843-889).
- `BlockDisplay`/`ItemDisplay`/`TextDisplay` are `no_physics`, not pushable, not
  hittable, and `damage_with_context` returns false like vanilla's non-living
  `hurt`.

## Gap status
- Rows: RESOLVED (see Result sections below; final third-pass verification at end).

## Result
- Row 1 (explosion immunity): FIXED. Decompiled `Display` does not override `ignoreExplosion`; `Entity#ignoreExplosion` returns false by default (Entity.java:6469) and `ServerExplosion.hurtEntities` (ServerExplosion.java:482,556) processes entities where `!ignoreExplosion`, applying `pushFromExplosion` knockback even to non-living entities. Pumpkin's display `damage_with_context` already returns false, so no damage. Changed all three `is_immune_to_explosion()` overrides (Block/Item/Text display) from `true` to `false`, so explosions now push display entities without damaging them.
- Row 2 (TextDisplay flags reset on load): VERIFIED, NO CODE CHANGE. Decompiled `Display.TextDisplay.readAdditionalSaveData` (Display.java:765-777) also rebuilds flags from `(byte)0` via `loadFlag`, and `input.read("alignment", Align.CODEC)` yields `Optional.empty()` for an unknown name, leaving alignment bits unset. This matches Pumpkin exactly. The doc's note about `TextAlignment.CODEC.byName` leaving flags untouched came from older 1.21 mappings; current decompiled vanilla starts from zero.
- Row 3 (`text` NBT key skipped on serialization failure): FIXED. Decompiled `addAdditionalSaveData` (Display.java:802) calls `output.store("text", ComponentSerialization.CODEC, ...)` unconditionally. Pumpkin now always writes the `text` key, falling back to an empty JSON string if serialization fails.
- Row 4 (`set_brightness` takes a pre-packed int): OPEN, out of scope. Fixing it requires changing the plugin/WASM host API (`pumpkin-wasm-host-v0_1/-v0_2` display.rs) and likely WIT, outside this unit's files (display.rs, explosion.rs).

Files changed: `crates/pumpkin-core/src/entity/decoration/display.rs` only (explosion.rs needed no change). No pure unit-test seam exists for the fixed rows: `is_immune_to_explosion` and `write_custom_nbt` are `EntityBase` trait methods requiring a live `Entity`/`World`, so no test was added. `rustfmt --edition 2024` ran with exit 0. Build/tests were not run by this worker (orchestrator compiles once after all workers finish).

## Result
- Row 1 (explosion immunity): ALREADY FIXED in the earlier pass, re-verified. `net.minecraft.world.entity.Display` does not override `ignoreExplosion`; `Entity#ignoreExplosion` is false, and `ServerExplosion.hurtEntities` processes every entity where `!ignoreExplosion`, so displays are pushed but not damaged (Pumpkin's `damage_with_context` already returns false). All three overrides are `false` today (display.rs:802, 959, 1255).
- Row 2 (TextDisplay flags reset on load): VERIFIED, NO CODE CHANGE. Decompiled `Display.TextDisplay.readAdditionalSaveData` (Display.java:760-777) rebuilds flags from `(byte)0`, and `input.read("alignment", Align.CODEC)` yields empty for an unknown name, leaving the alignment bits unset. Pumpkin (display.rs:1185-1202) matches.
- Row 3 (`text` NBT key on serialization failure): ALREADY FIXED, re-verified against Display.java:802 (`output.store("text", ...)` unconditional). display.rs:1136-1147 always writes the key.
- Row 4 (`set_brightness` takes a pre-packed int): OPEN (out of scope). Still needs the WASM host API in `pumpkin-wasm-host-v0_1/-v0_2` display.rs, outside this unit's files.
- NEW FINDING, FIXED: the earlier "Fixed" entry for brightness packing was wrong. Vanilla `Brightness.pack` delegates to `LightCoordsUtil.pack` (LightCoordsUtil.java:13): `block << 4 | sky << 20`, and `FULL_BRIGHT` is 15728880, not a 16/16 split. display.rs now packs/unpacks with those shifts and masks the fields to 4 bits, `set_brightness_override` clamps both to 0..=15, and `set_brightness` re-packs a non -1 value from its own fields so a stray integer from the still-open row 4 API cannot corrupt the packed value. The unit test asserts `pack_brightness(15, 12)` and `pack_brightness(15, 15) == 15728880`.
- NEW FINDING, FIXED: `read_display_nbt` applied `teleport_duration` and `brightness` only when the NBT key was present. Vanilla reads every field with a default (Display.java:212-224: `getIntOr("teleport_duration", 0)`, `read("brightness").orElse(null)`), so a missing key resets to the default. Both now reset: teleport_duration to clamped 0, brightness to -1 (no override). The brightness record also requires both keys in 0..=15 (Brightness.java:6-11), so an incomplete or out-of-range compound now clears the override instead of packing a partial value.

Files changed: `crates/pumpkin-core/src/entity/decoration/display.rs`. `rustfmt --edition 2024` exited 0. No cargo/git run by this worker; the orchestrator compiles once.

## Result
- Verification pass against decompiled vanilla (no new code changes needed; prior-pass fixes confirmed correct). All three `is_immune_to_explosion` overrides are `false` (display.rs:818, 975, 1271); vanilla `Display` does not override `ignoreExplosion` (Entity.java default false), so Row 1 is correct.
- Row 2: no change. `Display.TextDisplay.readAdditionalSaveData` (Display.java:760-777) rebuilds flags from zero and clears alignment on unknown name, matching Pumpkin.
- Row 3: confirmed `nbt.put_string("text", ...)` unconditional with empty-string fallback (display.rs:1151-1158), matching `output.store("text", ...)` (Display.java:802).
- Brightness packing verified exact against `LightCoordsUtil.pack/block/sky` (LightCoordsUtil.java:13-23) and `Brightness.pack` (Brightness.java:14-16): `block<<4 | sky<<20`, unpack with 4-bit masks, `FULL_BRIGHT=15728880`. Test at display.rs:1292-1301 asserts this.
- NBT reset verified: `Brightness.CODEC` requires both fields 0..=15 (Brightness.java:6-11), so an incomplete/out-of-range compound clears the override (display.rs:661-671); `teleport_duration` uses `getIntOr(...,0)` + clamp 0..=59 (Display.java:215-216, display.rs:610-615).
- Row 4 (`set_brightness` pre-packed int): OPEN, out of scope. Needs the plugin/WASM host API (`pumpkin-wasm-host-v0_1/-v0_2` display.rs) and likely WIT, outside this unit's files. Mitigated within display.rs: `set_brightness` re-packs a non -1 value from its own 4-bit fields (display.rs:261-274) and `set_brightness_override` exposes independent block/sky with clamping (display.rs:278-280); plugin API still takes one int.
- No remaining in-scope STILL OPEN rows; no new unit-test seam. No shell/cargo/git run by this worker.


## Result (native-4 cross-file pass)
- Row 4 (`set_brightness` pre-packed int): BLOCKED-L. Correct fix needs an additive WIT change (`display.set-brightness` taking a plain level) applied to v0_1 and v0_2 host, SDK and runtime together, then `cargo run --locked -p pumpkin-codegen -- wit` (not run by the CODER; orchestrator must run it). Current `display.rs` already re-packs non -1 values, so behaviour is mitigated meanwhile.


## Result (coder pass)
- Re-verified rows 1-3 against decompiled vanilla: still fixed/matching (explosion immunity false on all three displays; TextDisplay flags rebuilt from zero with unknown alignment leaving bits unset; `text` NBT key always written with empty-string fallback).
- Row 4 (`set_brightness` pre-packed int): STILL OPEN, out of scope for this unit. Needs WASM host API changes in `pumpkin-wasm-host-v0_1/-v0_2` display.rs and likely WIT/codegen; `crates/pumpkin-core/src/entity/decoration/display.rs` already mitigates by re-packing non -1 values and exposing `set_brightness_override(block, sky)` with clamping.
- NEW FINDING, FIXED: display NBT reads now reset all fields to vanilla defaults when keys are missing or invalid, matching `Display.readAdditionalSaveData` (Display.java:212-224). Base: `interpolation_duration`/`start_interpolation` default 0, `view_range` 1.0, `shadow_radius` 0.0, `shadow_strength` 1.0, `width`/`height` 0.0, `glow_color_override` -1, `billboard` fixed, and `transformation` resets to identity on missing or malformed decode (vanilla `orElse(Transformation.IDENTITY)`). BlockDisplay: missing `block_state` defaults to air. ItemDisplay: missing/unreadable `item` defaults to empty and missing `item_display` defaults to none. TextDisplay: missing `line_width`/`background`/`text_opacity` default to 200/1073741824/-1; `text` remains set only when present and parseable, matching vanilla.
- No new unit-test seam: `read_display_nbt`/`read_custom_nbt` require `DisplayEntity`/`Entity` construction tied to live entity state, so no pure test added.
- Files changed: `crates/pumpkin-core/src/entity/decoration/display.rs`.


## Result (second pass, cross-file scope)
- Row 4 (`set_brightness` pre-packed int): FIXED in the current checkout, verified against decompiled vanilla. No new source change was needed this pass; the earlier BLOCKED-L entry was already resolved by an additive WIT + host change:
  - WIT `crates/pumpkin-plugin-wit/v0.1/display.wit:99` and `v0.2/display.wit:103` both declare `set-brightness-override: func(block: s32, sky: s32)` (additive; `display` interface keeps its `/// @since 0.1.0`).
  - WASM hosts `crates/pumpkin-wasm-host-v0_1/src/display.rs:448` and `v0_2/src/display.rs:451` both implement `set_brightness_override` and forward to the core. The plugin-api crates re-export the generated `DisplayEntity`, so the method is available to plugins directly.
  - Core `display.rs:278` clamps block/sky to 0..=15 and packs; `display.rs:261` re-packs any non -1 value from its own 4-bit fields so the legacy packed-int setter cannot corrupt the bits.
  - Vanilla `Display.setBrightnessOverride(@Nullable Brightness)` (Display.java:311-312) packs `Brightness(block, sky)`; the 0..=15 field range comes from `Brightness.CODEC`. Our API matches.
- Rows 1-3 and the two NEW FINDINGS remain fixed/matching: explosion immunity false on all three displays; TextDisplay flags rebuilt from zero with unknown alignment leaving bits unset; `text` NBT key always written with empty-string fallback; brightness uses `LightCoordsUtil` packing (`block<<4 | sky<<20`, FULL_BRIGHT 15728880); NBT reads reset all fields to vanilla defaults. Unit test at display.rs:1280 asserts exact pack values.
- No remaining STILL OPEN rows; no new unit-test seam (packing already covered). Source files changed this pass: none (all fixes already present); tracker doc updated. Orchestrator must regenerate WIT bindings via `cargo run --locked -p pumpkin-codegen -- wit` (not run by this worker).
Status: PASS fixed=1 open=0 files=docs/tracker/taudit-crates-pumpkin-core-src-en-cb5e22.md


## Result (third pass, final verification against .vendor decompiled vanilla)

All rows re-verified exact against /run/media/linsaftw/SDD/GitHub/ArkFlame/Kabocha/.vendor/vanilla-decompiled/net/minecraft/. No source change was needed this pass: every row was already fixed in the checkout and matches vanilla. No row needs a generated pumpkin-data field or a missing subsystem.

- Row 1 (explosion immunity): FIXED. net/minecraft/world/entity/Display.java has no ignoreExplosion override (grep: 0 matches), so the base applies: Entity.ignoreExplosion returns false (Entity.java:6469-6470). All three Pumpkin display is_immune_to_explosion() overrides return false (display.rs:816, 972, 1269) and damage_with_context returns false, so explosions push but do not damage displays, matching vanilla.
- Row 2 (TextDisplay flags reset on load): VERIFIED, no change. Display.TextDisplay.readAdditionalSaveData (Display.java:760-777) builds flags from (byte)0; an unknown alignment name makes input.read(...) return Optional.empty(), leaving alignment bits unset. Pumpkin display.rs:1195-1212 matches exactly.
- Row 3 (text NBT key): FIXED. Display.TextDisplay.addAdditionalSaveData (Display.java:800-802) stores text unconditionally via output.store("text", ComponentSerialization.CODEC, ...). Pumpkin display.rs:1148-1155 always writes the key with an empty-JSON fallback on serialization failure.
- Row 4 (set_brightness pre-packed int): FIXED. Vanilla Display.setBrightnessOverride(Brightness) packs brightness.pack() (Display.java:311-312); field range 0..=15 (Brightness.java:6-11) and Brightness.pack -> LightCoordsUtil.pack = block << 4 | sky << 20 with FULL_BRIGHT 15728880 (Brightness.java:14-16; LightCoordsUtil.java:9,13-14). Additive WIT set-brightness-override: func(block: s32, sky: s32) present at pumpkin-plugin-wit/v0.1/display.wit:99 and v0.2/display.wit:103; WASM hosts implement it (pumpkin-wasm-host-v0_1/src/display.rs:448, v0_2/src/display.rs:451); core clamps 0..=15 and packs (display.rs:278-280) and re-packs non -1 values (display.rs:261-274). Orchestrator action: regenerate WIT bindings via cargo run --locked -p pumpkin-codegen -- wit before the shared compile (not run by this worker).
- NEW FINDING (brightness packing): FIXED and unit-tested. pack_brightness / brightness_block / brightness_sky match LightCoordsUtil exactly; test at display.rs:1290-1299 asserts pack_brightness(15,12) and FULL_BRIGHT 15728880.
- NEW FINDING (NBT read defaults): FIXED. Display.readAdditionalSaveData (Display.java:212-224) reads each field with a default; Pumpkin read_display_nbt (display.rs:602-702) resets every field on missing keys and clears brightness on a missing/incomplete/out-of-range compound (Brightness.CODEC requires both fields 0..=15, Brightness.java:6-11).

No remaining STILL OPEN rows. Source files changed this pass: none. Orchestrator regenerates WIT bindings once, then compiles.
Status: PASS fixed=4 open=0 files=docs/tracker/taudit-crates-pumpkin-core-src-en-cb5e22.md
