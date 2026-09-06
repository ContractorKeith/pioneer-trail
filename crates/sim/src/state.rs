//! Command-driven deterministic journey state.
use crate::{
    calendar::CalendarDate,
    content::*,
    economy::{Counteroffer, Market, NpcTrain},
    health::{advance, PartyMember},
    rng::SimRng,
    score,
    weather::{Terrain, WeatherState},
};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Pace {
    Steady,
    Strenuous,
    Grueling,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RationLevel {
    Filling,
    Meager,
    BareBones,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CrossMethod {
    Ford,
    Caulk,
    Ferry,
    Wait,
    Guide,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunStatus {
    Setup,
    Outfitting,
    Travelling,
    AtLandmark(String),
    AwaitingFork(String),
    AwaitingRiver(String),
    Arrived,
    Failed,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Inventory {
    pub quantities: BTreeMap<String, u32>,
}
impl Inventory {
    pub fn get(&self, id: &str) -> u32 {
        self.quantities.get(id).copied().unwrap_or(0)
    }
    fn add(&mut self, id: &str, n: u32) {
        let quantity = self.quantities.entry(id.into()).or_default();
        *quantity = quantity.saturating_add(n);
    }
    fn remove(&mut self, id: &str, n: u32) -> bool {
        let h = self.get(id);
        if h < n {
            return false;
        }
        self.quantities.insert(id.into(), h - n);
        true
    }
    fn take(&mut self, id: &str, n: u32) -> u32 {
        let have = self.get(id);
        let taken = have.min(n);
        self.quantities.insert(id.into(), have - taken);
        taken
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingEvent {
    pub event_id: String,
    pub due_day: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub rng: SimRng,
    pub content: GameContent,
    pub status: RunStatus,
    pub miles: u32,
    pub day: u32,
    pub departure_month: u8,
    pub era_id: Option<String>,
    pub trail_id: Option<String>,
    pub occupation_id: Option<String>,
    pub party: Vec<PartyMember>,
    pub cash_cents: i64,
    pub inventory: Inventory,
    pub pace: Pace,
    pub rations: RationLevel,
    pub weather: WeatherKind,
    pub current_node_id: Option<String>,
    /// Destination selected from the current node's route; never infer this from absolute miles.
    pub target_node_id: Option<String>,
    pub route_miles_remaining: u32,
    pub pending_event: Option<String>,
    pub scheduled_events: Vec<PendingEvent>,
    pub flags: BTreeSet<String>,
    #[serde(default)]
    pub weather_state: WeatherState,
    #[serde(default)]
    pub difficulty: Difficulty,
    #[serde(default)]
    pub ox_fatigue: u8,
    #[serde(default)]
    pub reputation: i16,
    #[serde(default)]
    pub markets: BTreeMap<String, Market>,
    #[serde(default)]
    pub npcs: Vec<NpcTrain>,
    #[serde(default)]
    pub pending_counteroffer: Option<Counteroffer>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Command {
    SetDifficulty(Difficulty),
    Forage,
    Fish,
    Sell {
        item_id: String,
        quantity: u32,
    },
    Barter {
        npc_id: String,
        offered_item: String,
        offered_quantity: u32,
        wanted_item: String,
        wanted_quantity: u32,
    },
    InviteNpc {
        npc_id: String,
    },
    DismissNpc {
        npc_id: String,
    },
    AcceptCounteroffer {
        npc_id: String,
        offered_item: String,
        offered_quantity: u32,
        wanted_item: String,
        wanted_quantity: u32,
    },
    Configure {
        trail_id: String,
        era_id: String,
        occupation_id: String,
        party: Vec<String>,
        departure_month: u8,
    },
    Buy {
        item_id: String,
        quantity: u32,
    },
    Depart,
    Continue,
    TravelDay,
    SetPace(Pace),
    SetRations(RationLevel),
    Rest {
        days: u32,
    },
    ChooseRoute {
        route_id: String,
    },
    CrossRiver {
        method: CrossMethod,
    },
    Respond {
        event_id: String,
        choice_id: String,
    },
    Talk,
    Treat {
        member_index: usize,
        ailment_id: String,
    },
    HuntResult {
        food_lbs: u32,
        ammo_boxes_used: u32,
    },
    RaftResult {
        cargo_lost_lbs: u32,
        casualties: u8,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Outcome {
    Configured,
    Purchased { item_id: String, quantity: u32, cost_cents: i64 },
    Departed,
    DayAdvanced { day: u32, miles: u32, weather: WeatherKind },
    ArrivedAt { landmark_id: String },
    ForkAvailable { landmark_id: String },
    RiverCrossingRequired { landmark_id: String },
    Event { event_id: String, text: String },
    Quote { quote_id: String, text: String },
    Treated { member_index: usize, ailment_id: String },
    Score { points: u32 },
    MemberDied { name: String },
    Message(String),
    Rejected(CommandError),
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Error)]
pub enum CommandError {
    #[error("command is not available now")]
    InvalidPhase,
    #[error("unknown content id: {0}")]
    UnknownId(String),
    #[error("invalid setup")]
    InvalidSetup,
    #[error("insufficient cash")]
    InsufficientCash,
    #[error("purchase exceeds limit or wagon capacity")]
    CapacityExceeded,
    #[error("no such pending event")]
    NoPendingEvent,
    #[error("choice is unavailable")]
    InvalidChoice,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        Self::with_content(seed, GameContent::starter())
    }
    pub fn with_content(seed: u64, content: GameContent) -> Self {
        Self {
            rng: SimRng::new(seed),
            content,
            status: RunStatus::Setup,
            miles: 0,
            day: 0,
            departure_month: 3,
            era_id: None,
            trail_id: None,
            occupation_id: None,
            party: vec![],
            cash_cents: 0,
            inventory: Inventory::default(),
            pace: Pace::Steady,
            rations: RationLevel::Filling,
            weather: WeatherKind::Clear,
            current_node_id: None,
            target_node_id: None,
            route_miles_remaining: 0,
            pending_event: None,
            scheduled_events: vec![],
            flags: BTreeSet::new(),
            weather_state: WeatherState::default(),
            difficulty: Difficulty::Normal,
            ox_fatigue: 0,
            reputation: 0,
            markets: BTreeMap::new(),
            npcs: vec![NpcTrain {
                id: "emigrant_train".into(),
                name: "Holloway family".into(),
                reputation: 0,
                inventory: BTreeMap::from([("food".into(), 100)]),
                recurring: true,
                last_reputation_day: None,
            }],
            pending_counteroffer: None,
        }
    }
    pub fn apply(&mut self, c: Command) -> Vec<Outcome> {
        self.try_apply(c).unwrap_or_else(|e| vec![Outcome::Rejected(e)])
    }
    pub fn try_apply(&mut self, c: Command) -> Result<Vec<Outcome>, CommandError> {
        if matches!(self.status, RunStatus::Arrived | RunStatus::Failed) {
            return Err(CommandError::InvalidPhase);
        }
        if self.pending_event.is_some() && !matches!(c, Command::Respond { .. }) {
            return Err(CommandError::InvalidPhase);
        }
        match c {
            Command::SetDifficulty(difficulty) => {
                if self.status != RunStatus::Setup {
                    return Err(CommandError::InvalidPhase);
                }
                self.difficulty = difficulty;
                Ok(vec![Outcome::Message("Difficulty changed".into())])
            }
            Command::Forage => self.forage(),
            Command::Fish => self.fish(),
            Command::Sell { item_id, quantity } => self.sell(&item_id, quantity),
            Command::Barter {
                npc_id,
                offered_item,
                offered_quantity,
                wanted_item,
                wanted_quantity,
            } => {
                self.barter(&npc_id, &offered_item, offered_quantity, &wanted_item, wanted_quantity)
            }
            Command::InviteNpc { npc_id } => self.invite_npc(&npc_id),
            Command::DismissNpc { npc_id } => self.dismiss_npc(&npc_id),
            Command::AcceptCounteroffer {
                npc_id,
                offered_item,
                offered_quantity,
                wanted_item,
                wanted_quantity,
            } => self.accept_counteroffer(
                &npc_id,
                &offered_item,
                offered_quantity,
                &wanted_item,
                wanted_quantity,
            ),
            Command::Configure { trail_id, era_id, occupation_id, party, departure_month } => {
                self.configure(trail_id, era_id, occupation_id, party, departure_month)
            }
            Command::Buy { item_id, quantity } => self.buy(&item_id, quantity),
            Command::Depart => {
                self.phase(RunStatus::Outfitting)?;
                if self.inventory.get("oxen") == 0 {
                    return Err(CommandError::InvalidChoice);
                }
                self.begin_only_route()?;
                Ok(vec![Outcome::Departed])
            }
            Command::Continue | Command::TravelDay => {
                if matches!(self.status, RunStatus::AtLandmark(_)) {
                    self.begin_only_route()?;
                    Ok(vec![Outcome::Message("Leaving landmark".into())])
                } else {
                    self.travel()
                }
            }
            Command::SetPace(v) => {
                self.at_camp()?;
                self.pace = v;
                Ok(vec![Outcome::Message("Pace changed".into())])
            }
            Command::SetRations(v) => {
                self.at_camp()?;
                self.rations = v;
                Ok(vec![Outcome::Message("Rations changed".into())])
            }
            Command::Rest { days } => self.rest(days),
            Command::ChooseRoute { route_id } => self.route(&route_id),
            Command::CrossRiver { method } => self.cross(method),
            Command::Respond { event_id, choice_id } => self.respond(&event_id, &choice_id),
            Command::Talk => self.talk(),
            Command::Treat { member_index, ailment_id } => self.treat(member_index, &ailment_id),
            Command::HuntResult { food_lbs, ammo_boxes_used } => {
                self.hunt(food_lbs, ammo_boxes_used)
            }
            Command::RaftResult { cargo_lost_lbs, casualties } => {
                self.raft(cargo_lost_lbs, casualties)
            }
        }
    }
    fn configure(
        &mut self,
        t: String,
        e: String,
        o: String,
        names: Vec<String>,
        month: u8,
    ) -> Result<Vec<Outcome>, CommandError> {
        if self.status != RunStatus::Setup
            || names.len() != 5
            || names.iter().any(|name| {
                name.trim().is_empty()
                    || name.chars().count() > 24
                    || name.chars().any(char::is_control)
            })
            || !(3..=7).contains(&month)
        {
            return Err(CommandError::InvalidSetup);
        }
        let trail = self
            .content
            .trails
            .iter()
            .find(|x| x.id == t)
            .ok_or_else(|| CommandError::UnknownId(t.clone()))?;
        if !self.content.eras.iter().any(|x| x.id == e) {
            return Err(CommandError::UnknownId(e));
        }
        let job = self
            .content
            .occupations
            .iter()
            .find(|x| x.id == o)
            .ok_or_else(|| CommandError::UnknownId(o.clone()))?;
        self.cash_cents = job.starting_cash_cents;
        self.party = names
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let mut member = PartyMember::new(name);
                member.age =
                    18 + ((self.rng.stream("party").gen_range(0..40) + index as u32) % 55) as u8;
                member.traits = match o.as_str() {
                    "doctor" => vec![crate::party::Trait::Herbalist, crate::party::Trait::Hardy],
                    "hunter" => vec![crate::party::Trait::Sharpshooter, crate::party::Trait::Hardy],
                    "preacher" => vec![crate::party::Trait::Devout, crate::party::Trait::Cheerful],
                    _ => vec![crate::party::Trait::Cheerful, crate::party::Trait::Hardy],
                };
                member
            })
            .collect();
        self.departure_month = month;
        self.current_node_id = Some(trail.start_node_id.clone());
        self.trail_id = Some(t);
        self.era_id = Some(e);
        self.occupation_id = Some(o);
        self.status = RunStatus::Outfitting;
        Ok(vec![Outcome::Configured])
    }
    fn buy(&mut self, id: &str, n: u32) -> Result<Vec<Outcome>, CommandError> {
        if !self.can_shop() || n == 0 {
            return Err(CommandError::InvalidPhase);
        }
        let item = self
            .content
            .items
            .iter()
            .find(|x| x.id == id)
            .ok_or_else(|| CommandError::UnknownId(id.into()))?
            .clone();
        let season_markup = match self.season() {
            Season::Winter => 25,
            Season::Summer => 10,
            _ => 0,
        };
        let current_weight = self.weight();
        let market_id = self.current_node_id.clone().unwrap_or_default();
        let mut market = self.market_at(&market_id);
        market.replenish(self.day);
        if market.stock.get(id).copied().unwrap_or_default() < n {
            return Err(CommandError::InvalidChoice);
        }
        let cost = market
            .price_for(id, item.price_cents, season_markup)
            .checked_mul(i64::from(n))
            .ok_or(CommandError::InsufficientCash)?;
        if self.inventory.get(id).checked_add(n).ok_or(CommandError::CapacityExceeded)? > item.limit
            || current_weight
                .checked_add(item.weight_lbs.checked_mul(n).ok_or(CommandError::CapacityExceeded)?)
                .ok_or(CommandError::CapacityExceeded)?
                > 2400
        {
            return Err(CommandError::CapacityExceeded);
        }
        if cost > self.cash_cents {
            return Err(CommandError::InsufficientCash);
        }
        self.cash_cents -= cost;
        self.inventory.add(id, n);
        let stock = market.stock.entry(id.into()).or_default();
        *stock = stock.checked_sub(n).ok_or(CommandError::InvalidChoice)?;
        self.markets.insert(market_id, market);
        Ok(vec![Outcome::Purchased { item_id: id.into(), quantity: n, cost_cents: cost }])
    }
    fn travel(&mut self) -> Result<Vec<Outcome>, CommandError> {
        self.traveling()?;
        self.day += 1;
        if self.inventory.get("oxen") == 0 {
            self.status = RunStatus::Failed;
            return Ok(vec![Outcome::Message("Your wagon cannot move without oxen.".into())]);
        }
        let season = self.season();
        self.weather_state.advance(&mut self.rng, season);
        self.weather = self.weather_state.kind;
        let eat = match self.rations {
            RationLevel::Filling => 3,
            RationLevel::Meager => 2,
            RationLevel::BareBones => 1,
        };
        let required_food = eat * self.party.iter().filter(|p| p.alive).count() as u32;
        let eaten_food = self.inventory.take("food", required_food);
        let mut out = vec![];
        let damages: Vec<u8> = self
            .party
            .iter()
            .map(|p| {
                p.ailments
                    .iter()
                    .filter_map(|id| {
                        self.content.ailments.iter().find(|a| &a.id == id).map(|a| a.daily_damage)
                    })
                    .fold(0u8, u8::saturating_add)
            })
            .collect();
        for (p, damage) in self.party.iter_mut().zip(damages) {
            if advance(p, damage) {
                out.push(Outcome::MemberDied { name: p.name.clone() })
            }
        }
        self.progress_ailments(&mut out);
        if eaten_food < required_food {
            for member in &mut self.party {
                if member.alive {
                    member.health = member.health.saturating_sub(8);
                }
            }
        }
        for member in &mut self.party {
            if member.alive && member.health == 0 {
                member.alive = false;
                out.push(Outcome::MemberDied { name: member.name.clone() });
            }
        }
        if !self.party.iter().any(|member| member.alive) {
            self.status = RunStatus::Failed;
            return Ok(out);
        }
        let base: u32 = match self.pace {
            Pace::Steady => 18,
            Pace::Strenuous => 25,
            Pace::Grueling => 30,
        };
        let terrain = self.terrain();
        let morale_penalty = if self
            .party
            .iter()
            .filter(|member| member.alive)
            .map(|member| member.morale)
            .sum::<i16>()
            / (self.party.iter().filter(|member| member.alive).count().max(1) as i16)
            < 25
        {
            3
        } else {
            0
        };
        let weight_penalty = self.weight().saturating_sub(2_000) / 200;
        let fatigue_penalty = u32::from(self.ox_fatigue / 20);
        let moved = base
            .saturating_sub(
                self.weather_state.travel_penalty(terrain)
                    + morale_penalty
                    + weight_penalty
                    + fatigue_penalty,
            )
            .min(self.route_miles_remaining);
        self.ox_fatigue = self
            .ox_fatigue
            .saturating_add(match self.pace {
                Pace::Steady => 2,
                Pace::Strenuous => 5,
                Pace::Grueling => 9,
            })
            .min(100);
        self.miles += moved;
        self.route_miles_remaining -= moved;
        out.push(Outcome::DayAdvanced { day: self.day, miles: self.miles, weather: self.weather });
        self.due(&mut out);
        if self.pending_event.is_none() {
            self.event(&mut out)
        }
        if !matches!(self.status, RunStatus::Failed) {
            self.landmark(&mut out);
        }
        Ok(out)
    }
    fn rest(&mut self, days: u32) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        if !(1..=30).contains(&days) {
            return Err(CommandError::InvalidChoice);
        }
        let mut outcomes = Vec::new();
        let started = self.day;
        for _ in 0..days {
            self.pass_camp_day(&mut outcomes, true);
            if self.status == RunStatus::Failed {
                break;
            }
            self.due(&mut outcomes);
            if self.pending_event.is_some() {
                break;
            }
        }
        outcomes.push(Outcome::Message(format!("Rested {} days", self.day - started)));
        Ok(outcomes)
    }
    fn route(&mut self, id: &str) -> Result<Vec<Outcome>, CommandError> {
        if !matches!(self.status, RunStatus::AwaitingFork(_)) {
            return Err(CommandError::InvalidPhase);
        }
        let node = self.node()?;
        let r = node
            .routes
            .iter()
            .find(|x| x.id == id)
            .ok_or_else(|| CommandError::UnknownId(id.into()))?;
        let target_id = r.target_id.clone();
        let distance = r.distance_miles;
        let label = r.label.clone();
        self.target_node_id = Some(target_id);
        self.route_miles_remaining = distance;
        self.status = RunStatus::Travelling;
        Ok(vec![Outcome::Message(format!("Taking {label}"))])
    }
    fn cross(&mut self, m: CrossMethod) -> Result<Vec<Outcome>, CommandError> {
        if !matches!(self.status, RunStatus::AwaitingRiver(_)) {
            return Err(CommandError::InvalidPhase);
        }
        let river = self.node()?.river.clone().ok_or(CommandError::InvalidPhase)?;
        if self.node()?.routes.len() != 1 {
            return Err(CommandError::InvalidPhase);
        }
        if m == CrossMethod::Wait {
            let mut out = Vec::new();
            self.pass_camp_day(&mut out, false);
            if self.status != RunStatus::Failed {
                self.due(&mut out);
            }
            out.push(Outcome::Message("You wait one day for lower water.".into()));
            return Ok(out);
        }
        if m == CrossMethod::Guide {
            if self.current_node_id.as_deref() != Some("snake_river")
                || self.inventory.get("clothing") < 3
            {
                return Err(CommandError::InvalidChoice);
            }
            self.inventory.remove("clothing", 3);
        }
        if m == CrossMethod::Ferry {
            let cost = river.ferry_cost_cents.ok_or(CommandError::InvalidChoice)?;
            if self.cash_cents < cost {
                return Err(CommandError::InsufficientCash);
            }
            self.cash_cents -= cost
        }
        let risk = match m {
            CrossMethod::Ferry => 0,
            CrossMethod::Wait => 0,
            CrossMethod::Guide => 10,
            CrossMethod::Ford => river.depth_feet * 20,
            CrossMethod::Caulk => river.width_feet / 20,
        };
        if self.rng.stream("rivers").gen_range(0..100) < risk.min(90) {
            self.inventory.take("food", 50);
        }
        self.begin_only_route()?;
        Ok(vec![Outcome::Message("The crossing is behind you.".into())])
    }
    fn respond(&mut self, event: &str, choice: &str) -> Result<Vec<Outcome>, CommandError> {
        if self.pending_event.as_deref() != Some(event) {
            return Err(CommandError::NoPendingEvent);
        }
        let e = self
            .content
            .events
            .iter()
            .find(|e| e.id == event)
            .ok_or_else(|| CommandError::UnknownId(event.into()))?
            .clone();
        let c = e.choices.iter().find(|c| c.id == choice).ok_or(CommandError::InvalidChoice)?;
        if !self.choice_available(c) {
            return Err(CommandError::InvalidChoice);
        }
        let mut out = vec![];
        self.effects(&c.effects, &mut out);
        self.pending_event = None;
        Ok(out)
    }
    fn talk(&mut self) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let candidates: Vec<_> = self
            .content
            .quotes
            .iter()
            .filter(|q| {
                q.landmark_id.is_none()
                    || q.landmark_id.as_deref() == self.current_node_id.as_deref()
            })
            .collect();
        let q = candidates.choose(self.rng.stream("quotes"));
        let Some(q) = q else {
            return Ok(vec![Outcome::Message("The camp is quiet tonight.".into())]);
        };
        Ok(vec![Outcome::Quote { quote_id: q.id.clone(), text: q.text.clone() }])
    }
    fn treat(&mut self, i: usize, a: &str) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let doctor_present = self
            .party
            .iter()
            .any(|member| member.alive && member.traits.contains(&crate::party::Trait::Herbalist));
        let p = self.party.get_mut(i).ok_or(CommandError::InvalidChoice)?;
        if !p.alive
            || !p.ailments.iter().any(|x| x == a)
            || (!doctor_present && !self.inventory.remove("medicine", 1))
        {
            return Err(CommandError::InvalidChoice);
        }
        p.ailments.retain(|x| x != a);
        p.health = p.health.saturating_add(15).min(100);
        Ok(vec![Outcome::Treated { member_index: i, ailment_id: a.into() }])
    }
    fn hunt(&mut self, food: u32, ammo: u32) -> Result<Vec<Outcome>, CommandError> {
        self.traveling()?;
        if (food > 0 && ammo == 0)
            || food > 500
            || self.weight().checked_add(food).ok_or(CommandError::CapacityExceeded)? > 2400
            || !self.inventory.remove("ammunition", ammo)
        {
            return Err(CommandError::InvalidChoice);
        }
        self.inventory.add("food", food);
        let mut outcomes = Vec::new();
        self.pass_camp_day(&mut outcomes, false);
        outcomes.push(Outcome::Message(format!("Brought back {} lbs of food", food)));
        Ok(outcomes)
    }
    fn raft(&mut self, lost: u32, casualties: u8) -> Result<Vec<Outcome>, CommandError> {
        if !matches!(self.status, RunStatus::AtLandmark(_))
            || !matches!(self.current_landmark().map(|node| node.kind), Some(LandmarkKind::Finale))
        {
            return Err(CommandError::InvalidPhase);
        }
        self.inventory.remove("food", lost);
        for p in self.party.iter_mut().filter(|p| p.alive).take(casualties as usize) {
            p.alive = false
        }
        Ok(vec![Outcome::Message("Rafting result recorded".into())])
    }
    fn forage(&mut self) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let bonus = self
            .party
            .iter()
            .filter(|member| {
                member.traits.contains(&crate::party::Trait::Herbalist)
                    || member.traits.contains(&crate::party::Trait::Sharpshooter)
            })
            .count() as u32
            * 10;
        let food = (self.rng.stream("forage").gen_range(5..=25) + bonus)
            .min(self.max_addable("food").unwrap_or(0));
        self.inventory.add("food", food);
        let mut outcomes = Vec::new();
        self.pass_camp_day(&mut outcomes, false);
        outcomes.push(Outcome::Message(format!("Foraged {food} lbs of food.")));
        Ok(outcomes)
    }
    fn fish(&mut self) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        if !matches!(self.terrain(), Terrain::RiverValley) {
            return Err(CommandError::InvalidPhase);
        }
        let food = self
            .rng
            .stream("fishing")
            .gen_range(10..=45)
            .min(self.max_addable("food").unwrap_or(0));
        self.inventory.add("food", food);
        let mut outcomes = Vec::new();
        self.pass_camp_day(&mut outcomes, false);
        outcomes.push(Outcome::Message(format!("Caught {food} lbs of fish.")));
        Ok(outcomes)
    }
    fn sell(&mut self, item_id: &str, quantity: u32) -> Result<Vec<Outcome>, CommandError> {
        if !self.can_shop() || quantity == 0 {
            return Err(CommandError::InvalidChoice);
        }
        let item = self
            .content
            .items
            .iter()
            .find(|item| item.id == item_id)
            .ok_or_else(|| CommandError::UnknownId(item_id.into()))?;
        if self.inventory.get(item_id) < quantity {
            return Err(CommandError::InvalidChoice);
        }
        let market_id = self.current_node_id.clone().unwrap_or_default();
        let mut market = self.market_at(&market_id);
        market.replenish(self.day);
        let stock = market.stock.get(item_id).copied().unwrap_or_default();
        let max_stock = item.limit.max(20);
        let new_stock = stock.checked_add(quantity).ok_or(CommandError::CapacityExceeded)?;
        if new_stock > max_stock {
            return Err(CommandError::CapacityExceeded);
        }
        let price = market.price_for(item_id, item.price_cents, self.season_markup()) / 2;
        let credit =
            price.checked_mul(i64::from(quantity)).ok_or(CommandError::CapacityExceeded)?;
        let cash = self.cash_cents.checked_add(credit).ok_or(CommandError::CapacityExceeded)?;
        self.inventory.remove(item_id, quantity);
        market.stock.insert(item_id.into(), new_stock);
        self.markets.insert(market_id, market);
        self.cash_cents = cash;
        Ok(vec![Outcome::Message(format!("Sold {quantity} {item_id}."))])
    }
    fn barter(
        &mut self,
        npc_id: &str,
        offered_item: &str,
        offered_quantity: u32,
        wanted_item: &str,
        wanted_quantity: u32,
    ) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        if offered_quantity == 0 || wanted_quantity == 0 || offered_item == wanted_item {
            return Err(CommandError::InvalidChoice);
        }
        let (offered_value, wanted_value) =
            self.trade_values(offered_item, offered_quantity, wanted_item, wanted_quantity)?;
        let npc = self
            .npcs
            .iter()
            .find(|npc| npc.id == npc_id)
            .ok_or_else(|| CommandError::UnknownId(npc_id.into()))?;
        if !self.trade_fits(npc, offered_item, offered_quantity, wanted_item, wanted_quantity)? {
            return Err(CommandError::InvalidChoice);
        }
        if !npc.accepts(offered_value, wanted_value, self.reputation) {
            let counteroffer = self.make_counteroffer(
                npc_id,
                offered_item,
                wanted_item,
                wanted_quantity,
                wanted_value,
            )?;
            let counteroffer = counteroffer.ok_or(CommandError::InvalidChoice)?;
            self.pending_counteroffer = Some(counteroffer);
            return Ok(vec![Outcome::Message("The emigrants make a counteroffer.".into())]);
        }
        self.execute_trade(npc_id, offered_item, offered_quantity, wanted_item, wanted_quantity)?;
        Ok(vec![Outcome::Message("Trade accepted.".into())])
    }
    fn invite_npc(&mut self, npc_id: &str) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let npc = self
            .npcs
            .iter()
            .find(|npc| npc.id == npc_id)
            .ok_or_else(|| CommandError::UnknownId(npc_id.into()))?;
        if i32::from(npc.reputation) + i32::from(self.reputation) < 0
            || self.party.len() >= 12
            || !npc.recurring
            || self.party.iter().any(|member| member.npc_id.as_deref() == Some(npc_id))
        {
            return Err(CommandError::InvalidChoice);
        }
        let name = npc.name.clone();
        let npc = self.npcs.iter_mut().find(|npc| npc.id == npc_id).expect("NPC was found above");
        npc.recurring = false;
        let mut member = PartyMember::new(name.clone());
        member.npc_id = Some(npc_id.into());
        self.party.push(member);
        Ok(vec![Outcome::Message(format!("{name} joins the party."))])
    }
    fn dismiss_npc(&mut self, npc_id: &str) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let npc = self
            .npcs
            .iter()
            .find(|npc| npc.id == npc_id)
            .ok_or_else(|| CommandError::UnknownId(npc_id.into()))?;
        if npc.recurring {
            return Err(CommandError::InvalidChoice);
        }
        let index = self
            .party
            .iter()
            .position(|member| member.alive && member.npc_id.as_deref() == Some(npc_id))
            .ok_or(CommandError::InvalidChoice)?;
        let name = self.party[index].name.clone();
        self.party.remove(index);
        let npc = self.npcs.iter_mut().find(|npc| npc.id == npc_id).expect("NPC was found above");
        npc.recurring = true;
        Ok(vec![Outcome::Message(format!("{name} leaves the party."))])
    }
    fn accept_counteroffer(
        &mut self,
        npc_id: &str,
        offered_item: &str,
        offered_quantity: u32,
        wanted_item: &str,
        wanted_quantity: u32,
    ) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let offer = self.pending_counteroffer.clone().ok_or(CommandError::InvalidChoice)?;
        if offer.quoted_day != self.day
            || offer.npc_id != npc_id
            || offer.offered_item != offered_item
            || offer.offered_quantity != offered_quantity
            || offer.wanted_item != wanted_item
            || offer.wanted_quantity != wanted_quantity
        {
            return Err(CommandError::InvalidChoice);
        }
        let (offered_value, wanted_value) =
            self.trade_values(offered_item, offered_quantity, wanted_item, wanted_quantity)?;
        if offered_value != offer.offered_value_cents || wanted_value != offer.wanted_value_cents {
            return Err(CommandError::InvalidChoice);
        }
        let npc = self
            .npcs
            .iter()
            .find(|npc| npc.id == npc_id)
            .ok_or_else(|| CommandError::UnknownId(npc_id.into()))?;
        if !self.trade_fits(npc, offered_item, offered_quantity, wanted_item, wanted_quantity)?
            || !npc.accepts(offered_value, wanted_value, self.reputation)
        {
            return Err(CommandError::InvalidChoice);
        }
        self.execute_trade(npc_id, offered_item, offered_quantity, wanted_item, wanted_quantity)?;
        Ok(vec![Outcome::Message("Counteroffer accepted.".into())])
    }
    fn trade_values(
        &self,
        offered_item: &str,
        offered_quantity: u32,
        wanted_item: &str,
        wanted_quantity: u32,
    ) -> Result<(u64, u64), CommandError> {
        if offered_item == wanted_item || offered_quantity == 0 || wanted_quantity == 0 {
            return Err(CommandError::InvalidChoice);
        }
        let price = |id: &str| -> Result<u64, CommandError> {
            let item = self
                .content
                .items
                .iter()
                .find(|item| item.id == id)
                .ok_or_else(|| CommandError::UnknownId(id.into()))?;
            u64::try_from(item.price_cents)
                .ok()
                .filter(|price| *price > 0)
                .ok_or(CommandError::InvalidChoice)
        };
        let offered = price(offered_item)?
            .checked_mul(u64::from(offered_quantity))
            .ok_or(CommandError::CapacityExceeded)?;
        let wanted = price(wanted_item)?
            .checked_mul(u64::from(wanted_quantity))
            .ok_or(CommandError::CapacityExceeded)?;
        Ok((offered, wanted))
    }
    fn trade_fits(
        &self,
        npc: &NpcTrain,
        offered_item: &str,
        offered_quantity: u32,
        wanted_item: &str,
        wanted_quantity: u32,
    ) -> Result<bool, CommandError> {
        let wanted = self
            .content
            .items
            .iter()
            .find(|item| item.id == wanted_item)
            .ok_or_else(|| CommandError::UnknownId(wanted_item.into()))?;
        let offered = self
            .content
            .items
            .iter()
            .find(|item| item.id == offered_item)
            .ok_or_else(|| CommandError::UnknownId(offered_item.into()))?;
        if self.inventory.get(offered_item) < offered_quantity
            || npc.inventory.get(wanted_item).copied().unwrap_or_default() < wanted_quantity
            || npc
                .inventory
                .get(offered_item)
                .copied()
                .unwrap_or_default()
                .checked_add(offered_quantity)
                .is_none()
            || self.inventory.get(wanted_item).checked_add(wanted_quantity).is_none()
            || self.inventory.get(wanted_item).saturating_add(wanted_quantity) > wanted.limit
        {
            return Ok(false);
        }
        let weight_after = u64::from(self.weight())
            .checked_sub(u64::from(offered.weight_lbs) * u64::from(offered_quantity))
            .and_then(|weight| {
                weight.checked_add(u64::from(wanted.weight_lbs) * u64::from(wanted_quantity))
            })
            .ok_or(CommandError::CapacityExceeded)?;
        Ok(weight_after <= 2400)
    }
    fn make_counteroffer(
        &self,
        npc_id: &str,
        offered_item: &str,
        wanted_item: &str,
        wanted_quantity: u32,
        wanted_value: u64,
    ) -> Result<Option<Counteroffer>, CommandError> {
        let offered_price = self
            .content
            .items
            .iter()
            .find(|item| item.id == offered_item)
            .ok_or_else(|| CommandError::UnknownId(offered_item.into()))?
            .price_cents;
        let offered_price =
            u64::try_from(offered_price).map_err(|_| CommandError::InvalidChoice)?;
        if offered_price == 0 {
            return Ok(None);
        }
        let npc = self
            .npcs
            .iter()
            .find(|npc| npc.id == npc_id)
            .ok_or_else(|| CommandError::UnknownId(npc_id.into()))?;
        let discount = u64::try_from(self.reputation.max(0)).unwrap_or(0).min(25);
        let required_value = wanted_value
            .checked_mul(100)
            .and_then(|value| value.checked_add(99 + discount))
            .ok_or(CommandError::CapacityExceeded)?
            / (100 + discount);
        let offered_quantity =
            required_value.checked_add(offered_price - 1).ok_or(CommandError::CapacityExceeded)?
                / offered_price;
        let offered_quantity =
            u32::try_from(offered_quantity).map_err(|_| CommandError::CapacityExceeded)?;
        let (offered_value, wanted_value) =
            self.trade_values(offered_item, offered_quantity, wanted_item, wanted_quantity)?;
        if !self.trade_fits(npc, offered_item, offered_quantity, wanted_item, wanted_quantity)?
            || !npc.accepts(offered_value, wanted_value, self.reputation)
        {
            return Ok(None);
        }
        Ok(Some(Counteroffer {
            npc_id: npc_id.into(),
            offered_item: offered_item.into(),
            offered_quantity,
            wanted_item: wanted_item.into(),
            wanted_quantity,
            offered_value_cents: offered_value,
            wanted_value_cents: wanted_value,
            quoted_day: self.day,
        }))
    }
    fn execute_trade(
        &mut self,
        npc_id: &str,
        offered_item: &str,
        offered_quantity: u32,
        wanted_item: &str,
        wanted_quantity: u32,
    ) -> Result<(), CommandError> {
        let npc = self
            .npcs
            .iter_mut()
            .find(|npc| npc.id == npc_id)
            .ok_or_else(|| CommandError::UnknownId(npc_id.into()))?;
        self.inventory.remove(offered_item, offered_quantity);
        self.inventory.add(wanted_item, wanted_quantity);
        let offered_stock = npc.inventory.entry(offered_item.into()).or_default();
        *offered_stock =
            offered_stock.checked_add(offered_quantity).ok_or(CommandError::CapacityExceeded)?;
        let wanted_stock = npc.inventory.entry(wanted_item.into()).or_default();
        *wanted_stock =
            wanted_stock.checked_sub(wanted_quantity).ok_or(CommandError::InvalidChoice)?;
        if npc.last_reputation_day != Some(self.day) {
            self.reputation = self.reputation.saturating_add(1);
            npc.reputation = npc.reputation.saturating_add(1);
            npc.last_reputation_day = Some(self.day);
        }
        self.pending_counteroffer = None;
        Ok(())
    }
    pub fn score(&self) -> u32 {
        let mul = self
            .content
            .occupations
            .iter()
            .find(|x| Some(&x.id) == self.occupation_id.as_ref())
            .map_or(1.0, |x| x.score_multiplier);
        score::calculate(
            &self.party,
            self.cash_cents,
            &self.inventory.quantities.iter().map(|(a, b)| (a.clone(), *b)).collect::<Vec<_>>(),
            mul,
        )
    }
    pub fn current_landmark(&self) -> Option<&LandmarkDefinition> {
        self.node().ok()
    }
    pub fn choice_available(&self, choice: &EventChoice) -> bool {
        if !choice.conditions.iter().all(|condition| self.matches(condition)) {
            return false;
        }
        let mut cash = self.cash_cents;
        let mut supplies = self.inventory.clone();
        for effect in &choice.effects {
            match effect {
                Effect::AdjustCash(change) => {
                    cash = cash.saturating_add(*change);
                    if cash < 0 {
                        return false;
                    }
                }
                Effect::AdjustItem { item_id, quantity } => {
                    if *quantity < 0 && !supplies.remove(item_id, quantity.unsigned_abs()) {
                        return false;
                    }
                    if *quantity > 0 {
                        supplies.add(item_id, *quantity as u32);
                    }
                }
                _ => {}
            }
        }
        true
    }
    pub fn wagon_weight(&self) -> u32 {
        self.weight()
    }
    pub fn date(&self) -> (i32, u8, u8) {
        let year = self
            .content
            .eras
            .iter()
            .find(|era| Some(&era.id) == self.era_id.as_ref())
            .map_or(1848, |era| era.year);
        let date = CalendarDate::new(year, self.departure_month, 1)
            .expect("validated departure month")
            .add_days(self.day);
        (date.year, date.month, date.day)
    }
    pub fn can_shop(&self) -> bool {
        matches!(self.status, RunStatus::Outfitting | RunStatus::AtLandmark(_))
            && self.current_landmark().is_some_and(|node| node.store)
    }
    pub fn price_cents(&self, item_id: &str) -> Option<i64> {
        self.content.items.iter().find(|item| item.id == item_id).map(|item| {
            let mut market = self.market_at(self.current_node_id.as_deref().unwrap_or_default());
            market.replenish(self.day);
            market.price_for(item_id, item.price_cents, self.season_markup())
        })
    }
    fn season_markup(&self) -> i64 {
        match self.season() {
            Season::Winter => 25,
            Season::Summer => 10,
            _ => 0,
        }
    }
    fn market_at(&self, market_id: &str) -> Market {
        self.markets.get(market_id).cloned().unwrap_or_else(|| Market {
            normal_stock: self.content.items.iter().map(|item| (item.id.clone(), item.limit.max(20))).collect(),
            stock: self
                .content
                .items
                .iter()
                .map(|item| (item.id.clone(), item.limit.max(20)))
                .collect(),
            reputation: self.reputation,
            last_restock_day: self.day,
        })
    }
    fn max_addable(&self, item_id: &str) -> Option<u32> {
        let item = self.content.items.iter().find(|item| item.id == item_id)?;
        let item_limit = item.limit.checked_sub(self.inventory.get(item_id))?;
        let weight_limit =
            2400u32.saturating_sub(self.weight()).checked_div(item.weight_lbs).unwrap_or(u32::MAX);
        Some(item_limit.min(weight_limit))
    }
    /// Reject corrupted saves before a UI attempts to navigate their content IDs.
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.party.len() > 12
            || !(3..=7).contains(&self.departure_month)
            || self.day > 3660
            || self.cash_cents < 0
            || self.party.iter().any(|member| {
                member.health > 100
                    || !(0..=100).contains(&member.morale)
                    || (member.alive && member.health == 0)
            })
        {
            return Err(CommandError::InvalidSetup);
        }
        let mut joined_npcs = BTreeSet::new();
        if self
            .party
            .iter()
            .filter_map(|member| member.npc_id.as_ref())
            .any(|id| !joined_npcs.insert(id))
            || self.npcs.iter().any(|npc| npc.id.is_empty())
        {
            return Err(CommandError::InvalidSetup);
        }
        if self.status != RunStatus::Setup
            && (self.trail_id.is_none()
                || self.era_id.is_none()
                || self.occupation_id.is_none()
                || self.current_node_id.is_none()
                || self.party.is_empty())
        {
            return Err(CommandError::InvalidSetup);
        }
        if let Some(id) = &self.trail_id {
            let trail = self
                .content
                .trails
                .iter()
                .find(|trail| &trail.id == id)
                .ok_or_else(|| CommandError::UnknownId(id.clone()))?;
            for node in &trail.nodes {
                for route in &node.routes {
                    if !trail.nodes.iter().any(|target| target.id == route.target_id) {
                        return Err(CommandError::UnknownId(route.target_id.clone()));
                    }
                }
            }
            if let Some(node) = &self.current_node_id {
                if !trail.nodes.iter().any(|candidate| &candidate.id == node) {
                    return Err(CommandError::UnknownId(node.clone()));
                }
            }
            if let Some(node) = &self.target_node_id {
                if !trail.nodes.iter().any(|candidate| &candidate.id == node) {
                    return Err(CommandError::UnknownId(node.clone()));
                }
            }
        }
        if let Some(id) = &self.era_id {
            if !self.content.eras.iter().any(|era| &era.id == id) {
                return Err(CommandError::UnknownId(id.clone()));
            }
        }
        if let Some(id) = &self.occupation_id {
            if !self.content.occupations.iter().any(|occupation| &occupation.id == id) {
                return Err(CommandError::UnknownId(id.clone()));
            }
        }
        if matches!(self.status, RunStatus::Travelling)
            && (self.target_node_id.is_none() || self.route_miles_remaining == 0)
        {
            return Err(CommandError::InvalidSetup);
        }
        if !matches!(self.status, RunStatus::Travelling | RunStatus::Failed)
            && self.target_node_id.is_some()
        {
            return Err(CommandError::InvalidSetup);
        }
        match &self.status {
            RunStatus::AtLandmark(id)
            | RunStatus::AwaitingFork(id)
            | RunStatus::AwaitingRiver(id)
                if Some(id) != self.current_node_id.as_ref() =>
            {
                return Err(CommandError::InvalidSetup)
            }
            _ => {}
        }
        for id in self
            .pending_event
            .iter()
            .chain(self.scheduled_events.iter().map(|event| &event.event_id))
        {
            if !self.content.events.iter().any(|event| &event.id == id) {
                return Err(CommandError::UnknownId(id.clone()));
            }
        }
        for member in &self.party {
            for id in &member.ailments {
                if !self.content.ailments.iter().any(|ailment| &ailment.id == id) {
                    return Err(CommandError::UnknownId(id.clone()));
                }
            }
        }
        if let Some(offer) = &self.pending_counteroffer {
            if !self.npcs.iter().any(|npc| npc.id == offer.npc_id)
                || self.trade_values(
                    &offer.offered_item,
                    offer.offered_quantity,
                    &offer.wanted_item,
                    offer.wanted_quantity,
                )? != (offer.offered_value_cents, offer.wanted_value_cents)
            {
                return Err(CommandError::InvalidSetup);
            }
        }
        Ok(())
    }
    fn phase(&self, want: RunStatus) -> Result<(), CommandError> {
        if self.status == want {
            Ok(())
        } else {
            Err(CommandError::InvalidPhase)
        }
    }
    fn traveling(&self) -> Result<(), CommandError> {
        self.phase(RunStatus::Travelling)
    }
    fn at_camp(&self) -> Result<(), CommandError> {
        if matches!(
            self.status,
            RunStatus::Travelling
                | RunStatus::AtLandmark(_)
                | RunStatus::AwaitingRiver(_)
                | RunStatus::AwaitingFork(_)
        ) {
            Ok(())
        } else {
            Err(CommandError::InvalidPhase)
        }
    }
    fn begin_only_route(&mut self) -> Result<(), CommandError> {
        let node = self.node()?;
        if node.routes.len() != 1 {
            return Err(CommandError::InvalidPhase);
        }
        let target_id = node.routes[0].target_id.clone();
        let distance = node.routes[0].distance_miles;
        self.target_node_id = Some(target_id);
        self.route_miles_remaining = distance;
        self.status = RunStatus::Travelling;
        Ok(())
    }
    pub fn weight(&self) -> u32 {
        self.inventory
            .quantities
            .iter()
            .map(|(id, n)| {
                self.content
                    .items
                    .iter()
                    .find(|i| &i.id == id)
                    .map_or(0, |i| i.weight_lbs.saturating_mul(*n))
            })
            .fold(0u32, u32::saturating_add)
    }
    fn node(&self) -> Result<&LandmarkDefinition, CommandError> {
        self.content
            .trails
            .iter()
            .find(|t| Some(&t.id) == self.trail_id.as_ref())
            .and_then(|t| t.nodes.iter().find(|n| Some(&n.id) == self.current_node_id.as_ref()))
            .ok_or(CommandError::InvalidPhase)
    }
    fn weather_roll(&mut self) -> WeatherKind {
        match self.rng.stream("weather").gen_range(0..8) {
            0 => WeatherKind::Rain,
            1 => WeatherKind::Storm,
            2 => WeatherKind::Cold,
            3 => WeatherKind::Warm,
            4 => WeatherKind::Hot,
            _ => WeatherKind::Clear,
        }
    }
    fn due(&mut self, out: &mut Vec<Outcome>) {
        if let Some(i) = self.scheduled_events.iter().position(|x| x.due_day <= self.day) {
            let e = self.scheduled_events.remove(i);
            self.trigger(&e.event_id, out)
        }
    }
    fn event(&mut self, out: &mut Vec<Outcome>) {
        let ids: Vec<(String, u32)> = self
            .content
            .events
            .iter()
            .filter(|e| e.weight > 0 && e.conditions.iter().all(|c| self.matches(c)))
            .map(|e| (e.id.clone(), e.weight))
            .collect();
        if !ids.is_empty() && self.rng.stream("events").gen_range(0..100) < 15 {
            let total: u64 = ids.iter().map(|(_, weight)| u64::from(*weight)).sum();
            let mut roll = self.rng.stream("events").gen_range(0..total);
            let id = ids
                .iter()
                .find_map(|(id, weight)| {
                    if roll < u64::from(*weight) {
                        Some(id.clone())
                    } else {
                        roll -= u64::from(*weight);
                        None
                    }
                })
                .expect("positive total");
            self.trigger(&id, out)
        }
    }
    fn trigger(&mut self, id: &str, out: &mut Vec<Outcome>) {
        if let Some(e) = self.content.events.iter().find(|e| e.id == id).cloned() {
            out.push(Outcome::Event { event_id: e.id.clone(), text: e.text.clone() });
            self.effects(&e.effects, out);
            if !e.choices.is_empty() {
                self.pending_event = Some(e.id)
            }
        }
    }
    fn matches(&self, c: &Condition) -> bool {
        match c {
            Condition::Always => true,
            Condition::All(x) => x.iter().all(|c| self.matches(c)),
            Condition::Any(x) => x.iter().any(|c| self.matches(c)),
            Condition::Not(x) => !self.matches(x),
            Condition::MilesAtLeast(x) => self.miles >= *x,
            Condition::FoodBelow(x) => self.inventory.get("food") < *x,
            Condition::DayAtLeast(x) => self.day >= *x,
            Condition::AtLandmark(x) => self.current_node_id.as_deref() == Some(x),
            Condition::HasAilment(x) => self.party.iter().any(|p| p.ailments.contains(x)),
            Condition::Flag(x) => self.flags.contains(x),
            Condition::Trail(x) => self.trail_id.as_deref() == Some(x),
            Condition::Era(x) => self.era_id.as_deref() == Some(x),
            Condition::Occupation(x) => self.occupation_id.as_deref() == Some(x),
            Condition::Season(x) => self.season() == *x,
            Condition::Weather(x) => self.weather == *x,
            Condition::InventoryAtLeast { item_id, quantity } => {
                self.inventory.get(item_id) >= *quantity
            }
            Condition::CashAtLeast(cents) => self.cash_cents >= *cents,
            Condition::MoraleBelow(morale) => {
                self.party.iter().any(|member| member.alive && member.morale < *morale)
            }
        }
    }
    fn effects(&mut self, es: &[Effect], out: &mut Vec<Outcome>) {
        for e in es {
            match e {
                Effect::Message(x) => out.push(Outcome::Message(x.clone())),
                Effect::AdjustFood(x) => {
                    if *x >= 0 {
                        self.inventory.add("food", *x as u32)
                    } else {
                        self.inventory.take("food", x.unsigned_abs());
                    }
                }
                Effect::AdjustItem { item_id, quantity } => {
                    if *quantity >= 0 {
                        self.inventory.add(item_id, *quantity as u32);
                    } else {
                        self.inventory.take(item_id, quantity.unsigned_abs());
                    }
                }
                Effect::AdjustCash(x) => {
                    self.cash_cents = self.cash_cents.saturating_add(*x).max(0)
                }
                Effect::AdjustMorale(x) => {
                    for p in &mut self.party {
                        if p.alive {
                            p.morale = p.morale.saturating_add(*x).clamp(0, 100);
                        }
                    }
                }
                Effect::InflictAilment(x) => {
                    if let Some(p) =
                        self.party.iter_mut().find(|p| p.alive && !p.ailments.contains(x))
                    {
                        p.ailments.push(x.clone())
                    }
                }
                Effect::HealAilment(x) => {
                    for p in &mut self.party {
                        p.ailments.retain(|a| a != x)
                    }
                }
                Effect::LoseDays(x) => {
                    for _ in 0..(*x).min(365) {
                        self.pass_camp_day(out, false);
                        if self.status == RunStatus::Failed {
                            break;
                        }
                    }
                }
                Effect::SetFlag(x) => {
                    self.flags.insert(x.clone());
                }
                Effect::ClearFlag(x) => {
                    self.flags.remove(x);
                }
                Effect::Schedule { event_id, days } => self
                    .scheduled_events
                    .push(PendingEvent { event_id: event_id.clone(), due_day: self.day + days }),
            }
        }
    }
    fn landmark(&mut self, out: &mut Vec<Outcome>) {
        if self.route_miles_remaining != 0 {
            return;
        }
        let Some(target) = self.target_node_id.take() else {
            return;
        };
        let n = self
            .content
            .trails
            .iter()
            .find(|t| Some(&t.id) == self.trail_id.as_ref())
            .and_then(|t| t.nodes.iter().find(|n| n.id == target))
            .cloned();
        let Some(n) = n else { return };
        if self.current_node_id.as_deref() == Some(&n.id) {
            return;
        }
        self.current_node_id = Some(n.id.clone());
        match n.kind {
            LandmarkKind::Fork | LandmarkKind::Finale if n.routes.len() > 1 => {
                self.status = RunStatus::AwaitingFork(n.id.clone());
                out.push(Outcome::ForkAvailable { landmark_id: n.id })
            }
            LandmarkKind::River => {
                self.status = RunStatus::AwaitingRiver(n.id.clone());
                out.push(Outcome::RiverCrossingRequired { landmark_id: n.id })
            }
            LandmarkKind::Finale
                if self
                    .content
                    .trails
                    .iter()
                    .find(|trail| Some(&trail.id) == self.trail_id.as_ref())
                    .is_some_and(|trail| trail.goal_node_id == n.id) =>
            {
                self.status = RunStatus::Arrived;
                out.push(Outcome::ArrivedAt { landmark_id: n.id });
                out.push(Outcome::Score { points: self.score() })
            }
            _ => {
                self.status = RunStatus::AtLandmark(n.id.clone());
                out.push(Outcome::ArrivedAt { landmark_id: n.id })
            }
        }
    }
    fn progress_ailments(&mut self, out: &mut Vec<Outcome>) {
        let mut spread = Vec::new();
        for index in 0..self.party.len() {
            if !self.party[index].alive {
                continue;
            }
            let ailments = self.party[index].ailments.clone();
            for id in ailments {
                let Some(definition) =
                    self.content.ailments.iter().find(|ailment| ailment.id == id)
                else {
                    continue;
                };
                let days = {
                    let entry = self.party[index].ailment_days.entry(id.clone()).or_default();
                    *entry = entry.saturating_add(1);
                    *entry
                };
                let trait_modifier =
                    if self.party[index].traits.contains(&crate::party::Trait::Hardy) {
                        2
                    } else if self.party[index].traits.contains(&crate::party::Trait::Sickly) {
                        -2
                    } else {
                        0
                    };
                if matches!(
                    crate::health::stage(days, definition.severity),
                    crate::health::AilmentStage::Acute
                ) && self.rng.stream("health").gen_range(0..1000)
                    < definition
                        .mortality_per_mille
                        .saturating_sub(trait_modifier.max(0) as u16)
                        .saturating_add((-trait_modifier.min(0)) as u16)
                {
                    self.party[index].health = 0;
                    self.party[index].alive = false;
                    out.push(Outcome::MemberDied { name: self.party[index].name.clone() });
                    // A person can die only once per day, even if several ailments are present.
                    break;
                }
                if days >= u16::from(definition.severity.max(2)) * 3
                    && self.party[index].health >= 35
                {
                    self.party[index].ailments.retain(|ailment| ailment != &id);
                    self.party[index].ailment_days.remove(&id);
                }
                if crate::health::contagious(&id)
                    && days <= u16::from(definition.severity.max(2)) * 2
                    && self.rng.stream("health").gen_range(0..100) < 18
                {
                    spread.push(id);
                }
            }
        }
        for id in spread {
            if let Some(target) =
                self.party.iter_mut().find(|member| member.alive && !member.ailments.contains(&id))
            {
                target.ailments.push(id.clone());
                target.ailment_days.insert(id, 0);
            }
        }
    }
    fn pass_camp_day(&mut self, out: &mut Vec<Outcome>, resting: bool) {
        if self.status == RunStatus::Failed {
            return;
        }
        self.day = self.day.saturating_add(1);
        self.weather = self.weather_roll();
        let ration = match self.rations {
            RationLevel::Filling => 3,
            RationLevel::Meager => 2,
            RationLevel::BareBones => 1,
        };
        let required = self.party.iter().filter(|member| member.alive).count() as u32 * ration;
        let eaten = self.inventory.take("food", required);
        let damages: Vec<u8> = self
            .party
            .iter()
            .map(|member| {
                member
                    .ailments
                    .iter()
                    .filter_map(|id| {
                        self.content
                            .ailments
                            .iter()
                            .find(|ailment| &ailment.id == id)
                            .map(|ailment| ailment.daily_damage)
                    })
                    .fold(0u8, u8::saturating_add)
            })
            .collect();
        for (member, damage) in self.party.iter_mut().zip(damages) {
            if !member.alive {
                continue;
            }
            if advance(member, damage) {
                out.push(Outcome::MemberDied { name: member.name.clone() });
                continue;
            }
            if eaten < required {
                member.health = member.health.saturating_sub(5);
            }
            if resting && eaten == required {
                member.health = member.health.saturating_add(5).min(100);
            }
            if member.health == 0 {
                member.alive = false;
                out.push(Outcome::MemberDied { name: member.name.clone() });
            }
        }
        self.progress_ailments(out);
        if !self.party.iter().any(|member| member.alive) {
            self.status = RunStatus::Failed;
        }
    }
    fn season(&self) -> Season {
        match self.date().1 {
            3..=5 => Season::Spring,
            6..=8 => Season::Summer,
            9..=11 => Season::Autumn,
            _ => Season::Winter,
        }
    }
    fn terrain(&self) -> Terrain {
        match self.current_node_id.as_deref().unwrap_or_default() {
            id if id.contains("mountain") || id.contains("pass") => Terrain::Mountains,
            id if id.contains("river") => Terrain::RiverValley,
            id if id.contains("fort") => Terrain::Plains,
            _ => Terrain::Plains,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn branch_content() -> GameContent {
        let mut content = GameContent::starter();
        content.trails[0].nodes = vec![
            LandmarkDefinition {
                id: "independence".into(),
                name: "Start".into(),
                mile: 0,
                kind: LandmarkKind::Town,
                routes: vec![RouteDefinition {
                    id: "to-river".into(),
                    label: "River".into(),
                    target_id: "river".into(),
                    distance_miles: 10,
                }],
                river: None,
                store: true,
            },
            LandmarkDefinition {
                id: "river".into(),
                name: "River".into(),
                mile: 10,
                kind: LandmarkKind::River,
                routes: vec![RouteDefinition {
                    id: "to-fork".into(),
                    label: "Fork".into(),
                    target_id: "fork".into(),
                    distance_miles: 10,
                }],
                river: Some(RiverDefinition {
                    width_feet: 10,
                    depth_feet: 1,
                    ferry_cost_cents: Some(100),
                }),
                store: false,
            },
            LandmarkDefinition {
                id: "fork".into(),
                name: "Fork".into(),
                mile: 20,
                kind: LandmarkKind::Fork,
                routes: vec![
                    RouteDefinition {
                        id: "short".into(),
                        label: "Short".into(),
                        target_id: "willamette".into(),
                        distance_miles: 5,
                    },
                    RouteDefinition {
                        id: "long".into(),
                        label: "Long".into(),
                        target_id: "detour".into(),
                        distance_miles: 100,
                    },
                ],
                river: None,
                store: false,
            },
            LandmarkDefinition {
                id: "detour".into(),
                name: "Detour".into(),
                mile: 120,
                kind: LandmarkKind::Landmark,
                routes: vec![RouteDefinition {
                    id: "finish".into(),
                    label: "Finish".into(),
                    target_id: "willamette".into(),
                    distance_miles: 5,
                }],
                river: None,
                store: false,
            },
            LandmarkDefinition {
                id: "willamette".into(),
                name: "Goal".into(),
                mile: 25,
                kind: LandmarkKind::Finale,
                routes: vec![],
                river: None,
                store: false,
            },
        ];
        content
    }
    fn run(seed: u64) -> GameState {
        let mut g = GameState::new(seed);
        g.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["Ada".into(), "Ben".into(), "Clara".into(), "David".into(), "Eve".into()],
            departure_month: 4,
        });
        g.apply(Command::Buy { item_id: "food".into(), quantity: 500 });
        g.apply(Command::Buy { item_id: "oxen".into(), quantity: 3 });
        g.apply(Command::Depart);
        assert_eq!(g.status, RunStatus::Travelling);
        g
    }
    fn trade_game(seed: u64) -> GameState {
        let mut game = run(seed);
        let npc = game.npcs.iter_mut().find(|npc| npc.id == "emigrant_train").unwrap();
        npc.inventory.insert("clothing".into(), 4);
        game
    }
    fn assert_rejected_without_mutation(game: &mut GameState, command: Command) {
        let before = serde_json::to_value(&*game).unwrap();
        assert!(matches!(game.apply(command).as_slice(), [Outcome::Rejected(_)]));
        assert_eq!(serde_json::to_value(&*game).unwrap(), before);
    }
    #[test]
    fn deterministic() {
        let (mut a, mut b) = (run(4), run(4));
        for _ in 0..5 {
            assert_eq!(a.apply(Command::TravelDay), b.apply(Command::TravelDay));
        }
    }
    #[test]
    fn resume() {
        let mut a = run(5);
        a.apply(Command::TravelDay);
        let mut b: GameState = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert_eq!(a.apply(Command::TravelDay), b.apply(Command::TravelDay));
    }
    #[test]
    fn river_wait_consumes_day_but_does_not_cross() {
        let mut game = GameState::with_content(1, branch_content());
        game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["Ada".into(), "Ben".into(), "Clara".into(), "David".into(), "Eve".into()],
            departure_month: 4,
        });
        game.apply(Command::Buy { item_id: "oxen".into(), quantity: 1 });
        game.apply(Command::Buy { item_id: "food".into(), quantity: 100 });
        game.apply(Command::Depart);
        game.apply(Command::TravelDay);
        assert!(matches!(game.status, RunStatus::AwaitingRiver(_)));
        let day = game.day;
        let food = game.inventory.get("food");
        game.apply(Command::CrossRiver { method: CrossMethod::Wait });
        assert!(matches!(game.status, RunStatus::AwaitingRiver(_)));
        assert_eq!(game.day, day + 1);
        assert!(game.inventory.get("food") < food);
    }
    #[test]
    fn selected_route_clamps_and_arrives_only_at_goal() {
        let mut game = GameState::with_content(2, branch_content());
        game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["Ada".into(), "Ben".into(), "Clara".into(), "David".into(), "Eve".into()],
            departure_month: 4,
        });
        game.apply(Command::Buy { item_id: "oxen".into(), quantity: 1 });
        game.apply(Command::Buy { item_id: "food".into(), quantity: 100 });
        game.apply(Command::Depart);
        game.apply(Command::TravelDay);
        game.apply(Command::CrossRiver { method: CrossMethod::Ferry });
        game.apply(Command::TravelDay);
        assert!(matches!(game.status, RunStatus::AwaitingFork(_)));
        game.apply(Command::ChooseRoute { route_id: "short".into() });
        game.apply(Command::TravelDay);
        assert_eq!(game.status, RunStatus::Arrived);
        assert_eq!(game.miles, 25);
    }
    #[test]
    fn rejected_command_does_not_mutate_state() {
        let mut game = GameState::new(2);
        let before = serde_json::to_string(&game).unwrap();
        assert!(matches!(game.apply(Command::Depart).as_slice(), [Outcome::Rejected(_)]));
        assert_eq!(before, serde_json::to_string(&game).unwrap());
    }
    #[test]
    fn event_weights_bias_deterministic_selection() {
        let mut game = run(19);
        game.content.events = vec![
            EventDefinition {
                id: "common".into(),
                text: "common".into(),
                weight: 100,
                conditions: vec![Condition::Always],
                effects: vec![],
                choices: vec![],
            },
            EventDefinition {
                id: "rare".into(),
                text: "rare".into(),
                weight: 1,
                conditions: vec![Condition::Always],
                effects: vec![],
                choices: vec![],
            },
        ];
        let mut common = 0;
        let mut rare = 0;
        for _ in 0..10_000 {
            let mut outcomes = Vec::new();
            game.event(&mut outcomes);
            for outcome in outcomes {
                if let Outcome::Event { event_id, .. } = outcome {
                    if event_id == "common" {
                        common += 1
                    } else {
                        rare += 1
                    }
                }
            }
        }
        assert!(common > rare * 20, "common={common}, rare={rare}");
    }

    #[test]
    fn starvation_ends_a_party_and_terminal_commands_are_inert() {
        let mut game = run(42);
        game.inventory.quantities.insert("food".into(), 0);
        for _ in 0..20 {
            game.apply(Command::Continue);
        }
        assert_eq!(game.status, RunStatus::Failed);
        assert!(game.party.iter().all(|member| !member.alive && member.health == 0));
        let before = serde_json::to_value(&game).unwrap();
        for command in [Command::Continue, Command::Rest { days: 10 }, Command::Talk] {
            assert!(matches!(game.apply(command).as_slice(), [Outcome::Rejected(_)]));
            assert_eq!(serde_json::to_value(&game).unwrap(), before);
        }
        game.validate().unwrap();
    }

    #[test]
    fn river_wait_can_starve_and_rest_does_not_restore_dead_members() {
        let mut game = run(3);
        game.status = RunStatus::AwaitingRiver("river".into());
        game.content = branch_content();
        game.current_node_id = Some("river".into());
        game.target_node_id = None;
        game.route_miles_remaining = 0;
        game.inventory.quantities.insert("food".into(), 0);
        for member in &mut game.party {
            member.health = 5;
        }
        game.apply(Command::CrossRiver { method: CrossMethod::Wait });
        assert_eq!(game.status, RunStatus::Failed);
        assert!(game.party.iter().all(|member| !member.alive));
        let mut game = run(5);
        game.party[0].alive = false;
        game.party[0].health = 0;
        game.apply(Command::Rest { days: 2 });
        assert!(!game.party[0].alive);
        assert_eq!(game.party[0].health, 0);
        assert_eq!(game.inventory.get("food"), 476);
    }

    #[test]
    fn market_stock_depletes_and_npc_trade_is_atomic() {
        let mut game = GameState::new(8);
        game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
            departure_month: 4,
        });
        game.apply(Command::Buy { item_id: "food".into(), quantity: 10 });
        assert_eq!(game.markets["independence"].stock["food"], 1_990);
        let before = serde_json::to_value(&game).unwrap();
        assert!(matches!(
            game.apply(Command::Barter {
                npc_id: "emigrant_train".into(),
                offered_item: "food".into(),
                offered_quantity: 0,
                wanted_item: "food".into(),
                wanted_quantity: 1
            })
            .as_slice(),
            [Outcome::Rejected(_)]
        ));
        assert_eq!(serde_json::to_value(&game).unwrap(), before);
    }

    #[test]
    fn barter_uses_priced_quantities_and_conserves_items() {
        let mut game = trade_game(9);
        let before_food = game.inventory.get("food");
        let before_clothing = game.inventory.get("clothing");
        assert!(matches!(
            game.apply(Command::Barter {
                npc_id: "emigrant_train".into(),
                offered_item: "food".into(),
                offered_quantity: 50,
                wanted_item: "clothing".into(),
                wanted_quantity: 1,
            })
            .as_slice(),
            [Outcome::Message(message)] if message == "Trade accepted."
        ));
        assert_eq!(game.inventory.get("food"), before_food - 50);
        assert_eq!(game.inventory.get("clothing"), before_clothing + 1);
        let npc = &game.npcs[0];
        assert_eq!(npc.inventory["food"], 150);
        assert_eq!(npc.inventory["clothing"], 3);
    }

    #[test]
    fn invalid_barter_is_atomic_and_same_item_is_rejected() {
        let mut game = trade_game(10);
        assert_rejected_without_mutation(
            &mut game,
            Command::Barter {
                npc_id: "emigrant_train".into(),
                offered_item: "food".into(),
                offered_quantity: 1,
                wanted_item: "food".into(),
                wanted_quantity: 1,
            },
        );
        assert_rejected_without_mutation(
            &mut game,
            Command::Barter {
                npc_id: "emigrant_train".into(),
                offered_item: "unknown".into(),
                offered_quantity: 1,
                wanted_item: "clothing".into(),
                wanted_quantity: 1,
            },
        );
    }

    #[test]
    fn overflowing_barter_quote_is_rejected_without_mutation() {
        let mut game = trade_game(10);
        game.content.items.push(ItemDefinition {
            id: "priceless".into(),
            name: "Priceless".into(),
            unit: "crate".into(),
            price_cents: i64::MAX,
            weight_lbs: 0,
            limit: u32::MAX,
        });
        game.inventory.quantities.insert("priceless".into(), 3);
        game.npcs[0].inventory.insert("priceless".into(), 2);
        assert_rejected_without_mutation(
            &mut game,
            Command::Barter {
                npc_id: "emigrant_train".into(),
                offered_item: "priceless".into(),
                offered_quantity: 3,
                wanted_item: "clothing".into(),
                wanted_quantity: 1,
            },
        );
    }

    #[test]
    fn counteroffer_is_persisted_and_revalidated_before_acceptance() {
        let mut game = trade_game(11);
        let result = game.apply(Command::Barter {
            npc_id: "emigrant_train".into(),
            offered_item: "food".into(),
            offered_quantity: 1,
            wanted_item: "clothing".into(),
            wanted_quantity: 1,
        });
        assert!(
            matches!(result.as_slice(), [Outcome::Message(message)] if message.contains("counteroffer"))
        );
        let offer = game.pending_counteroffer.clone().unwrap();
        assert_eq!(offer.offered_quantity, 50);
        assert_eq!(offer.offered_value_cents, offer.wanted_value_cents);
        assert_rejected_without_mutation(
            &mut game,
            Command::AcceptCounteroffer {
                npc_id: offer.npc_id.clone(),
                offered_item: offer.offered_item.clone(),
                offered_quantity: offer.offered_quantity - 1,
                wanted_item: offer.wanted_item.clone(),
                wanted_quantity: offer.wanted_quantity,
            },
        );
        assert!(matches!(
            game.apply(Command::AcceptCounteroffer {
                npc_id: offer.npc_id,
                offered_item: offer.offered_item,
                offered_quantity: offer.offered_quantity,
                wanted_item: offer.wanted_item,
                wanted_quantity: offer.wanted_quantity,
            })
            .as_slice(),
            [Outcome::Message(message)] if message == "Counteroffer accepted."
        ));
        assert!(game.pending_counteroffer.is_none());
    }

    #[test]
    fn npc_trade_reputation_can_only_increase_once_per_day() {
        let mut game = trade_game(12);
        for _ in 0..2 {
            game.apply(Command::Barter {
                npc_id: "emigrant_train".into(),
                offered_item: "food".into(),
                offered_quantity: 50,
                wanted_item: "clothing".into(),
                wanted_quantity: 1,
            });
        }
        assert_eq!(game.reputation, 1);
        assert_eq!(game.npcs[0].reputation, 1);
        assert_eq!(game.npcs[0].last_reputation_day, Some(game.day));
    }

    #[test]
    fn npc_identity_prevents_duplicate_invites_and_protects_same_named_leader() {
        let mut game = trade_game(13);
        game.party[0].name = "Holloway family".into();
        game.apply(Command::InviteNpc { npc_id: "emigrant_train".into() });
        assert_eq!(game.party.iter().filter(|member| member.npc_id.is_some()).count(), 1);
        assert_rejected_without_mutation(
            &mut game,
            Command::InviteNpc { npc_id: "emigrant_train".into() },
        );
        game.apply(Command::DismissNpc { npc_id: "emigrant_train".into() });
        assert_eq!(game.party.len(), 5);
        assert_eq!(game.party[0].name, "Holloway family");
        assert!(game.party.iter().all(|member| member.npc_id.is_none()));
    }

    #[test]
    fn market_preview_matches_buy_and_sellback_is_stock_bounded() {
        let mut game = GameState::new(14);
        game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
            departure_month: 4,
        });
        let quote = game.price_cents("food").unwrap();
        let cash_before = game.cash_cents;
        assert!(
            matches!(game.apply(Command::Buy { item_id: "food".into(), quantity: 10 }).as_slice(),
            [Outcome::Purchased { cost_cents, .. }] if *cost_cents == quote * 10)
        );
        game.apply(Command::Sell { item_id: "food".into(), quantity: 10 });
        assert!(game.cash_cents < cash_before);
        assert_eq!(game.markets["independence"].stock["food"], 2_000);
        assert_rejected_without_mutation(
            &mut game,
            Command::Sell { item_id: "food".into(), quantity: 1 },
        );
    }

    #[test]
    fn forage_and_fishing_keep_food_under_item_and_wagon_limits() {
        let mut game = run(15);
        game.inventory.quantities.insert("food".into(), 2_000);
        assert!(matches!(game.apply(Command::Forage).as_slice(), outcomes
            if outcomes.iter().any(|outcome| matches!(outcome, Outcome::Message(message) if message == "Foraged 0 lbs of food."))));
        game.content.items.iter_mut().find(|item| item.id == "food").unwrap().limit = 5_000;
        game.inventory.quantities.insert("food".into(), 2_400);
        game.current_node_id = Some("river".into());
        assert!(matches!(game.apply(Command::Fish).as_slice(), outcomes
            if outcomes.iter().any(|outcome| matches!(outcome, Outcome::Message(message) if message == "Caught 0 lbs of fish."))));
    }

    #[test]
    fn mortality_emits_one_death_for_a_member_with_multiple_ailments() {
        let mut game = run(16);
        game.content.ailments = vec![
            AilmentDefinition {
                id: "measles".into(),
                name: "Measles".into(),
                severity: 2,
                daily_damage: 0,
                mortality_per_mille: 1_000,
            },
            AilmentDefinition {
                id: "cholera".into(),
                name: "Cholera".into(),
                severity: 2,
                daily_damage: 0,
                mortality_per_mille: 1_000,
            },
        ];
        game.party[0].ailments = vec!["measles".into(), "cholera".into()];
        game.party[0].ailment_days = BTreeMap::from([("measles".into(), 1), ("cholera".into(), 1)]);
        let deaths = game
            .apply(Command::TravelDay)
            .into_iter()
            .filter(|outcome| matches!(outcome, Outcome::MemberDied { name } if name == "Ada"))
            .count();
        assert_eq!(deaths, 1);
    }

    #[test]
    fn contagious_disease_spreads_and_eventually_recovers_deterministically() {
        let mut game = run(17);
        game.content.ailments = vec![AilmentDefinition {
            id: "measles".into(),
            name: "Measles".into(),
            severity: 100,
            daily_damage: 0,
            mortality_per_mille: 0,
        }];
        game.party[0].ailments.push("measles".into());
        for _ in 0..30 {
            game.progress_ailments(&mut Vec::new());
        }
        assert!(game
            .party
            .iter()
            .skip(1)
            .any(|member| member.ailments.contains(&"measles".into())));
        game.content.ailments[0].severity = 2;
        game.party[0].ailment_days.insert("measles".into(), 5);
        game.progress_ailments(&mut Vec::new());
        assert!(!game.party[0].ailments.contains(&"measles".into()));
    }

    proptest::proptest! {
        #[test]
        fn travel_invariants_hold_for_seeded_journeys(seed in proptest::prelude::any::<u64>(), pace in 0u8..3) {
            let mut game = run(seed);
            game.apply(Command::SetPace([Pace::Steady, Pace::Strenuous, Pace::Grueling][pace as usize]));
            let mut prior_miles = 0;
            let mut prior_alive = 5;
            for _ in 0..300 {
                game.apply(Command::Continue);
                proptest::prop_assert!(game.miles >= prior_miles && game.miles <= 2040);
                let alive = game.party.iter().filter(|member| member.alive).count();
                proptest::prop_assert!(alive <= prior_alive);
                proptest::prop_assert!(game.party.iter().all(|member| member.health <= 100));
                prior_miles = game.miles;
                prior_alive = alive;
                if matches!(game.status, RunStatus::Arrived | RunStatus::Failed) { break; }
            }
            proptest::prop_assert!(matches!(game.status, RunStatus::Arrived | RunStatus::Failed));
        }
    }
}
