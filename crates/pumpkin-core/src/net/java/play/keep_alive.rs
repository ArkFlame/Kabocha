#[allow(clippy::wildcard_imports)]
use super::*;
use std::time::Instant;

/// Vanilla `ServerCommonPacketListenerImpl#handleKeepAlive` only accepts a reply
/// that matches an outstanding challenge; anything else disconnects with
/// `disconnect.timeout` (Paper) instead of being ignored.
fn outstanding_keep_alive(pending: &[(i64, Instant)], id: i64) -> Option<usize> {
    pending.iter().position(|(pending_id, _)| *pending_id == id)
}

impl JavaClient {
    pub fn handle_keep_alive(&self, player: &Player, keep_alive: &SKeepAlive) {
        let mut pending = self
            .pending_keep_alives
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(pos) = outstanding_keep_alive(&pending, keep_alive.keep_alive_id) {
            let (_, send_time) = pending.swap_remove(pos);
            let ping = send_time.elapsed().as_millis() as u32;
            // Vanilla logic
            player.ping.store(
                (player.ping.load(Ordering::Relaxed) * 3 + ping) / 4,
                Ordering::Relaxed,
            );
            self.wait_for_keep_alive.store(false, Ordering::Relaxed);
        } else {
            // Vanilla `ServerGamePacketListenerImpl#handleKeepAlive` disconnects on an
            // unsolicited, stale or duplicate keep-alive reply instead of ignoring it.
            drop(pending);
            debug!(
                "Unexpected or duplicate keep alive id {} from player {}",
                keep_alive.keep_alive_id, player.gameprofile.name
            );
            self.try_kick(&pumpkin_macros::translate_cross!(
                pumpkin_data::translation::java::DISCONNECT_TIMEOUT,
                pumpkin_data::translation::bedrock::DISCONNECT_TIMEOUT
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::outstanding_keep_alive;
    use std::time::Instant;

    #[test]
    fn only_an_outstanding_challenge_is_accepted() {
        let now = Instant::now();
        let pending = [(10, now), (20, now)];
        assert_eq!(outstanding_keep_alive(&pending, 10), Some(0));
        assert_eq!(outstanding_keep_alive(&pending, 20), Some(1));
        assert_eq!(outstanding_keep_alive(&pending, 30), None);
        assert_eq!(outstanding_keep_alive(&[], 10), None);
    }
}
