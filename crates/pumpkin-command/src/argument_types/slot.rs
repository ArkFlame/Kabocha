use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::slot_ranges::{
    SLOT_RANGE_ALL_NAMES, SLOT_RANGE_SINGLE_SLOT_NAMES, get_slot_range,
};
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

pub const UNKNOWN_SLOT_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::SLOT_UNKNOWN,
    translation::java::SLOT_UNKNOWN,
);

pub const ONLY_SINGLE_SLOT_ALLOWED_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::SLOT_ONLY_SINGLE_ALLOWED,
    translation::java::SLOT_ONLY_SINGLE_ALLOWED,
);

/// Resolves a slot name: the generated table, plus villager and horse inventory slots it lacks.
fn resolve_slot_range(name: &str) -> Option<&'static [usize]> {
    if let Some(index) = name.strip_prefix("villager.") {
        return villager_slot_range(index);
    }
    match name {
        "horse.saddle" => Some(&[400usize]),
        "horse.armor" => Some(&[401usize]),
        _ => get_slot_range(name),
    }
}

fn villager_slot_range(index: &str) -> Option<&'static [usize]> {
    match index {
        "0" => Some(&[300usize]),
        "1" => Some(&[301usize]),
        "2" => Some(&[302usize]),
        "3" => Some(&[303usize]),
        "4" => Some(&[304usize]),
        "5" => Some(&[305usize]),
        "6" => Some(&[306usize]),
        "7" => Some(&[307usize]),
        _ => None,
    }
}

pub struct SlotArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for SlotArgumentType {
    type Item = usize;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        reader.read_until_space();
        let name = &reader.string()[start..reader.cursor()];

        match resolve_slot_range(name) {
            Some([slot]) => Ok(*slot),
            Some(_) => Err(ONLY_SINGLE_SLOT_ALLOWED_ERROR_TYPE
                .create(reader, TextComponent::text(name.to_string()))),
            None => {
                Err(UNKNOWN_SLOT_ERROR_TYPE.create(reader, TextComponent::text(name.to_string())))
            }
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ItemSlot
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder
            .filter_and_suggest(&SLOT_RANGE_SINGLE_SLOT_NAMES)
            .build()
    }

    fn examples(&self) -> Vec<String> {
        examples!("weapon", "container.2", "enderchest.0")
    }
}

impl_copy_get!(SlotArgumentType, usize);

pub struct SlotsArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for SlotsArgumentType {
    type Item = &'static [usize];

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        reader.read_until_space();
        let name = &reader.string()[start..reader.cursor()];

        resolve_slot_range(name).ok_or_else(|| {
            let text = name.to_string();
            UNKNOWN_SLOT_ERROR_TYPE.create(reader, TextComponent::text(text))
        })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ItemSlots
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.filter_and_suggest(&SLOT_RANGE_ALL_NAMES).build()
    }

    fn examples(&self) -> Vec<String> {
        examples!("weapon", "container.2", "enderchest.*")
    }
}

impl_copy_get!(SlotsArgumentType, &'static [usize]);

#[cfg(test)]
mod tests {
    use super::resolve_slot_range;

    #[test]
    fn resolves_villager_slots_300_to_307() {
        assert_eq!(resolve_slot_range("villager.0"), Some(&[300usize][..]));
        assert_eq!(resolve_slot_range("villager.7"), Some(&[307usize][..]));
        assert_eq!(resolve_slot_range("villager.8"), None);
        assert_eq!(resolve_slot_range("villager.07"), None);
    }

    #[test]
    fn resolves_horse_saddle_and_armor_slots() {
        assert_eq!(resolve_slot_range("horse.saddle"), Some(&[400usize][..]));
        assert_eq!(resolve_slot_range("horse.armor"), Some(&[401usize][..]));
    }

    #[test]
    fn keeps_generated_slot_names() {
        assert_eq!(resolve_slot_range("armor.head"), Some(&[103usize][..]));
        assert_eq!(resolve_slot_range("horse.0"), Some(&[500usize][..]));
        assert_eq!(resolve_slot_range("bogus"), None);
    }
}
