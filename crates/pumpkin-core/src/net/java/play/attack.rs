#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_attack(&self, player: &Arc<Player>, attack: &SAttack, server: &Arc<Server>) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();
        let entity_id = attack.entity_id;
        let player_entity = &player.get_entity();
        let world = player_entity.world.load_full();

        let player_target = world.get_player_by_id(entity_id.0);
        let target: Option<Arc<dyn EntityBase>> = player_target
            .as_ref()
            .map(|p| Arc::clone(p) as Arc<dyn EntityBase>)
            .or_else(|| world.get_entity_by_id(entity_id.0));
        // Vanilla guards the whole attack branch with `target != null` and ignores ids it
        // cannot resolve, so damage-less targets (e.g. ender dragon parts) do not kick.
        let Some(target) = target else {
            return;
        };

        let event = PlayerInteractEntityEvent::new(
            player,
            target,
            ActionType::Attack,
            None,
            player_entity.is_sneaking(),
        );
        self.handle_entity_interaction(player, event, None, server);
    }
}
