# Parity rows for crates/pumpkin-core/src/block/blocks/farmland.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Farmland** [partial/FIXED-ON-MASTER] Water fluid hydration, rain hydration, crop check, fence-gate exception, and trampling are implemented. Entity upward displacement on dirt conversion remains a TODO. Water fluid hydration, rain hydration, crop check, fence-gate exception, and trampling are implemented.

## Result

- **blocks/Farmland** - VERIFIED. crates/pumpkin-core/src/block/blocks/farmland.rs now matches vanilla turnToBaseBlock (.vendor/vanilla-decompiled/net/minecraft/world/level/block/FarmlandBlock.java:146-155): every dirt conversion (on_scheduled_tick -> vanilla tick:92-96, on_landed_upon trampling -> fallOn:115-144, random_tick dry revert -> randomTick:99-112) calls push_entities_up before set_block_state, mirroring vanilla's pushEntitiesUp(state, baseBlock.defaultBlockState(), level, pos) (helper at crates/pumpkin-core/src/block/mod.rs:86-139). Removed the redundant hand-rolled teleport block in on_landed_upon, since vanilla does only pushEntitiesUp with no extra player teleport, so the duplicate was a non-vanilla divergence that could double-displace the trampler. All other rows (water/rain hydration, crop check, fence-gate exception, trampling) were already implemented and confirmed against FarmlandBlock.java. No generated-data or WIT changes needed.
