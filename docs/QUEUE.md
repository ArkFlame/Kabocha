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
- #4073 failed: 45 conflict hunks; #4595 failed: plugin manager diverged
- #3630 failed: ~50 conflicting files; #4054 skipped (item merge present entity/item.rs:44)
- #3887 failed: generated-code conflicts, needs codegen run
- batch 11 held out: #4349 (fox goals use removed weather is_thundering), #4115 (copper golem shear duplicate vs #4578), #3871 (cushion Weak get_entity); fix workers needed

## Failed this round (need redesign brief, no read budget)
- #2828 wasm host moved + git-only wit-bindgen deps; #3407 Mob trait lacks mob_damage_with_context/remove_when_far_away, 11 core conflicts; #4039 and #3794 relaunched as redesign workers.
- Issues needing log/video/runtime: #4005 #3989 #4006 #4020 #4048.
- Failed (squash conflicts, need redesign or codegen): #4379 (16 files), #4378 (14 files), #3057 (about 100 files, old crate paths), #3507 (stale base, loot codegen), #4512 (58 files, 1287 hunks in generated block.rs, needs codegen regeneration).
