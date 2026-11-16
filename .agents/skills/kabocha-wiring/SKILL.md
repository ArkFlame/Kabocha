---
name: kabocha-wiring
description: Where to register new blocks, items, block entities, mobs, goals, commands in Pumpkin. Use when adding any such thing.
---
# Wiring (paths under crates/pumpkin-core/src/)
| Add | Copy | Register |
|---|---|---|
| Block | block/blocks/dirt_path.rs | block/blocks/mod.rs, block/registry.rs |
| Block entity | block/entities/campfire.rs | block_entity_from_nbt() in block/entities/mod.rs |
| Item | item/items/bucket.rs | default_registry() in item/items/mod.rs |
| Mob | entity/mob/bat.rs | from_type() entity/type.rs; world/natural_spawner.rs |
| AI goal | entity/ai/goal/melee_attack.rs | add_goal/add_target_goal on MobEntity |
| Command | command/commands/time.rs | default_dispatcher() command/commands/mod.rs |
Notes: Block/Item/Goal methods are sync. `#[pumpkin_block]` makes BlockMetadata. Block entity `write_internal()` writes id/x/y/z; `write_nbt()` only own fields. Mob NBT in mob_write_nbt/mob_read_nbt. Forgetting registration is the #1 bug: grep the example's name to find every site.
