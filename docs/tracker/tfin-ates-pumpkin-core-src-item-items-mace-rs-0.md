# Final rows from docs/tracker/tpar-crates-pumpkin-core-src-item-items-mace-rs.md

Close each item below completely: read the vanilla class under .vendor/vanilla-decompiled/net/minecraft/, change the Rust to match (edit any file needed, no cargo, rustfmt touched files). Files: mace.rs

No source change was required this pass: every rule in mace.rs and the smash path already matches vanilla. No generated pumpkin-data field or subsystem is missing. No STILL OPEN rows remain.

## Result (final pass)

Row `items/Mace` (the only row in the source tracker): VERIFIED. Re-read mace.rs and the whole smash path directly against the decompiled vanilla class at `.vendor/vanilla-decompiled/net/minecraft/world/item/MaceItem.java` (26.3, Paper-patched).

- `MaceItem::is_mace` (mace.rs:13-17) identifies a mace through the item-behaviour registry class, matching vanilla's identification by item class (MaceItem.java:28) rather than a hardcoded id; pure unit test `mace_behaviour_is_identified_by_registry_class` (mace.rs:42-46) covers it.
- `ItemMetadata::ids` (mace.rs:20-24) registers only `Item::MACE`, correct.
- `can_mine` (mace.rs:26-28) mirrors the existing SwordItem/TridentItem pattern; the item's `Tool` component (`can_destroy_blocks_in_creative: false`, MaceItem.java:51-53 `createToolProperties`) already enforces the creative mining restriction, so this override is redundant but harmless.
- Smash behaviour (combat.rs / player.rs / mob/mod.rs) matches vanilla: `canSmashAttack` via `smash_fall_ready` (combat.rs:84-94; MaceItem.java:160-162), the three-segment `getAttackDamageBonus` curve (combat.rs:113-120; MaceItem.java:100-124), `getKnockbackPower` with full 3D distance and heavy doubling (combat.rs:126-127; MaceItem.java:156-158), knockback radius 3.5 / vertical 0.7 and predicate (combat.rs:131-184; MaceItem.java:126-153), `hurtEnemy` / `postHurtEnemy` (MaceItem.java:56-97), and `getItemDamageSource` (MaceItem.java:164-167).
- Deterministic rules already have pure-unit seams: `smash_needs_a_fall_over_one_and_a_half_blocks_and_no_glide`, `mace_smash_damage_curve_matches_vanilla_branches`, `smash_knockback_power_uses_full_3d_distance` (combat.rs:925, 932, 1003) plus the mace.rs registry test. Adding another test would duplicate them, so none was added.

No source change was required this pass: every rule in mace.rs and the smash path already matches vanilla. No generated pumpkin-data field or subsystem is missing. No STILL OPEN rows remain.
