//! Illustrated conversations with recurring travelers.
//!
//! Named speakers appear at forts/trading posts (data-driven roster) or as a nearby
//! recurring NPC train (a "neighboring wagon"). Every answer is grounded in real
//! [`GameState`] fields — the next route, current supplies, current weather — never
//! an invented forecast. [`ConversationMemory`] persists per speaker so a second
//! meeting is recognized and a one-time favor cannot be farmed by talking on repeat.
use crate::content::LandmarkKind;
use crate::state::{CommandError, GameState, Outcome, RationLevel};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConversationTopic {
    Route,
    Supplies,
    News,
}

/// Where a conversation is taking place, used to pick setting-specific scene art.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeakerSetting {
    Fort,
    Wagon,
}

/// Persisted memory of a speaker: recognizes prior meetings and gates the one-time favor.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationMemory {
    pub times_talked: u32,
    pub last_talked_day: Option<u32>,
    pub topics_discussed: BTreeSet<ConversationTopic>,
    pub favor_received: bool,
}

/// A named speaker the player can currently approach.
#[derive(Debug, Clone)]
pub struct SpeakerView {
    pub id: String,
    pub name: String,
    pub setting: SpeakerSetting,
}

impl GameState {
    /// Named speakers available right now, by setting: fort roster at a fort, or any
    /// recurring NPC train currently near the party (a neighboring wagon).
    pub fn available_speakers(&self) -> Vec<SpeakerView> {
        let mut speakers = Vec::new();
        if let Some(node) = self.current_landmark() {
            if node.kind == LandmarkKind::Fort {
                for def in self.content.speakers.iter().filter(|d| d.landmark_id == node.id) {
                    speakers.push(SpeakerView {
                        id: def.id.clone(),
                        name: def.name.clone(),
                        setting: SpeakerSetting::Fort,
                    });
                }
            }
        }
        for npc in self.npcs.iter().filter(|npc| self.npc_present(&npc.id)) {
            speakers.push(SpeakerView {
                id: npc.id.clone(),
                name: npc.name.clone(),
                setting: SpeakerSetting::Wagon,
            });
        }
        speakers
    }

    /// Approach a named speaker and ask about one grounded topic.
    pub fn converse(
        &mut self,
        speaker_id: &str,
        topic: ConversationTopic,
    ) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let speaker = self
            .available_speakers()
            .into_iter()
            .find(|s| s.id == speaker_id)
            .ok_or_else(|| CommandError::UnknownId(speaker_id.into()))?;
        let answer = self.topic_answer(topic);

        let memory = self.conversation_memory.entry(speaker_id.to_string()).or_default();
        let recognized = memory.times_talked > 0;
        let grant_favor = recognized && !memory.favor_received;
        memory.times_talked += 1;
        memory.last_talked_day = Some(self.day);
        memory.topics_discussed.insert(topic);
        if grant_favor {
            memory.favor_received = true;
        }

        let greeting = self.greeting_line(&speaker, recognized);
        let mut lines = vec![greeting, answer];
        let favor = if grant_favor {
            let line = self.grant_favor(&speaker);
            lines.push(line.clone());
            Some(line)
        } else {
            None
        };
        Ok(vec![Outcome::Conversation {
            speaker_id: speaker.id,
            speaker_name: speaker.name,
            setting: speaker.setting,
            recognized,
            topic,
            lines,
            favor,
        }])
    }

    fn topic_answer(&self, topic: ConversationTopic) -> String {
        match topic {
            ConversationTopic::Route => self.route_answer(),
            ConversationTopic::Supplies => self.supplies_answer(),
            ConversationTopic::News => self.news_answer(),
        }
    }

    fn route_answer(&self) -> String {
        if let Some(node) = self.current_landmark() {
            if node.routes.len() > 1 {
                let options = node
                    .routes
                    .iter()
                    .map(|route| {
                        format!(
                            "{} toward {} ({} miles)",
                            route.label,
                            self.landmark_name(&route.target_id),
                            route.distance_miles
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; or ");
                return format!("From here the trail forks: {options}.");
            }
            if let Some(route) = node.routes.first() {
                return format!(
                    "The road runs on to {}, {} miles further.",
                    self.landmark_name(&route.target_id),
                    route.distance_miles
                );
            }
        }
        if let Some(target) = &self.target_node_id {
            return format!(
                "Keep to the marked road; {} miles remain to {}.",
                self.route_miles_remaining,
                self.landmark_name(target)
            );
        }
        "The trail runs where it always has; watch for the ruts and you won't lose it.".into()
    }

    fn supplies_answer(&self) -> String {
        let food = self.inventory.get("food");
        let daily = self.daily_food_lbs().max(1);
        let days = food / daily;
        let ration = match self.rations {
            RationLevel::Filling => "filling",
            RationLevel::Meager => "meager",
            RationLevel::BareBones => "bare bones",
        };
        format!(
            "By my reckoning you're carrying {food} lbs of food, good for about {days} days at {ration} rations."
        )
    }

    fn news_answer(&self) -> String {
        let weather = weather_word(self.weather);
        match self.current_landmark() {
            Some(node) => format!(
                "Word here at {} is {weather} weather and {} miles behind you.",
                node.name, self.miles
            ),
            None => format!("Nothing to report but {weather} weather and open road."),
        }
    }

    /// Original, data-driven dialogue for a fort speaker's roster entry; a plain
    /// template for a wagon-train speaker, since `NpcTrain` names are code-seeded.
    fn greeting_line(&self, speaker: &SpeakerView, recognized: bool) -> String {
        if let SpeakerSetting::Fort = speaker.setting {
            if let Some(def) = self.content.speakers.iter().find(|d| d.id == speaker.id) {
                return if recognized {
                    def.returning_greeting.clone()
                } else {
                    def.greeting.clone()
                };
            }
        }
        if recognized {
            format!("{}: Back again, are you?", speaker.name)
        } else {
            format!("{}: Well met on the road.", speaker.name)
        }
    }

    fn landmark_name(&self, id: &str) -> String {
        self.content
            .trails
            .iter()
            .flat_map(|trail| trail.nodes.iter())
            .find(|node| node.id == id)
            .map_or_else(|| id.to_string(), |node| node.name.clone())
    }

    /// A small, bounded, one-time favor granted the first time a speaker is recognized
    /// on a return visit. `ConversationMemory::favor_received` prevents this from ever
    /// firing twice for the same speaker, even across repeated conversations or saves.
    fn grant_favor(&mut self, speaker: &SpeakerView) -> String {
        match speaker.setting {
            SpeakerSetting::Fort => {
                self.inventory.add("food", 15);
                let favor_text = self
                    .content
                    .speakers
                    .iter()
                    .find(|d| d.id == speaker.id)
                    .map_or("a sack of cornmeal for the road", |def| def.favor_text.as_str());
                format!("{} leaves you {favor_text}.", speaker.name)
            }
            SpeakerSetting::Wagon => {
                if let Some(npc) = self.npcs.iter_mut().find(|npc| npc.id == speaker.id) {
                    npc.reputation = npc.reputation.saturating_add(1);
                }
                format!("{} trusts you a little more for remembering them.", speaker.name)
            }
        }
    }
}

fn weather_word(weather: crate::content::WeatherKind) -> &'static str {
    use crate::content::WeatherKind;
    match weather {
        WeatherKind::Clear => "clear",
        WeatherKind::Warm => "warm",
        WeatherKind::Hot => "hot",
        WeatherKind::Rain => "rainy",
        WeatherKind::Storm => "stormy",
        WeatherKind::Snow => "snowy",
        WeatherKind::Cold => "cold",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{GameContent, RouteDefinition, SpeakerDefinition};
    use crate::state::RunStatus;

    fn fort_state() -> GameState {
        let mut content = GameContent::starter();
        content.trails[0].nodes[0].kind = LandmarkKind::Fort;
        content.trails[0].nodes[0].routes = vec![RouteDefinition {
            id: "main".into(),
            label: "Main trail".into(),
            target_id: "willamette".into(),
            distance_miles: 2_040,
        }];
        content.speakers.push(SpeakerDefinition {
            id: "orson".into(),
            name: "Orson Pike".into(),
            landmark_id: "independence".into(),
            greeting: "Well met on the road.".into(),
            returning_greeting: "Back again, are you?".into(),
            favor_text: "sends you off with cornmeal".into(),
        });
        let mut game = GameState::with_content(1, content);
        game.trail_id = Some("oregon".into());
        game.status = RunStatus::AtLandmark("independence".into());
        game.current_node_id = Some("independence".into());
        game.npcs.clear();
        game
    }

    #[test]
    fn fort_speakers_appear_only_at_their_landmark() {
        let game = fort_state();
        let names: Vec<_> = game.available_speakers().into_iter().map(|s| s.id).collect();
        assert_eq!(names, vec!["orson".to_string()]);
    }

    #[test]
    fn second_conversation_is_recognized_and_grants_one_favor_only() {
        let mut game = fort_state();
        let before_food = game.inventory.get("food");

        let first = game.converse("orson", ConversationTopic::Route).unwrap();
        let Outcome::Conversation { recognized, favor, .. } = &first[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(!recognized);
        assert!(favor.is_none());
        assert_eq!(game.inventory.get("food"), before_food);

        let second = game.converse("orson", ConversationTopic::Supplies).unwrap();
        let Outcome::Conversation { recognized, favor, .. } = &second[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(recognized);
        assert!(favor.is_some());
        assert_eq!(game.inventory.get("food"), before_food + 15);

        let third = game.converse("orson", ConversationTopic::News).unwrap();
        let Outcome::Conversation { favor, .. } = &third[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(favor.is_none(), "favor must not repeat on a third visit");
        assert_eq!(game.inventory.get("food"), before_food + 15);
    }

    #[test]
    fn route_answer_reflects_actual_trail_data() {
        let game = fort_state();
        let outcomes = game.clone().converse("orson", ConversationTopic::Route).unwrap();
        let Outcome::Conversation { lines, .. } = &outcomes[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(lines.iter().any(|line| line.contains("Willamette Valley")));
        assert!(lines.iter().any(|line| line.contains("2040")));
    }

    #[test]
    fn unknown_speaker_is_rejected() {
        let mut game = fort_state();
        assert!(matches!(
            game.converse("nobody", ConversationTopic::News),
            Err(CommandError::UnknownId(id)) if id == "nobody"
        ));
    }

    #[test]
    fn conversation_memory_round_trips_through_json() {
        let mut game = fort_state();
        game.converse("orson", ConversationTopic::Route).unwrap();
        game.converse("orson", ConversationTopic::Supplies).unwrap();
        let json = serde_json::to_string(&game).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.conversation_memory["orson"].times_talked, 2);
        assert!(restored.conversation_memory["orson"].favor_received);
    }

    #[test]
    fn old_saves_without_conversation_memory_still_load() {
        let game = fort_state();
        let mut value = serde_json::to_value(&game).unwrap();
        value.as_object_mut().unwrap().remove("conversation_memory");
        let restored: GameState = serde_json::from_value(value).unwrap();
        assert!(restored.conversation_memory.is_empty());
    }
}
