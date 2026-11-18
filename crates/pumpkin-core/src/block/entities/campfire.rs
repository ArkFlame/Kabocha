use super::BlockEntity;
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{CampfireLikeProperties, HorizontalFacing};
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::recipes::{CookingRecipeKind, get_cooking_recipe_with_ingredient};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::io::Dirtiable;
use std::sync::{Arc, Mutex};

pub struct CampfireBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::SLOT_COUNT],
    pub cooking_times: [Mutex<i32>; Self::SLOT_COUNT],
    pub cooking_total_times: [Mutex<i32>; Self::SLOT_COUNT],
    /// Paper `CampfireStartEvent` cancellation is sticky for the sitting: a
    /// cancelled insertion keeps the placed item from ever cooking.
    pub cooking_cancelled: [Mutex<bool>; Self::SLOT_COUNT],
}

impl BlockEntity for CampfireBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut entity = Self::new(position);

        if let Some(list) = nbt.get_list("Items") {
            for tag in list {
                if let Some(compound) = tag.extract_compound() {
                    let slot = compound.get_byte("Slot").unwrap_or(0) as usize;
                    if slot < Self::SLOT_COUNT
                        && let Some(stack) = ItemStack::read_item_stack(compound)
                    {
                        entity.items[slot] = Arc::new(Mutex::new(stack));
                    }
                }
            }
        }

        if let Some(arr) = nbt.get_int_array("CookingTimes") {
            for (slot, &value) in arr.iter().enumerate().take(Self::SLOT_COUNT) {
                entity.cooking_times[slot] = Mutex::new(value);
            }
        }
        if let Some(arr) = nbt.get_int_array("CookingTotalTimes") {
            for (slot, &value) in arr.iter().enumerate().take(Self::SLOT_COUNT) {
                entity.cooking_total_times[slot] = Mutex::new(value);
            }
        }
        // Paper persists `stopCooking` as a byte buffer; a cancelled slot must stay cancelled across a reload.
        if let Some(arr) = nbt.get_byte_array("Paper.StopCooking") {
            for (slot, &value) in arr.iter().enumerate().take(Self::SLOT_COUNT) {
                if value != 0 {
                    *entity.cooking_cancelled[slot]
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
                }
            }
        }

        entity
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_cooking_nbt(nbt);
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let items: [ItemStack; Self::SLOT_COUNT] = std::array::from_fn(|i| {
            self.items[i]
                .lock()
                .map_or_else(|_| ItemStack::EMPTY.clone(), |g| g.clone())
        });
        Some(build_campfire_actor(self.position, &items))
    }

    fn tick(&self, world: &Arc<World>) {
        let block = world.get_block(&self.position);
        if block != &Block::CAMPFIRE && block != &Block::SOUL_CAMPFIRE {
            return;
        }

        let state_id = world.get_block_state(&self.position).id;
        let properties = CampfireLikeProperties::from_state_id(state_id);

        if !properties.lit {
            // Vanilla `CampfireBlock.getTicker` runs `cooldownTick` on every unlit campfire.
            if self.cool_down() {
                self.mark_chunk_dirty(world);
            }
            return;
        }

        // Vanilla's `CampfireBlockEntity.particleTick` runs client-side; emit the
        // equivalent smoke packets from the server so players see the same effect.
        self.particle_tick(world, properties);

        let mut changed = false;
        let mut completed = false;

        for slot in 0..Self::SLOT_COUNT {
            let stack = self.items[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if stack.is_empty() {
                continue;
            }

            if *self.cooking_cancelled[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
            {
                continue;
            }

            // Vanilla `cookTick` increments every non-empty stack and resolves the
            // recipe only at completion; the total is set once at insertion.
            let total = *self.cooking_total_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            let finished = {
                let mut progress = self.cooking_times[slot]
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *progress += 1;
                changed = true;
                *progress >= total
            };

            if !finished {
                continue;
            }

            let recipe =
                get_cooking_recipe_with_ingredient(stack.item, CookingRecipeKind::CampfireCooking);
            let mut result = match &recipe {
                Some(recipe) => {
                    let Some(result_item) = Item::from_registry_key(recipe.result.id) else {
                        continue;
                    };
                    ItemStack::new(recipe.result.count, result_item)
                }
                // Vanilla `cookItems` falls back to the raw stack when no recipe matches
                None => stack.clone(),
            };
            if let Some(server) = world.server.upgrade() {
                let mut event = crate::plugin::api::events::block::block_cook::BlockCookEvent::new(
                    self.position,
                    world.clone(),
                    stack.clone(),
                    result.clone(),
                );
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    // Paper restarts the slot instead of leaving progress at the
                    // completed value, which would refire the event every tick.
                    *self.cooking_times[slot]
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
                    continue;
                }
                result = event.result;
            }
            Self::spawn_cooked_item(world, self.position, result);

            *self.items[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = ItemStack::EMPTY.clone();
            *self.cooking_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
            *self.cooking_total_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
            *self.cooking_cancelled[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = false;
            // Vanilla emits the block change event once per finished slot.
            world.emit_game_event(
                GameEvent::BlockChange.name(),
                self.position.to_centered_f64(),
            );
            completed = true;
        }

        if completed {
            if let Some(block_entity) = world.get_block_entity(&self.position) {
                world.update_block_entity(&block_entity);
            }
        } else if changed {
            self.mark_chunk_dirty(world);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_cooking_nbt(&mut nbt);
        Some(nbt)
    }

    fn apply_item_components(&self, stack: &ItemStack) {
        if let Some(container) =
            stack.get_data_component::<pumpkin_data::data_component_impl::ContainerImpl>()
        {
            for (slot, item) in &container.items {
                let slot = *slot as usize;
                if let Some(cell) = self.items.get(slot) {
                    *cell
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = item.clone();
                    if let Some(cancelled) = self.cooking_cancelled.get(slot) {
                        *cancelled
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = false;
                    }
                    // A fresh stack must not inherit the previous sitting's progress.
                    if let Some(progress) = self.cooking_times.get(slot) {
                        *progress
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
                    }
                    if let Some(total) = self.cooking_total_times.get(slot) {
                        *total
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
                    }
                }
            }
        }
    }

    fn collect_pick_item_components(&self, stack: &mut ItemStack) {
        let items: [ItemStack; Self::SLOT_COUNT] = std::array::from_fn(|i| {
            self.items[i]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        });
        super::collect_container_items_from_slice(&items, stack);
    }

    fn on_block_replaced(self: Arc<Self>, world: &Arc<World>, _position: &BlockPos) {
        for item in &self.items {
            let stack = std::mem::replace(
                &mut *item
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
                ItemStack::EMPTY.clone(),
            );
            if !stack.is_empty() {
                world.scatter_stack(
                    f64::from(self.position.0.x),
                    f64::from(self.position.0.y),
                    f64::from(self.position.0.z),
                    stack,
                );
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CampfireBlockEntity {
    pub const ID: &'static str = "minecraft:campfire";
    pub const SLOT_COUNT: usize = 4;

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: std::array::from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            cooking_times: std::array::from_fn(|_| Mutex::new(0)),
            cooking_total_times: std::array::from_fn(|_| Mutex::new(0)),
            cooking_cancelled: std::array::from_fn(|_| Mutex::new(false)),
        }
    }

    fn write_cooking_nbt(&self, nbt: &mut NbtCompound) {
        let mut items = Vec::new();
        for (slot, item) in self.items.iter().enumerate() {
            let stack = item
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !stack.is_empty() {
                let mut item_nbt = NbtCompound::new();
                item_nbt.put_byte("Slot", slot as i8);
                stack.write_item_stack(&mut item_nbt);
                items.push(NbtTag::Compound(item_nbt));
            }
        }
        nbt.put_list("Items", items);

        let cooking_times = self
            .cooking_times
            .iter()
            .map(|time| {
                *time
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
            })
            .collect();
        nbt.put("CookingTimes", NbtTag::IntArray(cooking_times));

        let cooking_total_times = self
            .cooking_total_times
            .iter()
            .map(|time| {
                *time
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
            })
            .collect();
        nbt.put("CookingTotalTimes", NbtTag::IntArray(cooking_total_times));

        let stop_cooking: Vec<i8> = self
            .cooking_cancelled
            .iter()
            .map(|flag| {
                i8::from(
                    *flag
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner),
                )
            })
            .collect();
        nbt.put("Paper.StopCooking", NbtTag::ByteArray(stop_cooking.into()));
    }

    /// Server-side counterpart of vanilla `CampfireBlockEntity.particleTick`
    /// (`CampfireBlockEntity.java:165-188`): occasional cosy/signal smoke puffs
    /// and smoke over each cooking slot.
    fn particle_tick(&self, world: &Arc<World>, props: CampfireLikeProperties) {
        let pos = self.position;
        let smoke = if props.signal_fire {
            Particle::CampfireSignalSmoke
        } else {
            Particle::CampfireCosySmoke
        };

        // Vanilla: `random.nextFloat() < 0.11F`, then `nextInt(2) + 2` puffs.
        if rand::random::<f32>() < 0.11 {
            let puffs: i32 = rand::random_range(2..=3);
            for _ in 0..puffs {
                let sign = |v: f64| if rand::random::<bool>() { v } else { -v };
                let x = f64::from(pos.0.x) + 0.5 + sign(rand::random::<f64>() / 3.0);
                let y = f64::from(pos.0.y) + rand::random::<f64>() + rand::random::<f64>();
                let z = f64::from(pos.0.z) + 0.5 + sign(rand::random::<f64>() / 3.0);
                world.spawn_particle(
                    Vector3::new(x, y, z),
                    Vector3::new(0.0, 0.07, 0.0),
                    0.0,
                    1,
                    smoke,
                );
            }
        }

        let rotation = Self::facing_2d_data_value(props.facing);
        #[expect(
            clippy::needless_range_loop,
            reason = "slot also derives the item position"
        )]
        for slot in 0..Self::SLOT_COUNT {
            let filled = !self.items[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty();
            if !filled || rand::random::<f32>() >= 0.2 {
                continue;
            }
            let (dx, dz) = Self::item_particle_offset(slot, rotation);
            world.spawn_particle(
                Vector3::new(
                    f64::from(pos.0.x) + 0.5 + f64::from(dx),
                    f64::from(pos.0.y) + 0.5,
                    f64::from(pos.0.z) + 0.5 + f64::from(dz),
                ),
                Vector3::new(0.0, 5.0e-4, 0.0),
                0.0,
                4,
                Particle::Smoke,
            );
        }
    }

    /// Vanilla `Direction.get2DDataValue` for the four horizontal facings.
    const fn facing_2d_data_value(facing: HorizontalFacing) -> usize {
        match facing {
            HorizontalFacing::South => 0,
            HorizontalFacing::West => 1,
            HorizontalFacing::North => 2,
            HorizontalFacing::East => 3,
        }
    }

    /// Vanilla item-smoke offset: 0.3125 along the slot's `Direction` step and
    /// its clockwise step (`CampfireBlockEntity.java:177-181`).
    const fn item_particle_offset(slot: usize, rotation: usize) -> (f32, f32) {
        const STEP: [(f32, f32); 4] = [(0.0, 1.0), (-1.0, 0.0), (0.0, -1.0), (1.0, 0.0)];
        let dir = (slot + rotation) % 4;
        let (sx, sz) = STEP[dir];
        let (cx, cz) = STEP[(dir + 1) % 4];
        (-sx * 0.3125 + cx * 0.3125, -sz * 0.3125 + cz * 0.3125)
    }

    fn cool_down(&self) -> bool {
        let mut changed = false;
        for (slot, progress) in self.cooking_times.iter().enumerate() {
            let total = *self.cooking_total_times[slot]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut progress = progress
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // `CampfireBlockEntity.cooldownTick` clamps to [0, cookingTime].
            let cooled = (*progress - 2).clamp(0, total.max(0));
            if cooled != *progress {
                *progress = cooled;
                changed = true;
            }
        }
        changed
    }

    fn mark_chunk_dirty(&self, world: &Arc<World>) {
        let chunk_position = self.position.chunk_position();
        let _ = world.level.read_chunk_sync(&chunk_position, |chunk| {
            chunk.mark_dirty(true);
        });
    }

    /// Vanilla `CampfireBlockEntity.cookItems` drops the result at the campfire
    /// block centre in all three axes.
    fn cooked_item_spawn_position(position: BlockPos) -> Vector3<f64> {
        Vector3::new(
            f64::from(position.0.x) + 0.5,
            f64::from(position.0.y) + 0.5,
            f64::from(position.0.z) + 0.5,
        )
    }

    /// Vanilla `cookItems` splits the result into random 10-30 item chunks.
    fn cooked_chunk_sizes(count: u8, mut next_chunk: impl FnMut() -> u8) -> Vec<u8> {
        let mut sizes = Vec::new();
        let mut remaining = count;
        while remaining > 0 {
            let size = next_chunk().clamp(1, 30).min(remaining);
            sizes.push(size);
            remaining -= size;
        }
        sizes
    }

    fn spawn_cooked_item(world: &Arc<World>, position: BlockPos, mut stack: ItemStack) {
        let spawn = Self::cooked_item_spawn_position(position);
        // Vanilla spreads the drop with `RandomSource.triangle` around that centre.
        let deviation = 0.05f64 * 2.297;
        let sizes = Self::cooked_chunk_sizes(stack.item_count, || rand::random_range(10u8..=30u8));
        for size in sizes {
            let chunk = stack.split(size);
            let entity = Entity::new(world.clone(), spawn, &EntityType::ITEM);
            let velocity = Vector3::new(
                deviation * (rand::random::<f64>() - rand::random::<f64>()),
                0.2 + deviation * (rand::random::<f64>() - rand::random::<f64>()),
                deviation * (rand::random::<f64>() - rand::random::<f64>()),
            );
            world.spawn_entity(Arc::new(ItemEntity::new_with_velocity(
                entity,
                chunk,
                velocity,
                ItemEntity::DEFAULT_PICKUP_DELAY,
            )));
        }
    }
}

/// Builds the Bedrock block-actor NBT for campfires.
#[must_use]
pub fn build_campfire_actor(position: BlockPos, items: &[ItemStack; 4]) -> NbtCompound {
    let mut builder = super::bedrock::BedrockBlockActorBuilder::new("Campfire", position);
    for (i, item) in items.iter().enumerate() {
        if !item.is_empty() {
            let mut item_comp = NbtCompound::new();
            item_comp.put_string(
                "Name",
                super::bedrock::bedrock_item_name(item.item.registry_key),
            );
            item_comp.put_byte("Count", item.item_count as i8);
            item_comp.put_short("Damage", 0);
            item_comp.put_byte("Slot", i as i8);
            let key = match i {
                0 => "Item1",
                1 => "Item2",
                2 => "Item3",
                _ => "Item4",
            };
            builder = builder.put_compound(key, item_comp);
        }
    }
    builder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campfire_actor_builder() {
        let pos = BlockPos::new(5, 65, 5);
        let beef = ItemStack::new(1, &Item::BEEF);
        let pork = ItemStack::new(1, &Item::PORKCHOP);
        let items = [
            beef,
            ItemStack::EMPTY.clone(),
            pork,
            ItemStack::EMPTY.clone(),
        ];
        let nbt = build_campfire_actor(pos, &items);

        assert_eq!(nbt.get_string("id"), Some("Campfire"));
        let item1 = nbt.get_compound("Item1").expect("Item1 missing");
        assert_eq!(item1.get_string("Name"), Some("minecraft:beef"));
        assert_eq!(item1.get_byte("Count"), Some(1));
        assert_eq!(item1.get_byte("Slot"), Some(0));

        assert!(nbt.get_compound("Item2").is_none());

        let item3 = nbt.get_compound("Item3").expect("Item3 missing");
        assert_eq!(item3.get_string("Name"), Some("minecraft:porkchop"));
        assert_eq!(item3.get_byte("Count"), Some(1));
        assert_eq!(item3.get_byte("Slot"), Some(2));
    }

    #[test]
    fn cancelled_cooking_survives_nbt_roundtrip() {
        let pos = BlockPos::new(1, 64, 1);
        let entity = CampfireBlockEntity::new(pos);
        *entity.cooking_cancelled[2]
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;

        let mut nbt = NbtCompound::new();
        entity.write_cooking_nbt(&mut nbt);
        let loaded = <CampfireBlockEntity as BlockEntity>::from_nbt(&nbt, pos);

        assert!(
            *loaded.cooking_cancelled[2]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        );
        assert!(
            !*loaded.cooking_cancelled[0]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        );
    }

    #[test]
    fn cooked_result_splits_into_random_chunks() {
        assert_eq!(CampfireBlockEntity::cooked_chunk_sizes(1, || 10), vec![1],);
        assert_eq!(
            CampfireBlockEntity::cooked_chunk_sizes(45, || 30),
            vec![30, 15],
        );
        // Chunks below the vanilla 10-30 range still make progress.
        assert_eq!(
            CampfireBlockEntity::cooked_chunk_sizes(7, || 0),
            vec![1, 1, 1, 1, 1, 1, 1],
        );
        assert!(CampfireBlockEntity::cooked_chunk_sizes(0, || 10).is_empty());
    }

    #[test]
    fn cooked_item_spawns_at_block_centre() {
        let spawn = CampfireBlockEntity::cooked_item_spawn_position(BlockPos::new(3, 64, -7));
        assert_eq!(spawn.x, 3.5);
        assert_eq!(spawn.y, 64.5);
        assert_eq!(spawn.z, -6.5);
    }

    #[test]
    fn facing_2d_data_value_matches_vanilla_order() {
        assert_eq!(
            CampfireBlockEntity::facing_2d_data_value(HorizontalFacing::South),
            0
        );
        assert_eq!(
            CampfireBlockEntity::facing_2d_data_value(HorizontalFacing::West),
            1
        );
        assert_eq!(
            CampfireBlockEntity::facing_2d_data_value(HorizontalFacing::North),
            2
        );
        assert_eq!(
            CampfireBlockEntity::facing_2d_data_value(HorizontalFacing::East),
            3
        );
    }

    #[test]
    fn item_particle_offset_follows_slot_and_facing() {
        // Facing SOUTH, slot 0 sits at the south-west corner of the campfire.
        assert_eq!(
            CampfireBlockEntity::item_particle_offset(0, 0),
            (-0.3125, -0.3125)
        );
        // A quarter turn moves the smoke to the next corner.
        assert_ne!(
            CampfireBlockEntity::item_particle_offset(0, 0),
            CampfireBlockEntity::item_particle_offset(0, 1)
        );
        // The four slots wrap around every four positions.
        assert_eq!(
            CampfireBlockEntity::item_particle_offset(0, 0),
            CampfireBlockEntity::item_particle_offset(4, 0)
        );
    }
}
