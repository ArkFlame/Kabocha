use pumpkin_data::Block;
use pumpkin_gametest::{GameTestContext, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

pub(super) async fn test_snow_golem_pattern_spawns(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let base_pos = BlockPos::new(2, 1, 2);
    let mid_pos = BlockPos::new(2, 2, 2);
    let head_pos = BlockPos::new(2, 3, 2);

    test.set_block_default(&base_pos, &Block::SNOW_BLOCK)
        .await?;
    test.set_block_default(&mid_pos, &Block::SNOW_BLOCK).await?;
    test.set_block_default(&head_pos, &Block::CARVED_PUMPKIN)
        .await?;

    test.assert_block_present(&base_pos, &Block::AIR).await?;
    test.assert_block_present(&mid_pos, &Block::AIR).await?;
    test.assert_block_present(&head_pos, &Block::AIR).await?;
    test.assert_entity_present("minecraft:snow_golem").await?;

    test.succeed();
    Ok(())
}

pub(super) async fn test_iron_golem_pattern_spawns(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let base_pos = BlockPos::new(2, 1, 2);
    let mid_pos = BlockPos::new(2, 2, 2);
    let left_arm = BlockPos::new(1, 2, 2);
    let right_arm = BlockPos::new(3, 2, 2);
    let head_pos = BlockPos::new(2, 3, 2);

    test.set_block_default(&base_pos, &Block::IRON_BLOCK)
        .await?;
    test.set_block_default(&mid_pos, &Block::IRON_BLOCK).await?;
    test.set_block_default(&left_arm, &Block::IRON_BLOCK)
        .await?;
    test.set_block_default(&right_arm, &Block::IRON_BLOCK)
        .await?;
    test.set_block_default(&head_pos, &Block::CARVED_PUMPKIN)
        .await?;

    for pos in [base_pos, mid_pos, left_arm, right_arm, head_pos] {
        test.assert_block_present(&pos, &Block::AIR).await?;
    }
    test.assert_entity_present("minecraft:iron_golem").await?;

    test.succeed();
    Ok(())
}

pub(super) async fn test_wither_pattern_spawns(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let base_pos = BlockPos::new(2, 1, 2);
    let mid_pos = BlockPos::new(2, 2, 2);
    let left_arm = BlockPos::new(1, 2, 2);
    let right_arm = BlockPos::new(3, 2, 2);
    let left_skull = BlockPos::new(1, 3, 2);
    let right_skull = BlockPos::new(3, 3, 2);
    let center_skull = BlockPos::new(2, 3, 2);

    for pos in [base_pos, mid_pos, left_arm, right_arm] {
        test.set_block_default(&pos, &Block::SOUL_SAND).await?;
    }
    for pos in [left_skull, right_skull, center_skull] {
        test.set_block_default(&pos, &Block::WITHER_SKELETON_SKULL)
            .await?;
    }

    for pos in [
        base_pos,
        mid_pos,
        left_arm,
        right_arm,
        left_skull,
        right_skull,
        center_skull,
    ] {
        test.assert_block_present(&pos, &Block::AIR).await?;
    }
    test.assert_entity_present("minecraft:wither").await?;

    test.succeed();
    Ok(())
}

pub(super) async fn test_copper_golem_collects_items_at_path_end(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;
    for x in 0..5 {
        for z in 0..5 {
            if x == 0 || x == 4 || z == 0 || z == 4 {
                for y in 1..3 {
                    test.set_block_default(&BlockPos::new(x, y, z), &Block::STONE)
                        .await?;
                }
            }
        }
    }
    let source = BlockPos::new(1, 1, 2);
    test.set_block_default(&source, &Block::WAXED_COPPER_CHEST)
        .await?;
    test.set_container_item(&source, 0, "minecraft:iron_ingot", 16)
        .await?;
    test.spawn_entity_at("minecraft:copper_golem", Vector3::new(3.5, 1.0, 2.5))
        .await?;

    test.start_sequence()
        .then_wait_until(|test| async move { test.assert_container_empty(&source).await })
        .then_succeed();
    Ok(())
}
