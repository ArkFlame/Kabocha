use super::BlockEntity;
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
}

impl VaultBlockEntity {
    /// `VaultServerData#MAX_REWARD_PLAYERS`.
    pub const MAX_REWARDED_PLAYERS: usize = 128;
}

/// Vanilla keeps rewarded players in `server_data` as a list of four-int UUID arrays.
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
    let excess = players.len().saturating_sub(VaultBlockEntity::MAX_REWARDED_PLAYERS);
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
        Self {
            position,
            config: Mutex::new(nbt.get_compound("config").cloned()),
            server_data: Mutex::new(server_data),
            shared_data: Mutex::new(nbt.get_compound("shared_data").cloned()),
            rewarded_players,
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
        if !rewarded.is_empty() {
            let mut data = server_data.take().unwrap_or_else(NbtCompound::new);
            data.put("rewarded_players", rewarded_players_tag(&rewarded));
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
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            config: Mutex::new(None),
            server_data: Mutex::new(None),
            shared_data: Mutex::new(None),
            rewarded_players: Mutex::new(Vec::new()),
        }
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
