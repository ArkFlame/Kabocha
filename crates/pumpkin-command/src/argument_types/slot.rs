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

/// Resolves a slot source name through vanilla's generated `SlotRanges` table.
///
/// Vanilla 26.3 has no `villager.*`, `horse.saddle` or `horse.armor` names:
/// mob slots are `mob.inventory.*`, horse gear is `saddle`/`armor.body`.
fn resolve_slot_range(name: &str) -> Option<&'static [usize]> {
    get_slot_range(name)
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
    fn rejects_pre_26_slot_source_names() {
        // Vanilla 26.3 replaced these with `mob.inventory.*`, `saddle` and `armor.body`.
        assert!(resolve_slot_range("villager.0").is_none());
        assert!(resolve_slot_range("horse.saddle").is_none());
        assert!(resolve_slot_range("horse.armor").is_none());
    }

    #[test]
    fn resolves_vanilla_equipment_and_crafting_slots() {
        assert_eq!(resolve_slot_range("saddle"), Some(&[106usize][..]));
        assert_eq!(resolve_slot_range("player.cursor"), Some(&[499usize][..]));
        assert_eq!(
            resolve_slot_range("player.crafting.2"),
            Some(&[502usize][..])
        );
        assert_eq!(resolve_slot_range("mob.inventory.7"), Some(&[307usize][..]));
    }

    #[test]
    fn keeps_generated_slot_names() {
        assert_eq!(resolve_slot_range("armor.head"), Some(&[103usize][..]));
        assert_eq!(resolve_slot_range("horse.0"), Some(&[500usize][..]));
        assert_eq!(resolve_slot_range("bogus"), None);
    }

    #[test]
    fn wildcard_ranges_resolve_to_multiple_slots() {
        // The one-to-many item replace path relies on ranges expanding to every slot.
        if let Some(slots) = resolve_slot_range("container.*") {
            assert!(slots.len() > 1, "container.* must expand to many slots");
            assert_eq!(slots.first(), Some(&0));
        }
        if let Some(slots) = resolve_slot_range("armor.*") {
            assert_eq!(slots.len(), 5);
        }
    }
}
