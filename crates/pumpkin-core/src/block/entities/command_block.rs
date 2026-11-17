use std::sync::{
    Mutex as StdMutex,
    atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering},
};

use pumpkin_data::data_component_impl::{CustomNameImpl, DataComponentImpl};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;

use super::BlockEntity;

pub struct CommandBlockEntity {
    pub position: BlockPos,
    pub powered: AtomicBool,
    pub condition_met: AtomicBool,
    pub auto: AtomicBool,
    pub dirty: AtomicBool,
    pub command: StdMutex<String>,
    pub custom_name: StdMutex<Option<TextComponent>>,
    pub last_output: StdMutex<Option<TextComponent>>,
    pub track_output: AtomicBool,
    pub success_count: AtomicU32,
    pub last_execution: AtomicI64,
    pub update_last_execution: AtomicBool,
}

impl CommandBlockEntity {
    pub const ID: &'static str = "minecraft:command_block";
    #[must_use]
    pub const fn new(position: BlockPos, track_output: bool, is_chain: bool) -> Self {
        Self {
            position,
            powered: AtomicBool::new(false),
            condition_met: AtomicBool::new(false),
            auto: AtomicBool::new(is_chain),
            dirty: AtomicBool::new(false),
            command: StdMutex::new(String::new()),
            custom_name: StdMutex::new(None),
            last_output: StdMutex::new(None),
            track_output: AtomicBool::new(track_output),
            success_count: AtomicU32::new(0),
            last_execution: AtomicI64::new(-1),
            update_last_execution: AtomicBool::new(true),
        }
    }

    /// Returns the last output component, or `None` when nothing was recorded.
    #[must_use]
    pub fn get_last_output(&self) -> Option<TextComponent> {
        self.last_output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn write_sync_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("auto", self.auto.load(Ordering::SeqCst));
        nbt.put_string(
            "Command",
            self.command
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .to_string(),
        );
        if let Some(name) = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            // Vanilla BaseCommandBlock saves CustomName via ComponentSerialization.CODEC.
            nbt.put("CustomName", CustomNameImpl { name }.write_data());
        }
        nbt.put_bool("conditionMet", self.condition_met.load(Ordering::SeqCst));
        // Vanilla BaseCommandBlock saves LastOutput as a text-component JSON via
        // ComponentSerialization.CODEC, and only when TrackOutput is enabled.
        if self.track_output.load(Ordering::SeqCst)
            && let Some(last_output) = self
                .last_output
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
        {
            nbt.put(
                "LastOutput",
                last_output.to_nbt_tag_for_version(&JavaMinecraftVersion::V_26_3),
            );
        }
        nbt.put_bool("powered", self.powered.load(Ordering::SeqCst));
        nbt.put_bool("TrackOutput", self.track_output.load(Ordering::SeqCst));
        let update_last_execution = self.update_last_execution.load(Ordering::SeqCst);
        nbt.put_bool("UpdateLastExecution", update_last_execution);
        if update_last_execution {
            let last_execution = self.last_execution.load(Ordering::SeqCst);
            if last_execution != -1 {
                nbt.put_long("LastExecution", last_execution);
            }
        }
        nbt.put_int(
            "SuccessCount",
            self.success_count.load(Ordering::SeqCst).cast_signed(),
        );
    }
}

impl BlockEntity for CommandBlockEntity {
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
        let condition_met = AtomicBool::new(nbt.get_bool("conditionMet").unwrap_or(false));
        let auto = AtomicBool::new(nbt.get_bool("auto").unwrap_or(false));
        let powered = AtomicBool::new(nbt.get_bool("powered").unwrap_or(false));
        let command = StdMutex::new(nbt.get_string("Command").unwrap_or("").to_string());
        let custom_name = nbt
            .get("CustomName")
            .and_then(CustomNameImpl::read_data)
            .map(|c| c.name);
        let track_output_val = nbt.get_bool("TrackOutput").unwrap_or(true);
        let last_output = if track_output_val {
            nbt.get("LastOutput").map(TextComponent::from_nbt)
        } else {
            None
        };
        let last_output = StdMutex::new(last_output);
        let track_output = AtomicBool::new(track_output_val);
        let success_count =
            AtomicU32::new(nbt.get_int("SuccessCount").unwrap_or(0).cast_unsigned());
        let update_last_execution =
            AtomicBool::new(nbt.get_bool("UpdateLastExecution").unwrap_or(true));
        let last_execution = AtomicI64::new(if update_last_execution.load(Ordering::Relaxed) {
            nbt.get_long("LastExecution").unwrap_or(-1)
        } else {
            -1
        });

        Self {
            position,
            condition_met,
            auto,
            powered,
            command,
            custom_name: StdMutex::new(custom_name),
            last_output,
            track_output,
            success_count,
            last_execution,
            update_last_execution,
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_sync_nbt(nbt);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_sync_nbt(&mut nbt);
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let conditional =
            pumpkin_data::block_properties::CommandBlockProperties::from_state_id(state_id)
                .conditional;
        let command = self
            .command
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let last_output = self
            .get_last_output()
            .map_or_else(String::new, TextComponent::get_text);
        let custom_name = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .map(TextComponent::get_text);
        Some(build_command_block_actor(
            self.position,
            &command,
            custom_name.as_deref(),
            self.auto.load(Ordering::Relaxed),
            self.condition_met.load(Ordering::Relaxed),
            conditional,
            self.powered.load(Ordering::Relaxed),
            self.success_count.load(Ordering::Relaxed),
            self.track_output.load(Ordering::Relaxed),
            &last_output,
        ))
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn on_nbt_loaded(
        &self,
        world: &std::sync::Arc<crate::world::World>,
        previous: Option<&dyn BlockEntity>,
    ) {
        let previous_auto = previous
            .and_then(|prev| prev.as_any().downcast_ref::<Self>())
            .is_some_and(|prev| prev.auto.load(Ordering::Relaxed));
        crate::block::blocks::command::CommandBlock::schedule_auto_if_needed(
            world,
            self,
            &self.position,
            previous_auto,
        );
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Builds the Bedrock block-actor NBT for command blocks.
#[expect(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
#[must_use]
pub fn build_command_block_actor(
    position: BlockPos,
    command: &str,
    custom_name: Option<&str>,
    auto: bool,
    condition_met: bool,
    conditional: bool,
    powered: bool,
    success_count: u32,
    track_output: bool,
    last_output: &str,
) -> NbtCompound {
    super::bedrock::BedrockBlockActorBuilder::new("CommandBlock", position)
        .movable(false)
        .custom_name(custom_name)
        .put_string("Command", command)
        .put_bool("auto", auto)
        .put_bool("conditionMet", condition_met)
        .put_bool("conditionalMode", conditional)
        .put_bool("powered", powered)
        .put_int("SuccessCount", success_count as i32)
        .put_bool("TrackOutput", track_output)
        .put_string("LastOutput", last_output)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_block_actor_builder() {
        let pos = BlockPos::new(6, 64, 6);
        let nbt = build_command_block_actor(
            pos,
            "say hi",
            Some("Cmd"),
            true,
            true,
            false,
            false,
            1,
            true,
            "hi",
        );
        assert_eq!(nbt.get_string("id"), Some("CommandBlock"));
        assert_eq!(nbt.get_string("Command"), Some("say hi"));
        assert_eq!(nbt.get_string("CustomName"), Some("Cmd"));
        assert_eq!(nbt.get_bool("auto"), Some(true));
        assert_eq!(nbt.get_bool("conditionMet"), Some(true));
        assert_eq!(nbt.get_bool("conditionalMode"), Some(false));
        assert_eq!(nbt.get_bool("powered"), Some(false));
        assert_eq!(nbt.get_int("SuccessCount"), Some(1));
        assert_eq!(nbt.get_bool("isMovable"), Some(false));
    }

    #[test]
    fn command_block_custom_name_nbt_round_trip() {
        let pos = BlockPos::new(6, 64, 6);
        let entity = CommandBlockEntity::new(pos, true, false);
        *entity
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(TextComponent::text("Cmd"));

        let mut nbt = NbtCompound::new();
        entity.write_sync_nbt(&mut nbt);
        assert!(nbt.get("CustomName").is_some());

        let restored = CommandBlockEntity::from_nbt(&nbt, pos);
        let name = restored
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert_eq!(name.map(TextComponent::get_text), Some("Cmd".to_string()));
    }

    #[test]
    fn command_block_last_execution_nbt_round_trip() {
        let pos = BlockPos::new(6, 64, 6);
        let entity = CommandBlockEntity::new(pos, true, false);
        entity.last_execution.store(1234, Ordering::Relaxed);

        let mut nbt = NbtCompound::new();
        entity.write_sync_nbt(&mut nbt);
        assert_eq!(nbt.get_long("LastExecution"), Some(1234));
        assert_eq!(nbt.get_bool("UpdateLastExecution"), Some(true));

        let restored = CommandBlockEntity::from_nbt(&nbt, pos);
        assert_eq!(restored.last_execution.load(Ordering::Relaxed), 1234);
        assert!(restored.update_last_execution.load(Ordering::Relaxed));

        // With UpdateLastExecution=false vanilla neither saves nor restores LastExecution.
        let mut suppressed = CommandBlockEntity::new(pos, true, false);
        suppressed
            .update_last_execution
            .store(false, Ordering::Relaxed);
        suppressed.last_execution.store(99, Ordering::Relaxed);
        let mut nbt = NbtCompound::new();
        suppressed.write_sync_nbt(&mut nbt);
        assert_eq!(nbt.get_bool("UpdateLastExecution"), Some(false));
        assert!(nbt.get("LastExecution").is_none());
        let restored = CommandBlockEntity::from_nbt(&nbt, pos);
        assert_eq!(restored.last_execution.load(Ordering::Relaxed), -1);
    }

    #[test]
    fn command_block_last_output_nbt_round_trip() {
        let pos = BlockPos::new(6, 64, 6);
        let entity = CommandBlockEntity::new(pos, true, false);
        *entity
            .last_output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(TextComponent::text("[12:34:56] ").add_child(TextComponent::text("hello")));

        let mut nbt = NbtCompound::new();
        entity.write_sync_nbt(&mut nbt);
        assert!(nbt.get("LastOutput").is_some());

        let restored = CommandBlockEntity::from_nbt(&nbt, pos);
        assert_eq!(
            restored.get_last_output().map(TextComponent::get_text),
            Some("[12:34:56] hello".to_string())
        );
    }
}
