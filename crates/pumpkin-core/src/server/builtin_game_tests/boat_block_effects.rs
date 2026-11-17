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

/// A birch boat dropped above water falls and snaps back to the surface, then drops its
/// own item at that position when destroyed (AbstractBoat.destroy spawns the boat item).
pub(super) async fn test_birch_boat_floats_then_drops_on_break(
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

    let drop_pos = BlockPos::new(2, 1, 2);
    helper
        .spawn_entity("minecraft:birch_boat", &BlockPos::new(2, 4, 2))
        .await?;

    helper
        .start_sequence()
        // Let the boat fall out of the air and snap to the water surface.
        .then_idle(40)
        .then_execute(move |helper| async move {
            helper
                .assert_entity_present_in_box(
                    "minecraft:birch_boat",
                    BlockPos::new(0, 1, 0),
                    BlockPos::new(4, 3, 4),
                )
                .await
        })
        .then_execute(move |helper| async move {
            helper
                .execute_command(
                    &drop_pos,
                    "damage @e[type=minecraft:birch_boat,distance=..3,limit=1] 100",
                )
                .await
                .map(|_| ())
        })
        .then_wait_until(move |helper| async move {
            helper
                .assert_entity_not_present("minecraft:birch_boat")
                .await?;
            helper
                .assert_item_entity_present("minecraft:birch_boat", &drop_pos, 1.0)
                .await
        })
        .then_succeed();
    Ok(())
}

/// A chest boat drops its own matching item when destroyed. `AbstractChestBoat.destroy`
/// spawns the chest-boat item (and scatters any stored slots) at the boat's position.
pub(super) async fn test_chest_boat_drops_item_on_break(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;
    let drop_pos = BlockPos::new(2, 1, 2);
    helper
        .spawn_entity("minecraft:oak_chest_boat", &BlockPos::new(2, 1, 2))
        .await?;

    helper
        .start_sequence()
        .then_execute(move |helper| async move {
            helper
                .execute_command(
                    &drop_pos,
                    "damage @e[type=minecraft:oak_chest_boat,distance=..3,limit=1] 100",
                )
                .await
                .map(|_| ())
        })
        .then_wait_until(move |helper| async move {
            helper
                .assert_entity_not_present("minecraft:oak_chest_boat")
                .await?;
            helper
                .assert_item_entity_present("minecraft:oak_chest_boat", &drop_pos, 1.0)
                .await
        })
        .then_succeed();
    Ok(())
}
