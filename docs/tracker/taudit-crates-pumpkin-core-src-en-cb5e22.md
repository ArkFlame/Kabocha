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
