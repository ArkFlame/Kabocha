# Queue
Seed with `scripts/upstream-queue.sh prs > docs/queue-prs.tsv` and `... issues > docs/queue-issues.tsv` (2026-11-16: 366 open PRs, 449 open issues upstream; Kabocha has issues disabled). Track state here.
Status: todo | running | verify | merged | blocked | invalid.
| id | source | size | files | status | owner/branch | note |
|---|---|---|---|---|---|---|
- #3755 (PR) needs re-port: PluginHostState moved to wasm-host-common, no inherent impl in v0_1 context.rs
- #3829 failed: Shearable trait conflicts with master inline shearing (dispenser, bogged, sulfur_cube, snow_golem); needs hand redesign
- #4393 witch and #3988 bogged: ported branches kab/n4393 kab/n3988 fail compile (witch.rs arity of 2 calls, Weak<dyn EntityBase>.get_entity, get_difficulty on arc_swap guard in bogged.rs:116); need fix worker
- #2792 partial: protocol types/encoder/pending.rs ported; pumpkin-core outbound queue and world/mod.rs still Bytes (branch kab/n2792, do not land until finished)
- #4392 partial (only magma cube spawn rule + fire immunity ported; AbstractCubeMob/iron golem/slime rewrite pending)
- #3821 failed (fishing bobber/projectile 52 conflicts; needs larger-scope port)
- #3428 failed: master uses HeightMap, PR uses HeightmapType + WG heightmaps; needs redesign
- #3697 follow-up: elytra boost_attached, attached rockets, deflection, NBT
- #3848 failed: palette/light conflicts with #3320 #3427 #4488; #3903 partial: regenerate configured_features_generated.rs via codegen
