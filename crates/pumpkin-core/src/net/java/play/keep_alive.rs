#[allow(clippy::wildcard_imports)]
use super::*;
use std::time::Instant;

/// Paper `ServerCommonPacketListenerImpl#handleKeepAlive` accepts a reply only for
/// the oldest outstanding challenge (`pendingKeepAlives.peek()`). A reply to any other
/// challenge is out of order and disconnects with `disconnect.timeout`, exactly like
/// an unsolicited or duplicate id.
fn is_oldest_challenge(pending: &[(i64, Instant)], id: i64) -> bool {
    pending
        .first()
        .is_some_and(|(pending_id, _)| *pending_id == id)
}

impl JavaClient {
    pub fn handle_keep_alive(&self, player: &Player, keep_alive: &SKeepAlive) {
        let mut pending = self
            .pending_keep_alives
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if is_oldest_challenge(&pending, keep_alive.keep_alive_id) {
            let (_, send_time) = pending.remove(0);
            let ping = send_time.elapsed().as_millis() as u32;
            player.update_ping(ping);
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
    use super::is_oldest_challenge;
    use std::time::Instant;

    #[test]
    fn only_the_oldest_challenge_is_accepted() {
        let now = Instant::now();
        let pending = [(10, now), (20, now)];
        assert!(is_oldest_challenge(&pending, 10));
        assert!(!is_oldest_challenge(&pending, 20));
        assert!(!is_oldest_challenge(&pending, 30));
        assert!(!is_oldest_challenge(&[], 10));
    }
}
