# Parity rows for combat-1.8 combat mode

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **combat/1.8 combat mode** [planned/DESIGN] No dedicated implementation source identified. This does not establish that the feature is missing. 

## Result

- **combat/1.8 combat mode** - VERIFIED (matches vanilla). Vanilla has no "1.8 combat mode" feature, toggle or game rule; it ships a single 1.21 melee ruleset. Evidence: `.vendor/vanilla-decompiled/net/minecraft/world/damagesource/CombatRules.java` (one armor/damage ruleset), `.vendor/vanilla-decompiled/net/minecraft/world/entity/player/Player.java:1989-1995` (`getAttackStrengthScale` always derives from `attackStrengthTicker`), `.vendor/vanilla-decompiled/net/minecraft/world/flag/FeatureFlags.java` (no combat-mode flag) and `.vendor/vanilla-decompiled/net/minecraft/world/level/gamerules/GameRules.java` (no combat-mode rule). The only "old attack delay" switch in the decompiled tree is the Canvas/Paper fork config `restoreOldAttackDelayMechanics` (`Player.java:291,1994`), an opt-in server policy, not vanilla and not a parity gap. Pumpkin already matches vanilla with one 1.21 combat ruleset; there is no dedicated implementation source to add. The design-only spec for that opt-in mode lives in `docs/COMBAT_1_8.md` and is explicitly not a gap (`docs/TODO_2.md:168`). No source file is named in this unit, so no code or test was added.
