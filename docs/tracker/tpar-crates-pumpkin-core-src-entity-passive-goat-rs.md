# Parity rows for crates/pumpkin-core/src/entity/passive/goat.rs

Close every row below (status partial/planned in docs/PARITY.tsv): read the vanilla class in .vendor/vanilla-decompiled/net/minecraft/, compare, and make the Rust match. Edit any source file a row needs. No cargo. rustfmt touched files.

- **entities/Goat** [partial/RE-REVIEW] The reviewed source has changed. The previous assessment needs a new review. The reviewed source has changed.

## Result

- `entities/Goat` [partial/RE-REVIEW] -> `FIXED + VERIFIED` (one row).
  - FIXED: `create_horn` now seeds the fresh `RandomSource` from Java `UUID.hashCode()`
    semantics (both 64-bit UUID halves XOR-folded to 32 bits, sign-extended to the `long` seed),
    instead of truncating the low 64 bits. Source: `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/goat/Goat.java:97`
    (`RandomSource.createThreadLocalInstance(this.getUUID().hashCode())`).
  - VERIFIED against `.vendor/vanilla-decompiled/net/minecraft/world/entity/animal/goat/Goat.java` and
    `.../goat/GoatAi.java`: constructor can-float + powder-snow pathfinding malus; created
    attributes (generated GOAT data HP 10 / speed 0.2 / attack 2 matches `createAttributes`);
    `ageBoundaryReached` (1/2 attack damage); `calculateFallDamage` (-10); ambient, hurt,
    death, milk, eat, step sounds; `getBreedOffspring` (random parent screaming trait, short-circuit
    0.02 roll); `isFood` / `GOAT_FOOD`; bucket milking; `finalizeSpawn` (0.02 scream,
    0.1 unihorn); NBT read/write (missing horn keys default true); synched-data defaults;
    hasLeft/RightHorn; `dropHorn` horn selection + velocity; `setScreamingGoat`;
    `getMaxHeadYRot` 15 with head-yaw clamp via `get_max_head_rotation`;
    `checkGoatSpawnRules` (entity/type.rs:676, GOATS_SPAWNABLE_ON + brightness); long-jump
    (600-1200, 5x5, 3.5714288x) and ram (cooldowns, knockback, horn break) goals are configured
    with vanilla constants.
  - Not server-observable / already handled elsewhere, left as-is by design: ramming head
    lowering (`isLoweringHead` / `getRammingXHeadRot` client-only animation; ram START/END
    events broadcast by ram_target.rs), Brain activities vs goal selector (architectural),
    LONG_JUMPING 0.7 pose scale and baby 0.55 age scale (present in generated GOAT entity_type
    data), and entity-RNG vs level-RNG source (crate-wide pattern). No missing generated
    `pumpkin-data` field or absent subsystem blocks this row.
