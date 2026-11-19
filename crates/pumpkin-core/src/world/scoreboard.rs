use rustc_hash::FxHashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::Ordering;

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::{
    BClientPacket, ClientPacket, NumberFormat,
    bedrock::client::{
        remove_objective::CRemoveObjective as BRemoveObjective,
        set_display_objective::CSetDisplayObjective as BSetDisplayObjective,
        set_score::{CSetScore as BSetScore, ScoreEntry as BScoreEntry},
    },
    codec::var_int::VarInt,
    java::client::play::{
        CDisplayObjective, CResetScore, CSetPlayerTeam, CUpdateObjectives, CUpdateScore, Mode,
        RenderType, TeamMethod, TeamParameters,
    },
};
use pumpkin_util::text::{TextComponent, color::NamedColor};
use tracing::warn;

use super::World;
use crate::{entity::player::Player, net::bedrock::BedrockClient};

#[cfg(test)]
mod tests;

pub trait ScoreboardTarget: Send + Sync {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    );
    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J);
}

impl ScoreboardTarget for World {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        ScoreboardTarget::send_je(self, je_packet);
        self.broadcast_bedrock_all(be_packet);
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        let players = self.players.load();
        Self::broadcast_java_players(
            je_packet,
            players
                .iter()
                .filter(|player| player.scoreboard_synced.load(Ordering::Relaxed)),
        );
    }
}

impl<T: ScoreboardTarget + ?Sized> ScoreboardTarget for &T {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        (*self).send_editioned(je_packet, be_packet);
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        (*self).send_je(je_packet);
    }
}

impl ScoreboardTarget for std::sync::Arc<World> {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        ScoreboardTarget::send_editioned(self.as_ref(), je_packet, be_packet);
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        ScoreboardTarget::send_je(self.as_ref(), je_packet);
    }
}

impl ScoreboardTarget for Player {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        // Before the initial scoreboard state is sent on join, changes are only stored;
        // the full state sent when the player spawns includes them.
        if self.scoreboard_synced.load(Ordering::Relaxed) {
            self.try_enqueue_packet_editioned(je_packet, be_packet);
        }
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        if self.scoreboard_synced.load(Ordering::Relaxed) {
            self.try_send_client_packet(je_packet);
        }
    }
}

impl ScoreboardTarget for std::sync::Arc<Player> {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        // Before the initial scoreboard state is sent on join, changes are only stored;
        // the full state sent when the player spawns includes them.
        if self.scoreboard_synced.load(Ordering::Relaxed) {
            self.try_enqueue_packet_editioned(je_packet, be_packet);
        }
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        if self.scoreboard_synced.load(Ordering::Relaxed) {
            self.try_send_client_packet(je_packet);
        }
    }
}

pub struct NoTarget;

impl ScoreboardTarget for NoTarget {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        _je_packet: &J,
        _be_packet: &B,
    ) {
    }

    fn send_je<J: ClientPacket + Sync>(&self, _je_packet: &J) {}
}

#[derive(Clone, Debug, Default)]
pub struct Scoreboard {
    objectives: FxHashMap<String, ScoreboardObjective>,
    display_slots: FxHashMap<ScoreboardDisplaySlot, String>,
    scores: FxHashMap<String, FxHashMap<String, ScoreboardScore>>,
    /// Scores of removed or replaced objectives, kept as vanilla `PlayerScores` does.
    /// Each entry is keyed by the objective instance (identity), not by name.
    orphan_scores: Vec<OrphanScore>,
    teams: FxHashMap<String, Team>,
}

#[derive(Clone, Debug)]
struct OrphanScore {
    objective: ScoreboardObjective,
    entity_name: String,
    value: i32,
}

impl Scoreboard {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn get_objectives(&self) -> &FxHashMap<String, ScoreboardObjective> {
        &self.objectives
    }

    #[must_use]
    pub fn get_objective(&self, name: &str) -> Option<&ScoreboardObjective> {
        self.objectives.get(name)
    }

    #[must_use]
    pub const fn get_display_slots(&self) -> &FxHashMap<ScoreboardDisplaySlot, String> {
        &self.display_slots
    }

    #[must_use]
    pub fn get_display_objective(&self, slot: ScoreboardDisplaySlot) -> Option<&str> {
        self.display_slots.get(&slot).map(String::as_str)
    }

    #[must_use]
    pub const fn get_scores(&self) -> &FxHashMap<String, FxHashMap<String, ScoreboardScore>> {
        &self.scores
    }

    #[must_use]
    pub fn get_score(&self, entity_name: &str, objective_name: &str) -> Option<&ScoreboardScore> {
        self.scores.get(objective_name)?.get(entity_name)
    }

    #[must_use]
    pub fn get_score_value(&self, entity_name: &str, objective_name: &str) -> Option<i32> {
        self.get_score(entity_name, objective_name)
            .map(|s| s.value.0)
    }

    #[must_use]
    pub fn get_scores_for_objective(
        &self,
        objective_name: &str,
    ) -> Option<&FxHashMap<String, ScoreboardScore>> {
        self.scores.get(objective_name)
    }

    #[must_use]
    pub fn get_scores_for_entity(&self, entity_name: &str) -> FxHashMap<String, &ScoreboardScore> {
        let mut entity_scores = FxHashMap::default();
        for (obj_name, obj_scores) in &self.scores {
            if let Some(score) = obj_scores.get(entity_name) {
                entity_scores.insert(obj_name.clone(), score);
            }
        }
        entity_scores
    }

    #[must_use]
    pub const fn get_teams(&self) -> &FxHashMap<String, Team> {
        &self.teams
    }

    #[must_use]
    pub fn get_team(&self, name: &str) -> Option<&Team> {
        self.teams.get(name)
    }

    pub fn get_team_mut(&mut self, name: &str) -> Option<&mut Team> {
        self.teams.get_mut(name)
    }

    #[must_use]
    pub fn get_entity_team(&self, entity_name: &str) -> Option<&Team> {
        self.teams
            .values()
            .find(|team| team.players.iter().any(|p| p == entity_name))
    }

    pub fn add_objective(
        &mut self,
        target: &impl ScoreboardTarget,
        objective: ScoreboardObjective,
    ) {
        if self.objectives.contains_key(&objective.name) {
            warn!(
                "Tried to create an objective which already exists: {}",
                &objective.name
            );
            return;
        }

        let je_update = CUpdateObjectives::new(
            objective.name.clone(),
            Mode::Add,
            objective.display_name.clone(),
            objective.render_type,
            objective.number_format.clone(),
        );

        // Adding an objective never writes a display slot in vanilla. A Bedrock
        // SetDisplayObjective(sidebar) here would reveal an undisplayed objective
        // and clobber the real Bedrock sidebar; send_to_player sends the actual
        // displays from the recorded slots.
        target.send_je(&je_update);

        self.objectives.insert(objective.name.clone(), objective);
    }

    pub fn update_objective(
        &mut self,
        target: &impl ScoreboardTarget,
        mut objective: ScoreboardObjective,
    ) {
        let Some(identity) = self
            .objectives
            .get(&objective.name)
            .map(|existing| existing.identity.clone())
        else {
            warn!(
                "Tried to update an objective which does not exist: {}",
                &objective.name
            );
            return;
        };
        objective.identity = identity;

        let je_update = CUpdateObjectives::new(
            objective.name.clone(),
            Mode::Update,
            objective.display_name.clone(),
            objective.render_type,
            objective.number_format.clone(),
        );

        // Only an explicit set_display_objective writes a Bedrock display slot.
        // Sending SetDisplayObjective(sidebar) on update would clobber the real
        // sidebar and show an otherwise-hidden objective.
        target.send_je(&je_update);

        self.objectives.insert(objective.name.clone(), objective);
    }

    pub fn set_display_objective(
        &mut self,
        target: &impl ScoreboardTarget,
        slot: ScoreboardDisplaySlot,
        objective_name: Option<&str>,
    ) {
        let obj_name_str = objective_name.unwrap_or("");

        let display_name = objective_name
            .and_then(|name| self.objectives.get(name))
            .map_or_else(
                || obj_name_str.to_string(),
                |o| o.display_name.clone().get_text(),
            );

        let je_display = CDisplayObjective::new(slot, obj_name_str.to_string());
        if let Some(be_slot) = bedrock_display_slot(slot) {
            let be_display = BSetDisplayObjective {
                display_slot_name: be_slot.to_str().to_string(),
                objective_name: obj_name_str.to_string(),
                objective_display_name: display_name,
                criteria_name: "dummy".to_string(),
                sort_order: VarInt(0),
            };
            target.send_editioned(&je_display, &be_display);
        } else {
            // A team-colour slot is Java-only; sending it to Bedrock as "sidebar"
            // would replace the real Bedrock sidebar.
            target.send_je(&je_display);
        }

        if let Some(name) = objective_name {
            self.display_slots.insert(slot, name.to_string());
        } else {
            self.display_slots.remove(&slot);
        }
    }

    pub fn clear_display_objective(
        &mut self,
        target: &impl ScoreboardTarget,
        slot: ScoreboardDisplaySlot,
    ) {
        self.set_display_objective(target, slot, None);
    }

    pub fn remove_objective(&mut self, target: &impl ScoreboardTarget, name: &str) {
        if !self.objectives.contains_key(name) {
            warn!(
                "Tried to remove an objective which does not exist: {}",
                name
            );
            return;
        }

        let je_packet = CUpdateObjectives::new(
            name.to_string(),
            Mode::Remove,
            TextComponent::empty(),
            RenderType::Integer,
            None,
        );

        let be_packet = BRemoveObjective {
            objective_name: name.to_string(),
        };

        target.send_editioned(&je_packet, &be_packet);

        if let Some(removed) = self.objectives.remove(name) {
            // Vanilla `PlayerScores#remove` drops the live scores but keeps the orphan Score
            // objects that stored-score commands already hold, keyed by the removed objective.
            if let Some(scores) = self.scores.remove(name) {
                for (entity_name, score) in scores {
                    self.orphan_scores.push(OrphanScore {
                        objective: removed.clone(),
                        entity_name,
                        value: score.value.0,
                    });
                }
            }
        }
        self.scores.remove(name);
        self.display_slots.retain(|_, obj| obj != name);
    }

    /// Writes a score for an objective instance that is no longer live (vanilla orphan score).
    pub fn set_orphan_score(
        &mut self,
        objective: &ScoreboardObjective,
        entity_name: impl Into<String>,
        value: i32,
    ) {
        let entity_name = entity_name.into();
        if let Some(existing) = self
            .orphan_scores
            .iter_mut()
            .find(|o| o.objective.is_same(objective) && o.entity_name == entity_name)
        {
            existing.value = value;
        } else {
            self.orphan_scores.push(OrphanScore {
                objective: objective.clone(),
                entity_name,
                value,
            });
        }
    }

    /// Returns the retained orphan score for an objective instance and holder.
    #[must_use]
    pub fn get_orphan_score(
        &self,
        objective: &ScoreboardObjective,
        entity_name: &str,
    ) -> Option<i32> {
        self.orphan_scores
            .iter()
            .find(|o| o.objective.is_same(objective) && o.entity_name == entity_name)
            .map(|o| o.value)
    }

    pub fn update_score(&mut self, target: &impl ScoreboardTarget, score: ScoreboardScore) {
        if !self.objectives.contains_key(&score.objective_name) {
            warn!(
                "Tried to place a score into an objective which does not exist: {}",
                &score.objective_name
            );
            return;
        }

        let je_packet = CUpdateScore::new(
            score.entity_name.clone(),
            score.objective_name.clone(),
            score.value,
            score.display_name.clone(),
            score.number_format.clone(),
        );

        let be_packet = BSetScore {
            action: VarInt(0), // Change
            entries: vec![bedrock_score_entry(
                &score.entity_name,
                &score.objective_name,
                score.value.0,
            )],
        };

        target.send_editioned(&je_packet, &be_packet);

        self.scores
            .entry(score.objective_name.clone())
            .or_default()
            .insert(score.entity_name.clone(), score);
    }

    pub fn set_score_value(
        &mut self,
        target: &impl ScoreboardTarget,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        value: i32,
    ) {
        let entity_s = entity_name.into();
        let obj_s = objective_name.into();
        let existing = self.get_score(&entity_s, &obj_s).cloned();
        let score = ScoreboardScore {
            entity_name: entity_s,
            objective_name: obj_s,
            value: VarInt(value),
            display_name: existing.as_ref().and_then(|s| s.display_name.clone()),
            number_format: existing.as_ref().and_then(|s| s.number_format.clone()),
            locked: existing.as_ref().is_none_or(|s| s.locked),
        };
        self.update_score(target, score);
    }

    pub fn add_score(
        &mut self,
        target: &impl ScoreboardTarget,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        delta: i32,
    ) -> i32 {
        let entity_s = entity_name.into();
        let obj_s = objective_name.into();
        let current_val = self.get_score_value(&entity_s, &obj_s).unwrap_or(0);
        let new_val = current_val.wrapping_add(delta);
        self.set_score_value(target, entity_s, obj_s, new_val);
        new_val
    }

    pub fn remove_score(
        &mut self,
        target: &impl ScoreboardTarget,
        entity_name: &str,
        objective_name: &str,
    ) {
        // Vanilla `ServerScoreboard.removePlayerScore` only announces removals of scores that exist.
        let existed = self
            .scores
            .get_mut(objective_name)
            .is_some_and(|objective_scores| objective_scores.remove(entity_name).is_some());
        if !existed {
            return;
        }

        let je_packet = CResetScore::new(entity_name.to_string(), Some(objective_name.to_string()));

        let be_packet = BSetScore {
            action: VarInt(1), // Remove
            entries: vec![bedrock_score_entry(entity_name, objective_name, 0)],
        };

        target.send_editioned(&je_packet, &be_packet);
    }

    pub fn reset_scores_for_entity(&mut self, target: &impl ScoreboardTarget, entity_name: &str) {
        let je_packet = CResetScore::new(entity_name.to_string(), None);

        let mut be_entries = Vec::new();
        for (obj_name, obj_scores) in &self.scores {
            if obj_scores.contains_key(entity_name) {
                be_entries.push(bedrock_score_entry(entity_name, obj_name, 0));
            }
        }

        let be_packet = BSetScore {
            action: VarInt(1), // Remove
            entries: be_entries,
        };

        target.send_editioned(&je_packet, &be_packet);

        for obj_scores in self.scores.values_mut() {
            obj_scores.remove(entity_name);
        }
    }

    pub fn add_team(&mut self, target: &impl ScoreboardTarget, team: Team) {
        if self.teams.contains_key(&team.name) {
            warn!("Tried to create Team which already exists, {}", team.name);
            return;
        }

        let parameters = TeamParameters {
            display_name: &team.display_name,
            options: team.options,
            nametag_visibility: team.nametag_visibility.to_str(),
            collision_rule: team.collision_rule.to_str(),
            color: team.color_id(),
            player_prefix: &team.player_prefix,
            player_suffix: &team.player_suffix,
        };

        target.send_je(&CSetPlayerTeam {
            team_name: team.name.clone(),
            method: TeamMethod::Create,
            parameters: Some(parameters),
            players: team.players.clone().into(),
        });

        self.teams.insert(team.name.clone(), team);
    }

    pub fn create_team(&mut self, target: &impl ScoreboardTarget, team: Team) {
        self.add_team(target, team);
    }

    pub fn update_team(&mut self, target: &impl ScoreboardTarget, team: Team) {
        if !self.teams.contains_key(&team.name) {
            warn!("Tried to update Team which does not exist, {}", team.name);
            return;
        }

        let parameters = TeamParameters {
            display_name: &team.display_name,
            options: team.options,
            nametag_visibility: team.nametag_visibility.to_str(),
            collision_rule: team.collision_rule.to_str(),
            color: team.color_id(),
            player_prefix: &team.player_prefix,
            player_suffix: &team.player_suffix,
        };

        target.send_je(&CSetPlayerTeam {
            team_name: team.name.clone(),
            method: TeamMethod::Update,
            parameters: Some(parameters),
            players: Box::new([]),
        });

        self.teams.insert(team.name.clone(), team);
    }

    pub fn remove_team(&mut self, target: &impl ScoreboardTarget, name: &str) {
        if !self.teams.contains_key(name) {
            warn!("Tried to remove Team which does not exist, {}", name);
            return;
        }

        target.send_je(&CSetPlayerTeam {
            team_name: name.to_string(),
            method: TeamMethod::Remove,
            parameters: None,
            players: Box::new([]),
        });

        self.teams.remove(name);
    }

    pub fn add_player_to_team(
        &mut self,
        target: &impl ScoreboardTarget,
        team_name: &str,
        player: String,
    ) {
        if !self.teams.contains_key(team_name) {
            warn!(
                "Tried to add player to Team which does not exist, {}",
                team_name
            );
            return;
        }

        // Vanilla `Scoreboard.addPlayerToTeam` removes the holder from any previous
        // team before adding to the target, and no-ops the add only if the holder was
        // already in the target team. A holder therefore never stays in two teams.
        let mut already_in_target = false;
        for team in self.teams.values_mut() {
            if team.name == team_name {
                already_in_target = team.players.iter().any(|member| member == &player);
            } else {
                team.players.retain(|member| member != &player);
            }
        }
        if already_in_target {
            return;
        }

        target.send_je(&CSetPlayerTeam {
            team_name: team_name.to_string(),
            method: TeamMethod::AddPlayers,
            parameters: None,
            players: vec![player.clone()].into(),
        });

        if let Some(team) = self.teams.get_mut(team_name) {
            team.players.push(player);
        }
    }

    pub fn remove_player_from_team(
        &mut self,
        target: &impl ScoreboardTarget,
        team_name: &str,
        player: &str,
    ) {
        let Some(team) = self.teams.get_mut(team_name) else {
            warn!(
                "Tried to remove player from Team which does not exist, {}",
                team_name
            );
            return;
        };

        if !team.players.iter().any(|p| p == player) {
            return;
        }

        target.send_je(&CSetPlayerTeam {
            team_name: team_name.to_string(),
            method: TeamMethod::RemovePlayers,
            parameters: None,
            players: vec![player.to_string()].into(),
        });

        team.players.retain(|p| p != player);
    }

    pub fn clear_team_players(&mut self, target: &impl ScoreboardTarget, team_name: &str) {
        let Some(team) = self.teams.get_mut(team_name) else {
            warn!(
                "Tried to clear players from Team which does not exist, {}",
                team_name
            );
            return;
        };

        if team.players.is_empty() {
            return;
        }

        let players_to_remove = team.players.clone();
        target.send_je(&CSetPlayerTeam {
            team_name: team_name.to_string(),
            method: TeamMethod::RemovePlayers,
            parameters: None,
            players: players_to_remove.into(),
        });

        team.players.clear();
    }

    pub fn send_to_player(&self, player: &Player) {
        for objective in self.objectives.values() {
            let je_update = CUpdateObjectives::new(
                objective.name.clone(),
                Mode::Add,
                objective.display_name.clone(),
                objective.render_type,
                objective.number_format.clone(),
            );
            // An objective add/update never displays itself on Bedrock: only an
            // explicit SetDisplayObjective does. The display-slot loop below sends
            // the real Bedrock displays.
            player.try_send_client_packet(&je_update);
        }

        for (slot, objective_name) in &self.display_slots {
            let display_name = self.objectives.get(objective_name).map_or_else(
                || objective_name.clone(),
                |o| o.display_name.clone().get_text(),
            );
            let je_display = CDisplayObjective::new(*slot, objective_name.clone());
            if let Some(be_slot) = bedrock_display_slot(*slot) {
                let be_display = BSetDisplayObjective {
                    display_slot_name: be_slot.to_str().to_string(),
                    objective_name: objective_name.clone(),
                    objective_display_name: display_name,
                    criteria_name: "dummy".to_string(),
                    sort_order: VarInt(0),
                };
                player.try_enqueue_packet_editioned(&je_display, &be_display);
            } else {
                player.try_send_client_packet(&je_display);
            }
        }

        for objective_scores in self.scores.values() {
            for score in objective_scores.values() {
                let je_packet = CUpdateScore::new(
                    score.entity_name.clone(),
                    score.objective_name.clone(),
                    score.value,
                    score.display_name.clone(),
                    score.number_format.clone(),
                );
                let be_packet = BSetScore {
                    action: VarInt(0),
                    entries: vec![bedrock_score_entry(
                        &score.entity_name,
                        &score.objective_name,
                        score.value.0,
                    )],
                };
                player.try_enqueue_packet_editioned(&je_packet, &be_packet);
            }
        }

        for team in self.teams.values() {
            let parameters = TeamParameters {
                display_name: &team.display_name,
                options: team.options,
                nametag_visibility: team.nametag_visibility.to_str(),
                collision_rule: team.collision_rule.to_str(),
                color: team.color_id(),
                player_prefix: &team.player_prefix,
                player_suffix: &team.player_suffix,
            };
            let je_packet = CSetPlayerTeam {
                team_name: team.name.clone(),
                method: TeamMethod::Create,
                parameters: Some(parameters),
                players: team.players.clone().into(),
            };
            player.try_send_client_packet(&je_packet);
        }
    }
}

/// Display slots with their vanilla serialized names, as used in `scoreboard.dat`.
const DISPLAY_SLOT_NAMES: [(ScoreboardDisplaySlot, &str); 19] = [
    (ScoreboardDisplaySlot::List, "list"),
    (ScoreboardDisplaySlot::Sidebar, "sidebar"),
    (ScoreboardDisplaySlot::BelowName, "below_name"),
    (ScoreboardDisplaySlot::TeamBlack, "sidebar.team.black"),
    (
        ScoreboardDisplaySlot::TeamDarkBlue,
        "sidebar.team.dark_blue",
    ),
    (
        ScoreboardDisplaySlot::TeamDarkGreen,
        "sidebar.team.dark_green",
    ),
    (
        ScoreboardDisplaySlot::TeamDarkAqua,
        "sidebar.team.dark_aqua",
    ),
    (ScoreboardDisplaySlot::TeamDarkRed, "sidebar.team.dark_red"),
    (
        ScoreboardDisplaySlot::TeamDarkPurple,
        "sidebar.team.dark_purple",
    ),
    (ScoreboardDisplaySlot::TeamGold, "sidebar.team.gold"),
    (ScoreboardDisplaySlot::TeamGray, "sidebar.team.gray"),
    (
        ScoreboardDisplaySlot::TeamDarkGray,
        "sidebar.team.dark_gray",
    ),
    (ScoreboardDisplaySlot::TeamBlue, "sidebar.team.blue"),
    (ScoreboardDisplaySlot::TeamGreen, "sidebar.team.green"),
    (ScoreboardDisplaySlot::TeamAqua, "sidebar.team.aqua"),
    (ScoreboardDisplaySlot::TeamRed, "sidebar.team.red"),
    (
        ScoreboardDisplaySlot::TeamLightPurple,
        "sidebar.team.light_purple",
    ),
    (ScoreboardDisplaySlot::TeamYellow, "sidebar.team.yellow"),
    (ScoreboardDisplaySlot::TeamWhite, "sidebar.team.white"),
];

/// Bedrock exposes only list/sidebar/belowname; `sidebar.team.*` slots are Java-only.
/// Sending a team slot as a Bedrock sidebar would clobber the real Bedrock sidebar.
const fn bedrock_display_slot(slot: ScoreboardDisplaySlot) -> Option<BedrockDisplaySlot> {
    match slot {
        ScoreboardDisplaySlot::List => Some(BedrockDisplaySlot::PlayerList),
        ScoreboardDisplaySlot::Sidebar => Some(BedrockDisplaySlot::Sidebar),
        ScoreboardDisplaySlot::BelowName => Some(BedrockDisplaySlot::BelowName),
        _ => None,
    }
}

/// Mirrors vanilla `NumberFormatTypes`: a tagged compound keyed by `type`.
/// `styled` is omitted because Pumpkin's `Style` has no NBT codec yet.
fn number_format_to_nbt(format: &NumberFormat) -> Option<NbtTag> {
    let mut nbt = NbtCompound::new();
    match format {
        NumberFormat::Blank => nbt.put_string("type", "blank".to_string()),
        NumberFormat::Fixed(text) => {
            nbt.put_string("type", "fixed".to_string());
            nbt.put("value", text.to_nbt_tag_for_version(&CURRENT_MC_VERSION));
        }
        NumberFormat::Styled(_) => return None,
    }
    Some(NbtTag::Compound(nbt))
}

fn number_format_from_nbt(tag: &NbtTag) -> Option<NumberFormat> {
    let NbtTag::Compound(nbt) = tag else {
        return None;
    };
    match nbt.get_string("type")? {
        "blank" => Some(NumberFormat::Blank),
        "fixed" => nbt
            .get("value")
            .map(|value| NumberFormat::Fixed(TextComponent::from_nbt(value))),
        _ => None,
    }
}

fn name_tag_visibility_from_str(value: Option<&str>) -> NameTagVisibility {
    match value {
        Some("never") => NameTagVisibility::Never,
        Some("hideForOtherTeams") => NameTagVisibility::HideForOtherTeams,
        Some("hideForOwnTeam") => NameTagVisibility::HideForOwnTeam,
        _ => NameTagVisibility::Always,
    }
}

fn collision_rule_from_str(value: Option<&str>) -> CollisionRule {
    match value {
        Some("never") => CollisionRule::Never,
        Some("pushOtherTeams") => CollisionRule::PushOtherTeams,
        Some("pushOwnTeam") => CollisionRule::PushOwnTeam,
        _ => CollisionRule::Always,
    }
}

fn objective_to_nbt(objective: &ScoreboardObjective) -> NbtTag {
    let mut nbt = NbtCompound::new();
    nbt.put_string("Name", objective.name.clone());
    nbt.put_string("CriteriaName", objective.criterion.clone());
    nbt.put(
        "DisplayName",
        objective
            .display_name
            .to_nbt_tag_for_version(&CURRENT_MC_VERSION),
    );
    let render_type = match objective.render_type {
        RenderType::Integer => "integer",
        RenderType::Hearts => "hearts",
    };
    nbt.put_string("RenderType", render_type.to_string());
    if let Some(format) = &objective.number_format
        && let Some(tag) = number_format_to_nbt(format)
    {
        nbt.put("format", tag);
    }
    NbtTag::Compound(nbt)
}

fn score_to_nbt(score: &ScoreboardScore) -> NbtTag {
    let mut nbt = NbtCompound::new();
    nbt.put_string("Name", score.entity_name.clone());
    nbt.put_string("Objective", score.objective_name.clone());
    nbt.put_int("Score", score.value.0);
    nbt.put_bool("Locked", score.locked);
    if let Some(display_name) = &score.display_name {
        nbt.put(
            "display",
            display_name.to_nbt_tag_for_version(&CURRENT_MC_VERSION),
        );
    }
    if let Some(format) = &score.number_format
        && let Some(tag) = number_format_to_nbt(format)
    {
        nbt.put("format", tag);
    }
    NbtTag::Compound(nbt)
}

fn team_to_nbt(team: &Team) -> NbtTag {
    let mut nbt = NbtCompound::new();
    nbt.put_string("Name", team.name.clone());
    nbt.put(
        "DisplayName",
        team.display_name
            .to_nbt_tag_for_version(&CURRENT_MC_VERSION),
    );
    if let Some(color) = team.color {
        nbt.put_string("TeamColor", color.name().to_string());
    }
    nbt.put_bool("AllowFriendlyFire", team.is_allow_friendly_fire());
    nbt.put_bool("SeeFriendlyInvisibles", team.can_see_friendly_invisibles());
    nbt.put(
        "MemberNamePrefix",
        team.player_prefix
            .to_nbt_tag_for_version(&CURRENT_MC_VERSION),
    );
    nbt.put(
        "MemberNameSuffix",
        team.player_suffix
            .to_nbt_tag_for_version(&CURRENT_MC_VERSION),
    );
    nbt.put_string(
        "NameTagVisibility",
        team.nametag_visibility.to_str().to_string(),
    );
    nbt.put_string(
        "DeathMessageVisibility",
        team.death_message_visibility().to_str().to_string(),
    );
    nbt.put_string("CollisionRule", team.collision_rule.to_str().to_string());
    nbt.put_list(
        "Players",
        team.players
            .iter()
            .map(|player| NbtTag::String(player.clone().into_boxed_str()))
            .collect(),
    );
    NbtTag::Compound(nbt)
}

impl Scoreboard {
    /// Serializes objectives, scores, display slots and teams into the `data`
    /// compound of vanilla's `scoreboard.dat`.
    #[must_use]
    pub fn to_saved_nbt(&self) -> NbtCompound {
        let objectives: Vec<NbtTag> = self.objectives.values().map(objective_to_nbt).collect();
        let scores: Vec<NbtTag> = self
            .scores
            .values()
            .flat_map(FxHashMap::values)
            .map(score_to_nbt)
            .collect();
        let teams: Vec<NbtTag> = self.teams.values().map(team_to_nbt).collect();

        let mut display_slots = NbtCompound::new();
        for (slot, name) in DISPLAY_SLOT_NAMES {
            if let Some(objective) = self.display_slots.get(&slot) {
                display_slots.put_string(name, objective.clone());
            }
        }

        let mut data = NbtCompound::new();
        data.put_list("Objectives", objectives);
        data.put_list("PlayerScores", scores);
        data.put_compound("DisplaySlots", display_slots);
        data.put_list("Teams", teams);
        data
    }

    /// Replaces the contents of this scoreboard with a `data` compound read from
    /// `scoreboard.dat`. Nothing is sent to clients, so call it before players join.
    pub fn load_saved_nbt(&mut self, data: &NbtCompound) {
        self.objectives.clear();
        self.scores.clear();
        self.display_slots.clear();
        self.teams.clear();
        self.orphan_scores.clear();

        for tag in data.get_list("Objectives").unwrap_or_default() {
            let NbtTag::Compound(nbt) = tag else { continue };
            let Some(name) = nbt.get_string("Name") else {
                continue;
            };
            let display_name = nbt.get("DisplayName").map_or_else(
                || TextComponent::text(name.to_string()),
                TextComponent::from_nbt,
            );
            let render_type = if nbt.get_string("RenderType") == Some("hearts") {
                RenderType::Hearts
            } else {
                RenderType::Integer
            };
            let criterion = nbt.get_string("CriteriaName").unwrap_or("dummy");
            let number_format = nbt.get("format").and_then(number_format_from_nbt);
            self.objectives.insert(
                name.to_string(),
                ScoreboardObjective::new(name, display_name, render_type, number_format, criterion),
            );
        }

        for tag in data.get_list("PlayerScores").unwrap_or_default() {
            let NbtTag::Compound(nbt) = tag else { continue };
            let (Some(entity_name), Some(objective_name)) =
                (nbt.get_string("Name"), nbt.get_string("Objective"))
            else {
                continue;
            };
            if !self.objectives.contains_key(objective_name) {
                continue;
            }
            let mut score = ScoreboardScore::new(
                entity_name,
                objective_name,
                VarInt(nbt.get_int("Score").unwrap_or(0)),
                nbt.get("display").map(TextComponent::from_nbt),
                nbt.get("format").and_then(number_format_from_nbt),
            );
            score.locked = nbt.get_bool("Locked").unwrap_or(true);
            self.scores
                .entry(objective_name.to_string())
                .or_default()
                .insert(entity_name.to_string(), score);
        }

        if let Some(slots) = data.get_compound("DisplaySlots") {
            for (slot, name) in DISPLAY_SLOT_NAMES {
                if let Some(objective) = slots.get_string(name)
                    && self.objectives.contains_key(objective)
                {
                    self.display_slots.insert(slot, objective.to_string());
                }
            }
        }

        self.load_teams(data.get_list("Teams").unwrap_or_default());
    }

    fn load_teams(&mut self, tags: &[NbtTag]) {
        for tag in tags {
            let NbtTag::Compound(nbt) = tag else { continue };
            let Some(name) = nbt.get_string("Name") else {
                continue;
            };
            let options = (if nbt.get_bool("AllowFriendlyFire").unwrap_or(true) {
                Team::ALLOW_FRIENDLY_FIRE
            } else {
                0
            }) | (if nbt.get_bool("SeeFriendlyInvisibles").unwrap_or(true) {
                Team::SEE_FRIENDLY_INVISIBLES
            } else {
                0
            });
            let team = Team {
                name: name.to_string(),
                display_name: nbt
                    .get("DisplayName")
                    .map_or_else(TextComponent::empty, TextComponent::from_nbt),
                options,
                nametag_visibility: name_tag_visibility_from_str(
                    nbt.get_string("NameTagVisibility"),
                ),
                collision_rule: collision_rule_from_str(nbt.get_string("CollisionRule")),
                color: nbt
                    .get_string("TeamColor")
                    .and_then(|color| NamedColor::try_from(color).ok()),
                death_message_visibility: name_tag_visibility_from_str(
                    nbt.get_string("DeathMessageVisibility"),
                ),
                player_prefix: nbt
                    .get("MemberNamePrefix")
                    .map_or_else(TextComponent::empty, TextComponent::from_nbt),
                player_suffix: nbt
                    .get("MemberNameSuffix")
                    .map_or_else(TextComponent::empty, TextComponent::from_nbt),
                players: nbt
                    .get_list("Players")
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|tag| match tag {
                        NbtTag::String(player) => Some(player.to_string()),
                        _ => None,
                    })
                    .collect(),
            };
            self.teams.insert(name.to_string(), team);
        }
    }
}

/// The resolved form of a criterion, mirroring vanilla `ObjectiveCriteria`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveCriterion {
    pub read_only: bool,
    pub render_type: RenderType,
}

const TEAM_COLORS: &[&str] = &[
    "black",
    "dark_blue",
    "dark_green",
    "dark_aqua",
    "dark_red",
    "dark_purple",
    "gold",
    "gray",
    "dark_gray",
    "blue",
    "green",
    "aqua",
    "red",
    "light_purple",
    "yellow",
    "white",
];

/// Resolves a criterion name the way vanilla `ObjectiveCriteria.byName` does.
///
/// Unknown names return `None`, which vanilla reports as `argument.criteria.invalid`.
/// Pumpkin has no stat registry, so colon-qualified stat names are accepted as
/// writable integer criteria.
#[must_use]
pub fn resolve_criterion(name: &str) -> Option<ObjectiveCriterion> {
    let read_only = match name {
        "health" | "food" | "air" | "armor" | "xp" | "level" => true,
        "dummy" | "trigger" | "deathCount" | "playerKillCount" | "totalKillCount" => false,
        _ if name
            .strip_prefix("teamkill.")
            .or_else(|| name.strip_prefix("killedByTeam."))
            .is_some_and(|color| TEAM_COLORS.contains(&color)) =>
        {
            false
        }
        _ if name.contains(':') => false,
        _ => return None,
    };
    Some(ObjectiveCriterion {
        read_only,
        render_type: if name == "health" {
            RenderType::Hearts
        } else {
            RenderType::Integer
        },
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreboardObjective {
    pub name: String,
    pub display_name: TextComponent,
    pub render_type: RenderType,
    pub number_format: Option<NumberFormat>,
    pub criterion: String,
    // Clones retain identity; recreating an objective with the same name gets a new token.
    identity: std::sync::Arc<()>,
}

impl ScoreboardObjective {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        display_name: TextComponent,
        render_type: RenderType,
        number_format: Option<NumberFormat>,
        criterion: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            display_name,
            render_type,
            number_format,
            criterion: criterion.into(),
            identity: std::sync::Arc::new(()),
        }
    }

    /// Returns whether both values refer to the same objective instance.
    pub(crate) fn is_same(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.identity, &other.identity)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreboardScore {
    pub entity_name: String,
    pub objective_name: String,
    pub value: VarInt,
    pub display_name: Option<TextComponent>,
    pub number_format: Option<NumberFormat>,
    pub locked: bool,
}

impl ScoreboardScore {
    #[must_use]
    pub fn new(
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        value: VarInt,
        display_name: Option<TextComponent>,
        number_format: Option<NumberFormat>,
    ) -> Self {
        Self {
            entity_name: entity_name.into(),
            objective_name: objective_name.into(),
            value,
            display_name,
            number_format,
            locked: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameTagVisibility {
    Always,
    Never,
    HideForOtherTeams,
    HideForOwnTeam,
}

impl NameTagVisibility {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::HideForOtherTeams => "hideForOtherTeams",
            Self::HideForOwnTeam => "hideForOwnTeam",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionRule {
    Always,
    Never,
    PushOtherTeams,
    PushOwnTeam,
}

impl CollisionRule {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::PushOtherTeams => "pushOtherTeams",
            Self::PushOwnTeam => "pushOwnTeam",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Team {
    pub name: String,
    pub display_name: TextComponent,
    pub options: i8,
    pub nametag_visibility: NameTagVisibility,
    pub collision_rule: CollisionRule,
    /// `None` is vanilla `ChatFormatting.RESET`, the default colour of a new team.
    pub color: Option<NamedColor>,
    pub death_message_visibility: NameTagVisibility,
    pub player_prefix: TextComponent,
    pub player_suffix: TextComponent,
    pub players: Vec<String>,
}

impl Team {
    pub(crate) const ALLOW_FRIENDLY_FIRE: i8 = 0x01;
    /// Wire id for vanilla `ChatFormatting.RESET`; the packet encoder maps it to 21.
    pub(crate) const RESET_COLOR_ID: i32 = -1;
    pub(crate) const SEE_FRIENDLY_INVISIBLES: i8 = 0x02;

    #[must_use]
    pub const fn is_allow_friendly_fire(&self) -> bool {
        self.options & Self::ALLOW_FRIENDLY_FIRE != 0
    }

    #[must_use]
    pub fn color_id(&self) -> i32 {
        self.color
            .map_or(Self::RESET_COLOR_ID, |color| color as i32)
    }

    #[must_use]
    pub const fn death_message_visibility(&self) -> NameTagVisibility {
        self.death_message_visibility
    }

    #[must_use]
    pub const fn can_see_friendly_invisibles(&self) -> bool {
        self.options & Self::SEE_FRIENDLY_INVISIBLES != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BedrockSortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BedrockDisplaySlot {
    PlayerList,
    Sidebar,
    BelowName,
}

impl BedrockDisplaySlot {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::PlayerList => "list",
            Self::Sidebar => "sidebar",
            Self::BelowName => "belowname",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BedrockObjective {
    pub name: String,
    pub display_name: String,
    pub sort_order: BedrockSortOrder,
}

#[derive(Clone, Debug, Default)]
pub struct BedrockScoreboard {
    pub objectives: FxHashMap<String, BedrockObjective>,
    pub display_slots: FxHashMap<BedrockDisplaySlot, String>,
    pub scores: FxHashMap<(String, String), i32>,
}

fn bedrock_score_entry(entity_name: &str, objective_name: &str, value: i32) -> BScoreEntry {
    // IDs must survive string reallocations, score changes and scoreboard replay.
    let mut hasher = DefaultHasher::new();
    (entity_name, objective_name).hash(&mut hasher);
    BScoreEntry {
        scoreboard_id: i64::from_ne_bytes(hasher.finish().to_ne_bytes()),
        objective_name: objective_name.to_owned(),
        score: VarInt(value),
        entry_type: VarInt(3), // Fake player / Literal
        entity_unique_id: 0,
        custom_name: entity_name.to_owned(),
    }
}

enum BedrockObjectivePacket {
    Remove(BRemoveObjective),
    Display(BSetDisplayObjective),
    Scores(BSetScore),
}

impl BedrockObjectivePacket {
    fn send(&self, client: &BedrockClient) {
        match self {
            Self::Remove(packet) => client.try_enqueue_client_packet(packet),
            Self::Display(packet) => client.try_enqueue_client_packet(packet),
            Self::Scores(packet) => client.try_enqueue_client_packet(packet),
        }
    }
}

impl BedrockScoreboard {
    fn objective_packets(&self, name: &str) -> Vec<BedrockObjectivePacket> {
        let objective = self.objectives.get(name);
        let mut slots: Vec<_> = self
            .display_slots
            .iter()
            .filter(|(_, objective_name)| objective_name.as_str() == name)
            .map(|(slot, _)| *slot)
            .collect();
        if slots.is_empty()
            && !self
                .display_slots
                .contains_key(&BedrockDisplaySlot::Sidebar)
        {
            slots.push(BedrockDisplaySlot::Sidebar);
        }

        // Bedrock objective changes require recreating the display and restoring its rows.
        let mut packets = vec![BedrockObjectivePacket::Remove(BRemoveObjective {
            objective_name: name.to_owned(),
        })];
        for slot in slots {
            packets.push(BedrockObjectivePacket::Display(BSetDisplayObjective {
                display_slot_name: slot.to_str().to_owned(),
                objective_name: name.to_owned(),
                objective_display_name: objective.map_or_else(
                    || name.to_owned(),
                    |objective| objective.display_name.clone(),
                ),
                criteria_name: "dummy".to_owned(),
                sort_order: VarInt(match objective.map(|objective| objective.sort_order) {
                    Some(BedrockSortOrder::Descending) => 1,
                    _ => 0,
                }),
            }));
        }
        packets.push(BedrockObjectivePacket::Scores(BSetScore {
            action: VarInt(0),
            entries: self
                .scores
                .iter()
                .filter(|((_, objective_name), _)| objective_name == name)
                .map(|((entity, objective), value)| bedrock_score_entry(entity, objective, *value))
                .collect(),
        }));
        packets
    }

    pub fn add_objective(&mut self, player: &Player, objective: BedrockObjective) {
        self.set_objective(player, objective, Mode::Add);
    }

    pub fn update_objective(&mut self, player: &Player, objective: BedrockObjective) {
        self.set_objective(player, objective, Mode::Update);
    }

    fn set_objective(&mut self, player: &Player, objective: BedrockObjective, mode: Mode) {
        player.try_send_client_packet(&CUpdateObjectives::new(
            objective.name.clone(),
            mode,
            TextComponent::text(objective.display_name.clone()),
            RenderType::Integer,
            None,
        ));
        let name = objective.name.clone();
        self.objectives.insert(name.clone(), objective);
        if let Some(client) = player.client.bedrock() {
            for packet in self.objective_packets(&name) {
                packet.send(client);
            }
        }
    }

    pub fn remove_objective(&mut self, player: &Player, name: &str) {
        let be_remove = BRemoveObjective {
            objective_name: name.to_string(),
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CUpdateObjectives::new(
                name.to_string(),
                pumpkin_protocol::java::client::play::Mode::Remove,
                TextComponent::text(""),
                pumpkin_protocol::java::client::play::RenderType::Integer,
                None,
            ),
            &be_remove,
        );
        self.objectives.remove(name);
        self.display_slots.retain(|_, v| v != name);
        self.scores.retain(|(_, obj), _| obj != name);
    }

    pub fn set_display_objective(
        &mut self,
        player: &Player,
        slot: BedrockDisplaySlot,
        objective_name: Option<&str>,
    ) {
        player.try_send_client_packet(&CDisplayObjective::new(
            match slot {
                BedrockDisplaySlot::PlayerList => ScoreboardDisplaySlot::List,
                BedrockDisplaySlot::Sidebar => ScoreboardDisplaySlot::Sidebar,
                BedrockDisplaySlot::BelowName => ScoreboardDisplaySlot::BelowName,
            },
            objective_name.unwrap_or("").to_owned(),
        ));

        if let Some(name) = objective_name {
            self.display_slots.insert(slot, name.to_owned());
            if let Some(client) = player.client.bedrock() {
                for packet in self.objective_packets(name) {
                    packet.send(client);
                }
            }
        } else {
            self.display_slots.remove(&slot);
            if let Some(client) = player.client.bedrock() {
                client.try_enqueue_client_packet(&BSetDisplayObjective {
                    display_slot_name: slot.to_str().to_owned(),
                    objective_name: String::new(),
                    objective_display_name: String::new(),
                    criteria_name: "dummy".to_owned(),
                    sort_order: VarInt(0),
                });
            }
        }
    }

    pub fn clear_display_objective(&mut self, player: &Player, slot: BedrockDisplaySlot) {
        self.set_display_objective(player, slot, None);
    }

    pub fn update_score(
        &mut self,
        player: &Player,
        entity_name: &str,
        objective_name: &str,
        value: i32,
    ) {
        let score = ScoreboardScore::new(
            entity_name.to_string(),
            objective_name.to_string(),
            VarInt(value),
            None,
            None,
        );
        let be_score = BSetScore {
            action: VarInt(0),
            entries: vec![bedrock_score_entry(entity_name, objective_name, value)],
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CUpdateScore::new(
                score.entity_name.clone(),
                score.objective_name.clone(),
                score.value,
                score.display_name.clone(),
                score.number_format,
            ),
            &be_score,
        );

        self.scores
            .insert((entity_name.to_string(), objective_name.to_string()), value);
    }

    pub fn add_score(
        &mut self,
        player: &Player,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        delta: i32,
    ) -> i32 {
        let entity_s = entity_name.into();
        let obj_s = objective_name.into();
        let current = self
            .scores
            .get(&(entity_s.clone(), obj_s.clone()))
            .copied()
            .unwrap_or(0);
        let new_val = current.wrapping_add(delta);
        self.update_score(player, &entity_s, &obj_s, new_val);
        new_val
    }

    pub fn remove_score(&mut self, player: &Player, entity_name: &str, objective_name: &str) {
        let be_score = BSetScore {
            action: VarInt(1),
            entries: vec![bedrock_score_entry(entity_name, objective_name, 0)],
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CResetScore::new(
                entity_name.to_string(),
                Some(objective_name.to_string()),
            ),
            &be_score,
        );
        self.scores
            .remove(&(entity_name.to_string(), objective_name.to_string()));
    }

    pub fn reset_scores_for_entity(&mut self, player: &Player, entity_name: &str) {
        let be_score = BSetScore {
            action: VarInt(1),
            entries: self
                .scores
                .iter()
                .filter(|((entity, _), _)| entity == entity_name)
                .map(|((entity, objective), value)| bedrock_score_entry(entity, objective, *value))
                .collect(),
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CResetScore::new(entity_name.to_string(), None),
            &be_score,
        );
        self.scores.retain(|(entity, _), _| entity != entity_name);
    }

    pub fn send_to_player(&self, player: &Player) {
        if let Some(client) = player.client.bedrock() {
            for name in self.objectives.keys() {
                for packet in self.objective_packets(name) {
                    packet.send(client);
                }
            }
            return;
        }

        for objective in self.objectives.values() {
            player.try_send_client_packet(&CUpdateObjectives::new(
                objective.name.clone(),
                Mode::Add,
                TextComponent::text(objective.display_name.clone()),
                RenderType::Integer,
                None,
            ));
        }
        for (slot, objective_name) in &self.display_slots {
            player.try_send_client_packet(&CDisplayObjective::new(
                match slot {
                    BedrockDisplaySlot::PlayerList => ScoreboardDisplaySlot::List,
                    BedrockDisplaySlot::Sidebar => ScoreboardDisplaySlot::Sidebar,
                    BedrockDisplaySlot::BelowName => ScoreboardDisplaySlot::BelowName,
                },
                objective_name.clone(),
            ));
        }
        for ((entity_name, objective_name), value) in &self.scores {
            player.try_send_client_packet(&CUpdateScore::new(
                entity_name.clone(),
                objective_name.clone(),
                VarInt(*value),
                None,
                None,
            ));
        }
    }
}

#[derive(Default, Debug)]
pub struct ScoreboardBuilder {
    scoreboard: Scoreboard,
}

impl ScoreboardBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn objective(mut self, name: impl Into<String>, display_name: TextComponent) -> Self {
        let obj = ScoreboardObjective::new(
            name.into(),
            display_name,
            RenderType::Integer,
            None,
            "dummy",
        );
        self.scoreboard.objectives.insert(obj.name.clone(), obj);
        self
    }

    #[must_use]
    pub fn objective_with_render(
        mut self,
        name: impl Into<String>,
        display_name: TextComponent,
        render_type: RenderType,
    ) -> Self {
        let obj = ScoreboardObjective::new(name.into(), display_name, render_type, None, "dummy");
        self.scoreboard.objectives.insert(obj.name.clone(), obj);
        self
    }

    #[must_use]
    pub fn display_slot(
        mut self,
        slot: ScoreboardDisplaySlot,
        objective_name: impl Into<String>,
    ) -> Self {
        self.scoreboard
            .display_slots
            .insert(slot, objective_name.into());
        self
    }

    #[must_use]
    pub fn score(
        mut self,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        value: i32,
    ) -> Self {
        let entity_s = entity_name.into();
        let obj_s = objective_name.into();
        let score =
            ScoreboardScore::new(entity_s.clone(), obj_s.clone(), VarInt(value), None, None);
        self.scoreboard
            .scores
            .entry(obj_s)
            .or_default()
            .insert(entity_s, score);
        self
    }

    #[must_use]
    pub fn team(mut self, team: Team) -> Self {
        self.scoreboard.teams.insert(team.name.clone(), team);
        self
    }

    #[must_use]
    pub fn build(self) -> Scoreboard {
        self.scoreboard
    }
}

#[derive(Default, Debug)]
pub struct BedrockScoreboardBuilder {
    scoreboard: BedrockScoreboard,
}

impl BedrockScoreboardBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn objective(
        mut self,
        name: impl Into<String>,
        display_name: impl Into<String>,
        sort_order: BedrockSortOrder,
    ) -> Self {
        let name = name.into();
        let obj = BedrockObjective {
            name: name.clone(),
            display_name: display_name.into(),
            sort_order,
        };
        self.scoreboard.objectives.insert(name, obj);
        self
    }

    #[must_use]
    pub fn display_slot(
        mut self,
        slot: BedrockDisplaySlot,
        objective_name: impl Into<String>,
    ) -> Self {
        self.scoreboard
            .display_slots
            .insert(slot, objective_name.into());
        self
    }

    #[must_use]
    pub fn score(
        mut self,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        value: i32,
    ) -> Self {
        self.scoreboard
            .scores
            .insert((entity_name.into(), objective_name.into()), value);
        self
    }

    #[must_use]
    pub fn build(self) -> BedrockScoreboard {
        self.scoreboard
    }
}

#[cfg(test)]
mod bedrock_slot_tests {
    use super::*;

    #[test]
    fn team_display_slots_have_no_bedrock_slot() {
        assert!(bedrock_display_slot(ScoreboardDisplaySlot::List).is_some());
        assert!(bedrock_display_slot(ScoreboardDisplaySlot::Sidebar).is_some());
        assert!(bedrock_display_slot(ScoreboardDisplaySlot::BelowName).is_some());
        assert!(bedrock_display_slot(ScoreboardDisplaySlot::TeamRed).is_none());
        assert!(bedrock_display_slot(ScoreboardDisplaySlot::TeamWhite).is_none());
    }
}

#[cfg(test)]
mod team_membership_tests {
    use super::*;

    #[must_use]
    fn empty_team(name: &str) -> Team {
        Team {
            name: name.to_string(),
            display_name: TextComponent::empty(),
            options: 0,
            nametag_visibility: NameTagVisibility::Always,
            collision_rule: CollisionRule::Always,
            color: None,
            death_message_visibility: NameTagVisibility::Always,
            player_prefix: TextComponent::empty(),
            player_suffix: TextComponent::empty(),
            players: Vec::new(),
        }
    }

    #[test]
    fn joining_a_team_removes_the_holder_from_its_previous_team() {
        let mut board = Scoreboard::new();
        let target = NoTarget;
        board.add_team(&target, empty_team("red"));
        board.add_team(&target, empty_team("blue"));
        board.add_player_to_team(&target, "red", "Alex".to_string());
        board.add_player_to_team(&target, "blue", "Alex".to_string());

        assert!(board.get_team("red").is_some_and(|t| t.players.is_empty()));
        assert!(
            board
                .get_team("blue")
                .is_some_and(|t| t.players.iter().any(|p| p.as_str() == "Alex"))
        );
    }

    #[test]
    fn rejoining_the_target_team_still_drops_a_stale_membership() {
        // A holder can only be in one team in vanilla; if the target already lists
        // the holder, a stale membership in another team must still be cleared.
        let mut board = Scoreboard::new();
        let target = NoTarget;
        board.add_team(&target, empty_team("red"));
        board.add_team(&target, empty_team("blue"));
        board.add_player_to_team(&target, "blue", "Alex".to_string());
        // Simulate a desynced state where Alex lingers in red.
        if let Some(red) = board.get_team_mut("red") {
            red.players.push("Alex".to_string());
        }
        board.add_player_to_team(&target, "blue", "Alex".to_string());

        assert!(board.get_team("red").is_some_and(|t| t.players.is_empty()));
        assert_eq!(
            board.get_team("blue").map(|t| t
                .players
                .iter()
                .filter(|p| p.as_str() == "Alex")
                .count()),
            Some(1)
        );
    }
}
