# Parity rows for crates/pumpkin-core/src/block/blocks/piston/piston.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Pistons** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result

- **blocks/Pistons** -> FIXED. Re-audited against .vendor/vanilla-decompiled/net/minecraft/world/level/block/piston/PistonBaseBlock.java (Paper-patched) and SignalGetter.java.
  - is_movable: added the world-border check on the destination block (pos.relative(direction)) alongside the source block, matching isPushable line 251 (getWorldBorder().isWithinBounds(pos) and isWithinBounds(pos.relative(direction)), Paper piston world-border fix).
  - handle_synced_block_event: added Paper triggerEvent guard (lines 143-146) so a queued piston event whose data and 7 direction differs from the piston current FACING is rejected (Direction.from3DDataValue(b1 and 7) vs state.getValue(FACING)).
  - VERIFIED unchanged paths match vanilla: should_extend mirrors getNeighborSignal loops 1 and 3 (loop 2 hasSignal(pos, DOWN) is dead for a non-conductor piston); extend/contract event bodies, sticky pull, arm removal guard, destroy/push ordering and sound pitches all match triggerEvent/moveBlocks. No generated pumpkin-data field, WIT change, or cross-crate refactor required.
Status: PASS fixed=1 open=0 files=crates/pumpkin-core/src/block/blocks/piston/piston.rs
