# Parity rows for crates/pumpkin-core/src/block/blocks/piston/piston_extension.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Moving Piston** [partial/UNCOVERED] Head/base removal exists. break_block receives no player, losing the player context; the piston system also has separate movement/persistence gaps. Head/base removal exists.

## Result
STILL OPEN until closed. Per row record: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)
