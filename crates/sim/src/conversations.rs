//! Illustrated conversations with recurring travelers.
//!
//! Named speakers appear at forts/trading posts (data-driven roster, only while
//! actually standing at that fort) or as a nearby recurring NPC train (a
//! "neighboring wagon"). Every answer is grounded in real [`GameState`] fields —
//! the party's live progress toward its chosen target, current supplies, real
//! trail/river data ahead — never an invented forecast. [`ConversationMemory`]
//! persists per speaker so a *later-day* return visit is recognized and a
//! one-time favor cannot be farmed by asking several topics in one sitting.
use crate::content::{LandmarkDefinition, LandmarkKind};
use crate::state::{CommandError, GameState, Outcome, RationLevel, RunStatus};
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
    /// Named speakers available right now, by setting: the fort roster while
    /// actually standing at that fort (and its store isn't closed this era), or
    /// any recurring NPC train currently near the party (a neighboring wagon).
    /// Empty during a pending decision or minigame, matching every other command
    /// that requires [`GameState::at_camp`].
    pub fn available_speakers(&self) -> Vec<SpeakerView> {
        let mut speakers = Vec::new();
        if self.active_minigame.is_some() || self.pending_event.is_some() {
            return speakers;
        }
        if let Some(node) = self.current_landmark() {
            let standing_here =
                matches!(&self.status, RunStatus::AtLandmark(id) if id.as_str() == node.id);
            let era_rules = self.era_rules();
            if standing_here
                && node.kind == LandmarkKind::Fort
                && !era_rules.unavailable_stores.contains(&node.id)
            {
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

    /// Approach a named speaker and ask about one grounded topic. Only reachable
    /// through [`crate::state::Command::Converse`] — this is the single mutation
    /// boundary for conversations, mirroring every other command handler.
    pub(crate) fn converse(
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
        // A favor recognizes an actual *return visit* on a later day, not a second
        // question asked in the same sitting; `favor_received` makes it one-shot.
        let is_later_day_return =
            memory.last_talked_day.is_some_and(|last_day| last_day < self.day);
        let favor_eligible = is_later_day_return && !memory.favor_received;
        memory.times_talked = memory.times_talked.saturating_add(1);
        memory.last_talked_day = Some(self.day);
        memory.topics_discussed.insert(topic);

        let has_traded_before = match speaker.setting {
            SpeakerSetting::Fort => true,
            // Wagon recognition must reflect a real prior trade, not just prior talk.
            SpeakerSetting::Wagon => self
                .npcs
                .iter()
                .any(|npc| npc.id == speaker.id && npc.last_reputation_day.is_some()),
        };

        let greeting = self.greeting_line(&speaker, recognized);
        let mut lines = vec![greeting, answer];
        let favor = if favor_eligible && has_traded_before {
            self.grant_favor(&speaker).inspect(|line| lines.push(line.clone()))
        } else {
            None
        };
        if favor.is_some() {
            self.conversation_memory.get_mut(speaker_id).expect("just inserted").favor_received =
                true;
        }
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

    /// Reports the party's live progress, not a fork's full posted distance: while
    /// actually mid-route, the chosen target and remaining miles take priority over
    /// whatever routes are still listed on the last landmark node.
    fn route_answer(&self) -> String {
        if self.route_miles_remaining > 0 {
            if let Some(target) = &self.target_node_id {
                return format!(
                    "Keep to the marked road; {} miles remain to {}.",
                    self.route_miles_remaining,
                    self.landmark_name(target)
                );
            }
        }
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

    /// Shares concrete, useful facts about whatever is actually ahead — a river's
    /// real width and depth, or whether the next fort's store is open this era —
    /// rather than restating the weather or mileage already visible on screen.
    /// Never claims to still be at a fort once the party has moved on.
    fn news_answer(&self) -> String {
        let weather = weather_word(self.weather);
        let ahead = self.target_node_id.clone().or_else(|| {
            let standing_here = matches!(&self.status, RunStatus::AtLandmark(_));
            if standing_here {
                self.current_landmark()
                    .and_then(|node| node.routes.first().map(|route| route.target_id.clone()))
            } else {
                None
            }
        });
        if let Some(node) = ahead.as_deref().and_then(|id| self.trail_node(id)) {
            if let Some(river) = &node.river {
                return format!(
                    "The {} ahead is about {} feet wide, with a usual depth of {} feet. Check conditions when you arrive.",
                    node.name, river.width_feet, river.depth_feet
                );
            }
            if node.kind == LandmarkKind::Fort {
                return if self.era_rules().unavailable_stores.contains(&node.id) {
                    format!(
                        "Travelers say {} keeps no store worth the stop this season.",
                        node.name
                    )
                } else {
                    format!("Travelers say {} still keeps a fair stock at its counter.", node.name)
                };
            }
        }
        format!("Not much news. Here at camp, the weather is {weather}.")
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

    /// Looks up a landmark only within the party's own chosen trail, so a shared id
    /// on another trail variant can never surface the wrong name.
    fn trail_node(&self, id: &str) -> Option<&LandmarkDefinition> {
        self.content
            .trails
            .iter()
            .find(|trail| Some(&trail.id) == self.trail_id.as_ref())
            .and_then(|trail| trail.nodes.iter().find(|node| node.id == id))
    }

    fn landmark_name(&self, id: &str) -> String {
        self.trail_node(id).map_or_else(|| id.to_string(), |node| node.name.clone())
    }

    /// A small, capacity-checked, one-time favor granted on a recognized later-day
    /// return visit. Never exceeds real wagon capacity or the item limit — if there
    /// is no room, no food is claimed and the one-shot flag is left untouched so
    /// the favor can still be offered once there's space. Returns `None` when
    /// nothing was actually granted.
    fn grant_favor(&mut self, speaker: &SpeakerView) -> Option<String> {
        match speaker.setting {
            SpeakerSetting::Fort => {
                let def = self.content.speakers.iter().find(|d| d.id == speaker.id)?;
                let favor_text = def.favor_text.clone();
                let wanted = def.favor_food_lbs;
                let amount = self.max_addable("food").unwrap_or(0).min(wanted);
                if amount == 0 {
                    return None;
                }
                self.inventory.add("food", amount);
                Some(format!("{} leaves you {amount} lbs of food: {favor_text}.", speaker.name))
            }
            SpeakerSetting::Wagon => {
                let npc = self.npcs.iter_mut().find(|npc| npc.id == speaker.id)?;
                npc.reputation = npc.reputation.saturating_add(1);
                Some(format!("{} trusts you more, remembering your past trade.", speaker.name))
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
    use crate::content::{
        EraRules, GameContent, RiverDefinition, RouteDefinition, SpeakerDefinition,
    };
    use crate::economy::NpcTrain;
    use crate::state::{Command, RunStatus};
    use std::collections::BTreeMap;

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
            favor_food_lbs: 15,
        });
        let mut game = GameState::with_content(1, content);
        game.trail_id = Some("oregon".into());
        game.era_id = Some("1848".into());
        game.status = RunStatus::AtLandmark("independence".into());
        game.current_node_id = Some("independence".into());
        game.npcs.clear();
        game
    }

    fn converse(game: &mut GameState, speaker_id: &str, topic: ConversationTopic) -> Vec<Outcome> {
        game.apply(Command::Converse { speaker_id: speaker_id.into(), topic })
    }

    #[test]
    fn fort_speakers_appear_only_at_their_landmark() {
        let game = fort_state();
        let names: Vec<_> = game.available_speakers().into_iter().map(|s| s.id).collect();
        assert_eq!(names, vec!["orson".to_string()]);
    }

    #[test]
    fn departed_fort_speakers_are_not_available_mid_leg() {
        let mut game = fort_state();
        // The party has left the fort; `current_node_id` still names it as the last
        // stop, but the party is no longer standing there.
        game.status = RunStatus::Travelling;
        game.target_node_id = Some("willamette".into());
        game.route_miles_remaining = 1_800;
        assert!(game.available_speakers().is_empty(), "a departed fort's speaker must vanish");
    }

    #[test]
    fn era_closed_stores_remove_the_fort_speaker() {
        let mut game = fort_state();
        game.era_id = Some("closed".into());
        game.content.era_rules.insert(
            "closed".into(),
            EraRules { unavailable_stores: vec!["independence".into()], ..EraRules::default() },
        );
        assert!(game.available_speakers().is_empty());
    }

    #[test]
    fn pending_decisions_and_minigames_hide_every_speaker() {
        let mut game = fort_state();
        game.pending_event = Some("wheel".into());
        assert!(game.available_speakers().is_empty());
    }

    #[test]
    fn converse_is_only_reachable_through_the_command_boundary() {
        let mut game = fort_state();
        let outcomes = converse(&mut game, "orson", ConversationTopic::Route);
        assert!(matches!(outcomes.as_slice(), [Outcome::Conversation { .. }]));
    }

    #[test]
    fn converse_is_rejected_outside_a_valid_phase() {
        let mut game = fort_state();
        game.status = RunStatus::Setup;
        let outcomes = converse(&mut game, "orson", ConversationTopic::Route);
        assert!(matches!(outcomes.as_slice(), [Outcome::Rejected(CommandError::InvalidPhase)]));
        assert!(game.conversation_memory.is_empty(), "a rejected command must not mutate state");
    }

    #[test]
    fn repeated_talk_same_day_does_not_grant_a_favor() {
        let mut game = fort_state();
        let before_food = game.inventory.get("food");

        converse(&mut game, "orson", ConversationTopic::Route);
        let same_day = converse(&mut game, "orson", ConversationTopic::Supplies);
        let Outcome::Conversation { favor, .. } = &same_day[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(favor.is_none(), "a second question in the same sitting is not a return visit");
        assert_eq!(game.inventory.get("food"), before_food);
        assert_eq!(game.conversation_memory["orson"].times_talked, 2);
    }

    #[test]
    fn a_later_day_return_is_recognized_and_grants_one_favor_only() {
        let mut game = fort_state();
        let before_food = game.inventory.get("food");

        let first = converse(&mut game, "orson", ConversationTopic::Route);
        let Outcome::Conversation { recognized, favor, .. } = &first[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(!recognized);
        assert!(favor.is_none());

        game.day += 1;
        let second = converse(&mut game, "orson", ConversationTopic::Supplies);
        let Outcome::Conversation { recognized, favor, .. } = &second[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(recognized);
        let favor_line = favor.clone().expect("a later-day return should grant a favor");
        assert!(favor_line.contains("15 lbs"));
        assert_eq!(game.inventory.get("food"), before_food + 15);

        game.day += 1;
        let third = converse(&mut game, "orson", ConversationTopic::News);
        let Outcome::Conversation { favor, .. } = &third[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(favor.is_none(), "favor must not repeat on a third, later-day visit");
        assert_eq!(game.inventory.get("food"), before_food + 15);
    }

    #[test]
    fn a_full_wagon_receives_no_food_and_keeps_the_favor_available() {
        let mut game = fort_state();
        game.content.items.iter_mut().find(|item| item.id == "food").unwrap().limit =
            game.inventory.get("food");
        converse(&mut game, "orson", ConversationTopic::Route);
        game.day += 1;
        let outcomes = converse(&mut game, "orson", ConversationTopic::Supplies);
        let Outcome::Conversation { favor, .. } = &outcomes[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(favor.is_none(), "no room means no cornmeal claim");
        assert!(
            !game.conversation_memory["orson"].favor_received,
            "an ungranted favor must remain available for later"
        );
    }

    #[test]
    fn wagon_favor_requires_an_actual_prior_trade_not_just_prior_talk() {
        let mut game = fort_state();
        game.npcs.push(NpcTrain {
            id: "wagon_a".into(),
            name: "Nora Bird".into(),
            reputation: 0,
            inventory: BTreeMap::new(),
            recurring: true,
            last_reputation_day: None,
            first_mile: 0,
            last_mile: 5_000,
            period_days: 0,
            day_window: 0,
        });

        converse(&mut game, "wagon_a", ConversationTopic::News);
        game.day += 5;
        let outcomes = converse(&mut game, "wagon_a", ConversationTopic::News);
        let Outcome::Conversation { favor, .. } = &outcomes[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(favor.is_none(), "mere repeated talk must not fabricate trade familiarity");

        // A real trade actually happened (mirrors `barter`'s bookkeeping).
        game.npcs[0].last_reputation_day = Some(game.day);
        game.npcs[0].reputation = 1;
        game.day += 5;
        let outcomes = converse(&mut game, "wagon_a", ConversationTopic::News);
        let Outcome::Conversation { favor, .. } = &outcomes[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(favor.is_some(), "a real prior trade should be recognized with a favor");
        assert_eq!(game.npcs[0].reputation, 2);
    }

    /// Fort speakers vanish once the party is underway (see the departed-fort
    /// test), so mid-leg route questions can only reach a wagon-train speaker —
    /// exactly the scenario that used to leak a stale fork node's full distance.
    fn traveling_state_with_a_wagon_speaker() -> GameState {
        let mut game = fort_state();
        game.npcs.push(NpcTrain {
            id: "wagon_a".into(),
            name: "Nora Bird".into(),
            reputation: 0,
            inventory: BTreeMap::new(),
            recurring: true,
            last_reputation_day: None,
            first_mile: 0,
            last_mile: 5_000,
            period_days: 0,
            day_window: 0,
        });
        game.status = RunStatus::Travelling;
        game
    }

    #[test]
    fn route_answer_prioritizes_live_progress_over_the_full_fork_distance() {
        let mut game = traveling_state_with_a_wagon_speaker();
        game.target_node_id = Some("willamette".into());
        game.route_miles_remaining = 340;
        let outcomes = converse(&mut game, "wagon_a", ConversationTopic::Route);
        let Outcome::Conversation { lines, .. } = &outcomes[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(
            lines.iter().any(|line| line.contains("340") && line.contains("Willamette Valley")),
            "must report live remaining miles, not the full 2040 mile fork distance:\n{lines:?}"
        );
        assert!(
            !lines.iter().any(|line| line.contains("2040")),
            "must not restate the full posted distance once underway:\n{lines:?}"
        );
    }

    #[test]
    fn route_answer_does_not_relist_a_fork_already_chosen() {
        let mut game = traveling_state_with_a_wagon_speaker();
        game.content.trails[0].nodes[0].routes.push(RouteDefinition {
            id: "alt".into(),
            label: "Alternate path".into(),
            target_id: "willamette".into(),
            distance_miles: 1_900,
        });
        game.route_miles_remaining = 500;
        game.target_node_id = Some("willamette".into());
        let outcomes = converse(&mut game, "wagon_a", ConversationTopic::Route);
        let Outcome::Conversation { lines, .. } = &outcomes[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(
            !lines.iter().any(|line| line.contains("forks")),
            "a chosen route must not re-offer both fork alternatives once underway:\n{lines:?}"
        );
    }

    #[test]
    fn news_never_names_the_departed_fort_while_traveling() {
        // `current_node_id` stays "independence" — the last stop — for the whole
        // leg; only `target_node_id`/`route_miles_remaining` move. News must never
        // describe the party as still standing at that stale current landmark.
        let mut game = fort_state();
        game.status = RunStatus::Travelling;
        game.target_node_id = Some("willamette".into());
        game.route_miles_remaining = 100;
        game.npcs.push(NpcTrain {
            id: "wagon_b".into(),
            name: "Content Ashby".into(),
            reputation: 0,
            inventory: BTreeMap::new(),
            recurring: true,
            last_reputation_day: None,
            first_mile: 0,
            last_mile: 5_000,
            period_days: 0,
            day_window: 0,
        });
        let outcomes = converse(&mut game, "wagon_b", ConversationTopic::News);
        let Outcome::Conversation { lines, .. } = &outcomes[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(
            !lines.iter().any(|line| line.to_lowercase().contains("independence")),
            "must not reference the departed fort as if the party is still there:\n{lines:?}"
        );
    }

    #[test]
    fn news_reports_real_river_data_ahead_instead_of_repeating_the_status_bar() {
        let mut game = fort_state();
        game.content.trails[0].nodes.push(LandmarkDefinition {
            id: "kansas_river".into(),
            name: "Kansas River".into(),
            mile: 102,
            kind: LandmarkKind::River,
            routes: vec![],
            river: Some(RiverDefinition {
                width_feet: 620,
                depth_feet: 4,
                ferry_cost_cents: Some(500),
            }),
            store: false,
        });
        game.target_node_id = Some("kansas_river".into());
        game.route_miles_remaining = 30;
        let outcomes = converse(&mut game, "orson", ConversationTopic::News);
        let Outcome::Conversation { lines, .. } = &outcomes[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(
            lines.iter().any(|line| line.contains("620") && line.contains("4 feet")),
            "should share the real river width/depth ahead:\n{lines:?}"
        );
    }

    #[test]
    fn news_reports_whether_the_next_forts_store_is_open_this_era() {
        let mut game = fort_state();
        game.content.trails[0].nodes.push(LandmarkDefinition {
            id: "fort_kearney".into(),
            name: "Fort Kearney".into(),
            mile: 304,
            kind: LandmarkKind::Fort,
            routes: vec![],
            river: None,
            store: true,
        });
        game.target_node_id = Some("fort_kearney".into());
        game.route_miles_remaining = 50;
        let open = converse(&mut game, "orson", ConversationTopic::News);
        let Outcome::Conversation { lines, .. } = &open[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(lines.iter().any(|line| line.contains("Fort Kearney") && line.contains("stock")));

        game.day += 1;
        game.content.era_rules.insert(
            game.era_id.clone().unwrap(),
            EraRules { unavailable_stores: vec!["fort_kearney".into()], ..EraRules::default() },
        );
        let closed = converse(&mut game, "orson", ConversationTopic::News);
        let Outcome::Conversation { lines, .. } = &closed[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(lines.iter().any(|line| line.contains("no store")));
    }

    #[test]
    fn landmark_name_never_resolves_a_different_trails_node() {
        let mut game = fort_state();
        // Another trail defines a node with the same id but a different name.
        content_with_conflicting_trail(&mut game);
        game.trail_id = Some("oregon".into());
        assert_eq!(game.landmark_name("shared_id"), "Oregon Name");
    }

    fn content_with_conflicting_trail(game: &mut GameState) {
        game.content.trails[0].nodes.push(LandmarkDefinition {
            id: "shared_id".into(),
            name: "Oregon Name".into(),
            mile: 500,
            kind: LandmarkKind::Landmark,
            routes: vec![],
            river: None,
            store: false,
        });
        game.content.trails.push(crate::content::TrailDefinition {
            id: "california".into(),
            name: "California Trail".into(),
            start_node_id: "shared_id".into(),
            goal_node_id: "shared_id".into(),
            nodes: vec![LandmarkDefinition {
                id: "shared_id".into(),
                name: "California Name".into(),
                mile: 0,
                kind: LandmarkKind::Landmark,
                routes: vec![],
                river: None,
                store: false,
            }],
        });
    }

    #[test]
    fn route_answer_reflects_actual_trail_data() {
        let mut game = fort_state();
        let outcomes = converse(&mut game, "orson", ConversationTopic::Route);
        let Outcome::Conversation { lines, .. } = &outcomes[0] else {
            panic!("expected a Conversation outcome");
        };
        assert!(lines.iter().any(|line| line.contains("Willamette Valley")));
        assert!(lines.iter().any(|line| line.contains("2040")));
    }

    #[test]
    fn unknown_speaker_is_rejected() {
        let mut game = fort_state();
        let outcomes = converse(&mut game, "nobody", ConversationTopic::News);
        assert!(matches!(
            outcomes.as_slice(),
            [Outcome::Rejected(CommandError::UnknownId(id))] if id == "nobody"
        ));
    }

    #[test]
    fn conversation_memory_round_trips_through_json() {
        let mut game = fort_state();
        converse(&mut game, "orson", ConversationTopic::Route);
        game.day += 1;
        converse(&mut game, "orson", ConversationTopic::Supplies);
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

    #[test]
    fn old_content_without_a_speaker_roster_still_loads() {
        let json = serde_json::to_string(&GameContent::starter()).unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        value.as_object_mut().unwrap().remove("speakers");
        let restored: GameContent = serde_json::from_value(value).unwrap();
        assert!(restored.speakers.is_empty());
    }

    #[test]
    fn old_speaker_definitions_without_favor_food_lbs_default_to_fifteen() {
        let json = r#"{
            "id": "orson",
            "name": "Orson Pike",
            "landmark_id": "independence",
            "greeting": "Well met on the road.",
            "returning_greeting": "Back again, are you?",
            "favor_text": "sends you off with cornmeal"
        }"#;
        let def: SpeakerDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.favor_food_lbs, 15);
    }
}
