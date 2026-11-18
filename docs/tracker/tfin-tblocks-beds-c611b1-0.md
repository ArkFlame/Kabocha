# Final rows from docs/tracker/tblocks-beds-c611b1.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: bed.rs dimension.rs environment.rs straw_bed.rs

| `bed.rs:263-264`, `straw_bed.rs:268-269` | `AbstractBedBlock#getBedRule` evaluates `EnvironmentAttributes.BED_RULE` / `STRAW_BED_RULE` at the bed position | Uses `world.dimension.bed_rule` (dimension-level); `world/environment.rs` ignores `pos` | Per-position environment attribute lookup for biome overrides | M | STILL OPEN (per-position environment attribute lookup; M, not attempted) |
| `crates/pumpkin-data/src/generated/dimension.rs:13-18` | `BedRule` record has `canSleep`, `canSetSpawn`, `destroyOnUse`, `destroyOnLeave`, `errorMessage` | Generated `BedRule` has `can_sleep`, `can_set_spawn`, `explodes` only; no `destroy_on_leave`/`error_message` | Extend `pumpkin-codegen` + regenerate from `assets/` | M | STILL OPEN (needs pumpkin-codegen change; generated code not hand-edited) |
| `bed.rs:352-361` | `ServerPlayer#getBedResult:2476` returns `rule.asProblem()`, the per-dimension message | Hardcoded `BLOCK_MINECRAFT_BED_NO_SLEEP`; nether/respawn-anchor style messages and `BedRule.errorMessage` ignored | Needs the `error_message` field above | M | STILL OPEN (blocked on BedRule error_message codegen; M) |
| `bed.rs` failure paths (307-386) | Paper `AbstractBedBlock:112-135` fires `PlayerBedFailEnterEvent` for each `BedSleepingProblem` and allows plugins to override/explode | Only `PlayerBedEnterEvent` is fired, and only on the success path | Add the fail event on each early return | M | STILL OPEN (PlayerBedFailEnterEvent not attempted; M) |

## Result
STILL OPEN until closed. Per item record FIXED, VERIFIED (vanilla path) or BLOCKED-DATA (exact missing generated field).
