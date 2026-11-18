# Parity rows for crates/pumpkin-core/src/block/blocks/beacon.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Beacon** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result
- blocks/Beacon FIXED. Vanilla path: .vendor/vanilla-decompiled/net/minecraft/world/level/block/BeaconBlock.java. useWithoutItem always returns InteractionResult.SUCCESS, and awards INTERACT_WITH_BEACON only when player.openMenu(beacon).isPresent(). Pumpkin now returns BlockActionResult::Success unconditionally (no longer Fail when no factory) and increments the stat only when open_handled_screen(...) returns Some (the equivalent of isPresent()), so a cancelled or failed menu open no longer awards the stat. No pure unit-test seam exists (needs a live Player/World), so none added. Verified by source comparison; not compiled by this worker (orchestrator compiles).
