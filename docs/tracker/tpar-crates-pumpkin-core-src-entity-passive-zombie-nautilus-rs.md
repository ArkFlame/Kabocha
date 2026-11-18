# Parity rows for crates/pumpkin-core/src/entity/passive/zombie_nautilus.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Zombie Nautilus** [partial/AUDITED] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result
THIRD PASS complete.

- **entities/Zombie Nautilus** [partial/AUDITED] -> **VERIFIED (matches vanilla)**. Full re-audit against
  `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/nautilus/ZombieNautilus.java`,
  `.../ZombieNautilusAi.java`, `.../AbstractNautilus.java`, `.../NautilusAi.java`. No code divergence found;
  no source edit required. Evidence per behaviour:
  - `sun_protection_slot()==BODY` matches `sunProtectionSlot()` (ZombieNautilus.java:59). `can_be_a_baby()==false`
    and `can_mate()==false` match `canBeABaby()` (177) and `getBreedOffspring()->null` (53-56).
  - `can_be_leashed()==!isAggravated && !isMobControlled` matches `canBeLeashed()` (171-174).
    `requires_custom_persistence()` matches `super.requiresCustomPersistence() || isTame()` (538) and the
    audited sibling `nautilus.rs`; `remove_when_far_away()==true` matches (452-454).
  - All sounds match `getAmbientSound`/`getHurtSound`/`getDeathSound`/`getDashSound`/`getDashReadySound`/
    `playEatingSound`/`getSwimSound` (81-112). `is_food()` matches `AbstractNautilus.isFood()` (99-101):
    ZombieNautilus is never a baby, so `!tame && !baby ? TAMING_ITEMS : FOOD` reduces to the Rust branch.
  - Variant: synced `DATA_VARIANT_ID`, default temperate, NBT key `variant`, biome selection in
    `finalize_spawn`, and `ZOMBIE_NAUTILUS_VARIANT` data-component get/apply match (41-43, 116-169).
    MOVEMENT_SPEED 1.1 from `createAttributes()` (50) is present in generated
    `crates/pumpkin-data/src/generated/entity_type.rs:13397` (1.100000023841858f64); no data fix needed.
  - Dash matches `AbstractNautilus.executeRidersJump()` (348-356): cooldown 40, momentum 1.2 in water /
    0.5 on land, rider `getLookAngle()` * MOVEMENT_SPEED * `getBlockSpeedFactor()`. `tick_abstract_nautilus`
    matches `AbstractNautilus.tick()` (300-320): cancel dash under 35, decrement cooldown + ready sound at 0,
    bubbles, plus the Breath of the Nautilus player effect.
  - AI constants match `ZombieNautilusAi.java`: temptation speed 0.9 (25), attack speed 0.5 (26),
    swim speed 1.0 (24). Brain activities (CORE/IDLE/FIGHT, StartAttacking, ChargeAttack) are adapted to
    the goal selector, the same documented engine adaptation used by the audited sibling `nautilus.rs`;
    no observable behaviour gap (ZombieNautilus has no panic/breed and Pumpkin movement is client authoritative).
  - Existing tests already cover variant/NBT round trip, biome spawn variant, submerged sounds and adult
    invariants; no new seam test warranted.
