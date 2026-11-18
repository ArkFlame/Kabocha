# Parity rows for crates/pumpkin-core/src/entity/boss/ender_dragon.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Ender Dragon** [partial/COVERED] Dragon parts are constructed but not registered in the world. Main-box collision handling does not implement the full part-based damage/contact semantics. 

## Result

- entities/Ender Dragon: FIXED / VERIFIED.
  - Part registration was already present: World.dragon_parts plus add_dragon_parts / remove_dragon_parts / dragon_part_by_id and part merging in get_entities_at_box, get_entities_of_type_at_box and the level-wide enumeration in crates/pumpkin-core/src/world/mod.rs. Verified against EnderDragon.getSubEntities and EnderDragonPart in .vendor/vanilla-decompiled/net/minecraft/world/entity/boss/enderdragon/.
  - Collision/contact semantics fixed in crates/pumpkin-core/src/entity/boss/ender_dragon.rs: handle_part_contacts now queries the wing parts with inflate(4,2,4).move(0,-2,0) for the 4.0 push plus 5.0 gated damage, and the head/neck parts with inflate(1.0) for 10.0 damage, instead of the main hitbox. Push uses push_by(xd/dd*4, 0.2, zd/dd*4) matching Entity.push (.vendor/vanilla-decompiled/net/minecraft/world/entity/Entity.java:2661). Added the wasHurtRecently, lastHurtByMobTimestamp < tickCount - 2, team, creative/spectator and per-part gating from EnderDragon.aiStep / knockBack / hurt (.vendor/vanilla-decompiled/net/minecraft/world/entity/boss/enderdragon/EnderDragon.java:294,423,444,552). Two pure unit tests cover the knockback impulse and part box inflation.
