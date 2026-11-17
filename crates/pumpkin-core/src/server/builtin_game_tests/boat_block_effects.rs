use pumpkin_data::Block;
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

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

/// A boat dropped into deep water snaps to the surface on first contact and stays there
/// instead of sinking to the floor (AbstractBoat.floatBoat in-air to in-water branch).
pub(super) async fn test_boat_floats_up_after_fall(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    // A three-block-deep pool so a boat that failed to snap would settle near y=1.
    for x in 1..=3 {
        for z in 1..=3 {
            for y in 1..=3 {
                helper
                    .set_block_default(&BlockPos::new(x, y, z), &Block::WATER)
                    .await?;
            }
        }
    }

    helper
        .spawn_entity("minecraft:oak_boat", &BlockPos::new(2, 4, 2))
        .await?;

    helper
        .start_sequence()
        .then_idle(60)
        .then_execute(move |helper| async move {
            // Water surface is at y=4; the snapped boat rests near y=3.5, while a boat that
            // failed to snap would sink to y=1 and fall outside this box.
            helper
                .assert_entity_present_in_box(
                    "minecraft:oak_boat",
                    BlockPos::new(1, 2, 1),
                    BlockPos::new(3, 4, 3),
                )
                .await
        })
        .then_succeed();
    Ok(())
}

/// A submerged boat with a rider ejects that rider after vanilla's 60 underwater ticks.
/// AbstractBoat.tick only advances outOfControlTicks while UNDER_WATER/UNDER_FLOWING_WATER.
pub(super) async fn test_boat_ejects_passenger_after_60_underwater_ticks(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    // Floor one block below the structure so the boat rests in water even at the bottom,
    // keeping it submerged for the whole countdown instead of switching to ON_LAND.
    for x in 0..5 {
        for z in 0..5 {
            helper
                .set_block_default(&BlockPos::new(x, -1, z), &Block::STONE)
                .await?;
        }
    }
    for x in 1..=3 {
        for z in 1..=3 {
            for y in 0..=4 {
                helper
                    .set_block_default(&BlockPos::new(x, y, z), &Block::WATER)
                    .await?;
            }
        }
    }

    let boat_pos = Vector3::new(2.5, 3.0, 2.5);
    helper
        .spawn_entity_at("minecraft:oak_boat", boat_pos)
        .await?;
    helper
        .spawn_entity_at("minecraft:armor_stand", boat_pos)
        .await?;

    let center = BlockPos::new(2, 3, 2);
    helper
        .execute_command(
            &center,
            "ride @e[type=minecraft:armor_stand,distance=..4,limit=1] mount @e[type=minecraft:oak_boat,distance=..4,limit=1]",
        )
        .await?;

    const RIDING: &str =
        "execute as @e[type=minecraft:armor_stand,distance=..4] on vehicle if entity @s";

    helper
        .start_sequence()
        .then_execute(move |helper| async move {
            let riding = helper.execute_command(&center, RIDING).await?;
            helper.assert_true(riding == 1, "expected the armor stand to board the boat")
        })
        .then_wait_until(move |helper| async move {
            let riding = helper.execute_command(&center, RIDING).await.unwrap_or(0);
            helper.assert_true(
                riding == 0,
                "expected the submerged boat to eject its rider after 60 ticks",
            )
        })
        .then_succeed();
    Ok(())
}

/// Destroying a chest boat scatters all 27 container slots as item entities.
/// AbstractChestBoat.remove calls chestVehicleDestroyed, which drops every filled slot.
pub(super) async fn test_chest_boat_scatters_27_slots_on_break(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 5).await?;

    // Entity inventories have no gametest helper, so fill the 27 slots through summon NBT.
    let mut items = String::from("Items:[");
    for slot in 0..27 {
        if slot > 0 {
            items.push(',');
        }
        items.push_str(&format!("{{Slot:{slot}b,id:\"minecraft:stone\"}}"));
    }
    items.push(']');
    let summon = format!("summon minecraft:oak_chest_boat ~2 ~2 ~2 {{{items}}}");
    helper
        .execute_command(&BlockPos::new(0, 0, 0), &summon)
        .await?;

    helper
        .start_sequence()
        .then_execute(move |helper| async move {
            helper
                .assert_entity_present("minecraft:oak_chest_boat")
                .await
        })
        .then_execute(move |helper| async move {
            helper
                .execute_command(
                    &BlockPos::new(2, 2, 2),
                    "damage @e[type=minecraft:oak_chest_boat,distance=..4,limit=1] 100",
                )
                .await
                .map(|_| ())
        })
        .then_wait_until(move |helper| async move {
            helper
                .assert_entity_not_present("minecraft:oak_chest_boat")
                .await?;
            helper
                .assert_item_entity_count("minecraft:stone", &BlockPos::new(2, 2, 2), 3.0, 27)
                .await
        })
        .then_succeed();
    Ok(())
}
