use crate::command::{
    CommandSource,
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    errors::error_types::CommandErrorType,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};
use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

pub(crate) const INVALID_DISPLAY_SLOT_ERROR: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_SCOREBOARDDISPLAYSLOT_INVALID,
    translation::java::ARGUMENT_SCOREBOARDDISPLAYSLOT_INVALID,
);

/// Every slot name accepted by vanilla's `ScoreboardSlot.byName`.
const SLOT_NAMES: [(&str, ScoreboardDisplaySlot); 19] = [
    ("list", ScoreboardDisplaySlot::List),
    ("sidebar", ScoreboardDisplaySlot::Sidebar),
    ("below_name", ScoreboardDisplaySlot::BelowName),
    ("sidebar.team.black", ScoreboardDisplaySlot::TeamBlack),
    ("sidebar.team.dark_blue", ScoreboardDisplaySlot::TeamDarkBlue),
    ("sidebar.team.dark_green", ScoreboardDisplaySlot::TeamDarkGreen),
    ("sidebar.team.dark_aqua", ScoreboardDisplaySlot::TeamDarkAqua),
    ("sidebar.team.dark_red", ScoreboardDisplaySlot::TeamDarkRed),
    ("sidebar.team.dark_purple", ScoreboardDisplaySlot::TeamDarkPurple),
    ("sidebar.team.gold", ScoreboardDisplaySlot::TeamGold),
    ("sidebar.team.gray", ScoreboardDisplaySlot::TeamGray),
    ("sidebar.team.dark_gray", ScoreboardDisplaySlot::TeamDarkGray),
    ("sidebar.team.blue", ScoreboardDisplaySlot::TeamBlue),
    ("sidebar.team.green", ScoreboardDisplaySlot::TeamGreen),
    ("sidebar.team.aqua", ScoreboardDisplaySlot::TeamAqua),
    ("sidebar.team.red", ScoreboardDisplaySlot::TeamRed),
    ("sidebar.team.light_purple", ScoreboardDisplaySlot::TeamLightPurple),
    ("sidebar.team.yellow", ScoreboardDisplaySlot::TeamYellow),
    ("sidebar.team.white", ScoreboardDisplaySlot::TeamWhite),
];

/// Parses the display slot names of vanilla's `ScoreboardSlotArgumentType`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ScoreboardSlotArgumentType;

impl ArgumentType<CommandSource> for ScoreboardSlotArgumentType {
    type Item = ScoreboardDisplaySlot;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let name = reader.read_unquoted_string();
        SLOT_NAMES
            .iter()
            .find_map(|(slot_name, slot)| (*slot_name == name.as_str()).then_some(*slot))
            .ok_or_else(|| {
                INVALID_DISPLAY_SLOT_ERROR.create_without_context(TextComponent::text(name))
            })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ScoreboardSlot
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> Suggestions {
        for (slot_name, _) in SLOT_NAMES {
            builder = builder.filter_and_suggest_one(slot_name);
        }
        builder.build()
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "sidebar".to_string(),
            "list".to_string(),
            "below_name".to_string(),
        ]
    }
}

impl ScoreboardSlotArgumentType {
    pub fn get(
        context: &CommandContext,
        name: &str,
    ) -> Result<ScoreboardDisplaySlot, CommandSyntaxError> {
        Ok(*context.get_argument::<ScoreboardDisplaySlot>(name)?)
    }
}

/// Returns the canonical name vanilla uses for a display slot.
#[must_use]
pub fn slot_name(slot: ScoreboardDisplaySlot) -> &'static str {
    SLOT_NAMES
        .iter()
        .find_map(|(slot_name, known)| (*known == slot).then_some(*slot_name))
        .unwrap_or("sidebar")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vanilla_slot_names() {
        let mut reader = StringReader::new("sidebar.team.red");
        assert_eq!(
            ScoreboardSlotArgumentType.parse(&mut reader),
            Ok(ScoreboardDisplaySlot::TeamRed)
        );
        let mut reader = StringReader::new("below_name");
        assert_eq!(
            ScoreboardSlotArgumentType.parse(&mut reader),
            Ok(ScoreboardDisplaySlot::BelowName)
        );
    }
}
