use crate::block::registry::BlockRegistry;
use crate::command::commands::default_dispatcher;
use crate::command::commands::defaultgamemode::DefaultGamemode;
use crate::data::VanillaData;
use crate::data::player_server::ServerPlayerData;
use crate::entity::NBTStorage;
use crate::item::registry::ItemRegistry;
use crate::net::authentication::fetch_mojang_public_keys;
use crate::net::{ClientPlatform, DisconnectReason, EncryptionError, GameProfile, PlayerConfig};
use crate::plugin::PluginManager;
use crate::plugin::loader::PluginLoader;
use crate::plugin::player::player_login::PlayerLoginEvent;
use crate::plugin::server::server_broadcast::ServerBroadcastEvent;
use crate::server::tick_rate_manager::ServerTickRateManager;
use crate::world::WorldPortal;
use crate::world::custom_bossbar::CustomBossbars;
use crate::world::scoreboard::Scoreboard;
use crate::world::weather::{SharedWeatherData, WeatherData, WeatherSetup};
use crate::world::weather_profile::{
    DEFAULT_WEATHER_PROFILE, WeatherCycle, WeatherProfile, WeatherProfiles, WorldWeatherSettings,
};
use crate::{
    command::node::dispatcher::CommandDispatcher, entity::player::Player, world::World,
    world::map::MapManager,
};
use arc_swap::ArcSwap;
use connection_cache::{CachedBranding, CachedStatus};
use key_store::KeyStore;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::game_rules::{GameRule, GameRuleValue};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::permission::PermissionManager;
use pumpkin_util::text::color::NamedColor;
use pumpkin_world::dimension::into_level;
use pumpkin_world::world::WorldPortalExt;
use tracing::{debug, error, info, warn};

use crate::world::weather::own_weather_folder;
use pumpkin_protocol::java::client::login::CEncryptionRequest;
use pumpkin_protocol::java::client::play::{CChangeDifficulty, CGameEvent, CTabList, GameEvent};
use pumpkin_protocol::{ClientPacket, java::client::config::CPluginMessage};
use pumpkin_util::text::TextComponent;
use pumpkin_util::{Difficulty, GameMode};
use pumpkin_world::world_info::anvil::{
    AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME,
};
use pumpkin_world::world_info::data_files::read_weather;
use pumpkin_world::world_info::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};
use rand::seq::IndexedRandom;
use rayon::prelude::*;
use rsa::PublicKey as RsaPublicKey;
use std::fs;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32};
use std::{future::Future, sync::atomic::Ordering, time::Duration};
use tokio::sync::OnceCell;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub mod builtin_game_tests;
mod connection_cache;
pub(crate) mod debug_profiler;
pub mod enchantment;
mod key_store;
pub mod perf_profiler;
#[cfg(profiler)]
pub(crate) mod profiler;
pub mod recipe;
pub mod scheduler;
pub mod seasonal_events;
pub mod server_test_manager;
pub mod tick_rate_manager;
pub mod ticker;

pub use recipe::RecipeManager;

use crate::data::advancement_data::AdvancementManager;

/// Saved scoreboard file in `<world>/data/minecraft/`.
const SCOREBOARD_FILE: &str = "scoreboard.dat";
/// Prefix of the per-namespace command storage files in `<world>/data/minecraft/`.
const COMMAND_STORAGE_PREFIX: &str = "command_storage_";
/// Force-loaded chunks of a dimension, saved under its own folder.
const FORCED_CHUNKS_FILE: &str = "chunks.dat";
/// Pending `/schedule` events, saved like vanilla's `TimerQueue`.
const SCHEDULED_EVENTS_FILE: &str = "scheduled_events.dat";
/// Rolling TPS window sizes, in ticks at the vanilla 20 TPS rate. Vanilla's
/// `tickTimes60s` is 1200 samples (one minute); Paper reports 1m/5m/15m TPS.
const TPS_SAMPLES_1M: usize = 20 * 60;
const TPS_SAMPLES_5M: usize = 20 * 60 * 5;
const TPS_SAMPLES_15M: usize = 20 * 60 * 15;

/// Represents a Minecraft server instance.
#[repr(C)]
pub struct Server {
    pub basic_config: BasicConfiguration,
    pub advanced_config: AdvancedConfiguration,
    /// Live resource pack configuration, mutable at runtime through the plugin API.
    pub resource_pack: pumpkin_config::resource_pack::ResourcePackSettings,
    pub telemetry_config: TelemetryConfig,
    /// Tracks active and completed player session durations for anonymous telemetry.
    pub telemetry_sessions: crate::telemetry::TelemetrySessionTracker,

    pub data: VanillaData,

    /// Plugin manager
    pub plugin_manager: Arc<PluginManager>,

    /// Permission manager for the server.
    pub permission_manager: Arc<PermissionManager>,

    /// Handles cryptographic keys for secure communication.
    key_store: OnceCell<Arc<KeyStore>>,
    /// Bedrock OIDC provider keys, fetched on startup for 1.26.10+ token validation.
    pub bedrock_oidc_keys: Arc<OnceCell<(String, pumpkin_auth::jwt::Jwks)>>,
    /// Cached Bedrock server private key (process-lifetime). Generated on first Bedrock login and reused.
    /// Manages server status information.
    listing: std::sync::Mutex<CachedStatus>,
    /// Makes the join checks and the insertion into a world atomic.
    player_join_lock: std::sync::Mutex<()>,
    /// Per-IP connection, status and login rate limiter with its ban list.
    pub rate_limiter: std::sync::Mutex<crate::net::rate_limit::RateLimiter>,
    /// Saves server branding information.
    branding: CachedBranding,
    /// Saves and dispatches commands to appropriate handlers.
    pub command_dispatcher: ArcSwap<CommandDispatcher>,
    /// Block behaviour.
    pub block_registry: Arc<BlockRegistry>,
    /// Item behaviour.
    pub item_registry: Arc<ItemRegistry>,
    /// Manages multiple worlds within the server.
    pub worlds: ArcSwap<Vec<Arc<World>>>,
    /// All the dimensions that exist on the server.
    pub dimensions: Vec<Dimension>,
    /// Assigns unique IDs to containers.
    container_id: AtomicU32,
    pub recipe_manager: Arc<recipe::RecipeManager>,
    pub datapack_manager: Arc<crate::data::datapack::DatapackManager>,
    pub enchantment_manager: Arc<enchantment::EnchantmentManager>,
    /// Assigns unique IDs to maps.
    map_id: AtomicI32,
    /// Mojang's public keys, used for chat session signing
    /// Pulled from Mojang API on startup
    pub mojang_public_keys: ArcSwap<Vec<RsaPublicKey>>,
    /// The server's custom bossbars
    pub bossbars: std::sync::Mutex<CustomBossbars>,
    /// Manages all maps on the server
    pub map_manager: MapManager,
    /// The default gamemode when a player joins the server (reset every restart)
    pub defaultgamemode: std::sync::Mutex<DefaultGamemode>,
    /// Manages player data storage
    pub player_data_storage: ServerPlayerData,
    /// Command storage for `/data` command
    pub command_storage:
        std::sync::Mutex<std::collections::HashMap<String, pumpkin_nbt::compound::NbtCompound>>,
    /// Global server stopwatches
    pub stopwatches: std::sync::Mutex<crate::world::stopwatches::Stopwatches>,
    /// Global server random sequences
    pub random_sequences: std::sync::Mutex<crate::world::random_sequences::RandomSequences>,
    // Manages player advancement
    pub advancement_manager: Arc<AdvancementManager>,
    // Whether the server whitelist is on or off
    pub white_list: AtomicBool,
    /// Manages the server's tick rate, freezing, and sprinting
    pub tick_rate_manager: Arc<ServerTickRateManager>,
    /// Stores the duration of the last 100 ticks for performance analysis
    pub tick_times_nanos: std::sync::Mutex<[i64; 100]>,
    /// Tick durations over the last 1, 5 and 15 minutes, backing Paper-style rolling TPS.
    /// Boxed so the large rings do not bloat `Server`'s (and `new`'s) stack frame.
    pub tick_times_1m_nanos: std::sync::Mutex<Box<[i64]>>,
    pub tick_times_5m_nanos: std::sync::Mutex<Box<[i64]>>,
    pub tick_times_15m_nanos: std::sync::Mutex<Box<[i64]>>,
    /// Aggregated tick times for efficient rolling average calculation
    pub aggregated_tick_times_nanos: AtomicI64,
    /// Wall-clock instant of the previous tick-time debug sample, used to
    /// measure the raw full tick interval for the client's `TICK_TIME` chart.
    last_tick_sample_instant: std::sync::Mutex<Option<std::time::Instant>>,
    /// Total number of ticks processed by the server
    pub tick_count: AtomicI32,
    /// Owns the server-wide tick profiling session used by `/debug`.
    pub(crate) debug_profiler: debug_profiler::DebugProfiler,
    /// The metrics recording session driven by `/perf`
    pub perf_profiler: perf_profiler::PerfProfiler,
    /// Random unique Server ID used by Bedrock Edition
    pub server_guid: u64,
    /// Player idle timeout in minutes (0 = disabled)
    pub player_idle_timeout: AtomicI32,
    /// Manages scheduled tasks (e.g. from plugins)
    /// Manages scheduled datapack functions (`/schedule`)
    pub scheduled_functions: Arc<crate::server::scheduler::ScheduledFunctionQueue>,
    tasks: TaskTracker,
    pub runtime: tokio::runtime::Handle,
    /// Dedicated Rayon thread pool for synchronous world and entity simulation (ticks).
    pub simulation_pool: Arc<rayon::ThreadPool>,
    /// Dedicated Rayon thread pool for background CPU computation (RSA, Bedrock crypto, compression).
    pub compute_pool: Arc<rayon::ThreadPool>,
    pub management_hub: Arc<crate::net::management::hub::ManagementHub>,
    /// Cancelled when this server should stop. A child of [`crate::STOP_INTERRUPT`], so a
    /// process-wide stop reaches it, while an integrated server can stop without ending the
    /// process.
    pub stop_token: CancellationToken,
    /// Set once shutdown closes admission, so the shutdown kick pass sees every admitted player.
    admission_closed: std::sync::RwLock<bool>,
    /// Stores the latest available Pumpkin release if the startup update checker found a newer version.
    pub available_update: std::sync::OnceLock<crate::update_checker::GitHubRelease>,

    // world stuff which maybe should be put into a struct
    pub level_info: Arc<ArcSwap<LevelData>>,
    /// The single scoreboard shared by every world, matching vanilla's server-wide scoreboard.
    pub scoreboard: Arc<std::sync::Mutex<Scoreboard>>,
    /// The weather cycle every world follows.
    pub weather_data: SharedWeatherData,
    /// Weather profiles, loaded from `<world>/weather_profiles` and registered by plugins.
    pub weather_profiles: WeatherProfiles,
    world_info_writer: Arc<dyn WorldInfoWriter>,
}

impl Server {
    #[expect(clippy::too_many_lines)]
    pub async fn new(
        basic_config: BasicConfiguration,
        advanced_config: AdvancedConfiguration,
        telemetry_config: TelemetryConfig,
        vanilla_data: VanillaData,
        plugin_loaders: Vec<Arc<dyn PluginLoader>>,
    ) -> Result<Arc<Self>, WorldInfoError> {
        let permission_manager = Arc::new(PermissionManager::new());
        // First register the default commands. After that, plugins can put in their own.
        let command_dispatcher = ArcSwap::from_pointee(default_dispatcher(
            &permission_manager,
            &advanced_config.commands,
        ));

        crate::command::set_broadcast_console_to_ops(
            advanced_config.commands.broadcast_console_to_ops,
        );

        let world_path = basic_config.get_world_path();

        let block_registry = super::block::registry::default_registry();

        let is_new_world;
        let level_info = match AnvilLevelInfo.read_world_info(&world_path) {
            Ok(level_info) => {
                is_new_world = false;
                let dat_path = world_path.join(LEVEL_DAT_FILE_NAME);
                if dat_path.exists() {
                    let backup_path = world_path.join(LEVEL_DAT_BACKUP_FILE_NAME);
                    if let Err(err) = fs::copy(&dat_path, &backup_path) {
                        warn!("Failed to create backup {LEVEL_DAT_BACKUP_FILE_NAME}: {err}");
                    }
                }
                level_info
            }
            Err(WorldInfoError::InfoNotFound) => {
                warn!(
                    "No {LEVEL_DAT_FILE_NAME} in {}, creating a new {} world with seed {}",
                    world_path.display(),
                    basic_config.level_type,
                    basic_config.seed.0 as i64
                );
                let mut default_data = LevelData::for_new_world(
                    basic_config.seed,
                    &basic_config.level_type,
                    &basic_config.generator_settings,
                    basic_config.generate_structures,
                );
                if basic_config.hardcore {
                    default_data.difficulty = Difficulty::Hard;
                    default_data.difficulty_locked = true;
                }
                AnvilLevelInfo.write_world_info(&default_data, &world_path)?;
                is_new_world = true;
                default_data
            }
            Err(
                error @ (WorldInfoError::UnsupportedDataVersion(_)
                | WorldInfoError::UnsupportedLevelVersion(_)),
            ) => {
                error!("Failed to load world info!");
                error!("{error}");
                error!("Unsupported world version! See the logs for more info.");
                return Err(error);
            }
            Err(error) => {
                error!("Failed to load the world data in {}!", world_path.display());
                error!("{error}");
                error!(
                    "Refusing to continue: a default world would generate different terrain on top of the existing region files. Restore {LEVEL_DAT_FILE_NAME} from {LEVEL_DAT_BACKUP_FILE_NAME}, which also holds a copy of the world seed, or move the world folder aside to start a new world."
                );
                error!("Failed to load the world data! See the logs for more info.");
                return Err(error);
            }
        };

        let mut level_info = level_info;
        if basic_config.hardcore {
            level_info.difficulty = Difficulty::Hard;
            level_info.difficulty_locked = true;
            if let Err(err) = AnvilLevelInfo.write_world_info(&level_info, &world_path) {
                error!("Failed to persist hardcore difficulty fix: {err}");
            }
        }

        // Vanilla applies the default-game-mode property to the level's own game type
        // when the world loads (MinecraftServer.loadLevel -> worldData.setGameType).
        level_info.game_type = basic_config.default_gamemode;

        let seed = level_info.world_gen_settings.seed;
        let weather_profiles = WeatherProfiles::default();
        weather_profiles.load_folder(&basic_config.get_world_path().join("weather_profiles"));
        // The `default` profile, if there is one, shapes the cycle the worlds share.
        let mut weather_data = WeatherData::from_level_data(&level_info);
        if let Some(profile) = weather_profiles.get(DEFAULT_WEATHER_PROFILE) {
            weather_data = weather_data.with_profile(profile);
            weather_data.restore_phases(&level_info.weather_phases);
        }
        let weather_data = Arc::new(std::sync::Mutex::new(weather_data));
        let level_info = Arc::new(ArcSwap::new(Arc::new(level_info)));

        let listing = std::sync::Mutex::new(CachedStatus::new(
            &basic_config,
            &advanced_config.networking.java.motd,
            advanced_config.networking.java.max_players,
        ));
        let defaultgamemode = std::sync::Mutex::new(DefaultGamemode {
            gamemode: basic_config.default_gamemode,
        });
        let players_dir = world_path.join("players");
        let player_data_storage = ServerPlayerData::new(
            players_dir.join("data"),
            Duration::from_secs(advanced_config.player_data.save_player_cron_interval),
            advanced_config.player_data.save_player_data,
        );
        let advancement_manager = Arc::new(AdvancementManager::new(
            players_dir.clone(),
            advanced_config.advancement.enabled,
            advanced_config.advancement.save_advancements,
        ));
        let white_list = AtomicBool::new(basic_config.white_list);

        let tick_rate_manager = Arc::new(ServerTickRateManager::new(basic_config.tps));

        let dimensions = {
            let mut dimensions = vec![Dimension::OVERWORLD];
            if basic_config.allow_nether {
                dimensions.push(Dimension::THE_NETHER);
            }
            if basic_config.allow_end {
                dimensions.push(Dimension::THE_END);
            }
            dimensions
        };
        info!(
            "Enabled dimensions: {:?}",
            dimensions
                .iter()
                .map(|d| d.minecraft_name)
                .collect::<Vec<_>>()
        );

        let management_settings = Arc::new(crate::net::management::hub::ManagementSettings::new(
            &basic_config,
            &advanced_config,
            &advanced_config.networking.management,
        ));
        let management_hub = Arc::new(crate::net::management::hub::ManagementHub::new(
            management_settings,
        ));

        let cpus = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
        let simulation_pool = Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(cpus)
                .thread_name(|i| format!("Simulation-{i}"))
                .build()
                .map_err(|e| WorldInfoError::IoError(std::io::Error::other(e)))?,
        );
        let compute_threads = (cpus / 2).clamp(2, 8);
        let compute_pool = Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(compute_threads)
                .thread_name(|i| format!("Compute-{i}"))
                .build()
                .map_err(|e| WorldInfoError::IoError(std::io::Error::other(e)))?,
        );

        let rate_limiter = std::sync::Mutex::new(crate::net::rate_limit::RateLimiter::from_config(
            &advanced_config.networking.java.rate_limit,
        ));

        let server = Self {
            basic_config,
            resource_pack: pumpkin_config::resource_pack::ResourcePackSettings::new(
                advanced_config.resource_pack.clone(),
            ),
            advanced_config,
            telemetry_config,
            telemetry_sessions: crate::telemetry::TelemetrySessionTracker::default(),
            data: vanilla_data,
            plugin_manager: Arc::new(PluginManager::new(plugin_loaders)),
            permission_manager,
            container_id: 0.into(),
            recipe_manager: Arc::new(recipe::RecipeManager::new()),
            datapack_manager: Arc::new(crate::data::datapack::DatapackManager::new()),
            enchantment_manager: Arc::new(enchantment::EnchantmentManager::new()),
            map_id: level_info.load().map_id.into(),
            worlds: ArcSwap::from_pointee(vec![]),
            dimensions,
            command_dispatcher,
            block_registry: block_registry.clone(),
            item_registry: super::item::items::default_registry(),
            key_store: OnceCell::new(),
            bedrock_oidc_keys: Arc::new(OnceCell::new()),
            listing,
            player_join_lock: std::sync::Mutex::new(()),
            rate_limiter,
            branding: CachedBranding::new(),
            bossbars: std::sync::Mutex::new(CustomBossbars::new()),
            map_manager: MapManager::new(),
            defaultgamemode,
            player_data_storage,
            command_storage: std::sync::Mutex::new(std::collections::HashMap::new()),
            stopwatches: std::sync::Mutex::new(crate::world::stopwatches::Stopwatches::new()),
            random_sequences: std::sync::Mutex::new(
                crate::world::random_sequences::RandomSequences::new(),
            ),
            advancement_manager,
            white_list,
            tick_rate_manager,
            tick_times_nanos: std::sync::Mutex::new([0; 100]),
            tick_times_1m_nanos: std::sync::Mutex::new(vec![0; TPS_SAMPLES_1M].into_boxed_slice()),
            tick_times_5m_nanos: std::sync::Mutex::new(vec![0; TPS_SAMPLES_5M].into_boxed_slice()),
            tick_times_15m_nanos: std::sync::Mutex::new(
                vec![0; TPS_SAMPLES_15M].into_boxed_slice(),
            ),
            aggregated_tick_times_nanos: AtomicI64::new(0),
            last_tick_sample_instant: std::sync::Mutex::new(None),
            tick_count: AtomicI32::new(0),
            debug_profiler: debug_profiler::DebugProfiler::new(),
            perf_profiler: perf_profiler::PerfProfiler::default(),
            tasks: TaskTracker::new(),
            stop_token: crate::STOP_INTERRUPT.child_token(),
            admission_closed: std::sync::RwLock::new(false),
            runtime: tokio::runtime::Handle::current(),
            simulation_pool,
            compute_pool,
            scheduled_functions: Arc::new(crate::server::scheduler::ScheduledFunctionQueue::new()),
            server_guid: rand::random(),
            player_idle_timeout: AtomicI32::new(0),
            mojang_public_keys: ArcSwap::from_pointee(Vec::new()),
            world_info_writer: Arc::new(AnvilLevelInfo),
            scoreboard: Arc::new(std::sync::Mutex::new(Scoreboard::default())),
            weather_data,
            weather_profiles,
            level_info,
            management_hub,
            available_update: std::sync::OnceLock::new(),
        };
        let server = Arc::new(server);

        // Fetch / generate keys in background tasks to avoid blocking startup
        let server_clone = server.clone();
        server.spawn_task(async move {
            match KeyStore::new() {
                Ok(key_store) => {
                    let _ = server_clone.key_store.set(Arc::new(key_store));
                }
                // `get_or_init_key_store` retries on first use and reports the failure there.
                Err(err) => error!("Failed to generate encryption keys: {err}"),
            }
        });

        if server.basic_config.allow_chat_reports {
            let server_clone = server.clone();
            server.spawn_task(async move {
                let auth_config = server_clone
                    .advanced_config
                    .networking
                    .java
                    .authentication
                    .clone();
                let keys = fetch_mojang_public_keys(&auth_config)
                    .await
                    .unwrap_or_else(|e| {
                        error!("Failed to fetch Mojang keys: {e}");
                        Vec::new()
                    });
                server_clone.mojang_public_keys.store(Arc::new(keys));
            });
        }

        if server.advanced_config.networking.bedrock.online_mode
            && server
                .advanced_config
                .networking
                .bedrock
                .authentication
                .enabled
        {
            let server_clone = server.clone();
            server.spawn_task(async move {
                let auth = server_clone
                    .advanced_config
                    .networking
                    .bedrock
                    .authentication
                    .clone();
                let keys = match pumpkin_auth::jwt::fetch_oidc_jwks(
                    auth.url.as_deref(),
                    auth.connect_timeout,
                    auth.read_timeout,
                )
                .await
                {
                    Ok(keys) => keys,
                    Err(error) => {
                        error!("Failed to fetch Bedrock OIDC keys: {error}");
                        (String::new(), pumpkin_auth::jwt::Jwks { keys: Vec::new() })
                    }
                };
                let _ = server_clone.bedrock_oidc_keys.set(keys);
            });
        }

        let mut worlds_vec = Vec::new();
        for dim in &server.dimensions {
            info!(
                "Loading {}",
                TextComponent::text(dim.minecraft_name.to_string())
                    .color_named(NamedColor::DarkGreen)
                    .to_pretty_console()
            );
            let config = Arc::new(server.advanced_config.world.clone());
            let level = into_level(dim.clone(), &config, world_path.clone(), seed);
            let mut world = World::load(
                level.clone(),
                server.level_info.clone(),
                dim.clone(),
                block_registry.clone(),
                WeatherSetup {
                    data: server.weather_data.clone(),
                    ..WeatherSetup::default()
                },
                Arc::downgrade(&server),
            );
            world.scoreboard = server.scoreboard.clone();
            let world = Arc::new(world);
            let portal: Arc<dyn WorldPortalExt> = Arc::new(WorldPortal(world.clone()));
            level.world_portal.store(Arc::new(Some(portal)));
            worlds_vec.push(world);
        }

        for world in &worlds_vec {
            let mut world_init_event =
                crate::plugin::api::events::world::world_init::WorldInitEvent::new(world.clone());
            server
                .plugin_manager
                .fire(&server, &mut world_init_event)
                .await;

            let mut world_load_event =
                crate::plugin::api::events::world::world_load::WorldLoadEvent::new(world.clone());
            server
                .plugin_manager
                .fire(&server, &mut world_load_event)
                .await;
        }

        server.worlds.store(Arc::new(worlds_vec));
        server.load_persistent_data(&world_path);

        // Vanilla puts a new world's spawn on the surface. The default level data is written
        // with `DEFAULT_SPAWN_Y` before any chunk exists, so replace it with the real spawn
        // height once the spawn chunk can be generated; the console command source uses this
        // position for relative coordinates.
        if is_new_world && let Some(world) = server.get_world_from_dimension(&Dimension::OVERWORLD)
        {
            let (spawn_x, spawn_z, fallback_y) = {
                let info = server.level_info.load();
                (info.spawn_x, info.spawn_z, info.spawn_y)
            };
            let position = world
                .get_safe_player_spawn_position(spawn_x, spawn_z, fallback_y)
                .await;
            let spawn_y = position.y.floor() as i32;
            server.level_info.rcu(|info| {
                let mut info = (**info).clone();
                info.spawn_y = spawn_y;
                info
            });
            let info = server.level_info.load();
            if let Err(err) = AnvilLevelInfo.write_world_info(&info, &world_path) {
                error!("Failed to persist the world spawn height: {err}");
            }
        }

        info!("All worlds loaded successfully.");

        let enabled_packs = server.level_info.load().data_packs.enabled.clone();
        if let Err(err) =
            server
                .datapack_manager
                .load_all(&world_path, &enabled_packs, &server.recipe_manager)
        {
            error!("Failed to load datapacks at startup: {err}");
        }

        // Vanilla runs the load tag from the function manager tick, not during startup.
        server.datapack_manager.mark_load_pending();

        Ok(server)
    }

    /// Loads the scoreboard and command storage that datapacks rely on across restarts.
    /// Vanilla keeps one scoreboard for the server, so every world starts from the saved copy.
    /// The overworld's game time, which vanilla uses as the clock for scheduled events.
    fn overworld_game_time(&self) -> i64 {
        self.worlds
            .load()
            .iter()
            .find(|world| world.dimension.minecraft_name == Dimension::OVERWORLD.minecraft_name)
            .map_or(0, |world| {
                world
                    .level_time
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .world_age
            })
    }

    /// Restores a world's forced chunks from its `<dim_folder>/data/minecraft/chunks.dat`.
    /// Vanilla restores these in every `ServerLevel` constructor, including levels created at
    /// runtime, so this cannot live only in the startup `load_persistent_data` pass.
    fn restore_forced_chunks(world: &Arc<World>) {
        use pumpkin_world::world_info::data_files::read_saved_data;

        let Some(data) = read_saved_data(&world.level.level_folder.dim_folder, FORCED_CHUNKS_FILE)
        else {
            return;
        };
        let Some(forced) = data.get_long_array("Forced") else {
            return;
        };
        let mut forced_chunks = world
            .forced_chunks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut chunk_loading = world
            .level
            .chunk_loading
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut newly_forced = Vec::new();
        for packed in forced {
            // Vanilla `ChunkPos.toLong`: x in the low 32 bits, z in the high 32 bits.
            let x = *packed as i32;
            let z = (*packed >> 32) as i32;
            let pos = pumpkin_util::math::vector2::Vector2::new(x, z);
            if forced_chunks.insert(pos) {
                chunk_loading.add_force_ticket(pos);
                newly_forced.push(pos);
            }
        }
        chunk_loading.send_change();
        drop(chunk_loading);
        drop(forced_chunks);
        world.update_active_chunks();
        world.load_forced_entity_chunks(newly_forced);
    }

    fn load_persistent_data(&self, world_path: &std::path::Path) {
        use pumpkin_world::world_info::data_files::{minecraft_data_dir, read_saved_data};

        if let Some(data) = read_saved_data(world_path, SCOREBOARD_FILE) {
            self.scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .load_saved_nbt(&data);
        }

        if let Some(data) = read_saved_data(world_path, SCHEDULED_EVENTS_FILE) {
            self.scheduled_functions.load_saved_nbt(
                &data,
                self.tick_count.load(Ordering::Relaxed) as u64,
                self.overworld_game_time(),
            );
        }

        for world in self.worlds.load().iter() {
            Self::restore_forced_chunks(world);
        }

        let Ok(entries) = std::fs::read_dir(minecraft_data_dir(world_path)) else {
            return;
        };
        let mut storage = self
            .command_storage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(namespace) = file_name
                .to_str()
                .and_then(|name| name.strip_prefix(COMMAND_STORAGE_PREFIX))
                .and_then(|name| name.strip_suffix(".dat"))
            else {
                continue;
            };
            let Some(mut data) = read_saved_data(
                world_path,
                &format!("{COMMAND_STORAGE_PREFIX}{namespace}.dat"),
            ) else {
                continue;
            };
            let Some(pumpkin_nbt::tag::NbtTag::Compound(contents)) =
                data.child_tags.remove("contents")
            else {
                continue;
            };
            for (path, tag) in contents.child_tags {
                if let pumpkin_nbt::tag::NbtTag::Compound(compound) = tag {
                    storage.insert(format!("{namespace}:{path}"), compound);
                }
            }
        }
    }

    /// Writes the scoreboard and command storage to `<world>/data/minecraft/`.
    pub fn save_persistent_data(&self) -> Result<(), String> {
        use pumpkin_world::world_info::data_files::write_saved_data;

        let world_path = self.basic_config.get_world_path();
        let data_version = self.level_info.load().data_version;

        if let Some(world) = self.worlds.load().first() {
            let data = world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .to_saved_nbt();
            write_saved_data(&world_path, SCOREBOARD_FILE, data_version, data)
                .map_err(|e| format!("Failed to save {SCOREBOARD_FILE}: {e}"))?;
        }

        let data = self.scheduled_functions.to_saved_nbt(
            self.tick_count.load(Ordering::Relaxed) as u64,
            self.overworld_game_time(),
        );
        write_saved_data(&world_path, SCHEDULED_EVENTS_FILE, data_version, data)
            .map_err(|e| format!("Failed to save {SCHEDULED_EVENTS_FILE}: {e}"))?;

        for world in self.worlds.load().iter() {
            let forced: Vec<i64> = world
                .forced_chunks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .iter()
                .map(|pos| i64::from(pos.x as u32) | (i64::from(pos.y as u32) << 32))
                .collect();
            let mut data = pumpkin_nbt::compound::NbtCompound::new();
            data.put("Forced", pumpkin_nbt::tag::NbtTag::LongArray(forced));
            write_saved_data(
                &world.level.level_folder.dim_folder,
                FORCED_CHUNKS_FILE,
                data_version,
                data,
            )
            .map_err(|e| format!("Failed to save {FORCED_CHUNKS_FILE}: {e}"))?;
        }

        let mut namespaces: std::collections::HashMap<String, pumpkin_nbt::compound::NbtCompound> =
            std::collections::HashMap::new();
        {
            let storage = self
                .command_storage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (id, compound) in storage.iter() {
                let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
                namespaces
                    .entry(namespace.to_string())
                    .or_default()
                    .put_compound(path, compound.clone());
            }
        }
        for (namespace, contents) in namespaces {
            let mut data = pumpkin_nbt::compound::NbtCompound::new();
            data.put_compound("contents", contents);
            let file_name = format!("{COMMAND_STORAGE_PREFIX}{namespace}.dat");
            write_saved_data(&world_path, &file_name, data_version, data)
                .map_err(|e| format!("Failed to save {file_name}: {e}"))?;
        }
        Ok(())
    }

    /// Spawns a task associated with this server. All tasks spawned with this method are awaited
    /// when the server stops. This means tasks should complete in a reasonable (no looping) amount of time.
    pub fn spawn_task<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tasks.spawn_on(task, &self.runtime)
    }

    /// Returns whether the dimension is enabled in the server configuration.
    #[must_use]
    pub fn is_dimension_enabled(&self, dimension: &Dimension) -> bool {
        match dimension.minecraft_name {
            name if name == Dimension::THE_NETHER.minecraft_name => self.basic_config.allow_nether,
            name if name == Dimension::THE_END.minecraft_name => self.basic_config.allow_end,
            _ => true,
        }
    }

    /// Returns the world for `dimension`, falling back to the first loaded world, or `None`
    /// when no world is loaded.
    #[must_use]
    pub fn get_world_from_dimension(&self, dimension: &Dimension) -> Option<Arc<World>> {
        let worlds = self.worlds.load();
        worlds
            .iter()
            .find(|w| w.dimension.minecraft_name == dimension.minecraft_name)
            .cloned()
            .or_else(|| worlds.first().cloned())
    }

    /// Creates a world, or returns the overworld if its dimension is disabled.
    pub fn create_world(self: &Arc<Self>, name: String, dimension: Dimension) -> Arc<World> {
        if !self.is_dimension_enabled(&dimension) {
            warn!(
                "Cannot create world '{name}': dimension {} is disabled",
                dimension.minecraft_name
            );
            if let Some(overworld) = self.get_world_from_dimension(&Dimension::OVERWORLD) {
                return overworld;
            }
        }

        self.spawn_world(name, dimension, &WorldWeatherSettings::default(), None)
    }

    /// Creates or loads a world with its own weather handling. An already loaded world of
    /// that name and dimension is returned as it is.
    pub fn create_world_with_weather(
        self: &Arc<Self>,
        name: String,
        dimension: Dimension,
        weather: &WorldWeatherSettings,
    ) -> Result<Arc<World>, String> {
        if !self.is_dimension_enabled(&dimension) {
            return Err(format!(
                "dimension {} is disabled",
                dimension.minecraft_name
            ));
        }
        let profile = match &weather.profile {
            Some(id) if weather.cycle == WeatherCycle::Shared => {
                return Err(format!(
                    "weather profile '{id}' needs an own or fixed weather cycle"
                ));
            }
            Some(id) => Some(
                self.weather_profiles
                    .get(id)
                    .ok_or_else(|| format!("unknown weather profile '{id}'"))?,
            ),
            None => None,
        };
        Ok(self.spawn_world(name, dimension, weather, profile))
    }

    fn spawn_world(
        self: &Arc<Self>,
        name: String,
        dimension: Dimension,
        weather: &WorldWeatherSettings,
        profile: Option<Arc<WeatherProfile>>,
    ) -> Arc<World> {
        {
            let worlds = self.worlds.load();
            let world = worlds
                .iter()
                .find(|w| w.get_world_name() == name && w.dimension == dimension)
                .cloned();
            if let Some(world) = world {
                return world;
            }
        }

        let world_path = self.basic_config.get_world_path().join(name);
        let registry = self.block_registry.clone();
        let l_info = self.level_info.clone();
        let weak = Arc::downgrade(self);
        let config = Arc::new(self.advanced_config.world.clone());
        let seed = self.level_info.load().world_gen_settings.seed;

        let level =
            pumpkin_world::dimension::into_level(dimension.clone(), &config, world_path, seed);
        let data = match weather.cycle {
            WeatherCycle::Shared => self.weather_data.clone(),
            WeatherCycle::Own | WeatherCycle::Fixed { .. } => {
                let saved = (weather.cycle == WeatherCycle::Own)
                    .then(|| read_weather(&own_weather_folder(&level, &dimension)));
                let mut data = saved
                    .as_ref()
                    .map_or_else(WeatherData::default, WeatherData::from_file);
                if let Some(profile) = profile {
                    data = data.with_profile(profile);
                    if let Some(saved) = &saved {
                        data.restore_phases(&saved.custom_phases);
                    }
                }
                if let WeatherCycle::Fixed {
                    raining,
                    thundering,
                } = weather.cycle
                {
                    data.set_parameters(0, 0, raining, thundering);
                }
                Arc::new(std::sync::Mutex::new(data))
            }
        };
        let mut world: World = World::load(
            level.clone(),
            l_info,
            dimension,
            registry,
            WeatherSetup {
                data,
                cycle: weather.cycle,
                enabled: weather.enabled,
            },
            weak,
        );
        world.scoreboard = self.scoreboard.clone();
        let world = Arc::new(world);
        let portal: Arc<dyn WorldPortalExt> = Arc::new(WorldPortal(world.clone()));
        level.world_portal.store(Arc::new(Some(portal)));
        self.worlds.rcu(|worlds| {
            let mut new_worlds = (**worlds).clone();
            new_worlds.push(world.clone());
            new_worlds
        });
        // Vanilla restores forced chunks in the ServerLevel constructor, so runtime worlds
        // need this too, not just the worlds present at startup.
        Self::restore_forced_chunks(&world);
        let mut event =
            crate::plugin::api::events::world::world_init::WorldInitEvent::new(world.clone());
        self.plugin_manager.fire_blocking(self, &mut event);
        // Startup worlds fire both events, so a world created at runtime must too.
        let mut load_event =
            crate::plugin::api::events::world::world_load::WorldLoadEvent::new(world.clone());
        self.plugin_manager.fire_blocking(self, &mut load_event);
        world
    }

    pub async fn unload_world(&self, name: &str) -> Result<(), String> {
        let worlds = self.worlds.load();
        let world_to_unload = worlds
            .iter()
            .find(|w| w.get_world_name() == name || w.dimension.minecraft_name == name)
            .cloned()
            .ok_or_else(|| format!("World '{name}' not found"))?;

        if let Some(first_world) = worlds.first()
            && Arc::ptr_eq(first_world, &world_to_unload)
        {
            return Err("Cannot unload the primary/default world".to_string());
        }

        let player_count = world_to_unload.players.load().len();
        if player_count > 0 {
            return Err(format!(
                "Cannot unload world '{name}': {player_count} players are still in this world"
            ));
        }

        world_to_unload.shutdown().await;
        world_to_unload.unload().await;

        self.worlds.rcu(|w_list| {
            let mut new_list = (**w_list).clone();
            new_list.retain(|w| !Arc::ptr_eq(w, &world_to_unload));
            new_list
        });

        Ok(())
    }

    /// Copies the overworld clock into the level data so `level.dat` keeps the
    /// game time and time of day across restarts, like vanilla.
    fn sync_level_time(&self) {
        let Some((day_time, game_time, partial_tick, rate, paused)) = self
            .worlds
            .load()
            .iter()
            .find(|world| world.dimension.minecraft_name == Dimension::OVERWORLD.minecraft_name)
            .map(|world| {
                let time = world
                    .level_time
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                (
                    time.time_of_day,
                    time.world_age,
                    time.partial_tick,
                    time.rate,
                    time.paused,
                )
            })
        else {
            return;
        };
        self.level_info.rcu(|info| {
            let mut info = (**info).clone();
            info.day_time = day_time;
            info.game_time = game_time;
            info.clock_partial_tick = partial_tick;
            info.clock_rate = rate;
            info.clock_paused = paused;
            info
        });
    }

    /// Copies the overworld world border into the level data so `/worldborder`
    /// changes survive a restart, like vanilla's level.dat border fields.
    fn sync_level_border(&self) {
        let Some(world) = self
            .worlds
            .load()
            .iter()
            .find(|world| world.dimension.minecraft_name == Dimension::OVERWORLD.minecraft_name)
            .cloned()
        else {
            return;
        };
        let border = world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let center_x = border.center_x;
        let center_z = border.center_z;
        let size = border.current_diameter();
        let lerp_target = border.new_diameter;
        let lerp_time = border.remaining_ms();
        let warning_blocks = f64::from(border.warning_blocks);
        let warning_time = f64::from(border.warning_time);
        let damage_per_block = f64::from(border.damage_per_block);
        let safe_zone = f64::from(border.buffer);
        drop(border);
        self.level_info.rcu(|info| {
            let mut info = (**info).clone();
            info.border_center_x = center_x;
            info.border_center_z = center_z;
            info.border_size = size;
            info.border_size_lerp_target = lerp_target;
            info.border_size_lerp_time = lerp_time;
            info.border_warning_blocks = warning_blocks;
            info.border_warning_time = warning_time;
            info.border_damage_per_block = damage_per_block;
            info.border_safe_zone = safe_zone;
            info
        });
    }

    /// The level data with the weather and clock, which only the world keeps.
    fn level_data_to_save(&self) -> LevelData {
        self.sync_level_time();
        self.sync_level_border();
        let mut level_data = (**self.level_info.load()).clone();
        self.weather_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .store_in(&mut level_data);
        level_data
    }

    pub fn save_world_info(&self) -> Result<(), WorldInfoError> {
        let level_data = self.level_data_to_save();
        self.world_info_writer
            .write_world_info(&level_data, &self.basic_config.get_world_path())
    }

    pub fn reload_datapacks(&self) -> Result<(), String> {
        let enabled_packs = self.level_info.load().data_packs.enabled.clone();
        let world_path = self.basic_config.get_world_path();
        let result =
            self.datapack_manager
                .load_all(&world_path, &enabled_packs, &self.recipe_manager);
        if let Err(err) = &result {
            error!("Failed to reload datapacks: {err}");
        }

        // Do not run here: `/reload` and `/datapack` do not tick, and the load tag must see the
        // enabled pack set at execution time, so it is consumed by `tick_worlds`.
        self.datapack_manager.mark_load_pending();

        let dynamic_recipes = self.recipe_manager.get_dynamic_recipes_internal();
        // The stonecutter list only depends on the client version, so build it once per version.
        let mut stonecutter_packets = std::collections::BTreeMap::new();
        for player in self.get_all_players() {
            if let crate::net::ClientPlatform::Java(java_client) = player.client.as_ref() {
                let add_packet = pumpkin_protocol::java::client::play::CRecipeBookAdd::new(
                    true,
                    &dynamic_recipes,
                );
                if let Ok(data) = java_client.serialize_packet(&add_packet) {
                    java_client.try_enqueue_packet(data);
                }
                if let Some(data) = stonecutter_packets
                    .entry(java_client.version.load())
                    .or_insert_with(|| java_client.stonecutter_recipes_packet(&dynamic_recipes))
                    .clone()
                {
                    java_client.try_enqueue_packet(data);
                }
            }
        }
        result
    }

    #[must_use]
    pub fn get_known_packs<'a>(
        &self,
        server_version: &'a str,
        loaded_packs: &'a [crate::data::datapack::LoadedDatapack],
    ) -> Vec<pumpkin_protocol::KnownPack<'a>> {
        self.datapack_manager
            .get_known_packs(self, server_version, loaded_packs)
    }

    #[must_use]
    pub fn get_enabled_features(&self) -> Vec<&'static str> {
        self.datapack_manager.get_enabled_features(self)
    }

    #[must_use]
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.datapack_manager.is_feature_enabled(self, feature)
    }

    pub async fn save_all(&self) -> Result<(), String> {
        self.management_hub.broadcast_server_saving();

        // Vanilla `saveEverything` keeps going after a failed step so one bad file does not
        // skip the world saves; the failures are reported together at the end.
        let mut errors: Vec<String> = Vec::new();

        if let Err(err) = self.save_world_info() {
            error!("Failed to save world info: {err}");
            errors.push(format!("Failed to save world info: {err}"));
        }

        if let Err(err) = self.player_data_storage.save_all_players(self) {
            error!("Failed to save player data: {err}");
            errors.push(format!("Failed to save player data: {err}"));
        }

        if let Err(err) = self
            .advancement_manager
            .save_all_players(&self.get_all_players())
            .await
        {
            error!("Failed to save player advancements: {err}");
            errors.push(format!("Failed to save player advancements: {err}"));
        }

        if let Err(err) = self.save_persistent_data() {
            error!("{err}");
            errors.push(err);
        }

        for world in self.worlds.load().iter() {
            world.save().await;
        }

        self.management_hub.broadcast_server_saved();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    /// Discards placement from a disabled dimension before falling back to world spawn.
    fn saved_login_world(&self, data: &mut NbtCompound) -> Option<Arc<World>> {
        let key = data.get_string("Dimension")?;
        let Some(dimension) = Dimension::from_name(key) else {
            warn!("Invalid dimension key in player data: {key}");
            return None;
        };
        if self.is_dimension_enabled(dimension) {
            return self.get_world_from_dimension(dimension);
        }

        for tag in [
            "Pos",
            "Rotation",
            "Motion",
            "OnGround",
            "fall_distance",
            "FallDistance",
            "RootVehicle",
            "SleepTimer",
        ] {
            data.child_tags.remove(tag);
        }
        None
    }

    /// Adds a new player to the server.
    ///
    /// This function takes an `Arc<Client>` representing the connected client and performs the following actions:
    ///
    /// 1. Generates a new entity ID for the player.
    /// 2. Determines the player's gamemode (defaulting to Survival if not specified in configuration).
    /// 3. Selects the world for the player: the saved dimension when enabled, else the first world.
    /// 4. Creates a new `Player` instance using the provided information.
    /// 5. Adds the player to the chosen world.
    /// 6. Updates server listing: the online count always, the sample only when the client allows listing.
    ///
    /// # Arguments
    ///
    /// * `client`: An `Arc<Client>` representing the connected client.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    ///
    /// - `Arc<Player>`: A reference to the newly created player object.
    /// - `Arc<World>`: A reference to the world the player was added to.
    ///
    /// # Note
    ///
    /// You still have to spawn the `Player` in a `World` to let them join and make them visible.
    fn load_player_world(
        &self,
        id: &uuid::Uuid,
    ) -> Option<(Arc<World>, Option<pumpkin_nbt::compound::NbtCompound>)> {
        let first_world = self.worlds.load().first().cloned()?;
        let Ok(Some(mut data)) = self.player_data_storage.load_data(id) else {
            return Some((first_world, None));
        };
        let world = self.saved_login_world(&mut data).unwrap_or(first_world);
        Some((world, Some(data)))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the join sequence is kept in one ordered pass"
    )]
    pub fn add_player(
        self: &Arc<Self>,
        client: Arc<ClientPlatform>,
        profile: GameProfile,
        config: Option<PlayerConfig>,
    ) -> Option<(Arc<Player>, Arc<World>)> {
        // The login handlers check this too, but long before the insertion
        // and not on every path (proxies, no encryption, Bedrock).
        let _join_guard = self
            .player_join_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(reason) = self.join_rejection(&client, &profile) {
            client.try_kick(DisconnectReason::Kicked, &reason);
            return None;
        }

        let default_gamemode = self
            .defaultgamemode
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .gamemode;
        let force_gamemode = self.basic_config.force_gamemode;
        let (world, nbt) = self.load_player_world(&profile.id)?;

        // Vanilla ServerPlayer.load: saved `playerGameType`, else the forced
        // game type, else the level default. read_nbt restores the saved value
        // too, so pass it in up front to avoid a transient default gamemode.
        let gamemode = join_gamemode(default_gamemode, force_gamemode, nbt.as_ref());

        let mut player = Player::new(
            client,
            profile,
            config.clone().unwrap_or_default(),
            &world,
            gamemode,
        )?;

        if let Some(mut nbt_data) = nbt {
            if force_gamemode {
                // getForcedGameType() wins on join; let read_nbt apply the
                // forced mode's abilities instead of the saved mode's.
                nbt_data.put_int("playerGameType", gamemode as i32);
            }
            player.read_nbt(&mut nbt_data);
            // Vanilla persists HasPlayedBefore. Only infer it from Pos for
            // older Bedrock saves that predate the tag.
            if infer_played_before(&nbt_data) {
                player.has_played_before.store(true, Ordering::Relaxed);
            }
            // A save whose dimension is disabled loses its Pos (saved_login_world), so
            // the join must place it at world spawn, not at the default entity position.
            if !has_saved_position(&nbt_data) {
                player.has_played_before.store(false, Ordering::Relaxed);
            }
        }

        // Wrap in Arc after data is loaded
        let player = Arc::new(player);
        // Joining already sends play packets, which must reach `PacketSentEvent` for translation.
        player.client.set_player(&player);
        // Only Bedrock viewers see this skin, and the join must not wait for the download.
        if self.advanced_config.networking.bedrock.enabled {
            let weak_player = Arc::downgrade(&player);
            let properties = player.gameprofile.properties.load_full();
            let auth_config = self.advanced_config.networking.java.authentication.clone();
            self.spawn_task(async move {
                if let Some(mut skin) = Player::fetch_skin(&properties, &auth_config).await
                    && let Some(player) = weak_player.upgrade()
                {
                    if skin.skin_id == "Standard_Custom" {
                        let skin_id = format!("pumpkin:{}", player.gameprofile.id);
                        skin.skin_id.clone_from(&skin_id);
                        skin.full_id = skin_id;
                    }
                    player.bedrock_skin.store(Arc::new(skin));
                    // Bedrock players already online were sent this player with the fallback skin.
                    player.world().update_java_player_skin_for_bedrock(&player);
                }
            });
        }
        {
            let mut advancements = player
                .advancements
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Err(e) = advancements.load() {
                warn!("Error loading player {}: {e}", player.gameprofile.id);
            }
            advancements.player = Arc::downgrade(&player);
        };

        send_cancellable_blocking! {{
            self;
            &mut PlayerLoginEvent::new(player.clone(), TextComponent::text("You have been kicked from the server"));
            'after: {
                player.screen_handler_sync_handler.store_player(&(player.clone() as Arc<dyn pumpkin_inventory::screen_handler::InventoryPlayer>));
                if !self.admit_player(&world, &player) {
                    player.kick(
                        DisconnectReason::Shutdown,
                        &TextComponent::translate(
                            pumpkin_data::translation::java::MULTIPLAYER_DISCONNECT_SERVER_SHUTDOWN,
                            [],
                        ),
                    );
                    player.client.clear_player();
                    return None;
                }
                Some({
                    {
                        let mut user_cache = self
                            .data
                            .user_cache
                            .write()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        user_cache.upsert(player.gameprofile.id, player.gameprofile.name.clone());
                    };

                    // Vanilla counts every player in the status response; only the
                    // sample lists players that allow server listing.
                    let allows_listing = config.is_some_and(|config| config.server_listing);
                    let listed = self
                        .listing
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .add_player(&player, allows_listing);
                    player.listed_in_sample.store(listed, Ordering::Relaxed);

                    let player_dto = crate::net::management::dto::PlayerDto::new(
                        Some(player.gameprofile.id),
                        Some(player.gameprofile.name.clone()),
                    );
                    self.management_hub.broadcast_player_joined(&player_dto);
                    if self.telemetry_config.enabled {
                        self.telemetry_sessions.record_join(player.gameprofile.id);
                    }

                    (player, world)
                })
            }

            'cancelled: {
                player.kick(DisconnectReason::Kicked, &event.kick_message);
                player.client.clear_player();
                None
            }
        }}
    }

    /// Why `profile` can't join right now, if anything. Call with the join lock held.
    fn join_rejection(
        &self,
        client: &ClientPlatform,
        profile: &GameProfile,
    ) -> Option<TextComponent> {
        // Vanilla PlayerList#canBypassFullServerLogin applies the full-server kick only when
        // the profile cannot bypass the player limit (an op with bypassesPlayerLimit set).
        let bypasses_limit = {
            let ops = self
                .data
                .operator_config
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            can_bypass_player_limit(&ops, &profile.id)
        };
        let max_players = match client {
            ClientPlatform::Java(_) => self.advanced_config.networking.java.max_players,
            ClientPlatform::Bedrock(_) => self.advanced_config.networking.bedrock.max_players,
        };
        if !bypasses_limit && max_players > 0 && self.get_player_count() >= max_players as usize {
            return Some(TextComponent::translate_cross(
                pumpkin_data::translation::java::MULTIPLAYER_DISCONNECT_SERVER_FULL,
                pumpkin_data::translation::bedrock::DISCONNECTIONSCREEN_SERVERFULL,
                [],
            ));
        }
        if self.get_player_by_uuid(profile.id).is_some()
            || self.get_player_by_name(&profile.name).is_some()
        {
            return Some(TextComponent::translate_cross(
                pumpkin_data::translation::java::MULTIPLAYER_DISCONNECT_DUPLICATE_LOGIN,
                pumpkin_data::translation::bedrock::DISCONNECTIONSCREEN_LOGGEDINOTHERLOCATION,
                [],
            ));
        }
        None
    }

    pub fn remove_player(&self, player: &Player) {
        if self.telemetry_config.enabled {
            self.telemetry_sessions.record_leave(player);
        }
        let player_dto = crate::net::management::dto::PlayerDto::new(
            Some(player.gameprofile.id),
            Some(player.gameprofile.name.clone()),
        );
        self.management_hub.broadcast_player_left(&player_dto);

        player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::LeaveGame as i32,
            1,
        );
        self.listing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove_player(player, player.listed_in_sample.load(Ordering::Relaxed));
    }

    /// Adds player to `world` unless shutdown already closed admission.
    fn admit_player(&self, world: &World, player: &Arc<Player>) -> bool {
        let closed = self
            .admission_closed
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !*closed && world.add_player(player).is_ok()
    }

    /// Refuses all later joins. Returns once every join in progress is in a world, so the
    /// shutdown kick pass sees every admitted player.
    pub fn close_admission(&self) {
        // Joining holds this lock from the admission check until the player is in a world,
        // so waiting for it guarantees no join is between the check and the insertion.
        let _join_guard = self
            .player_join_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *self
            .admission_closed
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
    }
    /// Whether the server has begun shutting down and closed admission.
    pub fn is_shutting_down(&self) -> bool {
        *self
            .admission_closed
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Asks this server to stop: the ticker ends and a standalone server stops accepting
    /// connections and shuts down.
    pub fn stop(&self) {
        self.stop_token.cancel();
    }

    pub async fn shutdown(&self) {
        self.close_admission();
        self.management_hub.broadcast_server_stopping();

        self.tasks.close();
        debug!("Awaiting tasks for server");
        self.tasks.wait().await;
        debug!("Done awaiting tasks for server");

        info!("Stopping worlds");
        for world in self.worlds.load().iter() {
            world.shutdown().await;
        }
        if let Err(err) = self.save_persistent_data() {
            error!("{err}");
        }
        let level_data = self.level_data_to_save();
        // then lets save the world info

        if let Err(err) = self
            .world_info_writer
            .write_world_info(&level_data, &self.basic_config.get_world_path())
        {
            error!("Failed to save level.dat: {err}");
        }
        info!("Stopped worlds");
        #[cfg(profiler)]
        {
            let counters = profiler::counters(self);
            match tokio::task::spawn_blocking(move || {
                pumpkin_profiler::PROFILER.shutdown(&counters)
            })
            .await
            {
                Ok(Ok(())) => {}
                Ok(Err(error)) => error!("Failed to save profiler report on shutdown: {error}"),
                Err(error) => error!("Profiler shutdown task failed: {error}"),
            }
        }
    }

    /// Broadcasts a packet to all players in all worlds.
    ///
    /// This function sends the specified packet to every connected player in every world managed by the server.
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to the packet to be broadcast. The packet must implement the `ClientPacket` trait.
    pub fn broadcast_packet_all<P: ClientPacket>(&self, packet: &P) {
        for world in self.worlds.load().iter() {
            world.broadcast_packet_all(packet);
        }
    }

    /// Broadcasts custom server links to all players on the server.
    pub fn broadcast_server_links(&self, links: &[pumpkin_protocol::Link<'_>]) {
        let packet = pumpkin_protocol::java::client::play::CPlayServerLinks::new(links);
        self.broadcast_packet_all(&packet);
    }

    pub fn broadcast_tab_list_header_footer(&self, header: &TextComponent, footer: &TextComponent) {
        let packet = CTabList::new(header, footer);
        for world in self.worlds.load().iter() {
            for player in world.players.load().iter() {
                *player
                    .tab_list_header
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = header.clone();
                *player
                    .tab_list_footer
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = footer.clone();
            }
            world.broadcast_packet_all(&packet);
        }
    }

    pub fn broadcast_message(
        self: &Arc<Self>,
        message: &TextComponent,
        sender_name: &TextComponent,
        chat_type: u8,
        target_name: Option<&TextComponent>,
    ) {
        let mut event = ServerBroadcastEvent::new(message.clone(), sender_name.clone());
        self.plugin_manager.fire_blocking(self, &mut event);
        if !event.cancelled {
            for world in self.worlds.load().iter() {
                world.broadcast_message(&event.message, &event.sender, chat_type, target_name);
            }
        }
    }

    pub fn broadcast_chat_message(
        self: &Arc<Self>,
        message: &crate::net::chat::PlayerChatMessage,
        is_filtered: impl Fn(&crate::entity::player::Player) -> bool,
        sender_player: Option<&Arc<crate::entity::player::Player>>,
        chat_type: pumpkin_protocol::codec::var_int::VarInt,
        sender_name: &TextComponent,
        target_name: Option<&TextComponent>,
    ) {
        for world in self.worlds.load().iter() {
            world.broadcast_chat_message(
                message,
                &is_filtered,
                sender_player,
                chat_type,
                sender_name,
                target_name,
            );
        }
    }

    /// Gets the current difficulty of the server.
    pub fn get_difficulty(&self) -> Difficulty {
        self.level_info.load().difficulty
    }

    /// Sets the difficulty of the server.
    ///
    /// This function updates the difficulty level of the server and broadcasts the change to all players.
    /// It also iterates through all worlds to ensure the difficulty is applied consistently.
    /// If `force_update` is `Some(true)`, the difficulty will be set regardless of the current state.
    /// If `force_update` is `Some(false)` or `None`, the difficulty will only be updated if it is not locked.
    ///
    /// # Arguments
    ///
    /// * `difficulty`: The new difficulty level to set. This should be one of the variants of the `Difficulty` enum.
    /// * `force_update`: An optional boolean that, if set to `Some(true)`, forces the difficulty to be updated even if it is currently locked.
    ///
    /// # Note
    ///
    /// Mob spawn flags need no separate update here: every spawn query derives them
    /// from the live difficulty (`World::should_spawn_monsters`), like vanilla's
    /// `updateMobSpawningFlags`.
    pub fn set_difficulty(&self, difficulty: Difficulty, force_update: bool) {
        let current_info = self.level_info.load();
        if current_info.difficulty_locked && !force_update {
            return;
        }

        let new_difficulty = if self.basic_config.hardcore {
            Difficulty::Hard
        } else {
            difficulty
        };

        let mut new_info = (**current_info).clone();

        new_info.difficulty = new_difficulty;
        let locked = new_info.difficulty_locked;
        self.level_info.store(Arc::new(new_info));

        for world in self.worlds.load().iter() {
            world.set_difficulty(new_difficulty);
            world.broadcast_editioned(
                &CChangeDifficulty::new(new_difficulty as u8, locked),
                &pumpkin_protocol::bedrock::client::CSetDifficulty {
                    difficulty: (new_difficulty as u32).into(),
                },
            );
        }
    }

    /// Notifies clients about game rule changes that they mirror locally.
    pub fn on_game_rule_changed(&self, rule: &GameRule, value: &GameRuleValue<i64, bool>) {
        if *rule != GameRule::ImmediateRespawn {
            return;
        }

        let GameRuleValue::Bool(immediate_respawn) = value else {
            return;
        };
        // Vanilla's `IMMEDIATE_RESPAWN` event: 1 hides the client's death screen.
        let packet = CGameEvent::new(
            GameEvent::EnabledRespawnScreen,
            if *immediate_respawn { 1.0 } else { 0.0 },
        );
        let players = self.get_all_players();
        World::broadcast_java_clients(
            &packet,
            players.iter().filter_map(|player| {
                player
                    .client
                    .java()
                    .filter(|client| client.login_sent.load(Ordering::Relaxed))
            }),
        );
    }

    /// Sets the difficulty lock status of the server and broadcasts the update to all players.
    pub fn set_difficulty_locked(&self, locked: bool) {
        let current_info = self.level_info.load();
        let mut new_info = (**current_info).clone();
        new_info.difficulty_locked = locked;
        let difficulty = new_info.difficulty;
        self.level_info.store(Arc::new(new_info));

        for world in self.worlds.load().iter() {
            world.broadcast_editioned(
                &CChangeDifficulty::new(difficulty as u8, locked),
                &pumpkin_protocol::bedrock::client::CSetDifficulty {
                    difficulty: (difficulty as u32).into(),
                },
            );
        }
    }

    /// Searches for a player by their username across all worlds.
    ///
    /// This function iterates through each world managed by the server and attempts to find a player with the specified username.
    /// If a player is found in any world, it returns an `Arc<Player>` reference to that player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `name`: The username of the player to search for.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not found.
    pub fn get_player_by_name(&self, name: &str) -> Option<Arc<Player>> {
        for world in self.worlds.load().iter() {
            if let Some(player) = world.get_player_by_name(name) {
                return Some(player);
            }
        }
        None
    }

    pub fn get_players_by_ip(&self, ip: IpAddr) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in self.worlds.load().iter() {
            for player in world.players.load().iter() {
                if player.client.address().ip() == ip {
                    players.push(player.clone());
                }
            }
        }

        players
    }

    /// Returns all players from all worlds.
    pub fn get_all_players(&self) -> Vec<Arc<Player>> {
        let mut players = Vec::<Arc<Player>>::new();

        for world in self.worlds.load().iter() {
            players.extend(world.players.load().iter().cloned());
        }

        players
    }

    pub fn for_each_player<F>(&self, mut f: F)
    where
        F: FnMut(&Arc<Player>),
    {
        let worlds = self.worlds.load();

        for world in worlds.iter() {
            let players = world.players.load();
            for player in players.iter() {
                f(player);
            }
        }
    }

    /// Returns a random player from any of the worlds, or `None` if all worlds are empty.
    pub fn get_random_player(&self) -> Option<Arc<Player>> {
        let players = self.get_all_players();
        players.choose(&mut rand::rng()).map(Arc::<_>::clone)
    }

    /// Searches for a player by their UUID across all worlds.
    ///
    /// This function iterates through each world managed by the server and attempts to find a player with the specified UUID.
    /// If a player is found in any world, it returns an `Arc<Player>` reference to that player. Otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `id`: The UUID of the player to search for.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<Player>>` containing the player if found, or `None` if not found.
    pub fn get_player_by_uuid(&self, id: uuid::Uuid) -> Option<Arc<Player>> {
        for world in self.worlds.load().iter() {
            if let Some(player) = world.get_player_by_uuid(id) {
                return Some(player);
            }
        }
        None
    }

    /// Counts the total number of players across all worlds.
    ///
    /// This function iterates through each world and sums up the number of players currently connected to that world.
    ///
    /// # Returns
    ///
    /// The total number of players connected to the server.
    pub fn get_player_count(&self) -> usize {
        let mut count = 0;
        for world in self.worlds.load().iter() {
            count += world.players.load().len();
        }
        count
    }

    /// Similar to [`Server::get_player_count`] >= n, but may be more efficient since it stops its iteration through all worlds as soon as n players were found.
    pub fn has_n_players(&self, n: usize) -> bool {
        let mut count = 0;
        for world in self.worlds.load().iter() {
            count += world.players.load().len();
            if count >= n {
                return true;
            }
        }
        false
    }

    /// Returns the maximum number of players allowed on the server.
    #[must_use]
    pub const fn max_players(&self) -> u32 {
        self.advanced_config.networking.java.max_players
    }

    /// Starts the background telemetry task if enabled in configuration.
    pub fn start_telemetry(self: &Arc<Self>) {
        crate::telemetry::start_telemetry(self.clone());
    }

    /// Starts the background update checker task if enabled in configuration.
    pub fn start_update_checker(self: &Arc<Self>) {
        crate::update_checker::start_update_checker(self);
    }

    /// Notifies a joining player in chat if they are an operator and a Pumpkin update is available.
    pub fn notify_update_if_op(&self, player: &Player) {
        crate::update_checker::notify_update_if_op(self, player);
    }

    /// Generates a new container id.
    pub fn new_container_id(&self) -> u32 {
        self.container_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Generates a new map id.
    pub fn next_map_id(&self) -> i32 {
        let id = self.map_id.fetch_add(1, Ordering::SeqCst);
        self.level_info.rcu(|level_info| {
            let mut new_level_info = (**level_info).clone();
            new_level_info.map_id = self.map_id.load(Ordering::SeqCst);
            new_level_info
        });
        id
    }

    pub const fn get_branding(&self) -> CPluginMessage<'_> {
        self.branding.get_branding()
    }

    pub const fn get_status(&self) -> &std::sync::Mutex<CachedStatus> {
        &self.listing
    }

    async fn get_or_init_key_store(&self) -> Result<&Arc<KeyStore>, rsa::Error> {
        self.key_store
            .get_or_try_init(|| async {
                let (tx, rx) = tokio::sync::oneshot::channel();
                self.compute_pool.spawn(move || {
                    let _ = tx.send(KeyStore::new().map(Arc::new));
                });
                rx.await.unwrap_or_else(|_| KeyStore::new().map(Arc::new))
            })
            .await
    }

    pub async fn encryption_request<'a>(
        &'a self,
        verification_token: &'a [u8; 4],
        should_authenticate: bool,
    ) -> Result<CEncryptionRequest<'a>, rsa::Error> {
        Ok(self.get_or_init_key_store().await?.encryption_request(
            "",
            verification_token,
            should_authenticate,
        ))
    }

    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let key_store = self
            .get_or_init_key_store()
            .await
            .map_err(|err| {
                error!("Failed to load encryption keys: {err}");
                EncryptionError::FailedDecrypt
            })?
            .clone();
        let data = data.to_vec();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.compute_pool.spawn(move || {
            let _ = tx.send(key_store.decrypt(&data));
        });
        rx.await.map_err(|_| EncryptionError::FailedDecrypt)?
    }

    pub fn digest_secret(&self, secret: &[u8]) -> Result<String, rsa::Error> {
        self.key_store.get().map_or_else(
            || KeyStore::new().map(|key_store| key_store.get_digest(secret)),
            |key_store| Ok(key_store.get_digest(secret)),
        )
    }

    /// Vanilla `ServerCommonPacketListenerImpl.suspendFlushing`. Returns the suspended players.
    fn suspend_player_flushes(&self) -> Vec<Arc<Player>> {
        let mut suspended = Vec::new();
        self.for_each_player(|player| {
            if let Some(java) = player.client.java() {
                java.suspend_flushing();
                suspended.push(player.clone());
            }
        });
        suspended
    }

    /// Vanilla `resumeFlushing` / `Connection.flushChannel` at tick end (~50ms).
    /// Resumes the suspended set, not a fresh world scan
    /// off-tick respawn or world change can leave a player in no world's list at tick end.
    fn resume_player_flushes(suspended: &[Arc<Player>]) {
        for player in suspended {
            if let Some(java) = player.client.java() {
                java.resume_flushing();
            }
        }
    }

    /// Main server tick method. This now handles both player/network ticking (which always runs)
    /// and world/game logic ticking (which is affected by freeze state).
    pub fn tick(self: &Arc<Self>) {
        self.simulation_pool.install(|| {
            // Do not flush mid-tick; `Flush` is `flushChannel`.
            let suspended = self.suspend_player_flushes();
            // Paper runs the plugin scheduler/functions heartbeat every tick,
            // outside `runsNormally()`, so loader `on_tick` keeps running while
            // `/tick freeze` pauses world game logic. Datapack tick/load tags and
            // `/schedule` stay gated: ServerFunctionManager.tick gates the tags and
            // the per-level TimerQueue is ticked under ServerLevel.tick's `runs` flag.
            self.plugin_manager.tick_loaders(self);
            // Vanilla `tickChildren` is gated on `runsNormally()` alone: a sprint
            // clears the freeze, so `is_sprinting()` would wrongly revive a world
            // tick after `/tick freeze` is issued mid-sprint.
            if self.tick_rate_manager.runs_normally() {
                self.tick_worlds();
                // Always run player and network ticking, even when game is frozen
            } else {
                self.sync_game_time();
                self.tick_players_and_network();
            }
            self.flush_pending_block_updates();
            Self::resume_player_flushes(&suspended);
        });
    }

    fn flush_pending_block_updates(&self) {
        for world in self.worlds.load().iter() {
            world.flush_block_updates();
        }
    }

    /// Ticks essential server functions that must run even when the game is frozen.
    /// This includes player ticking (network, keep-alives) and flushing world updates to clients.
    pub fn tick_players_and_network(self: &Arc<Self>) {
        let worlds = self.worlds.load();

        for world in worlds.iter() {
            // Block updates are flushed once at tick end by `flush_pending_block_updates`.
            world.flush_synced_block_events();

            let players = world.players.load();
            players.par_iter().for_each(|player| {
                player.process_inbound_packets();
            });
            players.par_iter().for_each(|player| {
                #[cfg(profiler)]
                let _profile = pumpkin_profiler::cpu::span(pumpkin_profiler::cpu::Phase::Players);
                player.tick(self);
            });
        }
    }

    /// Ticks the game logic for all worlds. This is the part that is affected by `/tick freeze`.
    pub fn tick_worlds(self: &Arc<Self>) {
        #[cfg(profiler)]
        let functions_profile =
            pumpkin_profiler::cpu::span(pumpkin_profiler::cpu::Phase::Functions);
        let Some(source) = crate::command::CommandSender::Console.into_source(self) else {
            return;
        };
        let source = source.with_silent();
        // `#minecraft:load` runs from the tick, before the automatic tick functions.
        if self.datapack_manager.take_load_pending() {
            let _ = self
                .datapack_manager
                .execute_function(self, &source, "#minecraft:load");
        }
        let _ = self
            .datapack_manager
            .execute_function(self, &source, "#minecraft:tick");

        self.scheduled_functions.tick(
            self,
            self.tick_count.load(std::sync::atomic::Ordering::Relaxed) as u64,
        );
        #[cfg(profiler)]
        drop(functions_profile);

        self.sync_game_time();

        let worlds = self.worlds.load();
        worlds.par_iter().for_each(|world| {
            world.tick(self);
        });

        // Global tasks
        self.player_data_storage.tick(self);

        // Entities, scoreboard and command storage follow the world autosave.
        self.autosave_if_due(false);
    }

    /// Vanilla `MinecraftServer.autoSave`. Runs on the autosave tick cadence
    /// during normal ticks and once when pause-when-empty begins, so a long
    /// empty period cannot lose unsaved progress.
    pub(crate) fn autosave_if_due(self: &Arc<Self>, force: bool) {
        let worlds = self.worlds.load();
        let Some(world) = worlds.first() else {
            return;
        };
        if !world.level.save_enabled.load(Ordering::Relaxed) {
            return;
        }
        let autosave = world.level.autosave_ticks;
        let tick = u64::try_from(self.tick_count.load(Ordering::Relaxed)).unwrap_or(0);
        let due = autosave > 0 && tick > 0 && tick.is_multiple_of(autosave);
        if !force && !due {
            return;
        }
        for world in worlds.iter() {
            let world = world.clone();
            self.spawn_task(async move { world.autosave_entities().await });
        }
        let server = self.clone();
        self.spawn_task(async move {
            // level.dat holds the game time that scheduled_events.dat is relative to,
            // so both are saved together; otherwise a crash restores an old time and
            // pushes every scheduled function far into the future.
            let result = tokio::task::spawn_blocking(move || {
                if let Err(err) = server.save_world_info() {
                    error!("Failed to save level.dat: {err}");
                }
                server.save_persistent_data()
            })
            .await;
            if let Ok(Err(err)) = result {
                error!("{err}");
            }
        });
    }

    /// Vanilla `tickChildren` "timeSync": every 20 ticks, also while frozen.
    /// Sync before `tickTime()`. `tick_count` is advanced at the top of the tick
    /// loop, so it already holds the current vanilla tick.
    fn sync_game_time(&self) {
        if self.tick_count.load(Ordering::Relaxed) % 20 == 0
            && let Some(overworld) =
                self.worlds.load().iter().find(|world| {
                    world.dimension.minecraft_name == Dimension::OVERWORLD.minecraft_name
                })
        {
            overworld.force_game_time_synchronization(self);
        }
    }

    /// Updates the tick time statistics with the duration of the last tick.
    pub fn update_tick_times(&self, tick_duration_nanos: i64) {
        // `tick_count` is advanced at the top of the tick loop, so in-tick
        // readers see the current vanilla tick. The ring write stays
        // unconditional; a frozen tick reuses the last tick_count slot.
        let tick_count = self.tick_count.load(Ordering::Relaxed);
        let index = (tick_count % 100) as usize;

        let mut tick_times = self
            .tick_times_nanos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let old_time = tick_times[index];
        tick_times[index] = tick_duration_nanos;
        drop(tick_times);

        // Mirror vanilla's parallel `tickTimes5s`/`tickTimes10s`/`tickTimes60s`
        // rings; Paper derives its 1m/5m/15m TPS from the longer windows.
        {
            let mut ring = self
                .tick_times_1m_nanos
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            ring[(tick_count as usize) % TPS_SAMPLES_1M] = tick_duration_nanos;
        };
        {
            let mut ring = self
                .tick_times_5m_nanos
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            ring[(tick_count as usize) % TPS_SAMPLES_5M] = tick_duration_nanos;
        };
        {
            let mut ring = self
                .tick_times_15m_nanos
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            ring[(tick_count as usize) % TPS_SAMPLES_15M] = tick_duration_nanos;
        };

        self.aggregated_tick_times_nanos
            .fetch_add(tick_duration_nanos - old_time, Ordering::Relaxed);

        // Vanilla's `TICK_TIME` sample measures the raw wall-clock interval
        // between consecutive ticks (FULL_TICK) via `currentTime - lastTickNanos`,
        // not a synthetic `work.max(target)`.
        let now = std::time::Instant::now();
        let mut last_sample = self
            .last_tick_sample_instant
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let full_tick_nanos = last_sample
            .map(|previous| now.saturating_duration_since(previous).as_nanos() as i64)
            .unwrap_or(0);
        *last_sample = Some(now);
        drop(last_sample);

        let sample_slice = tick_time_sample(full_tick_nanos, tick_duration_nanos);
        let packet = pumpkin_protocol::java::client::play::CDebugSample::new(
            &sample_slice,
            pumpkin_protocol::codec::var_int::VarInt(0),
        );
        for world in self.worlds.load().iter() {
            let players = world.players.load();
            let recipients = players
                .iter()
                .filter(|player| {
                    player.subscribed_debug_sample.load(Ordering::Relaxed)
                        && player.permission_lvl.load() >= pumpkin_util::PermissionLvl::Two
                })
                .filter_map(|player| player.client.java());
            World::broadcast_java_clients(&packet, recipients);
        }
    }

    /// Gets the rolling average tick time over the last 100 ticks, in nanoseconds.
    pub fn get_average_tick_time_nanos(&self) -> i64 {
        let tick_count = self.tick_count.load(Ordering::Relaxed);
        let sample_size = (tick_count as usize).min(100);
        if sample_size == 0 {
            return 0;
        }
        self.aggregated_tick_times_nanos.load(Ordering::Relaxed) / sample_size as i64
    }

    /// Returns the average Milliseconds Per Tick (MSPT).
    pub fn get_mspt(&self) -> f64 {
        let avg_nanos = self.get_average_tick_time_nanos();
        // Convert nanoseconds to decimal milliseconds
        avg_nanos as f64 / 1_000_000.0
    }

    /// Returns the Ticks Per Second (TPS).
    ///
    /// Tick time measures work only, without the idle wait before the next deadline, so the
    /// rate from MSPT alone can exceed the target. The ticker never runs faster than the
    /// target rate, so cap there.
    pub fn get_tps(&self) -> f64 {
        let mspt = self.get_mspt();
        if mspt <= 0.0 {
            return 0.0;
        }
        let target = 1_000_000_000.0 / self.tick_rate_manager.nanoseconds_per_tick().max(1) as f64;
        (1000.0 / mspt).min(target)
    }

    /// Rolling TPS over the last 1, 5 and 15 minutes, in Paper's `getTPS()` order.
    pub fn get_recent_tps(&self) -> [f64; 3] {
        [self.get_tps_1m(), self.get_tps_5m(), self.get_tps_15m()]
    }

    /// TPS averaged over the last minute of ticks.
    pub fn get_tps_1m(&self) -> f64 {
        ring_tps(
            &self.tick_times_1m_nanos,
            self.tick_count.load(Ordering::Relaxed),
            self.tick_rate_manager.nanoseconds_per_tick(),
        )
    }

    /// TPS averaged over the last five minutes of ticks.
    pub fn get_tps_5m(&self) -> f64 {
        ring_tps(
            &self.tick_times_5m_nanos,
            self.tick_count.load(Ordering::Relaxed),
            self.tick_rate_manager.nanoseconds_per_tick(),
        )
    }

    /// TPS averaged over the last fifteen minutes of ticks.
    pub fn get_tps_15m(&self) -> f64 {
        ring_tps(
            &self.tick_times_15m_nanos,
            self.tick_count.load(Ordering::Relaxed),
            self.tick_rate_manager.nanoseconds_per_tick(),
        )
    }

    /// Returns a copy of the last 100 tick times.
    pub fn get_tick_times_nanos_copy(&self) -> [i64; 100] {
        *self
            .tick_times_nanos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub async fn register_service(self: &Arc<Self>, service_name: String) {
        let mut service_event =
            crate::plugin::api::events::server::service_register::ServiceRegisterEvent::new(
                service_name,
            );
        self.plugin_manager.fire(self, &mut service_event).await;
    }

    pub async fn unregister_service(self: &Arc<Self>, service_name: String) {
        let mut service_event =
            crate::plugin::api::events::server::service_unregister::ServiceUnregisterEvent::new(
                service_name,
            );
        self.plugin_manager.fire(self, &mut service_event).await;
    }

    /// Fires `TabCompleteEvent`. Returns `false` when a plugin cancelled the suggestions.
    #[must_use]
    pub fn tab_complete(self: &Arc<Self>, buffer: String, completions: Vec<String>) -> bool {
        let mut tab_event = crate::plugin::api::events::server::tab_complete::TabCompleteEvent::new(
            buffer,
            completions,
        );
        self.plugin_manager.fire_blocking(self, &mut tab_event);
        !tab_event.cancelled
    }

    pub async fn enable_plugin(self: &Arc<Self>, plugin_name: String) {
        let mut enable_event =
            crate::plugin::api::events::server::plugin_enable::PluginEnableEvent::new(plugin_name);
        self.plugin_manager.fire(self, &mut enable_event).await;
    }

    pub async fn disable_plugin(self: &Arc<Self>, plugin_name: String) {
        let mut disable_event =
            crate::plugin::api::events::server::plugin_disable::PluginDisableEvent::new(
                plugin_name,
            );
        self.plugin_manager.fire(self, &mut disable_event).await;
    }
}

/// Vanilla `ServerOpList#canBypassPlayerLimit`: the profile is listed in ops.json and that
/// entry has `bypassesPlayerLimit` set. A missing field defaults to false, matching vanilla's
/// ops.json parsing (`object.has("bypassesPlayerLimit") && getAsBoolean()`).
fn can_bypass_player_limit(ops: &crate::data::op::OperatorConfig, uuid: &uuid::Uuid) -> bool {
    ops.get_entry(uuid)
        .is_some_and(|op| op.bypasses_player_limit)
}

/// Join gamemode selection mirroring `ServerPlayer.load` / `getForcedGameType`.
fn join_gamemode(default: GameMode, forced: bool, nbt: Option<&NbtCompound>) -> GameMode {
    if forced {
        return default;
    }
    nbt.and_then(|nbt| {
        nbt.get_int("playerGameType")
            .or_else(|| nbt.get_byte("playerGameType").map(i32::from))
    })
    .and_then(|value| GameMode::try_from(value).ok())
    .unwrap_or(default)
}

/// Only old Bedrock saves lack `HasPlayedBefore`; infer it from a saved Pos.
fn infer_played_before(nbt: &NbtCompound) -> bool {
    nbt.get_bool("HasPlayedBefore").is_none() && has_saved_position(nbt)
}

/// Whether the save still holds a usable `Pos`.
fn has_saved_position(nbt: &NbtCompound) -> bool {
    nbt.get_list("Pos").is_some_and(|pos| pos.len() >= 3)
}

/// TPS from a rolling tick-time ring. Paper's `TickData.getTPSAverage` returns the
/// ideal rate until the window has samples, then `1e9 / average tick time`.
fn ring_tps(ring: &std::sync::Mutex<Box<[i64]>>, tick_count: i32, target_nanos: i64) -> f64 {
    let ideal = 1_000_000_000.0 / target_nanos.max(1) as f64;
    let times = ring
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let samples = (tick_count as usize).min(times.len());
    if samples == 0 {
        return ideal;
    }
    let total: i64 = times[..samples].iter().sum();
    if total <= 0 {
        return ideal;
    }
    (1_000_000_000.0 / (total as f64 / samples as f64)).min(ideal)
}

/// Mirrors the vanilla `TICK_TIME` debug sample layout:
/// `[FULL_TICK, TICK_SERVER_METHOD, SCHEDULED_TASKS, IDLE]`.
/// Pumpkin does not track scheduled-task time separately, so it is always zero
/// and IDLE is the remainder of the raw full tick after the tick work.
const fn tick_time_sample(full_tick_nanos: i64, tick_method_nanos: i64) -> [i64; 4] {
    let scheduled_tasks_nanos: i64 = 0;
    let remainder = full_tick_nanos
        .saturating_sub(tick_method_nanos)
        .saturating_sub(scheduled_tasks_nanos);
    let idle_nanos = if remainder < 0 { 0 } else { remainder };
    [
        full_tick_nanos,
        tick_method_nanos,
        scheduled_tasks_nanos,
        idle_nanos,
    ]
}

/// Whether MinecraftServer.tickCount advances for a tick. Vanilla increments it
/// inside the `runsNormally()` branch of tickServer; a sprint clears the freeze
/// so sprint ticks advance it too, while frozen (non-stepping) ticks do not.
const fn advances_tick_count(runs_normally: bool, is_sprinting: bool) -> bool {
    runs_normally || is_sprinting
}

#[cfg(test)]
mod tests {
    use super::{
        advances_tick_count, can_bypass_player_limit, infer_played_before, join_gamemode, ring_tps,
        tick_time_sample,
    };
    use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
    use pumpkin_util::GameMode;

    fn saved_gamemode(mode: GameMode) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_int("playerGameType", mode as i32);
        nbt
    }

    #[test]
    fn tick_time_sample_mirrors_vanilla_dimensions() {
        // Normal tick: FULL_TICK is the raw interval, IDLE is the remainder.
        assert_eq!(
            tick_time_sample(50_000_000, 15_000_000),
            [50_000_000, 15_000_000, 0, 35_000_000]
        );
        // Overloaded tick: no idle, full tick equals the work time.
        assert_eq!(
            tick_time_sample(80_000_000, 80_000_000),
            [80_000_000, 80_000_000, 0, 0]
        );
        // A work time larger than the measured interval cannot yield negative idle.
        assert_eq!(tick_time_sample(10, 30), [10, 30, 0, 0]);
    }

    #[test]
    fn ring_tps_uses_filled_samples_and_caps_at_target() {
        const TARGET: i64 = 50_000_000; // 20 TPS
        let ring = std::sync::Mutex::new(vec![0i64; 4].into_boxed_slice());
        // No samples yet: like Paper, report the ideal rate.
        assert_eq!(ring_tps(&ring, 0, TARGET), 20.0);
        // Only filled samples count; trailing zero slots are ignored.
        ring.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[0] = 50_000_000;
        assert_eq!(ring_tps(&ring, 1, TARGET), 20.0);
        ring.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[1] = 100_000_000;
        assert!((ring_tps(&ring, 2, TARGET) - 13.333333333333334).abs() < 1e-9);
        // A ring faster than the target is capped, as Paper's getTPS() does.
        let fast = std::sync::Mutex::new(vec![10_000_000i64; 4].into_boxed_slice());
        assert_eq!(ring_tps(&fast, 4, TARGET), 20.0);
    }

    #[test]
    fn player_limit_bypass_requires_op_with_flag() {
        use crate::data::op::OperatorConfig;
        use pumpkin_config::op::Op;
        use pumpkin_util::PermissionLvl;
        use uuid::Uuid;

        let uuid = Uuid::from_u128(1);
        let mut config = OperatorConfig::default();

        // Not an op: no bypass.
        assert!(!can_bypass_player_limit(&config, &uuid));

        // Op without bypassesPlayerLimit (vanilla ops.json default): no bypass.
        config
            .ops
            .push(Op::new(uuid, "op".to_owned(), PermissionLvl::Four, false));
        assert!(!can_bypass_player_limit(&config, &uuid));

        // Op with the flag: bypass allowed.
        config.ops[0].bypasses_player_limit = true;
        assert!(can_bypass_player_limit(&config, &uuid));

        // A different op with the flag does not grant this uuid a bypass.
        config.ops.push(Op::new(
            Uuid::from_u128(2),
            "op2".to_owned(),
            PermissionLvl::Four,
            true,
        ));
        assert!(!can_bypass_player_limit(&config, &Uuid::from_u128(3)));
    }

    #[test]
    fn join_gamemode_prefers_forced_then_saved_then_default() {
        let nbt = saved_gamemode(GameMode::Adventure);
        assert_eq!(
            join_gamemode(GameMode::Survival, false, Some(&nbt)),
            GameMode::Adventure
        );
        assert_eq!(
            join_gamemode(GameMode::Survival, true, Some(&nbt)),
            GameMode::Survival
        );
        assert_eq!(
            join_gamemode(GameMode::Creative, false, None),
            GameMode::Creative
        );
    }

    #[test]
    fn played_before_uses_tag_when_present() {
        let mut nbt = NbtCompound::new();
        nbt.put_bool("HasPlayedBefore", false);
        nbt.put_list(
            "Pos",
            vec![
                NbtTag::Double(1.0),
                NbtTag::Double(2.0),
                NbtTag::Double(3.0),
            ],
        );
        assert!(!infer_played_before(&nbt));

        nbt.put_bool("HasPlayedBefore", true);
        assert!(!infer_played_before(&nbt));
    }

    #[test]
    fn played_before_falls_back_to_pos_only_without_tag() {
        let mut nbt = NbtCompound::new();
        nbt.put_list(
            "Pos",
            vec![
                NbtTag::Double(1.0),
                NbtTag::Double(2.0),
                NbtTag::Double(3.0),
            ],
        );
        assert!(infer_played_before(&nbt));

        nbt.put_list("Pos", vec![NbtTag::Double(1.0)]);
        assert!(!infer_played_before(&nbt));
    }

    #[test]
    fn tick_count_advances_only_when_game_runs() {
        // Normal tick and step-forward both report runsNormally.
        assert!(advances_tick_count(true, false));
        // A sprint clears the freeze, so sprint ticks advance the count too.
        assert!(advances_tick_count(false, true));
        // Frozen and not sprinting: tick_count must stay put.
        assert!(!advances_tick_count(false, false));
    }
}
