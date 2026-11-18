# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-bl-4d09af.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs shelf.rs

| Shelves placed in **flowing** water were not waterlogged | `shelf.rs:325` | `ShelfBlock.getStateForPlacement`: `getFluidState(...).is(Fluids.WATER)` is also true for flowing water (`FlowingFluid.isSame` covers still/flowing) | waterlogged now `matches!(replacing, BlockIsReplacing::Water(_))` instead of the source-only `water_source()` | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `block/mod.rs:667-673` `BlockIsReplacing::water_source` (46 call sites) | every `getStateForPlacement` that uses `FluidState#is(Fluids.WATER)` | only accepts level-0 sources, so all other waterloggable blocks still ignore flowing water | add `BlockIsReplacing::is_water()` and migrate the call sites; shelf already fixed locally | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| `shelf.rs` (absent) | `ShelfBlock.SHAPES` + `useShapeForLightOcclusion() = true` | shelf collision/occlusion shape comes from generated state data; not modelled in this file | verify the generated shape for `#minecraft:wooden_shelves`; engine/codegen work | L, STILL OPEN: size L, skipped per wave brief |
| `shelf.rs` (absent) | `BaseEntityBlock` + block entity | wiki: shelves cannot be pushed by pistons; Pumpkin has no block push-reaction/piston moveability concept | needs an engine-level piston push reaction, not a shelf-local fix | M, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
