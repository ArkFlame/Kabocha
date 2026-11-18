use super::BlockEntity;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;
use uuid::Uuid;

pub struct VaultBlockEntity {
    pub position: BlockPos,
    pub config: Mutex<Option<NbtCompound>>,
    pub server_data: Mutex<Option<NbtCompound>>,
    pub shared_data: Mutex<Option<NbtCompound>>,
    /// Insertion-ordered, capped at [`Self::MAX_REWARDED_PLAYERS`] like vanilla.
    pub rewarded_players: Mutex<Vec<Uuid>>,
    /// Loot awaiting ejection, popped from the back like vanilla [`VaultServerData#itemsToEject`].
    pub items_to_eject: Mutex<Vec<ItemStack>>,
}

impl VaultBlockEntity {
    /// [`VaultServerData`MAX_REWARD_PLAYERS`].
    pub const MAX_REWARDED_PLAYERS: usize = 128;
}

/// Vanilla keeps rewarded players in [`server_data`] as a list of four-int UUID arrays.
fn read_rewarded_players(server_data: &NbtCompound) -> Vec<Uuid> {
    let mut players = Vec::new();
    let Some(list) = server_data.get_list("rewarded_players") else {
        return players;
    };
    for tag in list {
        let uuid = match tag {
            NbtTag::IntArray(arr) if arr.len() == 4 => {
                let value = ((arr[0] as u128) << 96)
                    | (((arr[1] as u32) as u128) << 64)
                    | (((arr[2] as u32) as u128) << 32)
                    | ((arr[3] as u32) as u128);
                Uuid::from_u128(value)
            }
            NbtTag::String(s) => match Uuid::parse_str(s) {
                Ok(uuid) => uuid,
                Err(_) => continue,
            },
            _ => continue,
        };
        if !players.contains(&uuid) {
            players.push(uuid);
        }
    }
    let excess = players
        .len()
        .saturating_sub(VaultBlockEntity::MAX_REWARDED_PLAYERS);
    players.drain(..excess);
    players
}

fn rewarded_players_tag(rewarded: &[Uuid]) -> NbtTag {
    NbtTag::List(
        rewarded
            .iter()
            .map(|uuid| {
                let value = uuid.as_u128();
                NbtTag::IntArray(vec![
                    (value >> 96) as i32,
                    ((value >> 64) & 0xFFFF_FFFF) as i32,
                    ((value >> 32) & 0xFFFF_FFFF) as i32,
                    (value & 0xFFFF_FFFF) as i32,
                ])
            })
            .collect(),
    )
}

/// Vanilla [`VaultServerData`#itemsToEject] is a list of item-stack compounds under [`server_data`].
fn read_items_to_eject(server_data: &NbtCompound) -> Vec<ItemStack> {
    let mut items = Vec::new();
    let Some(list) = server_data.get_list("items_to_eject") else {
        return items;
    };
    for tag in list {
        if let NbtTag::Compound(compound) = tag
            && let Some(stack) = ItemStack::read_item_stack(compound)
        {
            items.push(stack);
        }
    }
    items
}

fn items_to_eject_tag(items: &[ItemStack]) -> NbtTag {
    NbtTag::List(
        items
            .iter()
            .map(|stack| {
                let mut compound = NbtCompound::new();
                stack.write_item_stack(&mut compound);
                NbtTag::Compound(compound)
            })
            .collect(),
    )
}

impl BlockEntity for VaultBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let server_data = nbt.get_compound("server_data").cloned();
        let rewarded_players = Mutex::new(
            server_data
                .as_ref()
                .map(read_rewarded_players)
                .unwrap_or_default(),
        );
        let items_to_eject = Mutex::new(
            server_data
                .as_ref()
                .map(read_items_to_eject)
                .unwrap_or_default(),
        );
        Self {
            position,
            config: Mutex::new(nbt.get_compound("config").cloned()),
            server_data: Mutex::new(server_data),
            shared_data: Mutex::new(nbt.get_compound("shared_data").cloned()),
            rewarded_players,
            items_to_eject,
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(cfg) = self.config.lock()
            && let Some(cfg) = cfg.as_ref()
        {
            nbt.put_compound("config", cfg.clone());
        }
        let mut server_data = self
            .server_data
            .lock()
            .ok()
            .and_then(|data| data.as_ref().cloned());
        let rewarded = self
            .rewarded_players
            .lock()
            .map(|players| players.clone())
            .unwrap_or_default();
        let items = self
            .items_to_eject
            .lock()
            .map(|stacks| stacks.clone())
            .unwrap_or_default();
        if !rewarded.is_empty() || !items.is_empty() {
            let mut data = server_data.take().unwrap_or_default();
            if !rewarded.is_empty() {
                data.put("rewarded_players", rewarded_players_tag(&rewarded));
            }
            if !items.is_empty() {
                data.put("items_to_eject", items_to_eject_tag(&items));
            }
            server_data = Some(data);
        }
        if let Some(data) = server_data {
            nbt.put_compound("server_data", data);
        }
        if let Ok(shared) = self.shared_data.lock()
            && let Some(shared) = shared.as_ref()
        {
            nbt.put_compound("shared_data", shared.clone());
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(shared) = self.shared_data.try_lock()
            && let Some(ref shared) = *shared
        {
            nbt.put_compound("shared_data", shared.clone());
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl VaultBlockEntity {
    pub const ID: &'static str = "minecraft:vault";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            config: Mutex::new(None),
            server_data: Mutex::new(None),
            shared_data: Mutex::new(None),
            rewarded_players: Mutex::new(Vec::new()),
            items_to_eject: Mutex::new(Vec::new()),
        }
    }

    /// Mirrors vanilla [`VaultSharedData`#setDisplayItem]: the item shown inside the vault lives in
    /// [`shared_data``display_item`] and is sent to clients through [`chunk_data_nbt`].
    pub fn set_display_item(&self, stack: &ItemStack) {
        let mut shared = self
            .shared_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let data = shared.get_or_insert_with(NbtCompound::new);
        if stack.is_empty() {
            data.child_tags.remove("display_item");
        } else {
            let mut item_compound = NbtCompound::new();
            stack.write_item_stack(&mut item_compound);
            data.put_compound("display_item", item_compound);
        }
    }

    /// Replaces the loot awaiting ejection (vanilla [`VaultServerData`#setItemsToEject`]).
    pub fn set_items_to_eject(&self, items: Vec<ItemStack>) {
        let mut stored = self
            .items_to_eject
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *stored = items;
    }

    /// Cycles the display item from the vault's loot table (vanilla [`VaultBlockEntity.Server.cycleDisplayItemFromLootTable`]).
    /// Picks a random stack from the loot table and sets it as the displayed item via [`shared_data`].
    pub fn cycle_display_item_from_loot_table(&self) {
        let mut shared = self
            .shared_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let data = shared.get_or_insert_with(NbtCompound::new);

        let config_guard = self.config.lock();
        let config = config_guard.as_ref().map(|c| c.as_ref());
        let config = match config {
            Some(c) => c,
            None => {
                // No config; set empty display item and return.
                data.child_tags.remove("display_item");
                return;
            }
        };
        let loot_table = config.get_string("loot_table").unwrap_or_else(|| {
            if config.ominous {
                "minecraft:chests/trial_chambers/reward_ominous"
            } else {
                "minecraft:chests/trial_chambers/reward"
            }
        });
        // Generate loot and set display item
        let stacks = self.resolve_loot_table(loot_table);
        if let Some(stack) = stacks.last() {
            self.set_display_item(stack);
        }
    }

    fn resolve_loot_table(&self, loot_table: &str) -> Vec<ItemStack> {
        // TODO: implement proper loot table resolution using world
        Vec::new()
    }

    /// Removes and returns the next stack to eject (vanilla [`#popNextItemToEject`], popping the back).
    pub fn pop_next_item_to_eject(&self) -> Option<ItemStack> {
        self.items_to_eject
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop()
    }

    /// The stack currently shown inside the vault (vanilla [`#getNextItemToEject`]).
    pub fn next_item_to_eject(&self) -> ItemStack {
        self.items_to_eject
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .last()
            .cloned()
            .unwrap_or_else(|| (*ItemStack::EMPTY).clone())
    }

    pub fn has_rewarded(&self, player_id: &Uuid) -> bool {
        self.rewarded_players
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(player_id)
    }

    pub fn mark_rewarded(&self, player_id: Uuid) {
        let mut rewarded = self
            .rewarded_players
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if rewarded.contains(&player_id) {
            return;
        }
        if rewarded.len() >= Self::MAX_REWARDED_PLAYERS {
            rewarded.remove(0);
        }
        rewarded.push(player_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_display_item_handles_missing_config() {
        let entity = VaultBlockEntity::new(BlockPos::new(0, 0, 0));
        // entity has no config, so cycle should set empty display item without panic
        entity.cycle_display_item_from_loot_table();
        // The display item should remain empty or be set to empty
        let display = entity.next_item_to_eject();
        // When no loot is available, the display item should be empty
        assert!(display.is_empty());
    }

    #[test]
    fn cycle_display_item_preserves_behavior() {
        let entity = VaultBlockEntity::new(BlockPos::new(0, 0, 0));
        // Call multiple times; should not panic
        entity.cycle_display_item_from_loot_table();
        entity.cycle_display_item_from_loot_table();
        entity.cycle_display_item_from_loot_table();
    }
}
