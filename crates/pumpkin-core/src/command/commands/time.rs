use pumpkin_data::environment_attribute::DayTimeline;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::argument_types::registry_id::RegistryIdArgumentType;
use crate::command::argument_types::time::TimeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::world::time::default_clock_name;
use pumpkin_util::identifier::Identifier;

const DESCRIPTION: &str = "Query or modify the world time and clocks.";
const PERMISSION: &str = "minecraft:command.time";

const DEFAULT_CLOCK: &str = "minecraft:overworld";
const DAY_TIMELINE: &str = "minecraft:day";

// Bedrock has no dedicated keys for these errors, so both editions use the Java text.
const ERROR_SET_ALREADY_AT_TIME: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_TIME_SET_ALREADY_AT_TIME,
    translation::java::COMMANDS_TIME_SET_ALREADY_AT_TIME,
);
const ERROR_PAUSE_ALREADY_PAUSED: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_TIME_PAUSE_ALREADY_PAUSED,
    translation::java::COMMANDS_TIME_PAUSE_ALREADY_PAUSED,
);
const ERROR_RESUME_ALREADY_RUNNING: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_TIME_PAUSE_ALREADY_RUNNING,
    translation::java::COMMANDS_TIME_PAUSE_ALREADY_RUNNING,
);
const ERROR_RATE_ALREADY_SAME: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_TIME_RATE_ALREADY_SAME,
    translation::java::COMMANDS_TIME_RATE_ALREADY_SAME,
);
// Bedrock has no time-marker keys, so both editions use the Java text.
const ERROR_NO_TIME_MARKER_FOUND: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_TIME_NO_TIME_MARKER_FOUND,
    translation::java::COMMANDS_TIME_NO_TIME_MARKER_FOUND,
);
const ERROR_ALREADY_AT_TIME_MARKER: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_TIME_SET_ALREADY_AT_TIME_MARKER,
    translation::java::COMMANDS_TIME_SET_ALREADY_AT_TIME_MARKER,
);
// Bedrock has no time/no-default-clock key, so both editions use the Java text.
const ERROR_NO_DEFAULT_CLOCK: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_TIME_NO_DEFAULT_CLOCK,
    translation::java::COMMANDS_TIME_NO_DEFAULT_CLOCK,
);

// Vanilla resolves `/time of <clock>` through ResourceArgument.resource(WORLD_CLOCK); an
// unknown id fails before any mutation. Bedrock's message uses only the id and ignores
// the extra registry argument that Java needs.
const ERROR_UNKNOWN_CLOCK: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_RESOURCE_NOT_FOUND,
    translation::bedrock::COMMANDS_TIME_CLOCK_FAILED,
);

// The vanilla `minecraft:day` timeline registry entry sets `clock` to
// `minecraft:overworld`, so a query through any other clock fails with this error.
// Bedrock has no matching key, so both editions use the Java text.
const ERROR_WRONG_TIMELINE_FOR_CLOCK: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_TIME_WRONG_TIMELINE_FOR_CLOCK,
    translation::java::COMMANDS_TIME_WRONG_TIMELINE_FOR_CLOCK,
);

#[derive(Clone, Copy)]
enum PresetTime {
    Day,
    Noon,
    Night,
    Midnight,
}

impl PresetTime {
    const fn to_ticks(self) -> i32 {
        match self {
            Self::Day => 1000,
            Self::Noon => 6000,
            Self::Night => 13000,
            Self::Midnight => 18000,
        }
    }
}

#[derive(Clone, Copy)]
enum Action {
    Set(Option<PresetTime>),
    Add,
    Pause,
    Resume,
    Rate,
}

#[derive(Clone, Copy)]
enum QueryMode {
    Time,
    GameTime,
    DayTime,
    Day,
}

const fn wrap_time(ticks: i64) -> i32 {
    (ticks % 2_147_483_647) as i32
}

/// True when the source is a Bedrock client, which translates its own time messages.
fn is_bedrock(context: &CommandContext) -> bool {
    context.source.output.as_player().is_some_and(|player| {
        matches!(
            player.client.as_ref(),
            crate::net::ClientPlatform::Bedrock(_)
        )
    })
}

/// The Java 26.3 client no longer has `commands.time.query`, so Java gets the text vanilla
/// prints for `/time query minecraft:day` (and `... repetitions`). Bedrock keeps its own
/// one-argument keys.
fn day_timeline_feedback(
    context: &CommandContext,
    java_key: &'static str,
    bedrock_key: &'static str,
    value: i64,
) -> TextComponent {
    let value = TextComponent::text(value.to_string());
    if is_bedrock(context) {
        TextComponent::translate_cross(java_key, bedrock_key, [value])
    } else {
        TextComponent::translate_cross(
            java_key,
            bedrock_key,
            [TextComponent::text(DAY_TIMELINE), value],
        )
    }
}

/// Resolves the `of <clock>` id against the `world_clock` registry. Without this check a
/// bogus id silently edited the current world's single `LevelTime`.
fn resolve_clock_name(
    context: &CommandContext,
    has_clock: bool,
) -> Result<String, CommandSyntaxError> {
    let name = if has_clock {
        IdentifierArgumentType::get(context, "clock")
            .map_or_else(|_| DEFAULT_CLOCK.to_string(), |id| id.to_string())
    } else {
        let world = context.source.world();
        // Vanilla getDefaultClock uses the dimension type's own default_clock:
        // the End clock in the End, and an error in the Nether.
        default_clock_name(world.dimension.minecraft_name)
            .map(std::string::ToString::to_string)
            .ok_or_else(|| {
                ERROR_NO_DEFAULT_CLOCK.create_without_context_args_slice(&[TextComponent::text(
                    world.dimension.minecraft_name,
                )])
            })?
    };
    if has_clock
        && !context
            .source
            .server()
            .datapack_manager
            .has_dynamic_registry_entry("minecraft:world_clock", &name)
    {
        return Err(ERROR_UNKNOWN_CLOCK.create_without_context_args_slice(&[
            TextComponent::text(name),
            TextComponent::text("minecraft:world_clock"),
        ]));
    }
    Ok(name)
}

/// Vanilla `queryTimelineTicks`/`queryTimelineRepetitions` reject a timeline whose
/// `clock` field differs from the requested clock. Pumpkin only models the
/// `minecraft:day` timeline, whose registry NBT sets `clock` to `minecraft:overworld`.
fn ensure_day_timeline_clock(clock_name: &str) -> Result<(), CommandSyntaxError> {
    if clock_name == DEFAULT_CLOCK {
        Ok(())
    } else {
        Err(
            ERROR_WRONG_TIMELINE_FOR_CLOCK.create_without_context_args_slice(&[
                TextComponent::text(DAY_TIMELINE),
                TextComponent::text(clock_name.to_string()),
            ]),
        )
    }
}

struct QueryExecutor {
    mode: QueryMode,
    has_clock: bool,
}

impl QueryExecutor {
    /// Reads a non-default clock from `LevelTime.clocks` instead of the world's
    /// own day time, matching vanilla `TimeCommand.query` with `of <clock>`.
    fn execute_other_clock(
        &self,
        context: &CommandContext,
        world: &crate::world::World,
        clock_name: &str,
    ) -> CommandExecutorResult {
        let guard = world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let total_ticks = guard
            .clocks
            .clock(clock_name)
            .map_or(0, |clock| clock.total_ticks);
        let game_time = guard.world_age;
        drop(guard);

        match self.mode {
            QueryMode::GameTime => {
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_QUERY_GAMETIME,
                        translation::bedrock::COMMANDS_TIME_QUERY_GAMETIME,
                        TextComponent::text(game_time.to_string())
                    ),
                    false,
                );
                Ok(wrap_time(game_time))
            }
            QueryMode::Time => {
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_QUERY_ABSOLUTE,
                        translation::bedrock::COMMANDS_TIME_CLOCK_QUERY_TIME,
                        TextComponent::text(clock_name.to_string()),
                        TextComponent::text(total_ticks.to_string())
                    ),
                    false,
                );
                Ok(wrap_time(total_ticks))
            }
            QueryMode::DayTime => {
                ensure_day_timeline_clock(clock_name)?;
                let curr_time = total_ticks % 24000;
                context.source.send_feedback(
                    day_timeline_feedback(
                        context,
                        translation::java::COMMANDS_TIME_QUERY_TIMELINE,
                        translation::bedrock::COMMANDS_TIME_QUERY_DAYTIME,
                        curr_time,
                    ),
                    false,
                );
                Ok(wrap_time(curr_time))
            }
            QueryMode::Day => {
                ensure_day_timeline_clock(clock_name)?;
                let curr_time = total_ticks / 24000;
                context.source.send_feedback(
                    day_timeline_feedback(
                        context,
                        translation::java::COMMANDS_TIME_QUERY_TIMELINE_REPETITIONS,
                        translation::bedrock::COMMANDS_TIME_QUERY_DAY,
                        curr_time,
                    ),
                    false,
                );
                Ok(curr_time as i32)
            }
        }
    }
}

impl CommandExecutor for QueryExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let clock_name = resolve_clock_name(context, self.has_clock)?;

        let world = context.source.world();
        if self.has_clock
            && clock_name
                != default_clock_name(world.dimension.minecraft_name).unwrap_or(DEFAULT_CLOCK)
        {
            return self.execute_other_clock(context, world, &clock_name);
        }
        let level_time = world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();

        match self.mode {
            QueryMode::GameTime => {
                let game_time = level_time.query_gametime();
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_QUERY_GAMETIME,
                        translation::bedrock::COMMANDS_TIME_QUERY_GAMETIME,
                        TextComponent::text(game_time.to_string())
                    ),
                    false,
                );
                Ok(wrap_time(game_time))
            }
            QueryMode::Time => {
                let total_ticks = level_time.time_of_day;
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_QUERY_ABSOLUTE,
                        translation::bedrock::COMMANDS_TIME_CLOCK_QUERY_TIME,
                        TextComponent::text(clock_name),
                        TextComponent::text(total_ticks.to_string())
                    ),
                    false,
                );
                Ok(wrap_time(total_ticks))
            }
            QueryMode::DayTime => {
                ensure_day_timeline_clock(&clock_name)?;
                let curr_time = level_time.query_daytime();
                context.source.send_feedback(
                    day_timeline_feedback(
                        context,
                        translation::java::COMMANDS_TIME_QUERY_TIMELINE,
                        translation::bedrock::COMMANDS_TIME_QUERY_DAYTIME,
                        curr_time,
                    ),
                    false,
                );
                Ok(wrap_time(curr_time))
            }
            QueryMode::Day => {
                ensure_day_timeline_clock(&clock_name)?;
                let curr_time = level_time.query_day();
                context.source.send_feedback(
                    day_timeline_feedback(
                        context,
                        translation::java::COMMANDS_TIME_QUERY_TIMELINE_REPETITIONS,
                        translation::bedrock::COMMANDS_TIME_QUERY_DAY,
                        curr_time,
                    ),
                    false,
                );
                Ok(curr_time as i32)
            }
        }
    }
}

struct ActionExecutor {
    action: Action,
    has_clock: bool,
}

impl ActionExecutor {
    /// Applies an `/time of <clock>` action to a non-default clock instance,
    /// leaving the world's own day time untouched.
    #[allow(clippy::too_many_lines)]
    fn execute_other_clock(
        &self,
        context: &CommandContext,
        world: &crate::world::World,
        clock_name: &str,
    ) -> CommandExecutorResult {
        let (total_ticks, feedback) = {
            let mut guard = world
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let clock = guard.clocks.clock_mut(clock_name);
            match self.action {
                Action::Set(preset) => {
                    let time = i64::from(match preset {
                        Some(p) => p.to_ticks(),
                        None => TimeArgumentType::get(context, "time")?,
                    });
                    if clock.total_ticks == time {
                        return Err(ERROR_SET_ALREADY_AT_TIME.create_without_context_args_slice(
                            &[
                                TextComponent::text(clock_name.to_string()),
                                TextComponent::text(time.to_string()),
                            ],
                        ));
                    }
                    clock.set_total_ticks(time);
                    (
                        time,
                        pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_SET_ABSOLUTE,
                            translation::bedrock::COMMANDS_TIME_CLOCK_SET,
                            TextComponent::text(clock_name.to_string()),
                            TextComponent::text(time.to_string())
                        ),
                    )
                }
                Action::Add => {
                    let amount = i64::from(TimeArgumentType::get(context, "time")?);
                    clock.total_ticks = (clock.total_ticks + amount).max(0);
                    let total = clock.total_ticks;
                    let feedback = if is_bedrock(context) {
                        TextComponent::translate(
                            translation::bedrock::COMMANDS_TIME_CLOCK_ADDED,
                            [
                                TextComponent::text(amount.to_string()),
                                TextComponent::text(clock_name.to_string()),
                                TextComponent::text(total.to_string()),
                            ],
                        )
                    } else {
                        TextComponent::translate(
                            translation::java::COMMANDS_TIME_SET_ABSOLUTE,
                            [
                                TextComponent::text(clock_name.to_string()),
                                TextComponent::text(total.to_string()),
                            ],
                        )
                    };
                    (total, feedback)
                }
                Action::Pause => {
                    if clock.paused {
                        return Err(
                            ERROR_PAUSE_ALREADY_PAUSED.create_without_context_args_slice(&[
                                TextComponent::text(clock_name.to_string()),
                            ]),
                        );
                    }
                    clock.set_paused(true);
                    let total = clock.total_ticks;
                    (
                        total,
                        pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_PAUSE,
                            translation::bedrock::COMMANDS_TIME_CLOCK_PAUSED,
                            TextComponent::text(clock_name.to_string()),
                            TextComponent::text(total.to_string())
                        ),
                    )
                }
                Action::Resume => {
                    if !clock.paused {
                        return Err(ERROR_RESUME_ALREADY_RUNNING
                            .create_without_context_args_slice(&[TextComponent::text(
                                clock_name.to_string(),
                            )]));
                    }
                    clock.set_paused(false);
                    let total = clock.total_ticks;
                    (
                        total,
                        pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_RESUME,
                            translation::bedrock::COMMANDS_TIME_CLOCK_RESUMED,
                            TextComponent::text(clock_name.to_string()),
                            TextComponent::text(total.to_string())
                        ),
                    )
                }
                Action::Rate => {
                    let rate = FloatArgumentType::get(context, "rate")?;
                    if (clock.rate - rate).abs() < f32::EPSILON {
                        return Err(ERROR_RATE_ALREADY_SAME.create_without_context_args_slice(&[
                            TextComponent::text(clock_name.to_string()),
                            TextComponent::text(super::data::java_float_to_string(rate)),
                        ]));
                    }
                    clock.set_rate(rate);
                    let feedback = if is_bedrock(context) {
                        TextComponent::translate(
                            translation::bedrock::COMMANDS_TIME_SET,
                            [TextComponent::text(super::data::java_float_to_string(rate))],
                        )
                    } else {
                        TextComponent::translate(
                            translation::java::COMMANDS_TIME_RATE,
                            [
                                TextComponent::text(clock_name.to_string()),
                                TextComponent::text(super::data::java_float_to_string(rate)),
                            ],
                        )
                    };
                    (clock.total_ticks, feedback)
                }
            }
        };
        context.source.send_feedback(feedback, true);
        match self.action {
            Action::Set(_) | Action::Add => Ok(wrap_time(total_ticks)),
            Action::Pause | Action::Resume | Action::Rate => Ok(1),
        }
    }
}

impl CommandExecutor for ActionExecutor {
    #[allow(clippy::too_many_lines)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let clock_name = resolve_clock_name(context, self.has_clock)?;

        let world = context.source.world();
        if self.has_clock
            && clock_name
                != default_clock_name(world.dimension.minecraft_name).unwrap_or(DEFAULT_CLOCK)
        {
            return self.execute_other_clock(context, world, &clock_name);
        }

        match self.action {
            Action::Set(preset) => {
                let time_count = if let Some(p) = preset {
                    p.to_ticks()
                } else {
                    TimeArgumentType::get(context, "time")?
                };
                let level_time = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if guard.time_of_day == i64::from(time_count) {
                        return Err(ERROR_SET_ALREADY_AT_TIME.create_without_context_args_slice(
                            &[
                                TextComponent::text(clock_name),
                                TextComponent::text(time_count.to_string()),
                            ],
                        ));
                    }
                    guard.set_time(time_count.into());
                    guard.clone()
                };
                level_time.send_time(world);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_SET_ABSOLUTE,
                        translation::bedrock::COMMANDS_TIME_CLOCK_SET,
                        TextComponent::text(clock_name),
                        TextComponent::text(time_count.to_string())
                    ),
                    true,
                );
                Ok(time_count)
            }
            Action::Add => {
                let time_count = TimeArgumentType::get(context, "time")?;
                let (level_time, total_ticks) = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.add_time(time_count.into());
                    let total_ticks = guard.time_of_day;
                    (guard.clone(), total_ticks)
                };
                level_time.send_time(world);
                let feedback = if is_bedrock(context) {
                    TextComponent::translate(
                        translation::bedrock::COMMANDS_TIME_CLOCK_ADDED,
                        [
                            TextComponent::text(time_count.to_string()),
                            TextComponent::text(clock_name),
                            TextComponent::text(total_ticks.to_string()),
                        ],
                    )
                } else {
                    TextComponent::translate(
                        translation::java::COMMANDS_TIME_SET_ABSOLUTE,
                        [
                            TextComponent::text(clock_name),
                            TextComponent::text(total_ticks.to_string()),
                        ],
                    )
                };
                context.source.send_feedback(feedback, true);
                Ok(wrap_time(total_ticks))
            }
            Action::Pause => {
                let level_time = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if guard.paused {
                        return Err(
                            ERROR_PAUSE_ALREADY_PAUSED.create_without_context_args_slice(&[
                                TextComponent::text(clock_name),
                            ]),
                        );
                    }
                    guard.set_paused(true);
                    guard.clone()
                };
                level_time.send_time(world);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_PAUSE,
                        translation::bedrock::COMMANDS_TIME_CLOCK_PAUSED,
                        TextComponent::text(clock_name),
                        TextComponent::text(level_time.time_of_day.to_string())
                    ),
                    true,
                );
                Ok(1)
            }
            Action::Resume => {
                let level_time = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if !guard.paused {
                        return Err(ERROR_RESUME_ALREADY_RUNNING
                            .create_without_context_args_slice(&[TextComponent::text(
                                clock_name,
                            )]));
                    }
                    guard.set_paused(false);
                    guard.clone()
                };
                level_time.send_time(world);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_RESUME,
                        translation::bedrock::COMMANDS_TIME_CLOCK_RESUMED,
                        TextComponent::text(clock_name),
                        TextComponent::text(level_time.time_of_day.to_string())
                    ),
                    true,
                );
                Ok(1)
            }
            Action::Rate => {
                let rate = FloatArgumentType::get(context, "rate")?;
                let level_time = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if (guard.rate - rate).abs() < f32::EPSILON {
                        return Err(ERROR_RATE_ALREADY_SAME.create_without_context_args_slice(&[
                            TextComponent::text(clock_name),
                            TextComponent::text(super::data::java_float_to_string(rate)),
                        ]));
                    }
                    guard.set_rate(rate);
                    guard.clone()
                };
                level_time.send_time(world);
                let feedback = if is_bedrock(context) {
                    TextComponent::translate(
                        translation::bedrock::COMMANDS_TIME_SET,
                        [TextComponent::text(super::data::java_float_to_string(rate))],
                    )
                } else {
                    TextComponent::translate(
                        translation::java::COMMANDS_TIME_RATE,
                        [
                            TextComponent::text(clock_name),
                            TextComponent::text(super::data::java_float_to_string(rate)),
                        ],
                    )
                };
                context.source.send_feedback(feedback, true);
                Ok(1)
            }
        }
    }
}

/// Vanilla `ClockTimeMarker.occursAt`. The day timeline markers repeat every
/// `period` ticks, so a marker matches at the same offset in any day.
const fn marker_occurs_at(total_ticks: i64, ticks: i64, period: i64) -> bool {
    ticks == total_ticks.rem_euclid(period)
}

/// Vanilla `ClockTimeMarker.resolveTimeToMoveTo`: advance forward to the next
/// occurrence without rewinding the clock.
const fn resolve_time_to_move_to(total_ticks: i64, ticks: i64, period: i64) -> i64 {
    let from = total_ticks.rem_euclid(period);
    let duration = ticks - from;
    if duration > 0 {
        total_ticks + duration
    } else {
        total_ticks + period + duration
    }
}

/// The vanilla `clock_time_marker` entries the `minecraft:day` timeline
/// attaches to the overworld clock. Datapack-defined markers are not loaded,
/// so any other id is reported as missing.
fn builtin_time_marker(id: &Identifier, clock: &str) -> Option<(i64, i64)> {
    if clock != DEFAULT_CLOCK || id.namespace() != "minecraft" {
        return None;
    }
    let period = i64::from(DayTimeline::PERIOD_TICKS);
    let ticks = match id.path() {
        "day" => DayTimeline::DAY_TIME_MARKER,
        "noon" => DayTimeline::NOON_TIME_MARKER,
        "night" => DayTimeline::NIGHT_TIME_MARKER,
        "midnight" => DayTimeline::MIDNIGHT_TIME_MARKER,
        "wake_up_from_sleep" => DayTimeline::WAKE_UP_FROM_SLEEP_TIME_MARKER,
        "roll_village_siege" => DayTimeline::ROLL_VILLAGE_SIEGE_TIME_MARKER,
        _ => return None,
    };
    Some((i64::from(ticks), period))
}

/// Reads one time marker from the `minecraft:day` timeline registry NBT. The
/// value is either a plain int or a compound with `ticks`, matching vanilla's
/// `Timeline.TimeMarkerInfo` codec. `period_ticks` on the root is shared by
/// every marker on the timeline.
fn day_timeline_marker(bytes: &[u8], marker_id: &str) -> Option<(i64, i64)> {
    use std::io::Cursor;

    use pumpkin_nbt::Nbt;

    let mut cursor = Cursor::new(bytes);
    let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
        pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
    );
    let root = Nbt::read_unnamed(&mut reader).ok()?.root_tag;
    let period = i64::from(root.get_int("period_ticks")?);
    let markers = root.get_compound("time_markers")?;
    let ticks = i64::from(match markers.get(marker_id)? {
        pumpkin_nbt::tag::NbtTag::Int(ticks) => *ticks,
        pumpkin_nbt::tag::NbtTag::Compound(info) => info.get_int("ticks")?,
        _ => return None,
    });
    Some((ticks, period))
}

/// Reads the markers flagged `show_in_commands` from the `minecraft:day`
/// timeline NBT, matching vanilla `commandTimeMarkersForClock`.
fn day_timeline_command_markers(bytes: &[u8]) -> Option<Vec<String>> {
    use std::io::Cursor;

    use pumpkin_nbt::Nbt;

    let mut cursor = Cursor::new(bytes);
    let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
        pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
    );
    let root = Nbt::read_unnamed(&mut reader).ok()?.root_tag;
    let markers = root.get_compound("time_markers")?;
    Some(
        markers
            .child_tags
            .iter()
            .filter(|(_, value)| match value {
                pumpkin_nbt::tag::NbtTag::Compound(info) => {
                    info.get_bool("show_in_commands").unwrap_or(false)
                }
                _ => false,
            })
            .map(|(name, _)| name.to_string())
            .collect(),
    )
}

/// The `minecraft:day` timeline registry NBT, when a datapack has loaded it.
fn day_timeline_data(context: &CommandContext) -> Option<Box<[u8]>> {
    context
        .source
        .server()
        .datapack_manager
        .get_dynamic_registry_entry_data("minecraft:timeline", DAY_TIMELINE)
}

/// Resolves a `/time set <timemarker>` marker against the `minecraft:day`
/// timeline data, falling back to the builtin markers. Vanilla registers every
/// timeline marker on its clock, so datapacks can move or add markers.
fn resolve_time_marker(
    context: &CommandContext,
    id: &Identifier,
    clock: &str,
) -> Option<(i64, i64)> {
    let builtin = builtin_time_marker(id, clock);
    if clock != DEFAULT_CLOCK {
        return builtin;
    }
    let Some(data) = day_timeline_data(context) else {
        return builtin;
    };
    day_timeline_marker(&data, &id.to_string()).or(builtin)
}

struct TimeMarkerSuggestionProvider;

impl SuggestionProvider for TimeMarkerSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let fallback = [
            "minecraft:day",
            "minecraft:noon",
            "minecraft:night",
            "minecraft:midnight",
        ];
        let markers = day_timeline_data(context)
            .and_then(|data| day_timeline_command_markers(&data))
            .filter(|markers| !markers.is_empty())
            .unwrap_or_else(|| {
                fallback
                    .iter()
                    .map(|marker| (*marker).to_string())
                    .collect()
            });
        for marker in markers {
            builder = builder.suggest(marker.as_str());
        }
        builder.build()
    }
}

struct SetTimeMarkerExecutor {
    has_clock: bool,
}

impl CommandExecutor for SetTimeMarkerExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let clock_name = resolve_clock_name(context, self.has_clock)?;
        let marker = IdentifierArgumentType::get(context, "timemarker")?;
        let marker_name = marker.to_string();
        let (ticks, period) =
            resolve_time_marker(context, &marker, &clock_name).ok_or_else(|| {
                ERROR_NO_TIME_MARKER_FOUND.create_without_context_args_slice(&[
                    TextComponent::text(marker_name.clone()),
                    TextComponent::text(clock_name.clone()),
                ])
            })?;

        let world = context.source.world();
        let (level_time, target_ticks) = {
            let mut guard = world
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if marker_occurs_at(guard.time_of_day, ticks, period) {
                return Err(
                    ERROR_ALREADY_AT_TIME_MARKER.create_without_context_args_slice(&[
                        TextComponent::text(clock_name),
                        TextComponent::text(marker_name),
                    ]),
                );
            }
            let target = resolve_time_to_move_to(guard.time_of_day, ticks, period);
            guard.set_time(target);
            (guard.clone(), target)
        };
        level_time.send_time(world);
        context.source.send_feedback(
            pumpkin_macros::translate_cross!(
                translation::java::COMMANDS_TIME_SET_TIME_MARKER,
                translation::java::COMMANDS_TIME_SET_TIME_MARKER,
                TextComponent::text(clock_name),
                TextComponent::text(marker_name)
            ),
            true,
        );
        Ok(wrap_time(target_ticks))
    }
}

macro_rules! build_branches {
    ($builder:expr, $has_clock:expr) => {{
        let set_node = literal("set")
            .then(literal("day").executes(ActionExecutor {
                action: Action::Set(Some(PresetTime::Day)),
                has_clock: $has_clock,
            }))
            .then(literal("noon").executes(ActionExecutor {
                action: Action::Set(Some(PresetTime::Noon)),
                has_clock: $has_clock,
            }))
            .then(literal("night").executes(ActionExecutor {
                action: Action::Set(Some(PresetTime::Night)),
                has_clock: $has_clock,
            }))
            .then(literal("midnight").executes(ActionExecutor {
                action: Action::Set(Some(PresetTime::Midnight)),
                has_clock: $has_clock,
            }))
            .then(
                argument("time", TimeArgumentType::any()).executes(ActionExecutor {
                    action: Action::Set(None),
                    has_clock: $has_clock,
                }),
            )
            .then(
                argument("timemarker", IdentifierArgumentType)
                    .suggests(TimeMarkerSuggestionProvider)
                    .executes(SetTimeMarkerExecutor {
                        has_clock: $has_clock,
                    }),
            );

        let add_node = literal("add").then(
            argument("time", TimeArgumentType::new(i32::MIN)).executes(ActionExecutor {
                action: Action::Add,
                has_clock: $has_clock,
            }),
        );

        let pause_node = literal("pause").executes(ActionExecutor {
            action: Action::Pause,
            has_clock: $has_clock,
        });
        let resume_node = literal("resume").executes(ActionExecutor {
            action: Action::Resume,
            has_clock: $has_clock,
        });

        let rate_node = literal("rate").then(
            argument("rate", FloatArgumentType::new(1.0e-5, 1000.0)).executes(ActionExecutor {
                action: Action::Rate,
                has_clock: $has_clock,
            }),
        );

        let query_node = literal("query")
            .then(literal("time").executes(QueryExecutor {
                mode: QueryMode::Time,
                has_clock: $has_clock,
            }))
            .then(literal("gametime").executes(QueryExecutor {
                mode: QueryMode::GameTime,
                has_clock: $has_clock,
            }))
            .then(literal("daytime").executes(QueryExecutor {
                mode: QueryMode::DayTime,
                has_clock: $has_clock,
            }))
            .then(literal("day").executes(QueryExecutor {
                mode: QueryMode::Day,
                has_clock: $has_clock,
            }));

        $builder
            .then(set_node)
            .then(add_node)
            .then(pause_node)
            .then(resume_node)
            .then(rate_node)
            .then(query_node)
    }};
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let of_clock_node = literal("of").then(build_branches!(
        argument(
            "clock",
            RegistryIdArgumentType::resource(Identifier::vanilla_static("world_clock"))
        ),
        true
    ));

    let time_cmd = build_branches!(command("time", DESCRIPTION).requires(PERMISSION), false)
        .then(of_clock_node);

    dispatcher.register(time_cmd);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_time_returns_ticks_until_i32_max_then_wraps_to_zero() {
        assert_eq!(wrap_time(0), 0);
        assert_eq!(wrap_time(2_147_483_646), 2_147_483_646);
        assert_eq!(wrap_time(2_147_483_647), 0);
        assert_eq!(wrap_time(2_147_483_648), 1);
    }

    #[test]
    fn day_timeline_query_rejects_other_clocks() {
        assert!(ensure_day_timeline_clock(DEFAULT_CLOCK).is_ok());
        assert!(ensure_day_timeline_clock("minecraft:the_end").is_err());
    }

    #[test]
    fn builtin_markers_resolve_against_the_day_timeline() {
        let day = Identifier::from_static("minecraft", "day");
        assert_eq!(
            builtin_time_marker(&day, DEFAULT_CLOCK),
            Some((1000, 24000))
        );
        let noon = Identifier::from_static("minecraft", "noon");
        assert_eq!(
            builtin_time_marker(&noon, DEFAULT_CLOCK),
            Some((6000, 24000))
        );
        let night = Identifier::from_static("minecraft", "night");
        assert_eq!(
            builtin_time_marker(&night, DEFAULT_CLOCK),
            Some((13000, 24000))
        );
        let midnight = Identifier::from_static("minecraft", "midnight");
        assert_eq!(
            builtin_time_marker(&midnight, DEFAULT_CLOCK),
            Some((18000, 24000))
        );
        let wake = Identifier::from_static("minecraft", "wake_up_from_sleep");
        assert_eq!(builtin_time_marker(&wake, DEFAULT_CLOCK), Some((0, 24000)));
        let siege = Identifier::from_static("minecraft", "roll_village_siege");
        assert_eq!(
            builtin_time_marker(&siege, DEFAULT_CLOCK),
            Some((18000, 24000))
        );
        assert_eq!(builtin_time_marker(&day, "minecraft:the_end"), None);
        let custom = Identifier::from_static("mymod", "day");
        assert_eq!(builtin_time_marker(&custom, DEFAULT_CLOCK), None);
    }

    #[test]
    fn time_marker_move_advances_forward_without_rewinding() {
        assert!(marker_occurs_at(1000, 1000, 24000));
        assert!(marker_occurs_at(25000, 1000, 24000));
        assert!(!marker_occurs_at(999, 1000, 24000));
        assert_eq!(resolve_time_to_move_to(0, 6000, 24000), 6000);
        assert_eq!(resolve_time_to_move_to(12000, 6000, 24000), 30000);
    }

    #[test]
    fn day_timeline_marker_reads_datapack_int_and_compound_forms() {
        let value = serde_json::json!({
            "period_ticks": 24000,
            "time_markers": {
                "minecraft:day": 1500,
                "minecraft:noon": {"ticks": 7000, "show_in_commands": true}
            }
        });
        let bytes =
            crate::data::datapack::dynamic_registry_loader::json_to_registry_nbt_bytes(&value);
        assert_eq!(
            day_timeline_marker(&bytes, "minecraft:day"),
            Some((1500, 24000))
        );
        assert_eq!(
            day_timeline_marker(&bytes, "minecraft:noon"),
            Some((7000, 24000))
        );
        assert_eq!(day_timeline_marker(&bytes, "minecraft:absent"), None);
    }

    #[test]
    fn day_timeline_command_markers_filters_show_in_commands() {
        let value = serde_json::json!({
            "period_ticks": 24000,
            "time_markers": {
                "minecraft:day": {"ticks": 1000, "show_in_commands": true},
                "minecraft:noon": {"ticks": 6000, "show_in_commands": true},
                "minecraft:wake_up_from_sleep": 0
            }
        });
        let bytes =
            crate::data::datapack::dynamic_registry_loader::json_to_registry_nbt_bytes(&value);
        let mut markers = day_timeline_command_markers(&bytes).unwrap_or_default();
        markers.sort();
        assert_eq!(markers, vec!["minecraft:day", "minecraft:noon"]);
    }
}
