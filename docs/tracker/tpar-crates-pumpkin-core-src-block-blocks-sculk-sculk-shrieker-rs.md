# Parity rows for crates/pumpkin-core/src/block/blocks/sculk/sculk_shrieker.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **blocks/Sculk Shrieker** [partial/AUDITED] Direct player stepping, warning levels, Darkness application, and Warden spawning are implemented, but world game events do not dispatch vibrations to shriekers. 

## Result
FIXED (third pass). Game-event vibration dispatch to shriekers is wired end to end and now matches vanilla.

Verification against decompiled vanilla:
- .vendor/vanilla-decompiled/net/minecraft/world/level/gameevent/GameEventDispatcher.java:22-64 posts an event to listeners in nearby chunk sections. Pumpkin world/game_event.rs::post_game_event does this and also collects block entities via game_event_radius(), reaching the shrieker.
- .vendor/vanilla-decompiled/net/minecraft/world/level/block/entity/SculkShriekerBlockEntity.java:196-235 VibrationUser: listener radius 8 (Pumpkin GAME_EVENT_RADIUS = 8), canReceiveVibration needs non-shrieking plus a player source, onReceiveVibration calls tryShriek. Pumpkin block/entities/sculk_shrieker.rs implements game_event_radius() and on_game_event accordingly, and block/blocks/sculk/sculk_shrieker.rs::try_shriek matches tryShriek/shriek.
- .vendor/vanilla-decompiled/net/minecraft/world/level/gameevent/vibrations/VibrationSystem.java:421-446 isValidVibration rejects spectators, sneaking players for IGNORE_VIBRATIONS_SNEAKING events, and vibration-dampening sources. Added these checks to the shrieker on_game_event this pass, mirroring entity/passive/allay/vibration.rs::is_valid_vibration.
- .vendor/vanilla-decompiled/net/minecraft/world/level/block/SculkShriekerBlock.java:53-71 direct stepOn and scheduled tick paths are implemented as on_entity_step and on_scheduled_tick.
- No pure unit-test seam: on_game_event needs a live Arc<World> and a player entity, so no test added.

Known simplifications outside this row: vibration travel delay and occlusion for shriekers are not simulated; the generated MINECRAFT_SHRIEKER_CAN_LISTEN tag is incomplete, so event-tag filtering is intentionally not applied (it would suppress footstep-triggered shrieks).
