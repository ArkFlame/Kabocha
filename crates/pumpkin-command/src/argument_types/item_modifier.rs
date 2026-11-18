use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::identifier::Identifier;

use crate::{
    argument_types::{
        FromStringReader,
        argument_type::{ArgumentType, JavaClientArgumentType},
    },
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    snbt::SnbtParser,
    string_reader::StringReader,
};

/// An item modifier given either by its registry id or inline, like vanilla's `ResourceOrInlineArgument`.
#[derive(Clone, Debug)]
pub enum ItemModifier {
    Reference(Identifier),
    Inline(NbtTag),
}

/// Parses a [`ItemModifier`].
pub struct ItemModifierArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for ItemModifierArgumentType {
    type Item = ItemModifier;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if matches!(reader.peek(), Some('{' | '[')) {
            SnbtParser::parse_for_commands(reader).map(ItemModifier::Inline)
        } else {
            Identifier::from_reader(reader).map(ItemModifier::Reference)
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::LootModifier
    }

    fn examples(&self) -> Vec<String> {
        examples!("foo:bar", "{function:\"minecraft:set_count\",count:2}")
    }
}

impl ItemModifierArgumentType {
    pub fn get<'a, S: crate::source::CommandSource>(
        context: &'a CommandContext<S>,
        name: &str,
    ) -> Result<&'a ItemModifier, CommandSyntaxError> {
        context.get_argument(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::DummySource;

    // Vanilla's ResourceOrIdArgument reads SNBT compounds inline and ids as references.
    #[test]
    fn parses_reference_or_inline() {
        let mut reference = StringReader::new("minecraft:set_count");
        let parsed = ArgumentType::<DummySource>::parse(&ItemModifierArgumentType, &mut reference);
        assert!(matches!(parsed, Ok(ItemModifier::Reference(_))));

        let mut inline = StringReader::new("{}");
        let parsed = ArgumentType::<DummySource>::parse(&ItemModifierArgumentType, &mut inline);
        assert!(matches!(parsed, Ok(ItemModifier::Inline(_))));
    }
}
