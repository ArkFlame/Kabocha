# Redstone timing sweep (tredstone2)

Scope: redstone timing constants in `crates/pumpkin-core/src/block/blocks/redstone`.

| Item | Result | Evidence |
|---|---|---|
| Repeater delay | OK, delay*2 ticks = 2/4/6/8 | `abstract_redstone_gate.rs` `get_update_delay_internal` |
| Comparator delay | OK, 2 ticks | `comparator.rs` `get_update_delay_internal` |
| Observer pulse | OK, 2 ticks | `observer.rs` `schedule_tick` |
| Redstone torch burnout | OK, 8 toggles in 60 ticks (existing test) | `redstone_torch.rs` test |
| Target block | OK, 8 ticks (arrow 20) | `target_block.rs` `on_projectile_hit` |
| Pressure plate release | OK, 20 ticks default | `pressure_plate/mod.rs` `tick_rate` |
| Weighted plate power | OK, ceil(min(n,max)/max*15) | `pressure_plate/weighted.rs` `signal_strength` |
| Stone button press | OK, 20 ticks | `buttons.rs` `press_duration` |
| Wood button press | OK, 30 ticks | `buttons.rs` `press_duration` |
| Held wood button reschedule | OK, hardcoded 30 only reached for wood (stone never stays pressed) | `buttons.rs` `on_scheduled_tick` |

Not verified in this pass (no divergence found or not reached): piston, note block, lightning rod,
daylight detector, hopper locking, dropper/dispenser, redstone lamp, powered rail, TNT, sculk sensor,
copper bulb, tripwire, comparator container formula. No code changes made.
