# Final rows from docs/tracker/taudit-crates-pumpkin-core-src-da-734ab8.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mod.rs

| mod.rs:1729 | `MacroFunction.stringify` (via `Tag.getAsString`) | Floats/doubles are printed with 15 fixed decimals and trailing zeros trimmed, so `1.0` becomes `1` and large values use Rust formatting. Java `Double.toString`/`Float.toString` produce `1.0` and differ at boundaries, changing macro-expanded command text. | Mirror Java `Double.toString`/`Float.toString` output (or add test fixtures from vanilla) before changing behaviour. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:1130 | `FunctionArgument#listSuggestions` | `get_function_names` returns `HashMap` iteration order; vanilla suggestions are alphabetical, so `/function <tab>` output is nondeterministic. | `names.sort_unstable()` before returning. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:2000-2003, 1352+ | `VanillaPackResources` feature packs | `"bundle"` is accepted by known-packs and feature resolution (`mod.rs:941`, `mod.rs:1000+`) but missing from `load_bundled_datapacks` and `get_all_known_packs`, so enabling the bundle experiment loads no data. | Add `"bundle"` to both bundled lists once the assets datapack exists. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |
| mod.rs:1806 | `RegistryDataLoader` raw ids | Custom damage type ids are `u8` and `saturating_add`, so beyond 205 custom types ids collide at 255; vanilla registry ids are ints. | Use a wider id type or reject overflow with a load error. | S, STILL OPEN: not reached in wave 2 h9 budget; needs its own pass |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
