# blocks / Infested Blocks

Audit result: no gap left.

- `EntityType::SILVERFISH` maps to `SilverfishEntity` in `from_type` (`crates/pumpkin-core/src/entity/type.rs:268`).
- `InfestedBlock::broken` checks Creative, `block_drops` and Silk Touch, then calls `spawn_infestation` (`crates/pumpkin-core/src/block/blocks/infested.rs:161-175`, `:77-98`).
- The Peaceful check landed in 281427878.

Status: FIXED (existing tree, no new commit). Silverfish spawn uses from_type(EntityType::SILVERFISH) (be572e868 completed the silverfish goal set). Vanilla spawnAfterBreak parity noted as covered by the broken hook.
