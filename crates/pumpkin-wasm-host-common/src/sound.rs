//! Conversion of the WIT `sound` enum to [`pumpkin_data::sound::Sound`].
//!
//! The WIT enum is generated from the same registry list as the internal enum,
//! in the same order, so a variant's position is its internal discriminant.

use pumpkin_data::sound::Sound;

/// Resolves a WIT `sound` variant index to the internal sound.
#[must_use]
pub fn from_index(index: usize) -> Option<Sound> {
    Sound::slice().get(index).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_indices_in_enum_order() {
        assert_eq!(from_index(0), Some(Sound::EntityAllayAmbientWithItem));
        assert_eq!(from_index(Sound::slice().len()), None);
    }
}
