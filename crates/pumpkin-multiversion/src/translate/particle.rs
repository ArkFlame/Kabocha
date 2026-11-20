//! Particles: id and options, LEVEL_PARTICLES.

use crate::legacy::particle::{ParticleOptionsLayout, particle_id_for_v1_8};
use pumpkin_data::particle::Particle;
use pumpkin_protocol::{
    VarInt,
    codec::item_stack_seralizer::ItemStackSerializer,
    java::client::play::CParticle,
    ser::{NetworkReadExt, NetworkWriteExt},
};
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::remap::{
    item_id_remap::remap_item_id_for_version, particle_id_remap::remap_particle_id_for_version,
};
use crate::translate::block::remap_state;

eras! {
    pub enum ParticleFormat {
        /// 1.7 names the particle and has no trailing data array.
        V1_7 = V_1_7_2,
        /// 1.8-1.12 use numeric ids and a trailing varint data array.
        V1_8 = V_1_8,
        /// Dust colors are three floats; dust color transitions are plain dust.
        V1_13 = V_1_13,
        /// Dust color transitions, with the scale between the colors.
        V1_17 = V_1_17,
        /// Block markers, barriers before.
        V1_18 = V_1_18,
        /// Sculk charge and shriek, ambient entity effects before.
        V1_19 = V_1_19,
        /// Dust color transitions end with the scale, entity effects gained a color.
        V1_20_5 = V_1_20_5,
        /// Dust colors are packed RGB ints.
        V1_21_2 = V_1_21_2,
        /// Tinted leaves, composter before.
        V1_21_5 = V_1_21_5,
        /// Effect, instant effect, dragon breath and flash gained options.
        V1_21_9 = V_1_21_9,
    }
}

/// First era in which the client's particle for `particle` takes its options.
const fn options_since(particle: Particle) -> ParticleFormat {
    match particle {
        Particle::BlockMarker => ParticleFormat::V1_18,
        Particle::SculkCharge | Particle::Shriek => ParticleFormat::V1_19,
        Particle::EntityEffect => ParticleFormat::V1_20_5,
        Particle::TintedLeaves => ParticleFormat::V1_21_5,
        Particle::Effect | Particle::InstantEffect | Particle::DragonBreath | Particle::Flash => {
            ParticleFormat::V1_21_9
        }
        _ => ParticleFormat::V1_13,
    }
}

/// Packed RGB as the three floats of older dust particles.
fn write_rgb_floats(color: i32, out: &mut Vec<u8>) -> Option<()> {
    for shift in [16, 8, 0] {
        out.write_f32_be(((color >> shift) & 0xFF) as f32 / 255.0)
            .ok()?;
    }
    Some(())
}

/// Varint data array appended after the count for 1.8-1.12 block/item crack particles.
fn legacy_extra_data(
    options: &[u8],
    legacy_id: u16,
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let mut read = options;
    let mut out = Vec::new();
    match legacy_id {
        // iconcrack_(id)_(data): item id then item damage.
        36 => {
            let stack = ItemStackSerializer::read_template_with_version(
                &mut read,
                &pumpkin_data::packet::CURRENT_MC_VERSION,
            )
            .ok()?;
            let item_id = remap_item_id_for_version(stack.0.item.id, version);
            out.write_var_int(&VarInt(i32::from(item_id))).ok()?;
            out.write_var_int(&VarInt(0)).ok()?;
        }
        // blockcrack_(id+(data<<12)) and blockdust_(id): one packed block value.
        37 | 38 => {
            let state = read.get_var_int().ok()?.0 as u32;
            let legacy_state = remap_state(state, version);
            let block_id = legacy_state >> 4;
            let meta = legacy_state & 0xF;
            let value = if legacy_id == 37 {
                block_id | (meta << 12)
            } else {
                block_id
            };
            out.write_var_int(&VarInt(value as i32)).ok()?;
        }
        _ => {}
    }
    Some(out)
}

/// Consumes a 26.3 particle (id and options) without translating it. Older
/// explosion layouts omit the particle entirely but still carry its bytes.
pub fn skip_particle(read: &mut &[u8]) -> Option<()> {
    let id = read.get_var_int().ok()?;
    ParticleOptionsLayout::skip_from_id(read, id).ok()
}

/// Reads one 26.3 particle and writes it for `version`.
pub fn write_particle(
    read: &mut &[u8],
    version: JavaMinecraftVersion,
    out: &mut Vec<u8>,
) -> Option<()> {
    let format = ParticleFormat::of(version);
    let id = read.get_var_int().ok()?.0;
    let particle = Particle::from_id(u16::try_from(id).ok()?)?;
    let options = ParticleOptionsLayout::of(particle);
    match format {
        ParticleFormat::V1_7 => {
            // The legacy writer derives the name from the 26.3 id; 1.7 has no data array.
            out.write_var_int(&VarInt(id)).ok()?;
            return options.skip(read).ok();
        }
        ParticleFormat::V1_8 => {
            let legacy_id = particle_id_for_v1_8(particle);
            out.write_var_int(&VarInt(i32::from(legacy_id))).ok()?;
            let option_bytes = ParticleOptionsLayout::read_bytes(read, VarInt(id)).ok()?;
            out.extend_from_slice(&legacy_extra_data(option_bytes, legacy_id, version)?);
            return Some(());
        }
        _ => {}
    }
    out.write_var_int(&VarInt(i32::from(remap_particle_id_for_version(
        id as u16, version,
    ))))
    .ok()?;
    if format < options_since(particle) {
        return options.skip(read).ok();
    }
    match options {
        ParticleOptionsLayout::None => {}
        ParticleOptionsLayout::BlockState => {
            let state = read.get_var_int().ok()?.0 as u32;
            out.write_var_int(&VarInt(remap_state(state, version) as i32))
                .ok()?;
        }
        ParticleOptionsLayout::Color => out.write_i32_be(read.get_i32_be().ok()?).ok()?,
        ParticleOptionsLayout::Dust => {
            let color = read.get_i32_be().ok()?;
            let scale = read.get_f32_be().ok()?;
            if format >= ParticleFormat::V1_21_2 {
                out.write_i32_be(color).ok()?;
            } else {
                write_rgb_floats(color, out)?;
            }
            out.write_f32_be(scale).ok()?;
        }
        ParticleOptionsLayout::DustColorTransition => {
            let from = read.get_i32_be().ok()?;
            let to = read.get_i32_be().ok()?;
            let scale = read.get_f32_be().ok()?;
            if format >= ParticleFormat::V1_21_2 {
                out.write_i32_be(from).ok()?;
                out.write_i32_be(to).ok()?;
                out.write_f32_be(scale).ok()?;
            } else if format >= ParticleFormat::V1_20_5 {
                write_rgb_floats(from, out)?;
                write_rgb_floats(to, out)?;
                out.write_f32_be(scale).ok()?;
            } else {
                write_rgb_floats(from, out)?;
                out.write_f32_be(scale).ok()?;
                // Plain dust before 1.17
                if format >= ParticleFormat::V1_17 {
                    write_rgb_floats(to, out)?;
                }
            }
        }
        ParticleOptionsLayout::Spell => {
            out.write_i32_be(read.get_i32_be().ok()?).ok()?;
            out.write_f32_be(read.get_f32_be().ok()?).ok()?;
        }
        ParticleOptionsLayout::Power => out.write_f32_be(read.get_f32_be().ok()?).ok()?,
        ParticleOptionsLayout::Float => out.write_f32_be(read.get_f32_be().ok()?).ok()?,
        ParticleOptionsLayout::VarInt => out.write_var_int(&read.get_var_int().ok()?).ok()?,
        // TODO: item, vibration, trail and geyser options
        ParticleOptionsLayout::Item
        | ParticleOptionsLayout::Vibration
        | ParticleOptionsLayout::Trail
        | ParticleOptionsLayout::Geyser
        | ParticleOptionsLayout::GeyserBase => return None,
    }
    Some(())
}

fn read_f32_vec3(read: &mut &[u8]) -> Option<Vector3<f32>> {
    Some(Vector3::new(
        read.get_f32_be().ok()?,
        read.get_f32_be().ok()?,
        read.get_f32_be().ok()?,
    ))
}

/// LEVEL_PARTICLES: 26.3 moved the particle to the front, split the speed per axis, made the
/// count a var int and added a randomization type. Core's writer handles every older layout.
pub fn level_particles_from_current(
    mut payload: &[u8],
    version: JavaMinecraftVersion,
) -> Option<Vec<u8>> {
    let mut particle = Vec::new();
    write_particle(&mut payload, version, &mut particle)?;
    let force_spawn = payload.get_bool().ok()?;
    let important = payload.get_bool().ok()?;
    let position = Vector3::new(
        payload.get_f64_be().ok()?,
        payload.get_f64_be().ok()?,
        payload.get_f64_be().ok()?,
    );
    let offset = read_f32_vec3(&mut payload)?;
    let speed = read_f32_vec3(&mut payload)?;
    let count = payload.get_var_int().ok()?.0;
    let _randomization = payload.get_var_int().ok()?;

    let (offset, max_speed) = if speed.x == speed.y && speed.y == speed.z {
        (offset, speed.x)
    } else if count <= 0 {
        // A count of 0 uses offset * speed as the velocity, so fold the speeds into the offsets
        (
            Vector3::new(offset.x * speed.x, offset.y * speed.y, offset.z * speed.z),
            1.0,
        )
    } else {
        // TODO: per axis speeds and the alternative randomization need one packet per
        // particle (ViaBackwards `BlockItemPacketRewriter26_3`); core sends neither yet
        (offset, speed.x)
    };

    let mut particle = particle.as_slice();
    let particle_id = particle.get_var_int().ok()?;
    super::legacy_bytes(
        &CParticle::new(
            force_spawn,
            important,
            position,
            offset,
            max_speed,
            count,
            particle_id,
            particle,
        ),
        version,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::LegacyWrite;
    use pumpkin_protocol::ClientPacket;

    #[test]
    fn legacy_particle_ids() {
        let flame = VarInt(i32::from(Particle::Flame.to_id()));
        let mut current = Vec::new();
        current.write_var_int(&flame).unwrap();

        // 1.7 keeps the current id so the writer can derive the particle name.
        let mut out = Vec::new();
        write_particle(
            &mut current.as_slice(),
            JavaMinecraftVersion::V_1_7_2,
            &mut out,
        )
        .unwrap();
        assert_eq!(out, current);

        // 1.8 maps through the legacy name ("flame" -> 26).
        let mut out = Vec::new();
        write_particle(
            &mut current.as_slice(),
            JavaMinecraftVersion::V_1_8,
            &mut out,
        )
        .unwrap();
        let mut read = out.as_slice();
        assert_eq!(read.get_var_int().unwrap().0, 26);
        assert!(read.is_empty());
    }

    #[test]
    fn legacy_block_crack_has_one_extra_varint() {
        let block = VarInt(i32::from(Particle::Block.to_id()));
        let mut current = Vec::new();
        current.write_var_int(&block).unwrap();
        current.write_var_int(&VarInt(0)).unwrap(); // 26.3 block state
        let mut out = Vec::new();
        write_particle(
            &mut current.as_slice(),
            JavaMinecraftVersion::V_1_8,
            &mut out,
        )
        .unwrap();
        let mut read = out.as_slice();
        assert_eq!(read.get_var_int().unwrap().0, 37); // blockcrack_
        assert!(read.get_var_int().is_ok());
        assert!(read.is_empty());
    }

    #[test]
    fn dust_color_becomes_floats_before_1_21_2() {
        let mut current = Vec::new();
        current
            .write_var_int(&VarInt(i32::from(Particle::Dust.to_id())))
            .unwrap();
        current.write_i32_be(0x00FF_0000).unwrap();
        current.write_f32_be(2.0).unwrap();

        let mut out = Vec::new();
        write_particle(
            &mut current.as_slice(),
            JavaMinecraftVersion::V_1_21,
            &mut out,
        )
        .unwrap();
        let id =
            remap_particle_id_for_version(Particle::Dust.to_id(), JavaMinecraftVersion::V_1_21);
        let mut expected = Vec::new();
        expected.write_var_int(&VarInt(i32::from(id))).unwrap();
        for value in [1.0f32, 0.0, 0.0, 2.0] {
            expected.write_f32_be(value).unwrap();
        }
        assert_eq!(out, expected);
    }

    /// Writes `particle` with `options` for `version`, or `None` if it was dropped.
    fn written(
        particle: Particle,
        options: &[u8],
        version: JavaMinecraftVersion,
    ) -> Option<Vec<u8>> {
        let mut current = Vec::new();
        current
            .write_var_int(&VarInt(i32::from(particle.to_id())))
            .unwrap();
        current.extend_from_slice(options);
        let mut read = current.as_slice();
        let mut out = Vec::new();
        write_particle(&mut read, version, &mut out)?;
        assert!(read.is_empty());
        let mut out = out.as_slice();
        // Before 1.13 particles use their own legacy numbering, checked by its own tests.
        if version >= JavaMinecraftVersion::V_1_13 {
            assert_eq!(
                out.get_var_int().unwrap().0,
                i32::from(remap_particle_id_for_version(particle.to_id(), version))
            );
        } else {
            out.get_var_int().unwrap();
        }
        Some(out.to_vec())
    }

    fn floats(values: &[f32]) -> Vec<u8> {
        let mut out = Vec::new();
        for value in values {
            out.write_f32_be(*value).unwrap();
        }
        out
    }

    #[test]
    fn dust_color_transition_per_version() {
        let mut options = Vec::new();
        options.write_i32_be(0x00FF_0000).unwrap();
        options.write_i32_be(0x0000_00FF).unwrap();
        options.write_f32_be(2.0).unwrap();
        let transition = |version| written(Particle::DustColorTransition, &options, version);

        assert_eq!(
            transition(JavaMinecraftVersion::V_1_21_2),
            Some(options.clone())
        );
        assert_eq!(
            transition(JavaMinecraftVersion::V_1_20_5),
            Some(floats(&[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0]))
        );
        assert_eq!(
            transition(JavaMinecraftVersion::V_1_17),
            Some(floats(&[1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 1.0]))
        );
        assert_eq!(
            transition(JavaMinecraftVersion::V_1_16),
            Some(floats(&[1.0, 0.0, 0.0, 2.0]))
        );
        assert!(transition(JavaMinecraftVersion::V_1_12).is_some());
    }

    #[test]
    fn options_are_dropped_for_option_less_fallbacks() {
        let color = 0x0012_3456i32.to_be_bytes();
        assert_eq!(
            written(
                Particle::TintedLeaves,
                &color,
                JavaMinecraftVersion::V_1_21_4
            ),
            Some(Vec::new())
        );
        assert_eq!(
            written(
                Particle::TintedLeaves,
                &color,
                JavaMinecraftVersion::V_1_21_5
            ),
            Some(color.to_vec())
        );
        assert_eq!(
            written(
                Particle::EntityEffect,
                &color,
                JavaMinecraftVersion::V_1_20_3
            ),
            Some(Vec::new())
        );
        assert_eq!(
            written(Particle::BlockMarker, &[1], JavaMinecraftVersion::V_1_17),
            Some(Vec::new())
        );
        assert_eq!(
            written(Particle::Shriek, &[5], JavaMinecraftVersion::V_1_18),
            Some(Vec::new())
        );
    }

    #[test]
    fn level_particles_use_the_26_2_layout() {
        let flame = VarInt(i32::from(Particle::Flame.to_id()));
        let packet = |id| {
            CParticle::new(
                true,
                false,
                Vector3::new(1.0, 2.0, 3.0),
                Vector3::new(0.1, 0.2, 0.3),
                0.5,
                10,
                id,
                &[],
            )
        };
        let mut current = Vec::new();
        packet(flame)
            .write_packet_data(&mut current, &pumpkin_data::packet::CURRENT_MC_VERSION)
            .unwrap();

        let version = JavaMinecraftVersion::V_26_2;
        let id = remap_particle_id_for_version(Particle::Flame.to_id(), version);
        let mut expected = Vec::new();
        packet(VarInt(i32::from(id)))
            .write_legacy(&mut expected, &version)
            .unwrap();
        assert_eq!(
            level_particles_from_current(&current, version),
            Some(expected)
        );
    }
}
