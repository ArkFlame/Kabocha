# Parity rows for crates/pumpkin-core/src/entity/boss/ender_dragon.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Ender Dragon** [partial/COVERED] Dragon parts are constructed but not registered in the world. Main-box collision handling does not implement the full part-based damage/contact semantics. 

## Result
(each row: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)
