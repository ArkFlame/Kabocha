# Parity rows for crates/pumpkin-core/src/block/blocks/redstone/sculk_sensor.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Sculk Sensor** [partial/AUDITED] Redstone signal output, calibrated directional filtering, and phase cooldown ticks are implemented, but World::post_game_event only notifies mob listeners and never triggers sculk sensors. 

## Result
(each row: FIXED, VERIFIED with the vanilla path, or BLOCKED-DATA with the exact missing generated field)
