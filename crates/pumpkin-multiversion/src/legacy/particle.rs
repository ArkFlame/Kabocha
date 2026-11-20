use pumpkin_data::particle::Particle;

use pumpkin_protocol::{
    VarInt,
    codec::item_stack_seralizer::ItemStackSerializer,
    ser::{NetworkReadExt, ReadingError},
};

use crate::legacy::packets::particle::particle_name_for_v1_7;

/// Numeric particle id used by 1.8-1.12 clients, derived from the legacy 1.7 name.
///
/// The generated 26.3 -> older remap is not usable below 1.13: the ViaBackwards
/// mapping files stop carrying a `particles` section at that boundary, so the
/// generated table is just the 1.13 table copied down. Names are stable across
/// 1.8-1.12, so the 1.7 name gives us the right numeric id for those clients.
#[must_use]
pub fn particle_id_for_v1_8(particle: Particle) -> u16 {
    match particle_name_for_v1_7(particle) {
        "explode" => 0,
        "largeexplode" => 1,
        "hugeexplosion" => 2,
        "fireworksSpark" => 3,
        "bubble" => 4,
        "splash" => 5,
        "wake" => 6,
        "suspended" => 7,
        "depthsuspend" => 8,
        "crit" => 9,
        "magicCrit" => 10,
        "smoke" => 11,
        "largesmoke" => 12,
        "spell" => 13,
        "instantSpell" => 14,
        "mobSpell" => 15,
        "mobSpellAmbient" => 16,
        "witchMagic" => 17,
        "dripWater" => 18,
        "dripLava" => 19,
        "angryVillager" => 20,
        "happyVillager" => 21,
        "townaura" => 22,
        "note" => 23,
        "portal" => 24,
        "enchantmenttable" => 25,
        "flame" => 26,
        "lava" => 27,
        "footstep" => 28,
        "cloud" => 29,
        "reddust" => 30,
        "snowballpoof" => 31,
        "snowshovel" => 32,
        "slime" => 33,
        "heart" => 34,
        "barrier" => 35,
        "iconcrack_" => 36,
        "blockcrack_" => 37,
        "blockdust_" => 38,
        "droplet" => 39,
        "take" => 40,
        "mobappearance" => 41,
        _ => 11,
    }
}

/// 26.3 option payload that follows a particle type id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleOptionsLayout {
    None,
    BlockState,
    Color,
    Dust,
    DustColorTransition,
    Spell,
    Power,
    Float,
    VarInt,
    Geyser,
    GeyserBase,
    Trail,
    Vibration,
    Item,
}

impl ParticleOptionsLayout {
    #[must_use]
    pub const fn of(particle: Particle) -> Self {
        use Particle as P;
        match particle {
            P::Block | P::BlockMarker | P::FallingDust | P::DustPillar | P::BlockCrumble => {
                Self::BlockState
            }
            P::EntityEffect | P::TintedLeaves | P::Flash => Self::Color,
            P::Dust => Self::Dust,
            P::DustColorTransition => Self::DustColorTransition,
            P::Effect | P::InstantEffect => Self::Spell,
            P::DragonBreath => Self::Power,
            P::SculkCharge => Self::Float,
            P::Shriek => Self::VarInt,
            P::Geyser | P::GeyserPlume => Self::Geyser,
            P::GeyserBase | P::GeyserPoof => Self::GeyserBase,
            P::Trail => Self::Trail,
            P::Vibration => Self::Vibration,
            P::Item => Self::Item,
            _ => Self::None,
        }
    }

    pub fn skip(self, bytebuf: &mut impl NetworkReadExt) -> Result<(), ReadingError> {
        match self {
            Self::None => {}
            Self::BlockState | Self::VarInt => {
                bytebuf.get_var_int()?;
            }
            Self::Color | Self::Geyser => {
                bytebuf.get_i32_be()?;
            }
            Self::Dust | Self::Spell | Self::GeyserBase => {
                bytebuf.get_i32_be()?;
                bytebuf.get_f32_be()?;
            }
            Self::DustColorTransition => {
                bytebuf.get_i32_be()?;
                bytebuf.get_i32_be()?;
                bytebuf.get_f32_be()?;
            }
            Self::Power | Self::Float => {
                bytebuf.get_f32_be()?;
            }
            Self::Trail => {
                bytebuf.get_f64_be()?;
                bytebuf.get_f64_be()?;
                bytebuf.get_f64_be()?;
                bytebuf.get_i32_be()?;
                bytebuf.get_var_int()?;
            }
            Self::Vibration => {
                match bytebuf.get_var_int()?.0 {
                    0 => {
                        bytebuf.get_block_pos(&pumpkin_data::packet::CURRENT_MC_VERSION)?;
                    }
                    1 => {
                        bytebuf.get_var_int()?;
                        bytebuf.get_f32_be()?;
                    }
                    source => {
                        return Err(ReadingError::Message(format!(
                            "Unknown vibration position source {source}"
                        )));
                    }
                }
                bytebuf.get_var_int()?;
            }
            Self::Item => {
                ItemStackSerializer::read_template_with_version(
                    bytebuf,
                    &pumpkin_data::packet::CURRENT_MC_VERSION,
                )?;
            }
        }
        Ok(())
    }

    pub fn skip_from_id(
        bytebuf: &mut impl NetworkReadExt,
        particle_id: VarInt,
    ) -> Result<(), ReadingError> {
        let particle = u16::try_from(particle_id.0)
            .ok()
            .and_then(Particle::from_id)
            .ok_or_else(|| {
                ReadingError::Message(format!("Unknown particle id {}", particle_id.0))
            })?;
        Self::of(particle).skip(bytebuf)
    }

    pub fn read_bytes<'a>(
        bytebuf: &mut &'a [u8],
        particle_id: VarInt,
    ) -> Result<&'a [u8], ReadingError> {
        let remaining_before = *bytebuf;
        Self::skip_from_id(bytebuf, particle_id)?;
        Ok(&remaining_before[..remaining_before.len() - bytebuf.len()])
    }
}
