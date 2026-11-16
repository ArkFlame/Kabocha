# Queue
Seed with `scripts/upstream-queue.sh prs > docs/queue-prs.tsv` and `... issues > docs/queue-issues.tsv` (2026-11-16: 366 open PRs, 449 open issues upstream; Kabocha has issues disabled). Track state here.
Status: todo | running | verify | merged | blocked | invalid.
| id | source | size | files | status | owner/branch | note |
|---|---|---|---|---|---|---|
- #3755 (PR) needs re-port: PluginHostState moved to wasm-host-common, no inherent impl in v0_1 context.rs
- #3829 failed: Shearable trait conflicts with master inline shearing (dispenser, bogged, sulfur_cube, snow_golem); needs hand redesign
- #4393 witch and #3988 bogged: ported branches kab/n4393 kab/n3988 fail compile (witch.rs arity of 2 calls, Weak<dyn EntityBase>.get_entity, get_difficulty on arc_swap guard in bogged.rs:116); need fix worker
