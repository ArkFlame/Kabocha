use std::sync::Arc;

use std::borrow::Cow;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::translation;
use pumpkin_data::{Block, Property};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CParticle, ParticleOptions};
use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::particle::{ParticleArgumentType, ParticleWithOptions};
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::player::Player;

const DESCRIPTION: &str = "Creates particles in the world.";
const PERMISSION: &str = "minecraft:command.particle";

const ERROR_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_PARTICLE_FAILED,
    translation::java::COMMANDS_PARTICLE_FAILED,
);

/// Reads a numeric tag as `f32`, whatever its NBT type.
fn tag_as_f32(tag: &NbtTag) -> Option<f32> {
    match tag {
        NbtTag::Byte(v) => Some(f32::from(*v)),
        NbtTag::Short(v) => Some(f32::from(*v)),
        NbtTag::Int(v) => Some(*v as f32),
        NbtTag::Long(v) => Some(*v as f32),
        NbtTag::Float(v) => Some(*v),
        NbtTag::Double(v) => Some(*v as f32),
        _ => None,
    }
}

/// Reads a numeric tag as `i32`, whatever its NBT type.
fn tag_as_i32(tag: &NbtTag) -> Option<i32> {
    match tag {
        NbtTag::Byte(v) => Some(i32::from(*v)),
        NbtTag::Short(v) => Some(i32::from(*v)),
        NbtTag::Int(v) => Some(*v),
        NbtTag::Long(v) => Some(*v as i32),
        NbtTag::Float(v) => Some(*v as i32),
        NbtTag::Double(v) => Some(*v as i32),
        _ => None,
    }
}

/// Decodes a vanilla color: a packed int or a list of `[r, g, b(, a)]` floats in `0..=1`.
/// Returns the packed ARGB value (alpha defaults to opaque).
fn tag_as_color(tag: &NbtTag) -> Option<i32> {
    match tag {
        NbtTag::Int(v) => Some(*v),
        NbtTag::Long(v) => Some(*v as i32),
        NbtTag::List(list) => color_from_floats(list.iter().map(tag_as_f32)),
        NbtTag::IntArray(arr) => color_from_floats(arr.iter().map(|v| Some(*v as f32))),
        _ => None,
    }
}

fn color_from_floats(parts: impl Iterator<Item = Option<f32>>) -> Option<i32> {
    let parts: Vec<f32> = parts.collect::<Option<Vec<_>>>()?;
    if parts.len() < 3 {
        return None;
    }
    let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
    let alpha = parts.get(3).map_or(255, |a| channel(*a));
    let rgb =
        (alpha << 24) | (channel(parts[0]) << 16) | (channel(parts[1]) << 8) | channel(parts[2]);
    Some(rgb as i32)
}

/// Reads a vanilla `Vec3` option stored as a list of three numbers.
fn tag_as_vec3(tag: &NbtTag) -> Option<Vector3<f64>> {
    let NbtTag::List(list) = tag else {
        return None;
    };
    let parts: Vec<f64> = list
        .iter()
        .map(|v| match v {
            NbtTag::Float(f) => Some(f64::from(*f)),
            NbtTag::Double(d) => Some(*d),
            NbtTag::Int(i) => Some(f64::from(*i)),
            _ => None,
        })
        .collect::<Option<_>>()?;
    match parts.as_slice() {
        [x, y, z] => Some(Vector3::new(*x, *y, *z)),
        _ => None,
    }
}

/// Resolves a vanilla block state (`"minecraft:stone"` or `{Name:..,Properties:{..}}`) to its id.
fn block_state_id(tag: &NbtTag) -> Option<u16> {
    let (name, props) = match tag {
        NbtTag::String(name) => (name.to_string(), None),
        NbtTag::Compound(c) => (
            c.get_string("Name")
                .or_else(|| c.get_string("id"))?
                .to_string(),
            c.get_compound("Properties")
                .or_else(|| c.get_compound("properties")),
        ),
        _ => return None,
    };
    let name = if name.contains(':') {
        name
    } else {
        format!("minecraft:{name}")
    };
    let block = Block::from_name(&name)?;
    let default = block.default_state.id;
    let Some(props) = props else {
        return Some(default.as_u16());
    };
    // Blocks without properties panic in `from_properties`, so only ask when they have some.
    block.properties(default)?;
    let props: Vec<Property> = props
        .child_tags
        .iter()
        .filter_map(|(key, value)| match value {
            NbtTag::String(v) => Property::from_str_pair(key, v),
            _ => None,
        })
        .collect();
    Some(block.from_properties(&props).to_state_id(block).as_u16())
}

/// Builds an item stack from vanilla's `item` option (`"stone"` or `{id:..,count:..}`).
fn item_stack(tag: &NbtTag) -> Option<ItemStack> {
    match tag {
        NbtTag::String(id) => {
            let mut c = NbtCompound::new();
            c.put_string("id", id.to_string());
            c.put_int("count", 1);
            ItemStack::read_item_stack(&c)
        }
        NbtTag::Compound(c) => {
            let mut c = c.clone();
            if c.get("count").is_none() {
                c.put_int("count", 1);
            }
            ItemStack::read_item_stack(&c)
        }
        _ => None,
    }
}

/// Encodes the network data of a particle with options, matching vanilla's stream codecs.
/// Missing options fall back to vanilla-like defaults so the client never gets a short packet.
fn encode_particle_options(particle: Particle, options: Option<&NbtCompound>) -> Vec<u8> {
    let opt = |key: &str| options.and_then(|o| o.get(key));
    let mut buf = Vec::new();
    let result = match particle {
        Particle::Dust => {
            let color = opt("color").and_then(tag_as_color).unwrap_or(0x00FF_0000);
            let scale = opt("scale").and_then(tag_as_f32).unwrap_or(1.0);
            buf.write_i32_be(color & 0x00FF_FFFF)
                .and_then(|()| buf.write_f32_be(scale))
        }
        Particle::DustColorTransition => {
            let from = opt("from_color")
                .and_then(tag_as_color)
                .unwrap_or(0x00FF_0000);
            let to = opt("to_color")
                .and_then(tag_as_color)
                .unwrap_or(0x00FF_FFFF);
            let scale = opt("scale").and_then(tag_as_f32).unwrap_or(1.0);
            buf.write_i32_be(from & 0x00FF_FFFF)
                .and_then(|()| buf.write_i32_be(to & 0x00FF_FFFF))
                .and_then(|()| buf.write_f32_be(scale))
        }
        Particle::EntityEffect | Particle::TintedLeaves => {
            let color = opt("color").and_then(tag_as_color).unwrap_or(-1);
            buf.write_i32_be(color)
        }
        Particle::Trail => {
            // Vanilla `TrailParticleOption`: target Vec3, colour INT, duration VAR_INT.
            let target = opt("target")
                .and_then(tag_as_vec3)
                .unwrap_or(Vector3::new(0.0, 0.0, 0.0));
            let color = opt("color").and_then(tag_as_color).unwrap_or(-1);
            let duration = opt("duration").and_then(tag_as_i32).unwrap_or(0);
            ParticleOptions::Trail {
                target,
                color,
                duration,
            }
            .write(&mut buf)
        }
        Particle::Effect | Particle::InstantEffect => {
            // Vanilla `SpellParticleOption`: colour, then the swirl power.
            let color = opt("color").and_then(tag_as_color).unwrap_or(-1);
            let power = opt("power").and_then(tag_as_f32).unwrap_or(1.0);
            ParticleOptions::Spell { color, power }.write(&mut buf)
        }
        Particle::SculkCharge => {
            // Vanilla `SculkChargeParticleOptions.STREAM_CODEC`: a single float roll.
            let roll = opt("roll").and_then(tag_as_f32).unwrap_or(0.0);
            buf.write_f32_be(roll)
        }
        Particle::Shriek => {
            // Vanilla `ShriekParticleOption.STREAM_CODEC`: a var-int delay.
            let delay = opt("delay").and_then(tag_as_i32).unwrap_or(0);
            buf.write_var_int(&VarInt(delay))
        }
        Particle::Block
        | Particle::BlockMarker
        | Particle::FallingDust
        | Particle::DustPillar
        | Particle::BlockCrumble => {
            let id = opt("block_state").and_then(block_state_id).unwrap_or(0);
            buf.write_var_int(&VarInt(i32::from(id)))
        }
        Particle::Item => {
            let stack = opt("item")
                .and_then(item_stack)
                .unwrap_or_else(|| ItemStack::EMPTY.clone());
            ItemStackSerializer(Cow::Owned(stack)).write(&mut buf)
        }
        _ => Ok(()),
    };
    if result.is_err() {
        buf.clear();
    }
    buf
}

/// Vanilla `ServerLevel#sendParticles` measures from the viewer's block centre
/// (`blockPosition().closerToCenterThan(pos, force ? 512 : 32)`), strictly less than the radius.
fn is_particle_viewer_in_range(player_pos: Vector3<f64>, pos: Vector3<f64>, force: bool) -> bool {
    const NORMAL_RADIUS: f64 = 32.0;
    const FORCED_RADIUS: f64 = 512.0;
    const BLOCK_CENTRE_OFFSET: f64 = 0.5;

    let radius = if force { FORCED_RADIUS } else { NORMAL_RADIUS };
    let dx = player_pos.x.floor() + BLOCK_CENTRE_OFFSET - pos.x;
    let dy = player_pos.y.floor() + BLOCK_CENTRE_OFFSET - pos.y;
    let dz = player_pos.z.floor() + BLOCK_CENTRE_OFFSET - pos.z;
    dx * dx + dy * dy + dz * dz < radius * radius
}

#[allow(clippy::too_many_arguments)]
fn send_particles(
    source: &CommandSource,
    particle: &ParticleWithOptions,
    pos: Vector3<f64>,
    delta: Vector3<f64>,
    speed: f32,
    count: i32,
    force: bool,
    viewers: Option<Vec<Arc<Player>>>,
) -> Result<i32, CommandSyntaxError> {
    let world = source.world().clone();
    let players: Vec<Arc<Player>> = viewers.unwrap_or_else(|| world.players.load().to_vec());

    let offset = Vector3::new(delta.x as f32, delta.y as f32, delta.z as f32);
    let data = encode_particle_options(particle.particle, particle.options.as_ref());
    let particle = particle.particle;
    let packet = CParticle::new(
        force,
        false,
        pos,
        offset,
        speed,
        count,
        VarInt(i32::from(particle.to_id())),
        &data,
    );

    let mut sent_count = 0;

    for player in &players {
        if !Arc::ptr_eq(&player.world(), &world) {
            continue;
        }
        if is_particle_viewer_in_range(player.position(), pos, force) {
            player.try_send_client_packet(&packet);
            sent_count += 1;
        }
    }

    if sent_count == 0 {
        return Err(ERROR_FAILED.create_without_context());
    }

    // Vanilla uses the registry key, e.g. `minecraft:dust_color_transition`.
    let particle_name = format!("minecraft:{}", particle.to_name());
    source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_PARTICLE_SUCCESS,
            translation::java::COMMANDS_PARTICLE_SUCCESS,
            [TextComponent::text(particle_name)],
        ),
        true,
    );

    Ok(sent_count)
}

#[derive(Clone, Copy)]
enum ParticleMode {
    Simple,
    WithPos,
    Full { force: bool, has_viewers: bool },
}

struct ParticleExecutor {
    mode: ParticleMode,
}

impl CommandExecutor for ParticleExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let particle = ParticleArgumentType::get_with_options(context, "name")?;

        match self.mode {
            ParticleMode::Simple => {
                let pos = context.source.position;
                send_particles(
                    &context.source,
                    &particle,
                    pos,
                    Vector3::new(0.0, 0.0, 0.0),
                    0.0,
                    0,
                    false,
                    None,
                )
            }
            ParticleMode::WithPos => {
                let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
                send_particles(
                    &context.source,
                    &particle,
                    pos,
                    Vector3::new(0.0, 0.0, 0.0),
                    0.0,
                    0,
                    false,
                    None,
                )
            }
            ParticleMode::Full { force, has_viewers } => {
                let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
                let delta = Vec3ArgumentType::get_vector3(context, "delta")?;
                let speed = FloatArgumentType::get(context, "speed")?;
                let count = IntegerArgumentType::get(context, "count")?;
                let viewers = if has_viewers {
                    Some(EntityArgumentType::get_players(context, "viewers")?)
                } else {
                    None
                };

                send_particles(
                    &context.source,
                    &particle,
                    pos,
                    delta,
                    speed,
                    count,
                    force,
                    viewers,
                )
            }
        }
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("particle", DESCRIPTION).requires(PERMISSION).then(
            argument("name", ParticleArgumentType)
                .executes(ParticleExecutor {
                    mode: ParticleMode::Simple,
                })
                .then(
                    argument("pos", Vec3ArgumentType::Default)
                        .executes(ParticleExecutor {
                            mode: ParticleMode::WithPos,
                        })
                        .then(
                            argument("delta", Vec3ArgumentType::Uncorrected).then(
                                argument("speed", FloatArgumentType::with_min(0.0)).then(
                                    argument("count", IntegerArgumentType::with_min(0))
                                        .executes(ParticleExecutor {
                                            mode: ParticleMode::Full {
                                                force: false,
                                                has_viewers: false,
                                            },
                                        })
                                        .then(
                                            literal("force")
                                                .executes(ParticleExecutor {
                                                    mode: ParticleMode::Full {
                                                        force: true,
                                                        has_viewers: false,
                                                    },
                                                })
                                                .then(
                                                    argument(
                                                        "viewers",
                                                        EntityArgumentType::Players,
                                                    )
                                                    .executes(ParticleExecutor {
                                                        mode: ParticleMode::Full {
                                                            force: true,
                                                            has_viewers: true,
                                                        },
                                                    }),
                                                ),
                                        )
                                        .then(
                                            literal("normal")
                                                .executes(ParticleExecutor {
                                                    mode: ParticleMode::Full {
                                                        force: false,
                                                        has_viewers: false,
                                                    },
                                                })
                                                .then(
                                                    argument(
                                                        "viewers",
                                                        EntityArgumentType::Players,
                                                    )
                                                    .executes(ParticleExecutor {
                                                        mode: ParticleMode::Full {
                                                            force: false,
                                                            has_viewers: true,
                                                        },
                                                    }),
                                                ),
                                        ),
                                ),
                            ),
                        ),
                ),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sculk_charge_writes_roll_float() {
        let mut options = NbtCompound::new();
        options.put_float("roll", 0.25);
        let data = encode_particle_options(Particle::SculkCharge, Some(&options));
        assert_eq!(data, 0.25f32.to_be_bytes());
    }

    #[test]
    fn shriek_writes_delay_var_int() {
        let mut options = NbtCompound::new();
        options.put_int("delay", 7);
        let data = encode_particle_options(Particle::Shriek, Some(&options));
        assert_eq!(data, [7]);
    }

    #[test]
    fn trail_writes_target_colour_and_duration() {
        let mut options = NbtCompound::new();
        options.put(
            "target",
            NbtTag::List(vec![
                NbtTag::Double(1.0),
                NbtTag::Double(2.0),
                NbtTag::Double(3.0),
            ]),
        );
        options.put_int("color", 0x00FF_0000);
        options.put_int("duration", 5);
        let data = encode_particle_options(Particle::Trail, Some(&options));
        let mut expected = Vec::new();
        for v in [1.0f64, 2.0, 3.0] {
            expected.extend_from_slice(&v.to_be_bytes());
        }
        expected.extend_from_slice(&0x00FF_0000i32.to_be_bytes());
        expected.push(5);
        assert_eq!(data, expected);
    }

    #[test]
    fn effect_writes_colour_var_int_and_power_float() {
        let mut options = NbtCompound::new();
        options.put_int("color", 1);
        let data = encode_particle_options(Particle::Effect, Some(&options));
        let mut expected = vec![1u8];
        expected.extend_from_slice(&1.0f32.to_be_bytes());
        assert_eq!(data, expected);
    }

    #[test]
    fn viewer_range_measures_from_block_centre() {
        // Viewer block x=0 has centre 0.5: 0.5 + 31.4 = 31.9 < 32 (raw 32.3 would miss).
        let viewer = Vector3::new(0.9, 0.0, 0.0);
        assert!(is_particle_viewer_in_range(
            viewer,
            Vector3::new(-31.4, 0.0, 0.0),
            false
        ));
        // Exactly 32 from the block centre is excluded (strict comparison).
        assert!(!is_particle_viewer_in_range(
            viewer,
            Vector3::new(-31.5, 0.0, 0.0),
            false
        ));
        assert!(is_particle_viewer_in_range(
            viewer,
            Vector3::new(-100.0, 0.0, 0.0),
            true
        ));
    }
}
