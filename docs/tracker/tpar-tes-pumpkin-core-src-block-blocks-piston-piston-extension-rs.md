# Parity rows for crates/pumpkin-core/src/block/blocks/piston/piston_extension.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Moving Piston** [partial/UNCOVERED] Head/base removal exists. break_block receives no player, losing the player context; the piston system also has separate movement/persistence gaps. Head/base removal exists.

## Result

- **blocks/Moving Piston** -> FIXED / VERIFIED. The `broken` callback now passes `Some(args.player)` into `break_block`, restoring the player context that the row flagged as lost. The base-removal predicate was extracted into a pure helper and aligned with vanilla `MovingPistonBlock.destroy`: it removes only an extended piston / sticky piston behind the moving-piston head and no longer cascade-removes a `MOVING_PISTON`, matching the vanilla `instanceof PistonBaseBlock && EXTENDED` check. Vanilla references: `.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/MovingPistonBlock.java` (`destroy`, lines 67-74) and `.vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonBaseBlock.java` (extended-head setup, lines 399-408). The creative gate is retained because `MOVING_PISTON` has hardness -1 (`.vendor/vanilla-decompiled/net/minecraft/world/level/block/Blocks.java`, lines 958-969) so only a creative break reaches this callback. Added a pure unit test `removes_only_extended_piston_base_for_creative_break`.
- The "separate movement/persistence gaps" mentioned in the row live in the piston movement / block-entity persistence code (other files and units), outside this unit's only editable basename `piston_extension.rs`; unchanged here and tracked in their own units.
