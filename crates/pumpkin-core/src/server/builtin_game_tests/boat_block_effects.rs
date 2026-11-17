use pumpkin_data::Block;
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::position::BlockPos;

use super::build_stone_floor;

/// A boat sweeping into a lily pad breaks it via `on_entity_collision`.
pub(super) async fn test_boat_breaks_lily_pad_on_landing(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    for x in 1..=3 {
        for z in 1..=3 {
            helper
                .set_block_default(&BlockPos::new(x, 1, z), &Block::WATER)
                .await?;
        }
    }

    let lily_pad = BlockPos::new(2, 1, 2);
    helper
        .set_block_default(&lily_pad, &Block::LILY_PAD)
        .await?;

    // Drop the boat so its movement sweep crosses the lily pad block.
    helper
        .spawn_entity("minecraft:oak_boat", &BlockPos::new(2, 3, 2))
        .await?;

    helper
        .start_sequence()
        .then_idle(40)
        .then_execute(move |helper| async move {
            helper
                .assert_block_not_present(&lily_pad, &Block::LILY_PAD)
                .await
        })
        .then_succeed();
    Ok(())
}
