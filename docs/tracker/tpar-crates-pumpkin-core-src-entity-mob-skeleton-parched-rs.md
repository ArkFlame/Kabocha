# Parity rows for crates/pumpkin-core/src/entity/mob/skeleton/parched.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Parched** [partial/COVERED] The shared bow attack uses a plain ARROW rather than the variant-specific payload. This was previously only marked unverified. The shared bow attack uses a plain ARROW rather than the variant-specific payload.

## Result
(each row: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)
