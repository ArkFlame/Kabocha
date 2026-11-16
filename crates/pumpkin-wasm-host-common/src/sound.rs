//! Conversion of the WIT `sound` enum to [`pumpkin_data::sound::Sound`].
//!
//! The WIT enum's variant names (`entity-experience-orb-pickup`) and the
//! internal enum's variant names (`EntityExperienceOrbPickup`) are generated
//! from the same registry list, so the internal variant can be found by name.
//! The dotted registry name (`entity.experience_orb.pickup`) cannot be
//! reconstructed from the variant name, because the split points differ.

use std::collections::HashMap;
use std::sync::LazyLock;

use pumpkin_data::sound::Sound;

/// Maps the CamelCase variant name of each sound to its internal value.
static BY_VARIANT_NAME: LazyLock<HashMap<String, Sound>> = LazyLock::new(|| {
    Sound::NAMES
        .iter()
        .filter_map(|name| Sound::from_name(name))
        .map(|sound| (variant_name(sound.to_name()), sound))
        .collect()
});

/// Turns a dotted registry name into the CamelCase variant name.
fn variant_name(registry_name: &str) -> String {
    let mut out = String::with_capacity(registry_name.len());
    for part in registry_name.split(['.', '_']) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

/// Resolves a WIT `sound` variant name (as produced by its `Debug` output,
/// e.g. `EntityExperienceOrbPickup`) to the internal sound.
#[must_use]
pub fn from_variant_name(variant: &str) -> Option<Sound> {
    BY_VARIANT_NAME.get(variant).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_wit_variant_names() {
        assert_eq!(
            from_variant_name("EntityExperienceOrbPickup"),
            Some(Sound::EntityExperienceOrbPickup)
        );
        assert_eq!(
            from_variant_name("AmbientUnderwaterLoopAdditionsUltraRare"),
            Some(Sound::AmbientUnderwaterLoopAdditionsUltraRare)
        );
        assert_eq!(from_variant_name("NotASound"), None);
    }
}
