use std::cell::RefCell;
use std::sync::Arc;

use crate::block::entities::BlockEntity;
use crate::block::entities::chest::ChestBlockEntity;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{ChestLikeProperties, ChestType, HorizontalFacing};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::{Block, BlockDirection, BlockId, translation};
use pumpkin_inventory::Inventory;
use pumpkin_inventory::double::DoubleInventory;
use pumpkin_inventory::generic_container_screen_handler::{create_generic_9x3, create_generic_9x6};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::{pumpkin_block, pumpkin_block_from_tag};
use pumpkin_util::GameMode;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use std::sync::Mutex;

use crate::block::{
    BlockBehaviour, BrokenArgs, EmitsRedstonePowerArgs, GetComparatorOutputArgs,
    GetRedstonePowerArgs, GetScreenHandlerFactoryArgs, GetStateForNeighborUpdateArgs,
    NormalUseArgs, OnPlaceArgs, OnSyncedBlockEventArgs, PathComputationType, PlacedArgs,
    RandomTickArgs, registry::BlockActionResult,
};
use crate::entity::EntityBase;
use crate::entity::passive::cat::CatEntity;
use crate::entity::passive::tamable::TamableAnimal;
use crate::entity::player::Player;
use crate::plugin::api::events::world::loot_generate::LootGenerateEvent;
use crate::world::World;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};
use pumpkin_data::BlockState;

// TODO vanilla `BaseContainerBlockEntity.canOpen`: the `lock` component, with `container.isLocked` and the
// `CHEST_LOCKED` sound on a failed open.
struct ChestScreenFactory {
    inventory: Arc<dyn Inventory>,
    loot_pending: bool,
    name: Option<TextComponent>,
    double_chest: bool,
}

impl ScreenHandlerFactory for ChestScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        if self.loot_pending && player.is_spectator() {
            return None;
        }

        let concrete_handler = if self.inventory.size() > 27 {
            create_generic_9x6(sync_id, player_inventory, self.inventory.clone(), player)
        } else {
            create_generic_9x3(sync_id, player_inventory, self.inventory.clone(), player)
        };

        let concrete_arc = Arc::new(Mutex::new(concrete_handler));

        Some(concrete_arc as SharedScreenHandler)
    }

    /// Vanilla `ChestBlock.MENU_PROVIDER_COMBINER.getDisplayName`: a custom name on
    /// either half wins; double chests without one use `container.chestDouble`
    /// ("Large Chest"), single chests use `container.chest`.
    fn get_display_name(&self) -> TextComponent {
        chest_display_name(self.name.as_ref(), self.double_chest)
    }
}

/// Resolves the chest menu title the way vanilla's menu providers do.
fn chest_display_name(custom_name: Option<&TextComponent>, double_chest: bool) -> TextComponent {
    if let Some(name) = custom_name {
        return name.clone();
    }
    if double_chest {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_CHESTDOUBLE,
            translation::bedrock::CONTAINER_CHESTDOUBLE
        )
    } else {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_CHEST,
            translation::bedrock::CONTAINER_CHEST
        )
    }
}

// Shared chest behavior implementations
const LID_ANIMATION_EVENT_TYPE: u8 = 1;

#[derive(Clone)]
pub(crate) struct ChestContainer {
    pub inventory: Arc<dyn Inventory>,
    pub positions: Vec<BlockPos>,
    halves: Vec<Arc<dyn Inventory>>,
}

impl ChestContainer {
    pub(crate) fn single(position: BlockPos, inventory: Arc<dyn Inventory>) -> Self {
        Self {
            inventory: inventory.clone(),
            positions: vec![position],
            halves: vec![inventory],
        }
    }

    pub fn same_inventory(&self, other: &Self) -> bool {
        self.halves.len() == other.halves.len()
            && self
                .halves
                .iter()
                .zip(&other.halves)
                .all(|(left, right)| Arc::ptr_eq(left, right))
    }

    pub fn try_start_open(&self) -> bool {
        use std::sync::atomic::Ordering;
        for (index, half) in self.halves.iter().enumerate() {
            let acquired = chest_reservation(half.as_ref()).is_some_and(|(reserved, viewers)| {
                viewers == 0
                    && reserved
                        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
            });
            if !acquired {
                for previous in &self.halves[..index] {
                    if let Some((reserved, _)) = chest_reservation(previous.as_ref()) {
                        reserved.store(false, Ordering::Release);
                    }
                }
                return false;
            }
        }
        self.inventory.on_open();
        true
    }

    pub fn stop_open(&self) {
        self.inventory.on_close();
        for half in &self.halves {
            if let Some((reserved, _)) = chest_reservation(half.as_ref()) {
                reserved.store(false, std::sync::atomic::Ordering::Release);
            }
        }
    }

    pub fn modify_stack<R>(
        &self,
        mut slot: usize,
        update: impl FnOnce(&mut ItemStack) -> R,
    ) -> Option<R> {
        for half in &self.halves {
            if slot >= half.size() {
                slot -= half.size();
                continue;
            }
            if let Some(chest) = half.as_any().downcast_ref::<ChestBlockEntity>() {
                return chest.modify_stack(slot, update);
            }
            if let Some(chest) = half
                .as_any()
                .downcast_ref::<crate::block::entities::trapped_chest::TrappedChestBlockEntity>(
            ) {
                return chest.modify_stack(slot, update);
            }
            return None;
        }
        None
    }

    pub fn is_open(&self, world: &World) -> bool {
        self.positions.iter().any(|position| {
            world.get_block_entity(position).is_some_and(|entity| {
                entity
                    .as_any()
                    .downcast_ref::<ChestBlockEntity>()
                    .is_some_and(|chest| chest.get_viewer_count() > 0)
                    || entity
                        .as_any()
                        .downcast_ref::<crate::block::entities::trapped_chest::TrappedChestBlockEntity>()
                        .is_some_and(|chest| chest.get_viewer_count() > 0)
            })
        })
    }

    pub fn unpack_loot(&self, world: &World) {
        for position in &self.positions {
            if let Some(entity) = world.get_block_entity(position) {
                unpack_loot_table(world, &entity, None);
            }
        }
    }
}

fn chest_reservation(inventory: &dyn Inventory) -> Option<(&std::sync::atomic::AtomicBool, u16)> {
    inventory
        .as_any()
        .downcast_ref::<ChestBlockEntity>()
        .map(|chest| (&chest.golem_reserved, chest.get_viewer_count()))
        .or_else(|| {
            inventory
                .as_any()
                .downcast_ref::<crate::block::entities::trapped_chest::TrappedChestBlockEntity>()
                .map(|chest| (&chest.golem_reserved, chest.get_viewer_count()))
        })
}

pub(crate) fn get_chest_container(world: &World, position: &BlockPos) -> Option<ChestContainer> {
    let (block, state) = world.get_block_and_state(position);
    if !block.id.has_tag(pumpkin_data::tag::Block::C_CHESTS_WOODEN)
        && !block
            .id
            .has_tag(pumpkin_data::tag::Block::MINECRAFT_COPPER_CHESTS)
        && block != &Block::TRAPPED_CHEST
    {
        return None;
    }
    if is_chest_blocked(world, position) {
        return None;
    }
    let first = world.get_block_entity(position)?.get_inventory()?;
    let props = ChestLikeProperties::from_state_id(state.id);
    let direction = match props.r#type {
        ChestType::Single => {
            return Some(ChestContainer::single(*position, first));
        }
        ChestType::Left => props.facing.rotate_clockwise(),
        ChestType::Right => props.facing.rotate_counter_clockwise(),
    };
    let neighbor = position.offset(direction.to_offset());
    get_chest_properties_if_can_connect(
        world,
        block,
        position,
        props.facing,
        direction,
        props.r#type.opposite(),
    )?;
    if is_chest_blocked(world, &neighbor) {
        return None;
    }
    let second = world.get_block_entity(&neighbor)?.get_inventory()?;
    let (first, second) = if props.r#type == ChestType::Right {
        (first, second)
    } else {
        (second, first)
    };
    let inventory = DoubleInventory::new(first.clone(), second.clone());
    Some(ChestContainer {
        inventory,
        positions: vec![*position, neighbor],
        halves: vec![first, second],
    })
}

/// Vanilla `RandomizableContainer.unpackLootTable`; luck and `this_entity` only come from a player.
fn on_place_chest_impl(args: &OnPlaceArgs<'_>) -> BlockStateId {
    let mut chest_props = ChestLikeProperties::default(args.block);
    chest_props.waterlogged = args.replacing.is_water();

    let (r#type, facing) = compute_chest_props(
        args.world,
        args.player,
        args.block,
        args.position,
        args.direction,
    );
    chest_props.facing = facing;
    chest_props.r#type = r#type;

    chest_props.to_state_id(args.block)
}

fn placed_chest_impl<E: BlockEntity + 'static>(
    args: &PlacedArgs<'_>,
    create_entity: impl FnOnce(BlockPos) -> E,
) {
    if args.world.get_block_entity(args.position).is_none() {
        let chest = create_entity(*args.position);
        args.world.add_block_entity(Arc::new(chest));
    }

    let chest_props = ChestLikeProperties::from_state_id(args.state_id);
    let connected_towards = match chest_props.r#type {
        ChestType::Single => return,
        ChestType::Left => chest_props.facing.rotate_clockwise(),
        ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
    };

    if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
        args.world,
        args.block,
        args.position,
        chest_props.facing,
        connected_towards,
        ChestType::Single,
    ) {
        neighbor_props.r#type = chest_props.r#type.opposite();

        args.world.set_block_state(
            &args.position.offset(connected_towards.to_offset()),
            neighbor_props.to_state_id(args.block),
            BlockFlags::NOTIFY_LISTENERS,
        );
    }
}

/// Direction to the other half of a double chest, `None` for a single chest.
const fn connected_direction(props: ChestLikeProperties) -> Option<HorizontalFacing> {
    match props.r#type {
        ChestType::Single => None,
        ChestType::Left => Some(props.facing.rotate_clockwise()),
        ChestType::Right => Some(props.facing.rotate_counter_clockwise()),
    }
}

/// Vanilla `DoubleBlockCombiner.combineWithNeigbour`: the other half counts only while it is the
/// same block, faces the same way and is the opposite half.
fn connected_half(
    world: &World,
    block: &Block,
    position: &BlockPos,
    props: ChestLikeProperties,
) -> Option<BlockPos> {
    let direction = connected_direction(props)?;
    get_chest_properties_if_can_connect(
        world,
        block,
        position,
        props.facing,
        direction,
        props.r#type.opposite(),
    )?;
    Some(position.offset(direction.to_offset()))
}

/// Vanilla `CompoundContainer` order: the `RIGHT` half comes first.
fn combine_halves(
    chest_type: ChestType,
    this: Arc<dyn Inventory>,
    other: Arc<dyn Inventory>,
) -> Arc<dyn Inventory> {
    if chest_type == ChestType::Right {
        DoubleInventory::new(this, other)
    } else {
        DoubleInventory::new(other, this)
    }
}

thread_local! {
    /// Chests (world address, position) whose `LootGenerateEvent` is being handled on this thread.
    static LOOT_EVENT_DISPATCH: RefCell<Vec<(usize, BlockPos)>> = const { RefCell::new(Vec::new()) };
}

/// Keeps a chest in [`LOOT_EVENT_DISPATCH`] while its event fires, also if a handler panics.
struct LootEventDispatch;

impl LootEventDispatch {
    fn enter(chest: (usize, BlockPos)) -> Self {
        LOOT_EVENT_DISPATCH.with_borrow_mut(|chests| chests.push(chest));
        Self
    }
}

impl Drop for LootEventDispatch {
    fn drop(&mut self) {
        LOOT_EVENT_DISPATCH.with_borrow_mut(Vec::pop);
    }
}

/// Vanilla `RandomizableContainer.unpackLootTable`: fills the chest from its pending loot table once.
fn unpack_loot_table(world: &World, entity: &Arc<dyn BlockEntity>, player: Option<&Player>) {
    // Fired before the loot lock -> a handler reading this chest would unpack it again.
    // A cancel still clears the key, the chest just stays empty.
    // TODO let `LootGenerateEvent` carry the container, context and generated items so a plugin can edit the loot.
    // TODO concurrent first unpacks on different threads each fire the event, only the one that takes the
    // key decides. One event per unpack needs an "unpacking" loot state that other threads wait on.
    let chest = (std::ptr::from_ref(world) as usize, entity.get_position());
    // A handler reading this chest lands here again: it stays packed, the outer call decides.
    if LOOT_EVENT_DISPATCH.with_borrow(|chests| chests.contains(&chest)) {
        return;
    }
    let cancelled_key = entity.loot_table_key().filter(|key| {
        let _dispatch = LootEventDispatch::enter(chest);
        world.cancelled_by_plugin(|| LootGenerateEvent::new(key.clone()))
    });
    let mut unpacked = None;
    entity.unpack_loot_table_with(&mut |loot_key, seed| {
        // The cancel only covers the key the event saw.
        let items = if cancelled_key.as_deref() == Some(loot_key.as_str()) {
            Vec::new()
        } else {
            generate_chest_loot(world, entity, player, &loot_key, seed)
        };
        unpacked = Some(loot_key);
        items
    });
    // Outside the loot lock: advancement handlers must not run while it is held.
    // TODO vanilla `CriteriaTriggers.GENERATE_LOOT` runs before `setLootTable(null)` and `LootTable.fill`.
    if let (Some(player), Some(loot_key)) = (player, unpacked) {
        player.trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::PlayerGeneratesContainerLoot {
                loot_table_id: loot_key,
            },
        );
    }
}

fn generate_chest_loot(
    world: &World,
    entity: &Arc<dyn BlockEntity>,
    player: Option<&Player>,
    loot_key: &str,
    seed: i64,
) -> Vec<ItemStack> {
    // Vanilla resolves an unknown key to `LootTable.EMPTY`: the key is cleared, nothing is placed.
    let Some(table) = world.get_loot_table(loot_key) else {
        return Vec::new();
    };
    let params = LootContextParameters {
        world: Some(world),
        // Vanilla `ORIGIN` is `Vec3.atCenterOf` of the half being unpacked.
        position: Some(entity.get_position().to_centered_f64()),
        this_entity: player.map(|player| player as &dyn EntityBase),
        luck: player.map_or(0.0, |player| {
            player
                .living_entity
                .get_attribute_value(&pumpkin_data::attributes::Attributes::LUCK) as f32
        }),
        ..Default::default()
    };
    generate_loot_from_handle(&table, seed, &params)
}

/// Vanilla `ChestBlock.getContainer` with `ignoreBeingBlocked = true`: a double chest is one container.
///
/// Vanilla unpacks a half on its first slot access. every caller reads slots right away, so both
/// halves unpack here.
pub fn chest_container(
    world: &World,
    block: &Block,
    position: &BlockPos,
    state_id: BlockStateId,
    entity: &Arc<dyn BlockEntity>,
) -> Option<Arc<dyn Inventory>> {
    unpack_loot_table(world, entity, None);
    let this = entity.clone().get_container()?;
    let props = ChestLikeProperties::from_state_id(state_id);
    let other = connected_half(world, block, position, props)
        .and_then(|other_pos| world.get_block_entity(&other_pos))
        .and_then(|other| {
            unpack_loot_table(world, &other, None);
            other.get_container()
        });
    Some(match other {
        Some(other) => combine_halves(props.r#type, this, other),
        None => this,
    })
}

/// Computes the comparator output for a chest, combining both halves for double chests.
fn get_chest_comparator_output(args: &GetComparatorOutputArgs<'_>) -> Option<u8> {
    let state = args.world.get_block_state_id(args.position);
    let first_chest = args.world.get_block_entity(args.position)?;

    let chest_props = ChestLikeProperties::from_state_id(state);
    let other_pos = connected_half(args.world, args.block, args.position, chest_props);

    // Vanilla passes `ignoreBeingBlocked = false`, so a blocked half reads zero.
    if is_chest_blocked(args.world, args.position)
        || other_pos.is_some_and(|other_pos| is_chest_blocked(args.world, &other_pos))
    {
        return Some(0);
    }

    // Vanilla reads every slot through `getItem`, which unpacks pending loot first.
    let second_chest = other_pos.and_then(|other_pos| args.world.get_block_entity(&other_pos));
    for entity in std::iter::once(&first_chest).chain(&second_chest) {
        unpack_loot_table(args.world, entity, None);
    }

    let first_inventory = first_chest.get_inventory()?;
    let inventory = match second_chest.and_then(BlockEntity::get_inventory) {
        Some(second_inventory) => {
            combine_halves(chest_props.r#type, first_inventory, second_inventory)
        }
        None => first_inventory,
    };
    Some(crate::block::calculate_comparator_output(
        inventory.as_ref(),
    ))
}

/// Returns a screen handler factory for opening a chest or double chest inventory.
///
/// Unpacks deferred loot tables on first open (for non-spectator players) and combines
/// connected inventories for double chests if neither chest half is obstructed.
fn get_chest_screen_handler_factory(
    args: GetScreenHandlerFactoryArgs<'_>,
) -> Option<Box<dyn ScreenHandlerFactory>> {
    let state = args.world.get_block_state_id(args.position);
    let first_chest = args.world.get_block_entity(args.position);

    let chest_props = ChestLikeProperties::from_state_id(state);
    let other_pos = connected_half(args.world, args.block, args.position, chest_props);
    let second = other_pos.and_then(|other_pos| args.world.get_block_entity(&other_pos));

    // Vanilla titles from the RIGHT half, then the LEFT half. Resolve both entities so a
    // custom name on either half wins regardless of which half the player clicked.
    let (first_half, second_half) = match chest_props.r#type {
        ChestType::Left => (second.clone(), first_chest.clone()),
        _ => (first_chest.clone(), second.clone()),
    };
    let name = first_half
        .and_then(|entity| entity.get_custom_name())
        .or_else(|| second_half.and_then(|entity| entity.get_custom_name()));

    // Vanilla gives a blocked chest no menu, so it stays packed too.
    if is_chest_blocked(args.world, args.position)
        || other_pos.is_some_and(|other_pos| is_chest_blocked(args.world, &other_pos))
    {
        return None;
    }

    // Both halves of a double chest are unpacked at once, like vanilla's CompoundContainer.
    if args.player.gamemode.load() != GameMode::Spectator {
        for entity in first_chest.iter().chain(&second) {
            unpack_loot_table(args.world, entity, Some(args.player));
        }
    }

    let loot_pending = first_chest
        .iter()
        .chain(&second)
        .any(|entity| entity.has_loot_table());
    let first_inventory = first_chest.and_then(BlockEntity::get_inventory)?;

    let inventory = match second.and_then(BlockEntity::get_inventory) {
        Some(second_inventory) => {
            combine_halves(chest_props.r#type, first_inventory, second_inventory)
        }
        None => first_inventory,
    };

    Some(Box::new(ChestScreenFactory {
        inventory,
        loot_pending,
        name,
        double_chest: other_pos.is_some(),
    }))
}

fn normal_use_chest_impl(args: &NormalUseArgs<'_>) -> BlockActionResult {
    if let Some(factory) = get_chest_screen_handler_factory(GetScreenHandlerFactoryArgs {
        server: args.server,
        world: args.world,
        block: args.block,
        position: args.position,
        player: args.player,
    }) {
        // Vanilla `ChestBlock.useWithoutItem` awards the stat only once a menu provider
        // exists, so a chest that cannot open does not raise OPEN_CHEST.
        let stat = if args.block.id == Block::TRAPPED_CHEST.id {
            pumpkin_data::statistic::CustomStatistic::TriggerTrappedChest
        } else {
            pumpkin_data::statistic::CustomStatistic::OpenChest
        };
        args.player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            stat as i32,
            1,
        );
        args.player
            .open_handled_screen(factory.as_ref(), Some(*args.position));
        // Vanilla `ChestBlock.useWithoutItem` angers nearby piglins on a successful open.
        crate::entity::mob::piglin_ai::PiglinAi::on_guarded_block_interacted(
            args.world,
            args.player,
        );
    }

    BlockActionResult::Success
}

fn broken_chest_impl(args: &BrokenArgs<'_>) {
    let chest_props = ChestLikeProperties::from_state_id(args.state.id);
    let connected_towards = match chest_props.r#type {
        ChestType::Single => return,
        ChestType::Left => chest_props.facing.rotate_clockwise(),
        ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
    };

    if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
        args.world,
        args.block,
        args.position,
        chest_props.facing,
        connected_towards,
        chest_props.r#type.opposite(),
    ) {
        neighbor_props.r#type = ChestType::Single;

        args.world.set_block_state(
            &args.position.offset(connected_towards.to_offset()),
            neighbor_props.to_state_id(args.block),
            BlockFlags::NOTIFY_LISTENERS,
        );
    }
}

/// Vanilla `ChestBlock.updateShape`: detach an orphaned half and adopt a neighbouring
/// single chest's orientation. Shared by wooden, trapped and copper chests.
fn chest_state_for_neighbor_update(args: &GetStateForNeighborUpdateArgs<'_>) -> BlockStateId {
    let mut props = ChestLikeProperties::from_state_id(args.state_id);
    if props.waterlogged {
        args.world.schedule_fluid_tick(
            &Fluid::WATER,
            *args.position,
            Fluid::WATER.flow_speed,
            TickPriority::Normal,
        );
    }
    let neighbor = Block::from_state_id(args.neighbor_state_id);
    let can_connect = chest_can_connect_to(args.block, neighbor);
    if can_connect && args.direction.is_horizontal() {
        let neighbor_props = ChestLikeProperties::from_state_id(args.neighbor_state_id);
        if props.r#type == ChestType::Single
            && neighbor_props.r#type != ChestType::Single
            && props.facing == neighbor_props.facing
            && connected_direction(neighbor_props)
                .is_some_and(|dir| dir.to_offset() == args.direction.opposite().to_offset())
        {
            props.r#type = neighbor_props.r#type.opposite();
        }
    } else if connected_direction(props)
        .is_some_and(|dir| dir.to_offset() == args.direction.to_offset())
    {
        props.r#type = ChestType::Single;
    }
    if can_connect
        && props.r#type != ChestType::Single
        && connected_direction(props)
            .is_some_and(|dir| dir.to_offset() == args.direction.to_offset())
    {
        props.to_state_id(neighbor)
    } else {
        props.to_state_id(args.block)
    }
}

#[pumpkin_block_from_tag("c:chests/wooden")]
pub struct ChestBlock;

impl BlockBehaviour for ChestBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        on_place_chest_impl(&args)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        chest_state_for_neighbor_update(&args)
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        args.r#type == LID_ANIMATION_EVENT_TYPE
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        placed_chest_impl(&args, ChestBlockEntity::new);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        normal_use_chest_impl(&args)
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        get_chest_screen_handler_factory(args)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        broken_chest_impl(&args);
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        get_chest_comparator_output(&args)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

/// Copper chests have the same behavior as wooden chests but also oxidize over time.
#[pumpkin_block_from_tag("minecraft:copper_chests")]
pub struct CopperChestBlock;

impl CopperChestBlock {
    pub(crate) fn get_from_copper_block(
        copper: &Block,
        facing: HorizontalFacing,
        world: &World,
        position: &BlockPos,
    ) -> BlockStateId {
        let chest = match copper.id {
            BlockId::EXPOSED_COPPER => &Block::EXPOSED_COPPER_CHEST,
            BlockId::WEATHERED_COPPER => &Block::WEATHERED_COPPER_CHEST,
            BlockId::OXIDIZED_COPPER => &Block::OXIDIZED_COPPER_CHEST,
            BlockId::WAXED_COPPER_BLOCK => &Block::WAXED_COPPER_CHEST,
            BlockId::WAXED_EXPOSED_COPPER => &Block::WAXED_EXPOSED_COPPER_CHEST,
            BlockId::WAXED_WEATHERED_COPPER => &Block::WAXED_WEATHERED_COPPER_CHEST,
            BlockId::WAXED_OXIDIZED_COPPER => &Block::WAXED_OXIDIZED_COPPER_CHEST,
            _ => &Block::COPPER_CHEST,
        };
        let mut props = ChestLikeProperties::default(chest);
        props.facing = facing;
        props.r#type = get_chest_type(world, chest, position, facing);
        Self::get_least_oxidized_chest_of_connected_blocks(
            props.to_state_id(chest),
            world,
            position,
        )
    }

    fn get_least_oxidized_chest_of_connected_blocks(
        state: BlockStateId,
        world: &World,
        position: &BlockPos,
    ) -> BlockStateId {
        let props = ChestLikeProperties::from_state_id(state);
        let direction = match props.r#type {
            ChestType::Single => return state,
            ChestType::Left => props.facing.rotate_clockwise(),
            ChestType::Right => props.facing.rotate_counter_clockwise(),
        };
        let block = Block::from_state_id(state);
        let neighbor = world.get_block(&position.offset(direction.to_offset()));
        if !neighbor
            .id
            .has_tag(pumpkin_data::tag::Block::MINECRAFT_COPPER_CHESTS)
        {
            return state;
        }
        let (age, waxed) = Self::weather_state(block);
        let (neighbor_age, neighbor_waxed) = Self::weather_state(neighbor);
        let least = if age <= neighbor_age { block } else { neighbor };
        let least = if waxed == neighbor_waxed {
            least
        } else {
            Self::unwax_block(least)
        };
        props.to_state_id(least)
    }

    fn weather_state(block: &Block) -> (super::weathering_copper::WeatherState, bool) {
        use super::weathering_copper::WeatherState;
        let unwaxed = Self::unwax_block(block);
        let age = super::weathering_copper::get_weather_state(unwaxed)
            .unwrap_or(WeatherState::Unaffected);
        (age, block != unwaxed)
    }

    const fn unwax_block(block: &Block) -> &Block {
        match block.id {
            BlockId::WAXED_COPPER_CHEST => &Block::COPPER_CHEST,
            BlockId::WAXED_EXPOSED_COPPER_CHEST => &Block::EXPOSED_COPPER_CHEST,
            BlockId::WAXED_WEATHERED_COPPER_CHEST => &Block::WEATHERED_COPPER_CHEST,
            BlockId::WAXED_OXIDIZED_COPPER_CHEST => &Block::OXIDIZED_COPPER_CHEST,
            _ => block,
        }
    }

    pub(crate) fn get_hinge_sound(block: &Block, open: bool) -> Sound {
        let (open_sound, close_sound) = if block == &Block::WEATHERED_COPPER_CHEST
            || block == &Block::WAXED_WEATHERED_COPPER_CHEST
        {
            (
                Sound::BlockCopperChestWeatheredOpen,
                Sound::BlockCopperChestWeatheredClose,
            )
        } else if block == &Block::OXIDIZED_COPPER_CHEST
            || block == &Block::WAXED_OXIDIZED_COPPER_CHEST
        {
            (
                Sound::BlockCopperChestOxidizedOpen,
                Sound::BlockCopperChestOxidizedClose,
            )
        } else {
            (Sound::BlockCopperChestOpen, Sound::BlockCopperChestClose)
        };

        if open { open_sound } else { close_sound }
    }
}

impl
    crate::block::blocks::weathering_copper::ChangeOverTimeBlock<
        crate::block::blocks::weathering_copper::WeatherState,
    > for CopperChestBlock
{
    fn get_age(
        &self,
        block: &Block,
    ) -> Option<crate::block::blocks::weathering_copper::WeatherState> {
        crate::block::blocks::weathering_copper::get_weather_state(block)
    }

    fn get_chance_modifier(
        &self,
        age: crate::block::blocks::weathering_copper::WeatherState,
    ) -> f32 {
        crate::block::blocks::weathering_copper::get_chance_modifier(age)
    }

    fn get_next(&self, block: &Block) -> Option<&'static Block> {
        crate::block::blocks::weathering_copper::get_next(block)
    }

    fn get_previous(&self, block: &Block) -> Option<&'static Block> {
        crate::block::blocks::weathering_copper::get_previous(block)
    }

    fn get_first(&self, block: &Block) -> Option<&'static Block> {
        crate::block::blocks::weathering_copper::get_first(block)
    }
}

impl crate::block::blocks::weathering_copper::WeatheringCopper for CopperChestBlock {}

impl BlockBehaviour for CopperChestBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        Self::get_least_oxidized_chest_of_connected_blocks(
            on_place_chest_impl(&args),
            args.world,
            args.position,
        )
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        chest_state_for_neighbor_update(&args)
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        args.r#type == LID_ANIMATION_EVENT_TYPE
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        placed_chest_impl(&args, ChestBlockEntity::new);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        normal_use_chest_impl(&args)
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        get_chest_screen_handler_factory(args)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        broken_chest_impl(&args);
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let current_state_id = args.world.get_block_state_id(args.position);
        let chest_props = ChestLikeProperties::from_state_id(current_state_id);

        // Only oxidize LEFT or SINGLE chests (not RIGHT) to prevent double oxidation
        if chest_props.r#type == ChestType::Right {
            return;
        }

        // Only oxidize if no players are viewing the chest
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(chest_entity) = block_entity.as_any().downcast_ref::<ChestBlockEntity>()
            && chest_entity.get_viewer_count() > 0
        {
            return;
        }

        crate::block::blocks::weathering_copper::change_over_time(
            args.world,
            args.position,
            args.block,
        );
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        get_chest_comparator_output(&args)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

/// Trapped chests have the same behavior as wooden chests but also emit redstone power based on viewer count.
#[pumpkin_block("minecraft:trapped_chest")]
pub struct TrappedChestBlock;

impl BlockBehaviour for TrappedChestBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        on_place_chest_impl(&args)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        chest_state_for_neighbor_update(&args)
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        args.r#type == LID_ANIMATION_EVENT_TYPE
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        use crate::block::entities::trapped_chest::TrappedChestBlockEntity;
        placed_chest_impl(&args, TrappedChestBlockEntity::new);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        normal_use_chest_impl(&args)
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        get_chest_screen_handler_factory(args)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        broken_chest_impl(&args);
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        use crate::block::entities::trapped_chest::TrappedChestBlockEntity;

        // Get viewer count from this chest
        let viewer_count = if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(trapped_chest) = block_entity
                .as_any()
                .downcast_ref::<TrappedChestBlockEntity>()
        {
            trapped_chest.get_viewer_count()
        } else {
            0
        };

        viewer_count.min(15) as u8
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        // Strong power emitted to the block beneath the trapped chest
        // The block below queries with direction Up (from below looking up at the chest)
        if args.direction == BlockDirection::Up {
            self.get_weak_redstone_power(args)
        } else {
            0
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        get_chest_comparator_output(&args)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn compute_chest_props(
    world: &World,
    player: &Player,
    block: &Block,
    block_pos: &BlockPos,
    face: BlockDirection,
) -> (ChestType, HorizontalFacing) {
    let player_facing = player.get_entity().get_horizontal_facing();
    let chest_facing = player_facing.opposite();

    // Vanilla `ChestBlock.getStateForPlacement` skips the neighbour join entirely for a
    // crouching player (`isShiftKeyDown`), even when clicking a single chest's side.
    if player.get_entity().pose.load() == EntityPose::Crouching {
        return (ChestType::Single, chest_facing);
    }

    // A chest placed against a horizontal face of a single chest always joins it and
    // adopts that chest's facing, whatever the player yaw is (so a diagonal placement
    // still extends instead of rotating).
    if let Some(clicked_face_horiz) = face.to_horizontal_facing() {
        let (clicked_block, clicked_block_state) =
            world.get_block_and_state_id(&block_pos.offset(clicked_face_horiz.to_offset()));

        if chest_can_connect_to(block, clicked_block) {
            let clicked_props = ChestLikeProperties::from_state_id(clicked_block_state);

            if clicked_props.r#type == ChestType::Single {
                if clicked_props.facing.rotate_clockwise() == clicked_face_horiz {
                    return (ChestType::Left, clicked_props.facing);
                } else if clicked_props.facing.rotate_counter_clockwise() == clicked_face_horiz {
                    return (ChestType::Right, clicked_props.facing);
                }
            }
        }
    }

    (
        get_chest_type(world, block, block_pos, chest_facing),
        chest_facing,
    )
}

fn get_chest_type(
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
    chest_facing: HorizontalFacing,
) -> ChestType {
    if get_chest_properties_if_can_connect(
        world,
        block,
        block_pos,
        chest_facing,
        chest_facing.rotate_clockwise(),
        ChestType::Single,
    )
    .is_some()
    {
        ChestType::Left
    } else if get_chest_properties_if_can_connect(
        world,
        block,
        block_pos,
        chest_facing,
        chest_facing.rotate_counter_clockwise(),
        ChestType::Single,
    )
    .is_some()
    {
        ChestType::Right
    } else {
        ChestType::Single
    }
}

fn get_chest_properties_if_can_connect(
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
    facing: HorizontalFacing,
    direction: HorizontalFacing,
    wanted_type: ChestType,
) -> Option<ChestLikeProperties> {
    let (neighbor_block, neighbor_block_state) =
        world.get_block_and_state_id(&block_pos.offset(direction.to_offset()));

    if !chest_can_connect_to(block, neighbor_block) {
        return None;
    }

    let neighbor_props = ChestLikeProperties::from_state_id(neighbor_block_state);
    if neighbor_props.facing == facing && neighbor_props.r#type == wanted_type {
        return Some(neighbor_props);
    }

    None
}

fn chest_can_connect_to(block: &Block, neighbor: &Block) -> bool {
    block == neighbor
        || (block
            .id
            .has_tag(pumpkin_data::tag::Block::MINECRAFT_COPPER_CHESTS)
            && neighbor
                .id
                .has_tag(pumpkin_data::tag::Block::MINECRAFT_COPPER_CHESTS))
}

fn is_chest_blocked(world: &World, block_pos: &BlockPos) -> bool {
    has_block_on_top(world, block_pos) || is_cat_sitting_on_chest(world, block_pos)
}
fn has_block_on_top(world: &World, block_pos: &BlockPos) -> bool {
    let above_pos = block_pos.up();
    let above_state = world.get_block_state(&above_pos);
    // Vanilla `ChestBlock.isBlockedChestByBlock` tests `isRedstoneConductor`; slabs,
    // stairs, walls, leaves and stained glass do not block a chest.
    above_state.is_redstone_conductor()
}

fn is_cat_sitting_on_chest(world: &World, block_pos: &BlockPos) -> bool {
    let search_box = BoundingBox::from_block(&block_pos.up());
    world.get_entities_at_box(&search_box).iter().any(|entity| {
        entity
            .cast_any()
            .downcast_ref::<CatEntity>()
            .is_some_and(TamableAnimal::is_in_sitting_pose)
    })
}

trait ChestTypeExt {
    fn opposite(&self) -> ChestType;
}

impl ChestTypeExt for ChestType {
    fn opposite(&self) -> Self {
        match self {
            Self::Single => Self::Single,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChestBlockEntity, ChestContainer, DoubleInventory};
    use pumpkin_inventory::Inventory;
    use pumpkin_util::math::position::BlockPos;
    use std::sync::Arc;

    #[test]
    fn chest_menu_title_matches_vanilla() {
        use super::chest_display_name;
        use pumpkin_util::text::{TextComponent, TextContent};

        fn key(component: &TextComponent) -> &str {
            match &*component.0.content {
                TextContent::Translate { translate, .. } => translate.as_ref(),
                _ => "",
            }
        }

        assert_eq!(key(&chest_display_name(None, false)), "container.chest");
        assert_eq!(
            key(&chest_display_name(None, true)),
            "container.chestDouble"
        );

        let custom = TextComponent::from_content(TextContent::Text {
            text: "Named".into(),
        });
        assert_eq!(chest_display_name(Some(&custom), false), custom);
        assert_eq!(chest_display_name(Some(&custom), true), custom);
    }

    #[test]
    fn failed_double_chest_reservation_releases_the_first_half() {
        let first = Arc::new(ChestBlockEntity::new(BlockPos::new(0, 0, 0)));
        let second = Arc::new(ChestBlockEntity::new(BlockPos::new(1, 0, 0)));
        let container = ChestContainer {
            inventory: DoubleInventory::new(first.clone(), second.clone()),
            positions: vec![first.position, second.position],
            halves: vec![first.clone(), second.clone()],
        };
        second.on_open();
        assert!(!container.try_start_open());
        assert_eq!(first.get_viewer_count(), 0);
        assert_eq!(second.get_viewer_count(), 1);
        second.on_close();
        assert!(container.try_start_open());
        assert!(!container.try_start_open());
        container.stop_open();
        assert_eq!(first.get_viewer_count(), 0);
        assert_eq!(second.get_viewer_count(), 0);
        assert!(container.try_start_open());
        container.stop_open();
    }
}
