# Parity rows for crates/pumpkin-core/src/item/items/mace.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **items/Mace** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result (third pass, final)

Row `items/Mace` (the only row): VERIFIED. Re-reviewed mace.rs and the whole smash path directly against the decompiled vanilla class at `.vendor/vanilla-decompiled/net/minecraft/world/item/MaceItem.java` (26.3, Paper-patched).

- `MaceItem::is_mace` (mace.rs:13-17) identifies a mace through the item-behaviour registry class, matching vanilla's identification by item class rather than a hardcoded id; covered by unit test `mace_behaviour_is_identified_by_registry_class` (mace.rs:42-46).
- `ItemMetadata::ids` (mace.rs:20-24) registers only `Item::MACE`, correct.
- `can_mine` (mace.rs:26-28) mirrors the existing SwordItem/TridentItem pattern; the item's `Tool` component (`can_destroy_blocks_in_creative: false`, MaceItem.java:51-53 `createToolProperties`) already enforces the creative mining restriction, so this override is redundant but harmless.
- The smash behaviour itself lives in combat.rs / player.rs / mob/mod.rs and matches vanilla: `canSmashAttack` (MaceItem.java:160-162), the three-segment `getAttackDamageBonus` (MaceItem.java:100-124), `getKnockbackPower` / `knockback` / `knockbackPredicate` (MaceItem.java:126-158), `hurtEnemy` / `postHurtEnemy` (MaceItem.java:56-97), and `getItemDamageSource` (MaceItem.java:164-167).
- Deterministic rules already have pure-unit seams: `smash_needs_a_fall_over_one_and_a_half_blocks_and_no_glide`, `mace_smash_damage_curve_matches_vanilla_branches`, `smash_knockback_power_uses_full_3d_distance` (combat.rs) plus the mace.rs registry test. Adding another test would duplicate them, so none was added.

No source change was required this pass: every rule in mace.rs and the smash path already matches vanilla. No generated pumpkin-data field or subsystem is missing. No STILL OPEN rows remain.
