# Entity TODO triage (crates/pumpkin-core/src/entity)

Resolved: parched skeleton attack interval (70/50), player portal cooldown pause, stale enchant-damage TODO removed.
Excluded (owned by other workers): hoglin.rs:196.

Left open (reason):
- predicate/mod.rs:34 ValidInventories: no vanilla predicate in .vendor decompile; needs data/design decision.
- mob/mod.rs:1063 defrost: needs powder-snow thaw path (Entity freeze/thaw), larger than one pass.
- mob/mod.rs:1402 / player.rs:9080 team color: needs team color data plumbing (scoreboard team -> name color).
- mob/mod.rs:1442 "remove once all entities implemented": design cleanup, not behaviour.
- mob/mod.rs:3147 move to on_stepped_on: refactor of step hooks, not a vanilla gap.
- mob/mod.rs:948 Bedrock climate_variant: Bedrock is separate and incomplete.
- mob/mod.rs:3510 SpawnReason attribute: needs spawn-reason plumbing through Mob construction.
- ai/goal/track_target.rs:88 "after implementing path": navigator returns pending paths via set_progress, no sync moveTo result.
- ai/goal/melee_attack.rs:254 +15 ticks on failed path: vanilla uses synchronous moveTo bool; Pumpkin navigator is async (set_progress), no equivalent result.
- ai/goal/active_target.rs:48 Mob::is_baby unification: refactor across baby-state sources.
- player.rs:1030 hunger load from previous instance: needs player save/load of hunger data.
- player.rs:1067 CPlayerSpawnPosition values: needs spawn-point packet values verified against vanilla in a runtime check.
- player.rs:5232 duplicate world code, player.rs:5356 change-world event ordering: refactor with event ordering risk.
- player.rs:5515, 5523 delta movement and relative flags: client-authoritative movement, needs protocol verification.
- player.rs:6001 difficulty scaling for hurtServer: needs scalesWithDifficulty damage-type data.
- player.rs:6525 mojang bug workaround: no concrete change identified.
- player.rs:7201 atomic ABA issue: concurrency redesign.
- player.rs:7330 broadcast metadata: needs entity metadata broadcast path verified in runtime.
- living.rs:1258 death config option: config design.
- living.rs:1948 FIXME raycast hit/miss: needs vanilla block-raycast ordering audit.
- living.rs:2296 gravity when world unloaded: needs vanilla unloaded-chunk behaviour.
- living.rs:2579 commented-out sneaking-on-block logic: needs block-under-entity audit before enabling.
- living.rs:5544 data-driven list: needs data pack system support.
- mob/vex.rs:105 VexCopyOwnerTargetGoal: needs owner tracking from evoker summon.
- mob/shulker.rs:110 defense search box: needs attach-face geometry audit.
- mob/skeleton/mod.rs:33-ish constants: resolved above.
- passive/strider.rs:98 jockey spawn roll: needs finalizeSpawn plumbing.
- tnt.rs:32 usedPortal / ignoreExplosion: needs dimension-change explosion state.
