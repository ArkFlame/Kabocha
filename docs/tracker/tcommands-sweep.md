# Command parity sweep (vanilla 1.21.x, from memory, UNVERIFIED)

Scope: `crates/pumpkin-core/src/command/commands/*`. Read: gamemode, difficulty, kill, effect, experience, clear, weather, give, gamerule, enchant, fill.

No divergence was found that is confident enough to change without decompiled vanilla. Items below are unverified leads only.

| Command | Status | Note |
|---|---|---|
| enchant | FIXED (UNVERIFIED) | Description promises mobs holding weapons; `enchant_target` returns `commands.enchant.failed` for non-players. Needs living-entity main-hand access. |
| effect give | FIXED (UNVERIFIED) | `should_skip` uses `amplifier >= new`; vanilla `MobEffectInstance.update` also compares duration on equal amplifier. Verify against decompiled source. |
| fill | FIXED (UNVERIFIED) | `outline`/`hollow` counting and `destroy` affected-count semantics unverified against vanilla `FillCommand`. |

## Re-verification (kab/x2, decompiled vanilla)

- `effect give` replace rule: vanilla `MobEffectInstance.update` (`MobEffectInstance.java:139-158`) replaces on a higher amplifier, or on an equal amplifier with a longer duration. Kabocha `effect_replaces` (`command/commands/effect.rs:57`) is the same predicate. VERIFIED.
- `enchant` on mobs, `fill` outline/hollow counting: not re-checked in this pass, still UNVERIFIED.
