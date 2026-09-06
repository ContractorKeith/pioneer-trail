//! Command-driven deterministic journey state.
use crate::{
    content::*,
    health::{advance, PartyMember},
    rng::SimRng,
    score,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Inventory {
    pub quantities: BTreeMap<String, u32>,
}
impl Inventory {
    pub fn get(&self, id: &str) -> u32 {
        self.quantities.get(id).copied().unwrap_or(0)
    }
    fn add(&mut self, id: &str, n: u32) {
        *self.quantities.entry(id.into()).or_default() += n
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
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Command {
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
        }
    }
    pub fn apply(&mut self, c: Command) -> Vec<Outcome> {
        self.try_apply(c).unwrap_or_else(|e| vec![Outcome::Rejected(e)])
    }
    pub fn try_apply(&mut self, c: Command) -> Result<Vec<Outcome>, CommandError> {
        if self.pending_event.is_some() && !matches!(c, Command::Respond { .. }) {
            return Err(CommandError::InvalidPhase);
        }
        match c {
            Command::Configure { trail_id, era_id, occupation_id, party, departure_month } => {
                self.configure(trail_id, era_id, occupation_id, party, departure_month)
            }
            Command::Buy { item_id, quantity } => self.buy(&item_id, quantity),
            Command::Depart => {
                self.phase(RunStatus::Outfitting)?;
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
                self.traveling()?;
                self.pace = v;
                Ok(vec![Outcome::Message("Pace changed".into())])
            }
            Command::SetRations(v) => {
                self.traveling()?;
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
            || names.is_empty()
            || names.len() > 5
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
        self.party = names.into_iter().map(PartyMember::new).collect();
        self.departure_month = month;
        self.current_node_id = Some(trail.start_node_id.clone());
        self.trail_id = Some(t);
        self.era_id = Some(e);
        self.occupation_id = Some(o);
        self.status = RunStatus::Outfitting;
        Ok(vec![Outcome::Configured])
    }
    fn buy(&mut self, id: &str, n: u32) -> Result<Vec<Outcome>, CommandError> {
        if !self.can_shop() {
            return Err(CommandError::InvalidPhase);
        }
        let item = self
            .content
            .items
            .iter()
            .find(|x| x.id == id)
            .ok_or_else(|| CommandError::UnknownId(id.into()))?;
        let cost =
            item.price_cents.checked_mul(i64::from(n)).ok_or(CommandError::InsufficientCash)?;
        if self.inventory.get(id) + n > item.limit || self.weight() + item.weight_lbs * n > 2400 {
            return Err(CommandError::CapacityExceeded);
        }
        if cost > self.cash_cents {
            return Err(CommandError::InsufficientCash);
        }
        self.cash_cents -= cost;
        self.inventory.add(id, n);
        Ok(vec![Outcome::Purchased { item_id: id.into(), quantity: n, cost_cents: cost }])
    }
    fn travel(&mut self) -> Result<Vec<Outcome>, CommandError> {
        self.traveling()?;
        self.day += 1;
        if self.inventory.get("oxen") == 0 {
            self.status = RunStatus::Failed;
            return Ok(vec![Outcome::Message("Your wagon cannot move without oxen.".into())]);
        }
        self.weather = self.weather_roll();
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
                    .sum()
            })
            .collect();
        for (p, damage) in self.party.iter_mut().zip(damages) {
            if advance(p, damage) {
                out.push(Outcome::MemberDied { name: p.name.clone() })
            }
        }
        if eaten_food < required_food {
            for member in &mut self.party {
                if member.alive {
                    member.health = member.health.saturating_sub(8);
                }
            }
        }
        if !self.party.iter().any(|member| member.alive) {
            self.status = RunStatus::Failed;
            return Ok(out);
        }
        let base = match self.pace {
            Pace::Steady => 15,
            Pace::Strenuous => 20,
            Pace::Grueling => 25,
        };
        let penalty = match self.weather {
            WeatherKind::Storm | WeatherKind::Snow => 7,
            WeatherKind::Rain | WeatherKind::Cold => 3,
            _ => 0,
        };
        let moved = (base - penalty).min(self.route_miles_remaining);
        self.miles += moved;
        self.route_miles_remaining -= moved;
        out.push(Outcome::DayAdvanced { day: self.day, miles: self.miles, weather: self.weather });
        self.due(&mut out);
        if self.pending_event.is_none() {
            self.event(&mut out)
        }
        if self.pending_event.is_none() {
            self.landmark(&mut out)
        }
        Ok(out)
    }
    fn rest(&mut self, days: u32) -> Result<Vec<Outcome>, CommandError> {
        self.traveling()?;
        for _ in 0..days.min(30) {
            self.day += 1;
            let required_food = self.party.iter().filter(|p| p.alive).count() as u32 * 3;
            let eaten_food = self.inventory.take("food", required_food);
            for p in &mut self.party {
                if p.alive {
                    p.health = if eaten_food < required_food {
                        p.health.saturating_sub(5)
                    } else {
                        (p.health + 5).min(100)
                    };
                }
            }
        }
        Ok(vec![Outcome::Message(format!("Rested {} days", days.min(30)))])
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
        if m == CrossMethod::Ferry {
            let cost = river.ferry_cost_cents.ok_or(CommandError::InvalidChoice)?;
            if self.cash_cents < cost {
                return Err(CommandError::InsufficientCash);
            }
            self.cash_cents -= cost
        }
        let risk = match m {
            CrossMethod::Ferry | CrossMethod::Wait => 0,
            CrossMethod::Guide => 10,
            CrossMethod::Ford => river.depth_feet * 20,
            CrossMethod::Caulk => river.width_feet / 20,
        };
        if self.rng.stream("rivers").gen_range(0..100) < risk.min(90) {
            self.inventory.remove("food", 50);
        }
        self.status = RunStatus::Travelling;
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
        let mut out = vec![];
        self.effects(&c.effects, &mut out);
        self.pending_event = None;
        Ok(out)
    }
    fn talk(&mut self) -> Result<Vec<Outcome>, CommandError> {
        self.traveling()?;
        let candidates: Vec<_> = self
            .content
            .quotes
            .iter()
            .filter(|q| {
                q.landmark_id.is_none()
                    || q.landmark_id.as_deref() == self.current_node_id.as_deref()
            })
            .collect();
        let q = candidates
            .choose(self.rng.stream("quotes"))
            .ok_or_else(|| CommandError::UnknownId("quote".into()))?;
        Ok(vec![Outcome::Quote { quote_id: q.id.clone(), text: q.text.clone() }])
    }
    fn treat(&mut self, i: usize, a: &str) -> Result<Vec<Outcome>, CommandError> {
        let p = self.party.get_mut(i).ok_or(CommandError::InvalidChoice)?;
        if !p.ailments.iter().any(|x| x == a) || !self.inventory.remove("medicine", 1) {
            return Err(CommandError::InvalidChoice);
        }
        p.ailments.retain(|x| x != a);
        p.health = (p.health + 15).min(100);
        Ok(vec![Outcome::Treated { member_index: i, ailment_id: a.into() }])
    }
    fn hunt(&mut self, food: u32, ammo: u32) -> Result<Vec<Outcome>, CommandError> {
        self.traveling()?;
        if !self.inventory.remove("ammunition", ammo) {
            return Err(CommandError::InvalidChoice);
        }
        self.inventory.add("food", food);
        Ok(vec![Outcome::Message(format!("Brought back {} lbs of food", food))])
    }
    fn raft(&mut self, lost: u32, casualties: u8) -> Result<Vec<Outcome>, CommandError> {
        self.traveling()?;
        self.inventory.remove("food", lost);
        for p in self.party.iter_mut().filter(|p| p.alive).take(casualties as usize) {
            p.alive = false
        }
        Ok(vec![Outcome::Message("Rafting result recorded".into())])
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
        let ordinal = u32::from(self.departure_month.saturating_sub(1)) * 30 + self.day;
        (
            year + (ordinal / 360) as i32,
            ((ordinal % 360) / 30 + 1) as u8,
            ((ordinal % 30) + 1) as u8,
        )
    }
    pub fn can_shop(&self) -> bool {
        matches!(self.status, RunStatus::Outfitting | RunStatus::AtLandmark(_))
            && self.current_landmark().is_some_and(|node| node.store)
    }
    pub fn price_cents(&self, item_id: &str) -> Option<i64> {
        self.content.items.iter().find(|item| item.id == item_id).map(|item| item.price_cents)
    }
    /// Reject corrupted saves before a UI attempts to navigate their content IDs.
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.party.len() > 5 || !(3..=7).contains(&self.departure_month) {
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
        for id in [&self.era_id, &self.occupation_id].into_iter().flatten() {
            let found = self.content.eras.iter().any(|x| &x.id == id)
                || self.content.occupations.iter().any(|x| &x.id == id);
            if !found {
                return Err(CommandError::UnknownId(id.clone()));
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
                self.content.items.iter().find(|i| &i.id == id).map_or(0, |i| i.weight_lbs * n)
            })
            .sum()
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
        let ids: Vec<String> = self
            .content
            .events
            .iter()
            .filter(|e| e.weight > 0 && e.conditions.iter().all(|c| self.matches(c)))
            .map(|e| e.id.clone())
            .collect();
        if !ids.is_empty() && self.rng.stream("events").gen_range(0..100) < 15 {
            let id = ids.choose(self.rng.stream("events")).unwrap().clone();
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
                        self.inventory.remove("food", x.unsigned_abs());
                    }
                }
                Effect::AdjustCash(x) => self.cash_cents = self.cash_cents.saturating_add(*x),
                Effect::AdjustMorale(x) => {
                    for p in &mut self.party {
                        p.morale = (p.morale + *x).clamp(0, 100)
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
                Effect::LoseDays(x) => self.day += *x,
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
            LandmarkKind::Fork => {
                self.status = RunStatus::AwaitingFork(n.id.clone());
                out.push(Outcome::ForkAvailable { landmark_id: n.id })
            }
            LandmarkKind::River => {
                self.status = RunStatus::AwaitingRiver(n.id.clone());
                out.push(Outcome::RiverCrossingRequired { landmark_id: n.id })
            }
            LandmarkKind::Finale => {
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
    fn season(&self) -> Season {
        match u32::from(self.departure_month) + self.day / 30 {
            0..=5 => Season::Spring,
            6..=8 => Season::Summer,
            9..=11 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn run(seed: u64) -> GameState {
        let mut g = GameState::new(seed);
        g.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["Ada".into()],
            departure_month: 4,
        });
        g.apply(Command::Buy { item_id: "food".into(), quantity: 500 });
        g.apply(Command::Depart);
        g
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
}
