use crate::argument_types::FromStringReader;
use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::particle::Particle;
use pumpkin_data::translation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

use crate::snbt::SnbtParser;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

pub const ERROR_UNKNOWN_PARTICLE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::PARTICLE_NOTFOUND,
    translation::bedrock::COMMANDS_PARTICLE_NOTFOUND,
);

pub const ERROR_INVALID_OPTIONS: CommandErrorType<1> = CommandErrorType::new(
    translation::java::PARTICLE_INVALIDOPTIONS,
    translation::java::PARTICLE_INVALIDOPTIONS,
);

/// Returns the first SNBT key vanilla's codec requires but the options lack.
///
/// Vanilla always decodes options through the particle type's codec
/// (`ParticleArgument.readParticle`), so a missing mandatory field is a parse error,
/// not a fall back to defaults. Simple particles and types whose codec keys are all
/// optional (`effect`, `instant_effect`, `dragon_breath`) accept an empty map.
fn missing_required_option(
    particle: Particle,
    options: Option<&NbtCompound>,
) -> Option<&'static str> {
    let required: &[&str] = match particle {
        // BlockParticleOption
        Particle::Block
        | Particle::BlockMarker
        | Particle::FallingDust
        | Particle::DustPillar
        | Particle::BlockCrumble => &["block_state"],
        // DustParticleOptions
        Particle::Dust => &["color", "scale"],
        // DustColorTransitionOptions
        Particle::DustColorTransition => &["from_color", "to_color", "scale"],
        // ColorParticleOption
        Particle::EntityEffect | Particle::TintedLeaves | Particle::Flash => &["color"],
        Particle::SculkCharge => &["roll"],
        Particle::Shriek => &["delay"],
        Particle::Item => &["item"],
        // VibrationParticleOption
        Particle::Vibration => &["destination", "arrival_in_ticks"],
        // TrailParticleOption
        Particle::Trail => &["target", "color", "duration"],
        // GeyserParticleOptions
        Particle::Geyser | Particle::GeyserPlume => &["water_blocks"],
        // GeyserBaseParticleOptions
        Particle::GeyserBase | Particle::GeyserPoof => &["water_blocks", "burst_impulse_base"],
        _ => return None,
    };
    required
        .iter()
        .copied()
        .find(|key| options.is_none_or(|o| !o.has(key)))
}

pub struct ParticleArgumentType;

/// A parsed particle argument: the particle type and its optional SNBT options.
#[derive(Clone, Debug)]
pub struct ParticleWithOptions {
    pub particle: Particle,
    pub options: Option<NbtCompound>,
}

impl<S: crate::source::CommandSource> ArgumentType<S> for ParticleArgumentType {
    type Item = ParticleWithOptions;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        let particle = Particle::from_name(identifier.path())
            .or_else(|| Particle::from_name(&identifier.to_string()))
            .ok_or_else(|| {
                ERROR_UNKNOWN_PARTICLE.create(reader, TextComponent::text(identifier.to_string()))
            })?;
        // Vanilla reads the particle options as an SNBT compound right after the id.
        let options = if reader.peek() == Some('{') {
            match SnbtParser::parse_for_commands(reader)? {
                NbtTag::Compound(compound) => Some(compound),
                _ => None,
            }
        } else {
            None
        };
        if let Some(field) = missing_required_option(particle, options.as_ref()) {
            return Err(ERROR_INVALID_OPTIONS.create(
                reader,
                TextComponent::text(format!("Missing option '{field}' for {particle:?}")),
            ));
        }
        Ok(ParticleWithOptions { particle, options })
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::Particle
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "foo".to_string(),
            "foo:bar".to_string(),
            "particle".to_string(),
        ]
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        // Vanilla suggests every registered particle as its resource key.
        let particles = (0u16..)
            .map_while(Particle::from_id)
            .map(|p| format!("minecraft:{}", p.to_name()))
            .collect();
        builder.filter_and_suggest_lowercase(particles).build()
    }
}

impl ParticleArgumentType {
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<Particle, CommandSyntaxError> {
        Ok(context.get_argument::<ParticleWithOptions>(name)?.particle)
    }

    /// Returns the particle together with its SNBT options (`dust{color:...}`).
    pub fn get_with_options<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<ParticleWithOptions, CommandSyntaxError> {
        Ok(context.get_argument::<ParticleWithOptions>(name)?.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(pairs: &[(&str, &str)]) -> NbtCompound {
        let mut compound = NbtCompound::new();
        for (key, value) in pairs {
            compound.put_string(key, value.to_string());
        }
        compound
    }

    #[test]
    fn required_option_missing_when_absent_or_incomplete() {
        assert_eq!(missing_required_option(Particle::Dust, None), Some("color"));
        assert_eq!(
            missing_required_option(Particle::Dust, Some(&options(&[("color", "1")]))),
            Some("scale")
        );
        assert_eq!(
            missing_required_option(Particle::Trail, Some(&options(&[("color", "1")]))),
            Some("target")
        );
        assert_eq!(
            missing_required_option(Particle::Vibration, Some(&options(&[("destination", "x")]))),
            Some("arrival_in_ticks")
        );
    }

    #[test]
    fn required_options_satisfied_or_not_needed() {
        let dust = options(&[("color", "1"), ("scale", "1")]);
        assert_eq!(missing_required_option(Particle::Dust, Some(&dust)), None);

        let transition = options(&[("from_color", "1"), ("to_color", "2"), ("scale", "1")]);
        assert_eq!(
            missing_required_option(Particle::DustColorTransition, Some(&transition)),
            None
        );

        // Simple particles and all-optional codecs accept no options at all.
        assert_eq!(missing_required_option(Particle::Crit, None), None);
        assert_eq!(missing_required_option(Particle::Effect, None), None);
        assert_eq!(missing_required_option(Particle::DragonBreath, None), None);
    }
}
