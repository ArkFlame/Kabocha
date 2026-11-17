# Player sweep (entity/player.rs, entity/player/*, hunger, breath, fall)

Read-only audit against vanilla FoodData, Player, LivingEntity and ExperienceData semantics.
No code changes: no confident divergence was found. Nothing compiled (CODER policy).

| Area | Location | Result |
|---|---|---|
| Exhaustion cost 4.0, saturation-then-food drain, peaceful skip | `entity/hunger.rs` tick | matches |
| Max exhaustion 40, saturation cap by food level on eat | `entity/hunger.rs` add_exhaustion, eat | matches |
| Natural regen 80 ticks at food>=18, fast regen 10 ticks at sat>0 and food>=20, exhaustion +6 / +saturation | `entity/hunger.rs` tick | matches |
| Starvation 80-tick timer, Easy/Normal/Hard floor rules | `entity/hunger.rs` tick | matches |
| Restart values (20, 5, 0) | `entity/hunger.rs` restart | matches |
| Jump exhaustion 0.05 / 0.2 sprinting | `entity/player.rs` jump | matches |
| Attack exhaustion 0.1 on landed hit | `entity/player.rs` attack | matches |
| Block-break exhaustion 0.005 | `entity/player.rs` MINE_BLOCK_EXHAUSTION | matches |
| Movement exhaustion: sprint 0.1/m on ground, 0.01/m swim or in water | `entity/player.rs` progress_motion | matches |
| Air supply 300, -1 per tick submerged, +4 recovery, damage 2 every 20 ticks once air <= 0 | `entity/breath.rs` | matches observable timing |
| Drowning reset on respawn and creative/spectator full air | `entity/breath.rs` | matches |
| Experience per-level points (2L+7, 5L-38, 9L-158) and total-to-level | `pumpkin-util/src/math/experience.rs` | matches |
| Death XP drop min(level*7, 100) | `entity/player.rs` get_experience_reward | matches |
| Item pickup delay 10, player drop delay 40, thrower set | `entity/item.rs`, `entity/player.rs` drop_item | matches |
| Swim/sprint pose rules (`update_swimming`) | `entity/player.rs` | matches |
| Spawn protection: overworld only, non-zero permission bypass, no ops disables, square radius | `world/mod.rs` is_in_spawn_protection | matches (bypass level wording unverified) |
| Gamemode switch: abilities reset, spectator clears fly/sneak/fall distance, camera reset | `entity/player.rs` set_gamemode | matches |
| Respawn resets mining and re-sends attack attributes | `entity/player.rs` respawn | matches |

## Uncertain, not changed

- Fall damage: resolved. `living.rs` `base_fall_damage` now uses vanilla `ceil((distance - safe - jumpBoostLevels) * modifier * FALL_DAMAGE_MULTIPLIER)` via `fall_damage_amount`, with Jump Boost subtracted (was `floor` with no Jump Boost term). Armor still applies, since vanilla fall damage is not in `bypasses_armor`; the brief's armor-ignoring claim was not adopted.
- Spawn protection bypass: kept. Pumpkin bypasses for any permission level above zero and applies in every gamemode. Vanilla bypasses ops by op-list membership (a level-0 op would bypass in vanilla only) and, per my knowledge, also restricts creative non-ops, so the adventure/survival-only claim was not adopted. Level-0 op edge unverified.
- Feather falling, slow falling, hay, slime reductions rely on `MINECRAFT_FALL_DAMAGE_RESETTING` tags and `PREVENT_AREA_FALL_DAMAGE_BLOCKS`; the tag contents were not audited here.
- Burn and lava damage paths were not read in this pass.
