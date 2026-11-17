# entities / Copper Golem

Superseded by [tentities-copper-golem-54a347.md](tentities-copper-golem-54a347.md) (2026-11-16
re-review). The earlier version's "weathering boundary is one tick early" item was wrong: vanilla
`updateWeathering` advances at `getGameTime() >= nextWeatheringTick`, which is what `mob_tick`
already does.
