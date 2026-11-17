#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_command_suggestion(
        &self,
        player: &Arc<Player>,
        packet: &SCommandSuggestion<'_>,
        server: &Arc<Server>,
    ) {
        let Some(cmd) = &packet.command.get(1..) else {
            return;
        };

        // Vanilla completes the command name too, so a first word without a space is not skipped.
        let word_start = last_word_start(cmd);

        let suggestions = server
            .command_dispatcher
            .load()
            .suggest(cmd, &player.get_command_source(server));

        let response = CCommandSuggestions::new(
            packet.id,
            ((word_start + 1) as i32).into(),
            ((cmd.len() - word_start) as i32).into(),
            suggestions.into(),
        );

        player.try_send_client_packet(&response);
    }
}

/// Byte offset in `cmd` (the command without its leading `/`) where the word being completed starts.
fn last_word_start(cmd: &str) -> usize {
    cmd.char_indices()
        .rfind(|(_, c)| c.is_whitespace())
        .map_or(0, |(i, c)| i + c.len_utf8())
}

#[cfg(test)]
mod tests {
    use super::last_word_start;

    #[test]
    fn first_word_starts_at_zero() {
        assert_eq!(last_word_start("he"), 0);
        assert_eq!(last_word_start(""), 0);
    }

    #[test]
    fn argument_starts_after_last_space() {
        assert_eq!(last_word_start("give a"), 5);
        assert_eq!(last_word_start("give  a"), 6);
    }
}
