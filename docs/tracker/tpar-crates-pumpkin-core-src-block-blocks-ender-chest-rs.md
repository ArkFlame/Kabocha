# Parity rows for crates/pumpkin-core/src/block/blocks/ender_chest.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Ender Chest** [partial/UNCOVERED] Personal inventory works. The opening path lacks the guarded-block interaction that angers piglins. Sitting cats do not obstruct vanilla ender chests, so that is not a parity gap. Personal inventory works.

## Result
(each row: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)
