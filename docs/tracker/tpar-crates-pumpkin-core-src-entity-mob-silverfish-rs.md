# Parity rows for crates/pumpkin-core/src/entity/mob/silverfish.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Silverfish** [partial/COVERED] The countdown decreases but does not scan/break infested blocks on expiry. No stone-merging behaviour is wired. 

## Result
(each row: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)
