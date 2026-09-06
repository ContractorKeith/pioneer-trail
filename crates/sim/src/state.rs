//! Command-driven deterministic journey state.
use crate::{
    calendar::CalendarDate,
    content::*,
    economy::{Counteroffer, Market, NpcTrain},
    family::{FamilyState, Pregnancy},
    health::{advance, PartyMember, Sex},
    minigame::{MinigameKind, MinigameSession},
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
    #[serde(default)]
    pub family: FamilyState,
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
    #[serde(default)]
    pub active_minigame: Option<MinigameSession>,
    #[serde(default)]
    pub loose_bullets: u8,
    #[serde(default)]
    pub last_fresh_food_day: Option<u32>,
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
    Repair,
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
    BeginHunt,
    HuntResult {
        food_lbs: u32,
        shots: u32,
    },
    RaftResult {
        cargo_lost_lbs: u32,
        casualties: u8,
        completed: bool,
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
            family: FamilyState::default(),
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
            npcs: vec![
                NpcTrain {
                    id: "emigrant_train".into(),
                    name: "Ruth Holloway".into(),
                    reputation: 0,
                    inventory: BTreeMap::from([("food".into(), 100)]),
                    recurring: true,
                    last_reputation_day: None,
                    first_mile: 0,
                    last_mile: 700,
                    period_days: 1,
                    day_window: 1,
                },
                NpcTrain {
                    id: "mercer_wagon".into(),
                    name: "Elias Mercer".into(),
                    reputation: 0,
                    inventory: BTreeMap::from([("clothing".into(), 2), ("food".into(), 60)]),
                    recurring: true,
                    last_reputation_day: None,
                    first_mile: 500,
                    last_mile: 1350,
                    period_days: 3,
                    day_window: 2,
                },
                NpcTrain {
                    id: "bird_traders".into(),
                    name: "Nora Bird".into(),
                    reputation: 0,
                    inventory: BTreeMap::from([("medicine".into(), 1), ("food".into(), 40)]),
                    recurring: true,
                    last_reputation_day: None,
                    first_mile: 1200,
                    last_mile: 1850,
                    period_days: 4,
                    day_window: 2,
                },
            ],
            pending_counteroffer: None,
            active_minigame: None,
            loose_bullets: 0,
            last_fresh_food_day: None,
        }
    }
    pub fn npc_present(&self, npc_id: &str) -> bool {
        self.npcs.iter().find(|npc| npc.id == npc_id).is_some_and(|npc| {
            npc.recurring
                && (npc.period_days == 0
                    || (self.miles >= npc.first_mile
                        && self.miles <= npc.last_mile
                        && self.day % npc.period_days < npc.day_window.max(1)))
        })
    }
    fn present_npc_ids(&self) -> BTreeSet<String> {
        self.npcs.iter().filter(|npc| self.npc_present(&npc.id)).map(|npc| npc.id.clone()).collect()
    }
    fn announce_npc_arrivals(&self, before: &BTreeSet<String>, out: &mut Vec<Outcome>) {
        for npc in
            self.npcs.iter().filter(|npc| self.npc_present(&npc.id) && !before.contains(&npc.id))
        {
            out.push(Outcome::Message(format!("{} draws near again.", npc.name)));
        }
    }
    pub fn apply(&mut self, c: Command) -> Vec<Outcome> {
        let mut outcomes = self.try_apply(c).unwrap_or_else(|e| vec![Outcome::Rejected(e)]);
        if matches!(self.status, RunStatus::Arrived | RunStatus::Failed) {
            self.pending_event = None;
            self.active_minigame = None;
        }
        let deaths = outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                Outcome::MemberDied { name } => Some(name.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !deaths.is_empty() {
            outcomes.extend(
                crate::party::mourn(&mut self.party, &deaths).into_iter().map(Outcome::Message),
            );
        }
        outcomes
    }
    fn try_apply(&mut self, c: Command) -> Result<Vec<Outcome>, CommandError> {
        if matches!(self.status, RunStatus::Arrived | RunStatus::Failed) {
            return Err(CommandError::InvalidPhase);
        }
        if let Some(session) = &self.active_minigame {
            let matching_result = matches!(
                (&session.kind, &c),
                (MinigameKind::Hunt, Command::HuntResult { .. })
                    | (MinigameKind::Raft, Command::RaftResult { .. })
            );
            if !matching_result {
                return Err(CommandError::InvalidPhase);
            }
        }
        if self.pending_event.is_some() && !matches!(c, Command::Respond { .. } | Command::Repair) {
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
            Command::Repair => self.repair(),
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
            Command::BeginHunt => self.begin_hunt(),
            Command::HuntResult { food_lbs, shots } => self.hunt(food_lbs, shots),
            Command::RaftResult { cargo_lost_lbs, casualties, completed } => {
                self.raft(cargo_lost_lbs, casualties, completed)
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
        if (o == "soldier" && e != "1866") || (t == "mormon" && e == "1843") {
            return Err(CommandError::InvalidSetup);
        }
        let job = self
            .content
            .occupations
            .iter()
            .find(|x| x.id == o)
            .ok_or_else(|| CommandError::UnknownId(o.clone()))?;
        self.cash_cents = job.starting_cash_cents;
        self.party = names.into_iter().map(PartyMember::new).collect();
        crate::party::initialize(&mut self.party, &o, &mut self.rng);
        self.family = FamilyState::default();
        if self.rng.stream("family").gen_range(0..100) < 15 {
            if let Some(mother) = self.party.iter().find(|member| {
                member.alive && member.sex == Sex::Female && (18..=40).contains(&member.age)
            }) {
                self.family.pregnancies.push(Pregnancy {
                    mother: mother.name.clone(),
                    due_day: self.day + self.rng.stream("family").gen_range(60..=130),
                });
            }
        }
        self.departure_month = month;
        self.current_node_id = Some(trail.start_node_id.clone());
        self.trail_id = Some(t);
        self.era_id = Some(e);
        self.occupation_id = Some(o);
        if self.occupation_id.as_deref() == Some("merchant") {
            self.inventory.add("trade_goods", 2);
        }
        if self.occupation_id.as_deref() == Some("preacher") {
            self.reputation = self.reputation.saturating_add(10);
        }
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
        let current_weight = self.weight();
        let market_id = self.current_node_id.clone().unwrap_or_default();
        let mut market = self.market_at(&market_id);
        market.replenish(self.day);
        if market.stock.get(id).copied().unwrap_or_default() < n {
            return Err(CommandError::InvalidChoice);
        }
        let cost = self
            .shop_unit_price(&market, &item)
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
        let present_before = self.present_npc_ids();
        self.day += 1;
        if self.inventory.get("oxen") == 0 {
            self.status = RunStatus::Failed;
            return Ok(vec![Outcome::Message("Your wagon cannot move without oxen.".into())]);
        }
        let season = self.season();
        let climate = self.region().climate;
        self.weather_state.advance_in(&mut self.rng, season, climate);
        self.weather = self.weather_state.kind;
        self.spoil_food();
        let eat = match self.rations {
            RationLevel::Filling => 3,
            RationLevel::Meager => 2,
            RationLevel::BareBones => 1,
        };
        let required_food = eat * self.party.iter().filter(|p| p.alive).count() as u32;
        let eaten_food = self.inventory.take("food", required_food);
        let mut out = vec![];
        let damages: Vec<u8> =
            self.party.iter().map(|member| self.ailment_damage(member)).collect();
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
        if eaten_food == required_food && self.pace == Pace::Steady {
            for member in self.party.iter_mut().filter(|member| member.alive) {
                member.health = member.health.saturating_add(1).min(100);
            }
        }
        for member in &mut self.party {
            if member.alive && member.health == 0 {
                member.alive = false;
                out.push(Outcome::MemberDied { name: member.name.clone() });
            }
        }
        self.party_daily(&mut out, false, eaten_food == required_food);
        self.family_daily(&mut out);
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
        let fatigue_gain = match self.pace {
            Pace::Steady => 2,
            Pace::Strenuous => 5,
            Pace::Grueling => 9,
        };
        self.ox_fatigue = self
            .ox_fatigue
            .saturating_add(if self.occupation_id.as_deref() == Some("farmer") {
                fatigue_gain / 2
            } else {
                fatigue_gain
            })
            .min(100);
        if self.ox_fatigue >= 50 && self.pace != Pace::Steady {
            let fatigue_damage = if self.pace == Pace::Grueling { 2 } else { 1 };
            for member in self.party.iter_mut().filter(|member| member.alive) {
                member.health = member.health.saturating_sub(fatigue_damage);
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
        self.weather_illness();
        self.miles += moved;
        self.route_miles_remaining -= moved;
        out.push(Outcome::DayAdvanced { day: self.day, miles: self.miles, weather: self.weather });
        self.announce_npc_arrivals(&present_before, &mut out);
        self.landmark(&mut out);
        if !matches!(self.status, RunStatus::Arrived | RunStatus::Failed) {
            self.due(&mut out);
            if self.pending_event.is_none() {
                self.event(&mut out)
            }
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
        let (at_dalles, route) = {
            let node = self.node()?;
            let route = node
                .routes
                .iter()
                .find(|route| route.id == id)
                .ok_or_else(|| CommandError::UnknownId(id.into()))?
                .clone();
            (node.id == "the_dalles", route)
        };
        if at_dalles && id == "columbia" {
            self.active_minigame = Some(MinigameSession {
                kind: MinigameKind::Raft,
                seed: self.rng.stream("rafting").gen(),
                ammo_available: 0,
            });
            return Ok(vec![Outcome::Message("The Columbia current takes hold.".into())]);
        }
        if at_dalles && id == "barlow" {
            if self.era_id.as_deref() == Some("1843") {
                return Err(CommandError::InvalidChoice);
            }
            if self.cash_cents < 500 {
                return Err(CommandError::InsufficientCash);
            }
            self.cash_cents -= 500;
        }
        let target_id = route.target_id;
        let distance = route.distance_miles;
        let label = route.label;
        self.target_node_id = Some(target_id);
        self.route_miles_remaining = distance;
        self.status = RunStatus::Travelling;
        Ok(vec![Outcome::Message(format!("Taking {label}"))])
    }
    fn cross(&mut self, m: CrossMethod) -> Result<Vec<Outcome>, CommandError> {
        if !matches!(self.status, RunStatus::AwaitingRiver(_)) {
            return Err(CommandError::InvalidPhase);
        }
        self.node()?.river.as_ref().ok_or(CommandError::InvalidPhase)?;
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
        let risk = self.crossing_risk(m).ok_or(CommandError::InvalidPhase)?;
        if m == CrossMethod::Guide {
            if self.current_node_id.as_deref() != Some("snake_river")
                || self.inventory.get("clothing") < self.guide_cost()
            {
                return Err(CommandError::InvalidChoice);
            }
            self.inventory.remove("clothing", self.guide_cost());
        }
        if m == CrossMethod::Ferry {
            let cost = self.ferry_cost().ok_or(CommandError::InvalidChoice)?;
            if self.cash_cents < cost {
                return Err(CommandError::InsufficientCash);
            }
            self.cash_cents -= cost;
            self.flags.insert("ferry_used".into());
        }
        let mut out = Vec::new();
        if self.rng.stream("rivers").gen_range(0..100) < risk {
            let lost = self.inventory.take("food", 50);
            out.push(Outcome::Message(format!("The river carries away {lost} lbs of food.")));
            if m == CrossMethod::Ford && self.rng.stream("rivers").gen_range(0..100) < 5 {
                let living = self
                    .party
                    .iter()
                    .enumerate()
                    .filter_map(|(index, member)| member.alive.then_some(index))
                    .collect::<Vec<_>>();
                if let Some(&index) = living.choose(self.rng.stream("rivers")) {
                    let member = &mut self.party[index];
                    member.health = 0;
                    member.alive = false;
                    out.push(Outcome::MemberDied { name: member.name.clone() });
                }
            }
        }
        self.pass_camp_day(&mut out, false);
        if self.status != RunStatus::Failed {
            self.begin_only_route()?;
        }
        out.push(Outcome::Message("The crossing is behind you.".into()));
        Ok(out)
    }
    pub fn effective_depth(&self) -> Option<u32> {
        let river = self.node().ok()?.river.as_ref()?;
        let spring_runoff = u32::from(self.season() == Season::Spring);
        Some(
            river
                .depth_feet
                .saturating_add_signed(i32::from(self.weather_state.river_depth_bonus))
                .saturating_add(spring_runoff),
        )
    }
    pub fn crossing_risk(&self, method: CrossMethod) -> Option<u32> {
        let river = self.node().ok()?.river.as_ref()?;
        let depth = self.effective_depth()?;
        let weight_penalty = self.weight().saturating_sub(1_200) / 120;
        let ox_penalty = match self.inventory.get("oxen") {
            0..=1 => 20,
            2 => 10,
            _ => 0,
        };
        let risk = match method {
            CrossMethod::Ferry | CrossMethod::Wait => 0,
            CrossMethod::Guide => 10,
            CrossMethod::Ford => {
                depth.saturating_mul(12).saturating_add(weight_penalty).saturating_add(ox_penalty)
            }
            CrossMethod::Caulk => {
                river.width_feet / 25 + weight_penalty.saturating_mul(2) + ox_penalty
            }
        };
        Some(risk.min(90))
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
    /// Whether the current mandatory wagon event can be addressed with a spare or a repair attempt.
    pub fn can_repair(&self) -> bool {
        if self.at_camp().is_err() || self.pending_wagon_part().is_none() {
            return false;
        }
        let part = self.pending_wagon_part().expect("checked above");
        self.inventory.get(&part) > 0
            || self.occupation_id.as_deref() == Some("blacksmith")
            || self.inventory.get("tools") > 0
    }
    fn pending_wagon_part(&self) -> Option<String> {
        let event_id = self.pending_event.as_deref()?;
        self.content
            .events
            .iter()
            .find(|event| event.id == event_id)?
            .choices
            .iter()
            .flat_map(|choice| &choice.effects)
            .find_map(|effect| match effect {
                Effect::AdjustItem { item_id, quantity }
                    if *quantity < 0 && matches!(item_id.as_str(), "wheel" | "axle" | "tongue") =>
                {
                    Some(item_id.clone())
                }
                _ => None,
            })
    }
    fn repair(&mut self) -> Result<Vec<Outcome>, CommandError> {
        if !self.can_repair() {
            return Err(CommandError::InvalidChoice);
        }
        let part = self.pending_wagon_part().expect("can_repair requires a wagon part");
        let used_spare = self.inventory.get(&part) > 0;
        if used_spare && !self.inventory.remove(&part, 1) {
            return Err(CommandError::InvalidChoice);
        }

        let chance = if used_spare {
            100
        } else if self.occupation_id.as_deref() == Some("blacksmith") {
            80
        } else {
            let best_repair = self
                .party
                .iter()
                .filter(|member| member.alive)
                .map(|member| member.skills.repair)
                .max()
                .unwrap_or(0);
            (25 + u32::from(best_repair) * 8
                + u32::from(self.occupation_id.as_deref() == Some("carpenter")) * 20)
                .min(95)
        };
        let succeeded = self.rng.stream("repairs").gen_range(0..100) < chance;
        let mut out = Vec::new();
        self.pass_camp_day(&mut out, false);
        if matches!(self.status, RunStatus::Arrived | RunStatus::Failed) {
            self.pending_event = None;
            return Ok(out);
        }
        if succeeded {
            if let Some(member) = self
                .party
                .iter_mut()
                .filter(|member| member.alive)
                .max_by_key(|member| member.skills.repair)
            {
                member.skills.repair = member.skills.repair.saturating_add(1);
            }
            self.pending_event = None;
            let source = if used_spare { "A spare" } else { "The repair" };
            out.push(Outcome::Message(format!("{source} sets the {part} right.")));
        } else {
            out.push(Outcome::Message(format!(
                "The {part} repair fails. The wagon still cannot move."
            )));
        }
        Ok(out)
    }
    fn talk(&mut self) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let candidates: Vec<_> = self
            .content
            .quotes
            .iter()
            .filter(|q| {
                (q.landmark_id.is_none()
                    || q.landmark_id.as_deref() == self.current_node_id.as_deref())
                    && (q.seasons.is_empty() || q.seasons.contains(&self.season()))
                    && q.state_tags.iter().all(|tag| self.quote_tag_matches(tag))
                    && (q.landmark_id.is_some()
                        || !q.seasons.is_empty()
                        || !q.state_tags.is_empty())
            })
            .collect();
        let fallback: Vec<_> = self
            .content
            .quotes
            .iter()
            .filter(|q| q.landmark_id.is_none() && q.seasons.is_empty() && q.state_tags.is_empty())
            .collect();
        let q = candidates
            .choose(self.rng.stream("quotes"))
            .or_else(|| fallback.choose(self.rng.stream("quotes")));
        let Some(q) = q else {
            return Ok(vec![Outcome::Message("The camp is quiet tonight.".into())]);
        };
        Ok(vec![Outcome::Quote { quote_id: q.id.clone(), text: q.text.clone() }])
    }
    fn quote_tag_matches(&self, tag: &str) -> bool {
        match tag {
            "hungry" => {
                self.inventory.get("food")
                    < 3 * match self.rations {
                        RationLevel::Filling => 3,
                        RationLevel::Meager => 2,
                        RationLevel::BareBones => 1,
                    } * self.party.iter().filter(|member| member.alive).count() as u32
            }
            "sick" => self.party.iter().any(|member| member.alive && !member.ailments.is_empty()),
            "wealthy" => self.cash_cents >= 80_000,
            "late" => self.date().1 >= 9,
            "cold" => matches!(self.weather, WeatherKind::Cold | WeatherKind::Snow),
            "weary" => {
                self.ox_fatigue >= 50
                    || self.party.iter().any(|member| member.alive && member.health < 60)
            }
            "homesick" => self.party.iter().any(|member| member.alive && member.morale < 40),
            "grieving" => self.party.iter().any(|member| !member.alive),
            "hopeful" => self.party.iter().any(|member| member.alive && member.morale >= 60),
            "thirsty" => self.weather == WeatherKind::Hot,
            _ => false,
        }
    }
    fn treat(&mut self, i: usize, a: &str) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let doctor_present =
            self.party.iter().any(|member| member.alive && member.skills.medicine >= 4);
        let p = self.party.get_mut(i).ok_or(CommandError::InvalidChoice)?;
        if !p.alive
            || !p.ailments.iter().any(|x| x == a)
            || (!doctor_present && !self.inventory.remove("medicine", 1))
        {
            return Err(CommandError::InvalidChoice);
        }
        p.ailments.retain(|x| x != a);
        p.ailment_days.remove(a);
        p.skills.medicine = p.skills.medicine.saturating_add(1);
        p.health = p.health.saturating_add(15).min(100);
        Ok(vec![Outcome::Treated { member_index: i, ailment_id: a.into() }])
    }
    fn begin_hunt(&mut self) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        if self.pending_event.is_some() || self.active_minigame.is_some() {
            return Err(CommandError::InvalidPhase);
        }
        let ammunition = self
            .inventory
            .get("ammunition")
            .checked_mul(20)
            .and_then(|shots| shots.checked_add(u32::from(self.loose_bullets)))
            .ok_or(CommandError::CapacityExceeded)?;
        if ammunition == 0 {
            return Err(CommandError::InvalidChoice);
        }
        self.active_minigame = Some(MinigameSession {
            kind: MinigameKind::Hunt,
            seed: self.rng.stream("hunting").gen(),
            ammo_available: ammunition,
        });
        Ok(vec![Outcome::Message("Hunt begun.".into())])
    }
    fn hunt(&mut self, food: u32, shots: u32) -> Result<Vec<Outcome>, CommandError> {
        let session = self.active_minigame.clone().ok_or(CommandError::InvalidPhase)?;
        if session.kind != MinigameKind::Hunt
            || shots > session.ammo_available.min(20)
            || (food > 0 && shots == 0)
            || food > shots.saturating_mul(100)
        {
            return Err(CommandError::InvalidChoice);
        }
        let ammunition = self
            .inventory
            .get("ammunition")
            .checked_mul(20)
            .and_then(|available| available.checked_add(u32::from(self.loose_bullets)))
            .ok_or(CommandError::CapacityExceeded)?;
        if ammunition < session.ammo_available || shots > ammunition {
            return Err(CommandError::InvalidChoice);
        }
        let bag_limit = if self.occupation_id.as_deref() == Some("hunter") { 200 } else { 150 };
        if food > bag_limit {
            return Err(CommandError::InvalidChoice);
        }
        let added_food = food.min(self.max_addable("food").unwrap_or(0));
        let ammunition_after = ammunition - shots;
        let boxes_after = ammunition_after / 20;
        let loose_after = u8::try_from(ammunition_after % 20).expect("remainder is below 20");
        self.inventory.quantities.insert("ammunition".into(), boxes_after);
        self.loose_bullets = loose_after;
        self.inventory.add("food", added_food);
        if added_food > 0 {
            if let Some((index, _)) = self
                .party
                .iter()
                .enumerate()
                .filter(|(_, member)| member.alive)
                .max_by_key(|(_, member)| member.skills.hunting)
            {
                self.party[index].skills.hunting =
                    self.party[index].skills.hunting.saturating_add(1);
            }
        }
        if shots > 0
            && self.rng.stream("health").gen_range(0..100) < 1
            && self.content.ailments.iter().any(|ailment| ailment.id == "gunshot")
        {
            let candidates = self
                .party
                .iter()
                .enumerate()
                .filter_map(|(index, member)| {
                    (member.alive && !member.ailments.iter().any(|ailment| ailment == "gunshot"))
                        .then_some(index)
                })
                .collect::<Vec<_>>();
            if let Some(&index) = candidates.choose(self.rng.stream("health")) {
                self.party[index].ailments.push("gunshot".into());
                self.party[index].ailment_days.insert("gunshot".into(), 0);
            }
        }
        self.active_minigame = None;
        let mut outcomes = Vec::new();
        self.pass_camp_day(&mut outcomes, false);
        if added_food > 0 {
            self.last_fresh_food_day = Some(self.day);
        }
        outcomes.push(Outcome::Message(format!("Brought back {added_food} lbs of food")));
        Ok(outcomes)
    }
    fn raft(
        &mut self,
        lost: u32,
        casualties: u8,
        completed: bool,
    ) -> Result<Vec<Outcome>, CommandError> {
        let session = self.active_minigame.clone().ok_or(CommandError::InvalidPhase)?;
        if session.kind != MinigameKind::Raft
            || self.current_node_id.as_deref() != Some("the_dalles")
            || !matches!(self.status, RunStatus::AwaitingFork(_))
            || lost > 120
            || casualties > 4
        {
            return Err(CommandError::InvalidChoice);
        }
        let route = self
            .node()?
            .routes
            .iter()
            .find(|route| route.id == "columbia")
            .ok_or(CommandError::InvalidChoice)?
            .clone();
        self.inventory.take("food", lost);
        let mut outcomes = Vec::new();
        let casualty_count = usize::from(casualties);
        let living = self
            .party
            .iter()
            .enumerate()
            .filter_map(|(index, member)| member.alive.then_some(index))
            .collect::<Vec<_>>();
        for index in
            living.choose_multiple(self.rng.stream("rafting"), casualty_count.min(living.len()))
        {
            let member = &mut self.party[*index];
            member.health = 0;
            member.alive = false;
            outcomes.push(Outcome::MemberDied { name: member.name.clone() });
        }
        self.active_minigame = None;
        self.pass_camp_day(&mut outcomes, false);
        if completed && self.party.iter().any(|member| member.alive) {
            self.miles = self.miles.saturating_add(route.distance_miles);
            self.current_node_id = Some(route.target_id.clone());
            self.target_node_id = None;
            self.route_miles_remaining = 0;
            self.status = RunStatus::Arrived;
            outcomes.push(Outcome::ArrivedAt { landmark_id: route.target_id });
            outcomes.push(Outcome::Score { points: self.score() });
        } else if self.party.iter().any(|member| member.alive) {
            self.current_node_id = Some("the_dalles".into());
            self.target_node_id = None;
            self.route_miles_remaining = 0;
            self.status = RunStatus::AwaitingFork("the_dalles".into());
        }
        outcomes.push(Outcome::Message("Rafting result recorded".into()));
        Ok(outcomes)
    }
    fn forage(&mut self) -> Result<Vec<Outcome>, CommandError> {
        self.at_camp()?;
        let bonus = self
            .party
            .iter()
            .filter(|member| {
                member.alive
                    && (member.traits.contains(&crate::party::Trait::Herbalist)
                        || member.traits.contains(&crate::party::Trait::Sharpshooter))
            })
            .count() as u32
            * 10;
        let occupation_bonus = u32::from(self.occupation_id.as_deref() == Some("farmer")) * 10;
        let food = (self.rng.stream("forage").gen_range(5..=25) + bonus + occupation_bonus)
            .min(self.max_addable("food").unwrap_or(0));
        self.inventory.add("food", food);
        let mut outcomes = Vec::new();
        self.pass_camp_day(&mut outcomes, false);
        if food > 0 {
            self.last_fresh_food_day = Some(self.day);
        }
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
        if food > 0 {
            self.last_fresh_food_day = Some(self.day);
        }
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
        let price = self.shop_unit_price(&market, item) / 2;
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
        if !self.npc_present(npc_id) {
            return Err(CommandError::InvalidChoice);
        }
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
        if !self.npc_present(npc_id) {
            return Err(CommandError::InvalidChoice);
        }
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
        member.traits.push(crate::party::Trait::Sociable);
        member.skills.animals = 2;
        for companion in self.party.iter_mut().filter(|companion| companion.alive) {
            member.relationships.affinity.insert(companion.name.clone(), 5);
            companion.relationships.affinity.insert(name.clone(), 5);
        }
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
        if !self.npc_present(npc_id) {
            return Err(CommandError::InvalidChoice);
        }
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
        let mut offered = price(offered_item)?
            .checked_mul(u64::from(offered_quantity))
            .ok_or(CommandError::CapacityExceeded)?;
        if self.occupation_id.as_deref() == Some("merchant") {
            offered = offered.checked_mul(6).ok_or(CommandError::CapacityExceeded)? / 5;
        }
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
        score::breakdown(self).total
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
        let mut open_party_slots = self.available_party_slots();
        let mut added_names = BTreeSet::new();
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
                Effect::AddMember { name, .. }
                    if open_party_slots == 0
                        || self.party.iter().any(|member| member.name == *name)
                        || !added_names.insert(name) =>
                {
                    return false;
                }
                Effect::AddMember { .. } => open_party_slots -= 1,
                _ => {}
            }
        }
        true
    }
    pub fn wagon_weight(&self) -> u32 {
        self.weight()
    }
    /// Open seats after reserving room for every pending birth.
    pub fn available_party_slots(&self) -> usize {
        12usize.saturating_sub(self.party.len().saturating_add(self.family.pregnancies.len()))
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
        let era_rules = self.era_rules();
        matches!(self.status, RunStatus::Outfitting | RunStatus::AtLandmark(_))
            && self
                .current_landmark()
                .is_some_and(|node| node.store && !era_rules.unavailable_stores.contains(&node.id))
    }
    /// Current ferry fee after the selected era's game-balance modifier.
    pub fn ferry_cost(&self) -> Option<i64> {
        let rules = self.era_rules();
        let base = self.node().ok()?.river.as_ref()?.ferry_cost_cents?;
        rules.ferries_available.then(|| Self::apply_percent(base, rules.ferry_fee_percent))
    }
    /// Clothing sets required for the Snake River guide in the selected era.
    pub fn guide_cost(&self) -> u32 {
        self.era_rules().guide_cost_clothing
    }
    pub fn price_cents(&self, item_id: &str) -> Option<i64> {
        self.content.items.iter().find(|item| item.id == item_id).map(|item| {
            let mut market = self.market_at(self.current_node_id.as_deref().unwrap_or_default());
            market.replenish(self.day);
            self.shop_unit_price(&market, item)
        })
    }
    pub fn sell_price_cents(&self, item_id: &str) -> Option<i64> {
        self.price_cents(item_id).map(|price| price / 2)
    }
    fn shop_unit_price(&self, market: &Market, item: &ItemDefinition) -> i64 {
        let mut price = market.price_for(&item.id, item.price_cents, self.season_markup());
        price = Self::apply_percent(price, self.era_rules().price_percent);
        let reputation_percent = self.reputation.clamp(-10, 10);
        price = Self::apply_percent(price, (100 - reputation_percent) as u16);
        let discount = match self.occupation_id.as_deref() {
            Some("banker")
                if matches!(
                    self.current_landmark().map(|node| node.kind),
                    Some(LandmarkKind::Fort)
                ) =>
            {
                10
            }
            Some("carpenter") if matches!(item.id.as_str(), "wheel" | "axle" | "tongue") => 20,
            Some("soldier") if item.id == "ammunition" => 20,
            _ => 0,
        };
        if discount > 0 {
            price = (i128::from(price) * i128::from(100 - discount) / 100)
                .clamp(1, i128::from(i64::MAX)) as i64;
        }
        price
    }
    fn apply_percent(amount: i64, percent: u16) -> i64 {
        (i128::from(amount) * i128::from(percent) / 100).clamp(1, i128::from(i64::MAX)) as i64
    }
    fn era_rules(&self) -> EraRules {
        self.era_id
            .as_ref()
            .and_then(|id| self.content.era_rules.get(id))
            .cloned()
            .unwrap_or_default()
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
            normal_stock: self
                .content
                .items
                .iter()
                .map(|item| (item.id.clone(), item.limit.max(20)))
                .collect(),
            stock: self
                .content
                .items
                .iter()
                .map(|item| (item.id.clone(), item.limit.max(20)))
                .collect(),
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
    pub fn has_fresh_food(&self) -> bool {
        self.last_fresh_food_day.is_some_and(|day| self.day.saturating_sub(day) <= 3)
    }
    /// Reject corrupted saves before a UI attempts to navigate their content IDs.
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.party.len() > 12
            || self.party.len().saturating_add(self.family.pregnancies.len()) > 12
            || !(3..=7).contains(&self.departure_month)
            || self.day > 3660
            || self.cash_cents < 0
            || self.loose_bullets >= 20
            || self.last_fresh_food_day.is_some_and(|day| day > self.day)
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
        let mut mothers = BTreeSet::new();
        for pregnancy in &self.family.pregnancies {
            if pregnancy.due_day < self.day
                || pregnancy.due_day > self.day.saturating_add(130)
                || !mothers.insert(&pregnancy.mother)
            {
                return Err(CommandError::InvalidSetup);
            }
            let Some(mother) = self.party.iter().find(|member| member.name == pregnancy.mother)
            else {
                return Err(CommandError::InvalidSetup);
            };
            if mother.sex != Sex::Female || !(18..=50).contains(&mother.age) {
                return Err(CommandError::InvalidSetup);
            }
        }
        let mut marriages = BTreeSet::new();
        for (left, right) in &self.family.marriages {
            if left.is_empty()
                || right.is_empty()
                || left == right
                || left.len() > 24
                || right.len() > 24
                || left.chars().any(char::is_control)
                || right.chars().any(char::is_control)
            {
                return Err(CommandError::InvalidSetup);
            }
            let pair = if left < right { (left, right) } else { (right, left) };
            if !marriages.insert(pair) {
                return Err(CommandError::InvalidSetup);
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
        if let Some(session) = &self.active_minigame {
            if self.pending_event.is_some() {
                return Err(CommandError::InvalidSetup);
            }
            match session.kind {
                MinigameKind::Hunt => {
                    let available = self
                        .inventory
                        .get("ammunition")
                        .checked_mul(20)
                        .and_then(|shots| shots.checked_add(u32::from(self.loose_bullets)))
                        .ok_or(CommandError::InvalidSetup)?;
                    if session.ammo_available == 0
                        || session.ammo_available != available
                        || self.at_camp().is_err()
                    {
                        return Err(CommandError::InvalidSetup);
                    }
                }
                MinigameKind::Raft => {
                    if session.ammo_available != 0
                        || self.current_node_id.as_deref() != Some("the_dalles")
                        || !matches!(self.status, RunStatus::AwaitingFork(_))
                    {
                        return Err(CommandError::InvalidSetup);
                    }
                }
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
            .saturating_add(u32::from(self.loose_bullets > 0))
    }
    fn node(&self) -> Result<&LandmarkDefinition, CommandError> {
        self.content
            .trails
            .iter()
            .find(|t| Some(&t.id) == self.trail_id.as_ref())
            .and_then(|t| t.nodes.iter().find(|n| Some(&n.id) == self.current_node_id.as_ref()))
            .ok_or(CommandError::InvalidPhase)
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
            .filter_map(|e| {
                let weight = self.event_weight(e);
                (weight > 0).then(|| (e.id.clone(), weight))
            })
            .collect();
        let event_odds = match self.difficulty {
            Difficulty::Easy => 10,
            Difficulty::Normal => 15,
            Difficulty::Hard => 22,
        };
        if !ids.is_empty() && self.rng.stream("events").gen_range(0..100) < event_odds {
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
    fn event_weight(&self, event: &EventDefinition) -> u32 {
        let percent = self.era_rules().event_weight_percent.get(&event.id).copied().unwrap_or(100);
        ((u64::from(event.weight) * u64::from(percent) / 100).max(1)).min(u64::from(u32::MAX))
            as u32
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
            Condition::RelationshipAtLeast(affinity) => {
                self.live_adult_pair(true).is_some_and(|(_, _, current)| current >= *affinity)
            }
            Condition::PartySizeBelow(size) => self.party.len() < usize::from(*size),
            Condition::HasAdultPair => self.live_adult_pair(true).is_some(),
        }
    }
    fn effects(&mut self, es: &[Effect], out: &mut Vec<Outcome>) {
        for e in es {
            match e {
                Effect::Message(x) => out.push(Outcome::Message(x.clone())),
                Effect::AdjustFood(x) => {
                    if *x >= 0 {
                        self.inventory
                            .add("food", (*x as u32).min(self.max_addable("food").unwrap_or(0)))
                    } else {
                        self.inventory.take("food", x.unsigned_abs());
                    }
                }
                Effect::AdjustItem { item_id, quantity } => {
                    if *quantity >= 0 {
                        self.inventory.add(
                            item_id,
                            (*quantity as u32).min(self.max_addable(item_id).unwrap_or(0)),
                        );
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
                        p.ailments.retain(|a| a != x);
                        p.ailment_days.remove(x);
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
                Effect::AdjustRelationship(change) => {
                    if let Some((left, right, _)) = self.adjust_live_relationship(*change) {
                        out.push(Outcome::Message(format!(
                            "{} and {} grow {}.",
                            left,
                            right,
                            if *change > 0 { "closer" } else { "more distant" }
                        )));
                    }
                }
                Effect::CelebrateWedding => self.celebrate_wedding(out),
                Effect::MemberLeaves => self.member_leaves(out),
                Effect::AddMember { name, age } => {
                    if self.available_party_slots() > 0
                        && !self.party.iter().any(|member| member.name == *name)
                    {
                        let mut member = PartyMember::new(name.clone());
                        member.age = *age;
                        crate::party::initialize_joiner(
                            &mut member,
                            &mut self.party,
                            &mut self.rng,
                        );
                        self.party.push(member);
                        out.push(Outcome::Message(format!("{} joins the party.", name)));
                    }
                }
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
        let (_, month, day) = self.date();
        if n.id == "independence_rock" && (month < 7 || (month == 7 && day <= 4)) {
            self.flags.insert("independence_rock_early".into());
            for member in self.party.iter_mut().filter(|member| member.alive) {
                member.morale = member.morale.saturating_add(10).min(100);
            }
        }
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
                let difficulty_multiplier = match self.difficulty {
                    Difficulty::Easy => 75,
                    Difficulty::Normal => 100,
                    Difficulty::Hard => 125,
                };
                let mut mortality = (u32::from(definition.mortality_per_mille)
                    .saturating_mul(difficulty_multiplier)
                    / 100)
                    .min(1_000) as u16;
                if self.party.iter().any(|member| member.alive && member.skills.medicine >= 4) {
                    mortality /= 2;
                }
                if mortality < 1_000 {
                    mortality = mortality
                        .saturating_sub(trait_modifier.max(0) as u16)
                        .saturating_add((-trait_modifier.min(0)) as u16);
                }
                if matches!(
                    crate::health::stage(days, definition.severity),
                    crate::health::AilmentStage::Acute
                ) && self.rng.stream("health").gen_range(0..1000) < mortality
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
        let present_before = self.present_npc_ids();
        self.day = self.day.saturating_add(1);
        let season = self.season();
        let climate = self.region().climate;
        self.weather_state.advance_in(&mut self.rng, season, climate);
        self.weather = self.weather_state.kind;
        self.spoil_food();
        if resting {
            self.ox_fatigue = self.ox_fatigue.saturating_sub(25);
        }
        let ration = match self.rations {
            RationLevel::Filling => 3,
            RationLevel::Meager => 2,
            RationLevel::BareBones => 1,
        };
        let required = self.party.iter().filter(|member| member.alive).count() as u32 * ration;
        let eaten = self.inventory.take("food", required);
        let damages: Vec<u8> =
            self.party.iter().map(|member| self.ailment_damage(member)).collect();
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
        self.weather_illness();
        self.party_daily(out, resting, eaten == required);
        self.family_daily(out);
        if !self.party.iter().any(|member| member.alive) {
            self.status = RunStatus::Failed;
        }
        self.announce_npc_arrivals(&present_before, out);
    }
    fn spoil_food(&mut self) {
        let rate_per_mille = match self.weather {
            WeatherKind::Hot => 3,
            WeatherKind::Rain => 1,
            _ => 0,
        };
        if rate_per_mille > 0 {
            let food = self.inventory.get("food");
            self.inventory.take("food", food.saturating_mul(rate_per_mille) / 1_000);
        }
    }
    fn party_daily(&mut self, out: &mut Vec<Outcome>, resting: bool, filling: bool) {
        let varied_food = self.has_fresh_food();
        out.extend(
            crate::party::daily(
                &mut self.party,
                resting,
                filling && self.rations == RationLevel::Filling,
                varied_food,
                &mut self.rng,
            )
            .into_iter()
            .map(Outcome::Message),
        );
    }
    /// Resolve pregnancies after each completed game day. A due pregnancy is consumed even when
    /// the mother has died or the wagon is full, so old saves cannot repeatedly retry a birth.
    fn family_daily(&mut self, out: &mut Vec<Outcome>) {
        if matches!(self.status, RunStatus::Arrived | RunStatus::Failed) {
            return;
        }
        let pregnancies = std::mem::take(&mut self.family.pregnancies);
        for pregnancy in pregnancies {
            let Some(mother_index) =
                self.party.iter().position(|member| member.name == pregnancy.mother)
            else {
                continue;
            };
            let mother = &self.party[mother_index];
            if !mother.alive || mother.sex != Sex::Female || !(18..=50).contains(&mother.age) {
                continue;
            }
            if pregnancy.due_day > self.day {
                self.family.pregnancies.push(pregnancy);
                continue;
            }
            if self.party.len() >= 12 {
                if mother_index != 0 {
                    let mother_name = mother.name.clone();
                    self.party.remove(mother_index);
                    out.push(Outcome::Message(format!(
                        "{} leaves the party with her newborn child to find care.",
                        mother_name
                    )));
                } else {
                    self.family.pregnancies.push(pregnancy);
                    out.push(Outcome::Message(format!(
                        "The full wagon leaves no safe place for {} to give birth.",
                        mother.name
                    )));
                }
                continue;
            }
            let mother_name = mother.name.clone();
            let baby_name = self.next_baby_name();
            let mut baby = PartyMember::new(baby_name.clone());
            baby.age = 0;
            baby.health = 80;
            crate::party::initialize_joiner(&mut baby, &mut self.party, &mut self.rng);
            self.party.push(baby);
            out.push(Outcome::Message(format!("{} gives birth to {}.", mother_name, baby_name)));

            let complication_chance =
                if self.party.iter().any(|member| member.alive && member.skills.medicine >= 4) {
                    5
                } else {
                    10
                };
            if self.rng.stream("family").gen_range(0..100) < complication_chance
                && self
                    .content
                    .ailments
                    .iter()
                    .any(|ailment| ailment.id == "childbirth_complications")
            {
                let mother = &mut self.party[mother_index];
                if !mother.ailments.iter().any(|ailment| ailment == "childbirth_complications") {
                    mother.ailments.push("childbirth_complications".into());
                    mother.ailment_days.insert("childbirth_complications".into(), 0);
                    out.push(Outcome::Message(format!(
                        "{} suffers childbirth complications.",
                        mother.name
                    )));
                }
            }
        }
    }
    fn next_baby_name(&self) -> String {
        let mut ordinal = 1;
        loop {
            let candidate = format!("Baby {ordinal}");
            if !self.party.iter().any(|member| member.name == candidate) {
                return candidate;
            }
            ordinal += 1;
        }
    }
    fn live_adult_pair(&self, prefer_highest: bool) -> Option<(usize, usize, i16)> {
        let mut selected = None;
        for left in 0..self.party.len() {
            for right in (left + 1)..self.party.len() {
                let a = &self.party[left];
                let b = &self.party[right];
                if !a.alive || !b.alive || a.age < 18 || b.age < 18 {
                    continue;
                }
                let affinity = a.relationships.affinity.get(&b.name).copied().unwrap_or(0);
                if selected.is_none_or(|(_, _, current)| {
                    if prefer_highest {
                        affinity > current
                    } else {
                        affinity < current
                    }
                }) {
                    selected = Some((left, right, affinity));
                }
            }
        }
        selected
    }
    fn adjust_live_relationship(&mut self, change: i16) -> Option<(String, String, i16)> {
        let (left, right, affinity) = self.live_adult_pair(change >= 0)?;
        let left_name = self.party[left].name.clone();
        let right_name = self.party[right].name.clone();
        let updated = (affinity + change).clamp(-100, 100);
        self.party[left].relationships.affinity.insert(right_name.clone(), updated);
        self.party[right].relationships.affinity.insert(left_name.clone(), updated);
        Some((left_name, right_name, updated))
    }
    fn celebrate_wedding(&mut self, out: &mut Vec<Outcome>) {
        let Some((left, right, affinity)) = self.live_adult_pair(true) else { return };
        if affinity < 20 {
            return;
        }
        let pair = (self.party[left].name.clone(), self.party[right].name.clone());
        if self
            .family
            .marriages
            .iter()
            .any(|existing| existing == &pair || existing == &(pair.1.clone(), pair.0.clone()))
        {
            return;
        }
        self.family.marriages.push(pair.clone());
        out.push(Outcome::Message(format!("{} and {} celebrate their union.", pair.0, pair.1)));
    }
    fn member_leaves(&mut self, out: &mut Vec<Outcome>) {
        let Some(index) = self
            .party
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, member)| member.alive && member.age >= 18)
            .min_by_key(|(_, member)| member.morale)
            .map(|(index, _)| index)
        else {
            return;
        };
        let member = self.party.remove(index);
        self.family.pregnancies.retain(|pregnancy| pregnancy.mother != member.name);
        out.push(Outcome::Message(format!("{} leaves the party.", member.name)));
    }
    fn ailment_damage(&self, member: &PartyMember) -> u8 {
        let damage = member
            .ailments
            .iter()
            .filter_map(|id| {
                self.content.ailments.iter().find(|ailment| &ailment.id == id).map(|ailment| {
                    let days = member.ailment_days.get(id).copied().unwrap_or(0).saturating_add(1);
                    match crate::health::stage(days, ailment.severity) {
                        crate::health::AilmentStage::Acute => ailment.daily_damage,
                        crate::health::AilmentStage::Symptoms
                        | crate::health::AilmentStage::Recovering => {
                            (ailment.daily_damage.saturating_add(1)) / 2
                        }
                    }
                })
            })
            .fold(0u8, u8::saturating_add);
        if damage == 0 {
            return 0;
        }
        match self.difficulty {
            Difficulty::Easy => damage.saturating_sub(1),
            Difficulty::Normal => damage,
            Difficulty::Hard => damage.saturating_add(1),
        }
    }
    fn weather_illness(&mut self) {
        let ailment = match self.weather {
            WeatherKind::Cold | WeatherKind::Snow => Some("pneumonia"),
            WeatherKind::Hot => Some("heat_stroke"),
            _ => None,
        };
        let Some(ailment) = ailment else { return };
        let protected = self.inventory.get("clothing")
            >= self.party.iter().filter(|member| member.alive).count() as u32;
        let chance = if protected { 1 } else { 5 };
        if self.rng.stream("health").gen_range(0..100) < chance {
            let candidates = self
                .party
                .iter()
                .enumerate()
                .filter_map(|(index, member)| {
                    (member.alive && !member.ailments.iter().any(|id| id == ailment))
                        .then_some(index)
                })
                .collect::<Vec<_>>();
            if let Some(&index) = candidates.choose(self.rng.stream("health")) {
                let member = &mut self.party[index];
                if self.content.ailments.iter().any(|definition| definition.id == ailment) {
                    member.ailments.push(ailment.into());
                    member.ailment_days.insert(ailment.into(), 0);
                }
            }
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
    pub fn terrain(&self) -> Terrain {
        self.region().terrain
    }
    fn region(&self) -> Region {
        if self.content.regions.is_empty() {
            return Region::plains();
        }
        let node_id = if self.status == RunStatus::Travelling {
            self.target_node_id.as_deref().or(self.current_node_id.as_deref())
        } else {
            self.current_node_id.as_deref()
        }
        .unwrap_or_default();
        self.content.regions.get(node_id).copied().unwrap_or_else(Region::plains)
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
        content.regions = BTreeMap::from([
            ("independence".into(), Region::plains()),
            ("river".into(), Region::river_valley()),
            ("fork".into(), Region::plains()),
            ("detour".into(), Region::hills()),
            ("willamette".into(), Region::forest()),
        ]);
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
    fn repair_game(seed: u64) -> GameState {
        let mut game = run(seed);
        game.content.items.extend([
            ItemDefinition {
                id: "wheel".into(),
                name: "Wheel".into(),
                unit: "each".into(),
                price_cents: 1_000,
                weight_lbs: 30,
                limit: 3,
            },
            ItemDefinition {
                id: "tools".into(),
                name: "Tools".into(),
                unit: "set".into(),
                price_cents: 2_500,
                weight_lbs: 15,
                limit: 1,
            },
        ]);
        game.content.events.push(EventDefinition {
            id: "broken_wheel".into(),
            text: "A wheel splits on the rocks.".into(),
            weight: 0,
            conditions: vec![Condition::Always],
            effects: vec![],
            choices: vec![EventChoice {
                id: "spare".into(),
                label: "Fit a spare wheel".into(),
                conditions: vec![Condition::InventoryAtLeast {
                    item_id: "wheel".into(),
                    quantity: 1,
                }],
                effects: vec![Effect::AdjustItem { item_id: "wheel".into(), quantity: -1 }],
            }],
        });
        game.pending_event = Some("broken_wheel".into());
        game
    }
    fn deep_ford_game(seed: u64) -> GameState {
        let mut game = run(seed);
        game.content = branch_content();
        game.current_node_id = Some("river".into());
        game.target_node_id = None;
        game.route_miles_remaining = 0;
        game.status = RunStatus::AwaitingRiver("river".into());
        game.content.trails[0].nodes[1].river.as_mut().unwrap().depth_feet = 8;
        game
    }
    fn minigame_content() -> GameContent {
        let mut content = GameContent::starter();
        content.items.push(ItemDefinition {
            id: "ammunition".into(),
            name: "Ammunition".into(),
            unit: "box".into(),
            price_cents: 200,
            weight_lbs: 1,
            limit: 99,
        });
        content.occupations.push(OccupationDefinition {
            id: "hunter".into(),
            name: "Hunter".into(),
            starting_cash_cents: 40_000,
            score_multiplier: 2.5,
            perk: "Carries more meat".into(),
        });
        content.trails[0].goal_node_id = "willamette".into();
        content.trails[0].nodes = vec![
            LandmarkDefinition {
                id: "independence".into(),
                name: "Start".into(),
                mile: 0,
                kind: LandmarkKind::Town,
                routes: vec![RouteDefinition {
                    id: "main".into(),
                    label: "Dalles".into(),
                    target_id: "the_dalles".into(),
                    distance_miles: 10,
                }],
                river: None,
                store: true,
            },
            LandmarkDefinition {
                id: "the_dalles".into(),
                name: "The Dalles".into(),
                mile: 1813,
                kind: LandmarkKind::Finale,
                routes: vec![
                    RouteDefinition {
                        id: "barlow".into(),
                        label: "Barlow".into(),
                        target_id: "willamette".into(),
                        distance_miles: 72,
                    },
                    RouteDefinition {
                        id: "columbia".into(),
                        label: "Columbia".into(),
                        target_id: "willamette".into(),
                        distance_miles: 72,
                    },
                ],
                river: None,
                store: false,
            },
            LandmarkDefinition {
                id: "willamette".into(),
                name: "Goal".into(),
                mile: 1885,
                kind: LandmarkKind::Finale,
                routes: vec![],
                river: None,
                store: false,
            },
        ];
        content.regions = BTreeMap::from([
            ("independence".into(), Region::plains()),
            ("the_dalles".into(), Region::river_valley()),
            ("willamette".into(), Region::forest()),
            (
                "sierra".into(),
                Region {
                    terrain: Terrain::Mountains,
                    climate: crate::weather::ClimateZone::Mountain,
                },
            ),
            (
                "salt_desert".into(),
                Region { terrain: Terrain::Desert, climate: crate::weather::ClimateZone::Arid },
            ),
        ]);
        content
    }
    fn minigame_game(seed: u64, occupation: &str) -> GameState {
        let mut game = GameState::with_content(seed, minigame_content());
        game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: occupation.into(),
            party: vec!["Ada".into(), "Ben".into(), "Clara".into(), "David".into(), "Eve".into()],
            departure_month: 4,
        });
        game.apply(Command::Buy { item_id: "oxen".into(), quantity: 1 });
        game.apply(Command::Buy { item_id: "food".into(), quantity: 100 });
        game.apply(Command::Buy { item_id: "ammunition".into(), quantity: 2 });
        game.apply(Command::Depart);
        game
    }
    fn modifier_game(seed: u64, occupation: &str, era: &str) -> GameState {
        let mut content = GameContent::starter();
        content.trails[0].nodes[0].kind = LandmarkKind::Fort;
        content.eras.push(EraDefinition { id: "1843".into(), name: "1843".into(), year: 1843 });
        content.eras.push(EraDefinition { id: "1866".into(), name: "1866".into(), year: 1866 });
        for id in ["banker", "merchant", "doctor", "carpenter", "preacher", "soldier"] {
            content.occupations.push(OccupationDefinition {
                id: id.into(),
                name: id.into(),
                starting_cash_cents: 100_000,
                score_multiplier: 1.0,
                perk: String::new(),
            });
        }
        content.items.extend([
            ItemDefinition {
                id: "ammunition".into(),
                name: "Ammo".into(),
                unit: "box".into(),
                price_cents: 200,
                weight_lbs: 1,
                limit: 99,
            },
            ItemDefinition {
                id: "wheel".into(),
                name: "Wheel".into(),
                unit: "each".into(),
                price_cents: 1_000,
                weight_lbs: 30,
                limit: 3,
            },
            ItemDefinition {
                id: "trade_goods".into(),
                name: "Goods".into(),
                unit: "lot".into(),
                price_cents: 2_000,
                weight_lbs: 10,
                limit: 10,
            },
            ItemDefinition {
                id: "medicine".into(),
                name: "Medicine".into(),
                unit: "kit".into(),
                price_cents: 1_500,
                weight_lbs: 2,
                limit: 5,
            },
        ]);
        content.ailments.push(AilmentDefinition {
            id: "fever".into(),
            name: "Fever".into(),
            severity: 3,
            daily_damage: 4,
            mortality_per_mille: 100,
        });
        let mut game = GameState::with_content(seed, content);
        game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: era.into(),
            occupation_id: occupation.into(),
            party: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
            departure_month: 4,
        });
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
    fn seeded_family_pregnancy_is_eligible_and_reproducible() {
        let seed = (0..1_000)
            .find(|seed| {
                let mut game = GameState::new(*seed);
                game.apply(Command::Configure {
                    trail_id: "oregon".into(),
                    era_id: "1848".into(),
                    occupation_id: "farmer".into(),
                    party: vec![
                        "Ada".into(),
                        "Ben".into(),
                        "Clara".into(),
                        "David".into(),
                        "Eve".into(),
                    ],
                    departure_month: 4,
                });
                !game.family.pregnancies.is_empty()
            })
            .expect("a seeded family pregnancy should be reachable");
        let mut left = GameState::new(seed);
        let mut right = GameState::new(seed);
        let command = Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["Ada".into(), "Ben".into(), "Clara".into(), "David".into(), "Eve".into()],
            departure_month: 4,
        };
        left.apply(command.clone());
        right.apply(command);
        assert_eq!(left.family, right.family);
        let pregnancy = &left.family.pregnancies[0];
        let mother = left.party.iter().find(|member| member.name == pregnancy.mother).unwrap();
        assert_eq!(mother.sex, Sex::Female);
        assert!((18..=40).contains(&mother.age));
        assert!((60..=130).contains(&pregnancy.due_day));
    }
    #[test]
    fn due_pregnancy_births_once_per_completed_day() {
        let mut game = run(71);
        game.party[0].sex = Sex::Female;
        game.party[0].age = 30;
        game.family.pregnancies =
            vec![Pregnancy { mother: game.party[0].name.clone(), due_day: game.day + 1 }];
        let mut outcomes = Vec::new();
        game.pass_camp_day(&mut outcomes, false);
        assert_eq!(game.party.iter().filter(|member| member.name == "Baby 1").count(), 1);
        assert!(game.family.pregnancies.is_empty());
        assert!(outcomes.iter().any(|outcome| matches!(outcome, Outcome::Message(message) if message.contains("gives birth to Baby 1"))));
        game.family_daily(&mut outcomes);
        assert_eq!(game.party.iter().filter(|member| member.name == "Baby 1").count(), 1);
        let baby = game.party.iter().find(|member| member.name == "Baby 1").unwrap();
        assert_eq!((baby.age, baby.health), (0, 80));
        assert!(baby.traits.is_empty());
        assert_eq!(baby.skills, crate::party::Skills::default());
        for other in game.party.iter().filter(|member| member.name != "Baby 1") {
            assert_eq!(
                baby.relationships.affinity[&other.name],
                other.relationships.affinity["Baby 1"]
            );
        }
    }
    #[test]
    fn due_pregnancy_does_not_revive_dead_mothers_and_handles_corrupt_full_parties() {
        let mut dead_mother = run(72);
        dead_mother.party[0].sex = Sex::Female;
        dead_mother.party[0].alive = false;
        dead_mother.family.pregnancies =
            vec![Pregnancy { mother: dead_mother.party[0].name.clone(), due_day: dead_mother.day }];
        dead_mother.family_daily(&mut Vec::new());
        assert!(dead_mother.family.pregnancies.is_empty());
        assert!(!dead_mother.party[0].alive);
        assert_eq!(dead_mother.party.len(), 5);

        let mut full_party = run(73);
        full_party.party[1].sex = Sex::Female;
        full_party.party[1].age = 30;
        let mother = full_party.party[1].name.clone();
        while full_party.party.len() < 12 {
            full_party.party.push(PartyMember::new(format!("Extra {}", full_party.party.len())));
        }
        full_party.family.pregnancies =
            vec![Pregnancy { mother: mother.clone(), due_day: full_party.day }];
        assert!(matches!(full_party.validate(), Err(CommandError::InvalidSetup)));
        let mut outcomes = Vec::new();
        full_party.family_daily(&mut outcomes);
        assert!(full_party.family.pregnancies.is_empty());
        assert_eq!(full_party.party.len(), 11);
        assert!(!full_party.party.iter().any(|member| member.name == mother));
        assert!(outcomes.iter().any(|outcome| matches!(outcome, Outcome::Message(message) if message.contains("newborn child"))));
    }
    #[test]
    fn wedding_and_departure_keep_durable_family_and_party_records() {
        let mut game = run(74);
        for member in &mut game.party {
            member.age = 30;
        }
        let first = game.party[0].name.clone();
        let second = game.party[1].name.clone();
        game.party[0].relationships.affinity.insert(second.clone(), 25);
        game.party[1].relationships.affinity.insert(first, 25);
        game.effects(&[Effect::CelebrateWedding], &mut Vec::new());
        assert_eq!(
            game.family.marriages,
            vec![(game.party[0].name.clone(), game.party[1].name.clone())]
        );

        let leader = game.party[0].name.clone();
        let dead_name = game.party[1].name.clone();
        let departing = game.party[2].name.clone();
        game.party[0].morale = 0;
        game.party[1].alive = false;
        game.party[2].morale = 1;
        game.effects(&[Effect::MemberLeaves], &mut Vec::new());
        assert!(game.party.iter().any(|member| member.name == leader));
        assert!(game.party.iter().any(|member| member.name == dead_name && !member.alive));
        assert!(!game.party.iter().any(|member| member.name == departing));
        assert_eq!(game.family.marriages.len(), 1);
    }
    #[test]
    fn family_dsl_effects_target_deterministic_live_pairs_and_add_members() {
        let mut game = run(76);
        for member in &mut game.party {
            member.age = 30;
        }
        let names = game.party.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
        game.party[0].relationships.affinity.insert(names[1].clone(), 20);
        game.party[1].relationships.affinity.insert(names[0].clone(), 20);
        game.party[2].relationships.affinity.insert(names[3].clone(), -20);
        game.party[3].relationships.affinity.insert(names[2].clone(), -20);
        game.effects(&[Effect::AdjustRelationship(5)], &mut Vec::new());
        game.effects(&[Effect::AdjustRelationship(-5)], &mut Vec::new());
        assert_eq!(game.party[0].relationships.affinity[&names[1]], 25);
        assert_eq!(game.party[2].relationships.affinity[&names[3]], -25);
        game.effects(&[Effect::AddMember { name: "Elias".into(), age: 22 }], &mut Vec::new());
        let elias = game.party.iter().find(|member| member.name == "Elias").unwrap();
        assert_eq!(elias.age, 22);
        assert_eq!(elias.traits.len(), 2);
        for other in game.party.iter().filter(|member| member.name != "Elias") {
            assert_eq!(
                elias.relationships.affinity[&other.name],
                other.relationships.affinity["Elias"]
            );
        }
    }
    #[test]
    fn invalid_saved_pregnancy_reference_or_due_date_is_rejected() {
        let mut game = run(75);
        game.family.pregnancies =
            vec![Pregnancy { mother: "Missing".into(), due_day: game.day + 60 }];
        assert!(matches!(game.validate(), Err(CommandError::InvalidSetup)));
        game.family.pregnancies =
            vec![Pregnancy { mother: game.party[0].name.clone(), due_day: game.day + 131 }];
        assert!(matches!(game.validate(), Err(CommandError::InvalidSetup)));
    }
    #[test]
    fn pending_birth_reserves_an_invitation_slot_and_marriage_history_is_validated() {
        let mut game = run(77);
        game.party[0].sex = Sex::Female;
        game.party[0].age = 30;
        game.family.pregnancies =
            vec![Pregnancy { mother: game.party[0].name.clone(), due_day: game.day + 60 }];
        while game.party.len() < 11 {
            game.party.push(PartyMember::new(format!("Extra {}", game.party.len())));
        }
        assert_eq!(game.available_party_slots(), 0);
        game.effects(&[Effect::AddMember { name: "No Room".into(), age: 22 }], &mut Vec::new());
        assert!(!game.party.iter().any(|member| member.name == "No Room"));

        game.family.marriages = vec![("Departed".into(), "Partner".into())];
        assert!(game.validate().is_ok());
        game.family.marriages.push(("Partner".into(), "Departed".into()));
        assert!(matches!(game.validate(), Err(CommandError::InvalidSetup)));
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
    fn arrival_does_not_leave_an_unanswerable_pending_event() {
        let mut content = GameContent::starter();
        content.trails[0].nodes[0].routes[0].distance_miles = 1;
        content.events.push(EventDefinition {
            id: "last_day".into(),
            text: "Too late".into(),
            weight: 0,
            conditions: vec![Condition::Always],
            effects: vec![],
            choices: vec![EventChoice {
                id: "answer".into(),
                label: "Answer".into(),
                conditions: vec![],
                effects: vec![],
            }],
        });
        let mut game = GameState::with_content(2, content);
        game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
            departure_month: 4,
        });
        game.apply(Command::Buy { item_id: "oxen".into(), quantity: 1 });
        game.apply(Command::Buy { item_id: "food".into(), quantity: 100 });
        game.apply(Command::Depart);
        game.scheduled_events.push(PendingEvent { event_id: "last_day".into(), due_day: 1 });
        let outcomes = game.apply(Command::TravelDay);
        assert_eq!(game.status, RunStatus::Arrived);
        assert!(game.pending_event.is_none());
        assert!(outcomes.iter().all(
            |outcome| !matches!(outcome, Outcome::Event { event_id, .. } if event_id == "last_day")
        ));
    }

    #[test]
    fn terminal_event_effects_clear_the_pending_event() {
        let mut game = run(202);
        game.content.events.push(EventDefinition {
            id: "fatal_delay".into(),
            text: "The trail closes behind you.".into(),
            weight: 0,
            conditions: vec![Condition::Always],
            effects: vec![Effect::LoseDays(1)],
            choices: vec![EventChoice {
                id: "wait".into(),
                label: "Wait".into(),
                conditions: vec![],
                effects: vec![],
            }],
        });
        game.inventory.quantities.insert("food".into(), 0);
        for member in &mut game.party {
            member.health = 10;
        }
        game.scheduled_events.push(PendingEvent { event_id: "fatal_delay".into(), due_day: 1 });
        game.apply(Command::Rest { days: 1 });
        assert_eq!(game.status, RunStatus::Failed);
        assert!(game.pending_event.is_none());
        assert!(game.active_minigame.is_none());
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
        game.party[0].name = "Ruth Holloway".into();
        game.apply(Command::InviteNpc { npc_id: "emigrant_train".into() });
        assert_eq!(game.party.iter().filter(|member| member.npc_id.is_some()).count(), 1);
        let joined = game.party.iter().find(|member| member.npc_id.is_some()).unwrap();
        assert!(!joined.traits.is_empty());
        assert!(!joined.relationships.affinity.is_empty());
        assert_rejected_without_mutation(
            &mut game,
            Command::InviteNpc { npc_id: "emigrant_train".into() },
        );
        game.apply(Command::DismissNpc { npc_id: "emigrant_train".into() });
        assert_eq!(game.party.len(), 5);
        assert_eq!(game.party[0].name, "Ruth Holloway");
        assert!(game.party.iter().all(|member| member.npc_id.is_none()));
    }

    #[test]
    fn dialogue_tags_require_the_full_context_and_fall_back_to_generic() {
        let mut game = run(130);
        game.content.quotes = vec![
            QuoteDefinition {
                id: "hungry".into(),
                text: "Hungry.".into(),
                landmark_id: None,
                seasons: vec![Season::Spring],
                state_tags: vec!["hungry".into()],
            },
            QuoteDefinition {
                id: "wrong-place".into(),
                text: "Wrong place.".into(),
                landmark_id: Some("willamette".into()),
                seasons: vec![Season::Spring],
                state_tags: vec![],
            },
            QuoteDefinition {
                id: "generic".into(),
                text: "Generic.".into(),
                landmark_id: None,
                seasons: vec![],
                state_tags: vec![],
            },
        ];
        game.inventory.quantities.insert("food".into(), 44);
        assert!(
            matches!(game.apply(Command::Talk).as_slice(), [Outcome::Quote { quote_id, .. }] if quote_id == "hungry")
        );

        game.inventory.quantities.insert("food".into(), 45);
        assert!(
            matches!(game.apply(Command::Talk).as_slice(), [Outcome::Quote { quote_id, .. }] if quote_id == "generic")
        );

        game.party[0].ailments.push("fever".into());
        game.cash_cents = 80_000;
        game.weather = WeatherKind::Cold;
        game.ox_fatigue = 50;
        game.party[1].morale = 30;
        game.party[2].alive = false;
        game.party[3].morale = 60;
        game.day = 153;
        assert!(["sick", "wealthy", "late", "cold", "weary", "homesick", "grieving", "hopeful"]
            .into_iter()
            .all(|tag| game.quote_tag_matches(tag)));
        game.weather = WeatherKind::Hot;
        assert!(game.quote_tag_matches("thirsty"));
        assert!(!game.quote_tag_matches("unknown"));
    }

    #[test]
    fn npc_schedules_recur_across_day_transitions_and_saves() {
        let mut game = run(131);
        let npc = game.npcs.iter_mut().find(|npc| npc.id == "emigrant_train").unwrap();
        npc.period_days = 2;
        npc.day_window = 1;
        npc.last_mile = 2_040;
        game.day = 1;
        assert!(!game.npc_present("emigrant_train"));
        let outcomes = game.apply(Command::Rest { days: 1 });
        assert!(game.npc_present("emigrant_train"));
        assert!(outcomes.iter().any(|outcome| matches!(outcome, Outcome::Message(message) if message == "Ruth Holloway draws near again.")));
        let restored: GameState =
            serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
        assert!(restored.npc_present("emigrant_train"));
        assert_eq!(restored.npcs, game.npcs);
    }

    #[test]
    fn away_npc_trades_are_rejected_without_mutation() {
        let mut game = trade_game(132);
        let npc = game.npcs.iter_mut().find(|npc| npc.id == "emigrant_train").unwrap();
        npc.period_days = 2;
        npc.day_window = 1;
        game.day = 1;
        assert!(!game.npc_present("emigrant_train"));
        assert_rejected_without_mutation(
            &mut game,
            Command::Barter {
                npc_id: "emigrant_train".into(),
                offered_item: "food".into(),
                offered_quantity: 50,
                wanted_item: "clothing".into(),
                wanted_quantity: 1,
            },
        );
    }

    #[test]
    fn repair_uses_real_resources_and_keeps_or_clears_the_mandatory_event() {
        let mut unavailable = repair_game(133);
        assert!(!unavailable.can_repair());
        assert_rejected_without_mutation(&mut unavailable, Command::Repair);

        let mut spare = repair_game(134);
        spare.inventory.add("wheel", 1);
        let day = spare.day;
        let food = spare.inventory.get("food");
        let skill = spare.party.iter().map(|member| member.skills.repair).max().unwrap();
        assert!(spare.can_repair());
        let outcomes = spare.apply(Command::Repair);
        assert!(outcomes.iter().any(
            |outcome| matches!(outcome, Outcome::Message(message) if message.contains("spare"))
        ));
        assert_eq!(spare.day, day + 1);
        assert_eq!(spare.inventory.get("food"), food - 15);
        assert_eq!(spare.inventory.get("wheel"), 0);
        assert!(spare.pending_event.is_none());
        assert_eq!(spare.party.iter().map(|member| member.skills.repair).max().unwrap(), skill + 1);

        let mut tool_success = None;
        let mut tool_failure = None;
        for seed in 0..200 {
            let mut candidate = repair_game(seed);
            candidate.inventory.add("tools", 1);
            candidate.inventory.quantities.insert("food".into(), 0);
            candidate.party[0].health = 50;
            let outcomes = candidate.apply(Command::Repair);
            if candidate.pending_event.is_none() {
                tool_success.get_or_insert((candidate, outcomes));
            } else {
                tool_failure.get_or_insert((candidate, outcomes));
            }
            if tool_success.is_some() && tool_failure.is_some() {
                break;
            }
        }
        let (success, success_outcomes) =
            tool_success.expect("a tool repair should succeed for a seeded run");
        assert_eq!(success.inventory.get("wheel"), 0);
        assert_eq!(success.inventory.get("tools"), 1);
        assert!(success_outcomes.iter().any(|outcome| matches!(outcome, Outcome::Message(message) if message.contains("sets the wheel right"))));
        let (failure, failure_outcomes) =
            tool_failure.expect("a tool repair should fail for a seeded run");
        assert_eq!(failure.day, 1);
        assert!(failure.party[0].health < 50);
        assert_eq!(failure.pending_event.as_deref(), Some("broken_wheel"));
        assert!(failure_outcomes.iter().any(|outcome| matches!(outcome, Outcome::Message(message) if message.contains("repair fails"))));

        let mut terminal = repair_game(135);
        terminal.inventory.add("tools", 1);
        terminal.inventory.quantities.insert("food".into(), 0);
        for member in &mut terminal.party {
            member.health = 5;
        }
        terminal.apply(Command::Repair);
        assert_eq!(terminal.status, RunStatus::Failed);
        assert!(terminal.pending_event.is_none());
    }

    #[test]
    fn deep_fords_can_kill_but_ferries_do_not_without_illness() {
        let ford_deaths = (0..500)
            .filter(|seed| {
                deep_ford_game(*seed)
                    .apply(Command::CrossRiver { method: CrossMethod::Ford })
                    .iter()
                    .any(|outcome| matches!(outcome, Outcome::MemberDied { .. }))
            })
            .count();
        assert!(ford_deaths > 0, "deep ford deaths={ford_deaths}");
        let ferry_deaths = (0..500)
            .filter(|seed| {
                deep_ford_game(*seed)
                    .apply(Command::CrossRiver { method: CrossMethod::Ferry })
                    .iter()
                    .any(|outcome| matches!(outcome, Outcome::MemberDied { .. }))
            })
            .count();
        assert_eq!(ferry_deaths, 0);
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
        assert_eq!(game.price_cents("food"), Some(20));
        game.reputation = 10;
        let quote = game.price_cents("food").unwrap();
        assert_eq!(quote, 18);
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
        assert!(game.last_fresh_food_day.is_none());
        game.content.items.iter_mut().find(|item| item.id == "food").unwrap().limit = 5_000;
        game.inventory.quantities.insert("food".into(), 2_400);
        game.content.regions.insert("river".into(), Region::river_valley());
        game.current_node_id = Some("river".into());
        game.target_node_id = Some("river".into());
        assert!(matches!(game.apply(Command::Fish).as_slice(), outcomes
            if outcomes.iter().any(|outcome| matches!(outcome, Outcome::Message(message) if message == "Caught 0 lbs of fish."))));
        assert!(game.last_fresh_food_day.is_none());
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
    fn full_mortality_remains_certain_on_normal_and_hard() {
        for difficulty in [Difficulty::Normal, Difficulty::Hard] {
            let mut game = run(161);
            game.difficulty = difficulty;
            game.content.ailments = vec![AilmentDefinition {
                id: "fatal".into(),
                name: "Fatal".into(),
                severity: 2,
                daily_damage: 0,
                mortality_per_mille: 1_000,
            }];
            game.party[0].ailments = vec!["fatal".into()];
            game.party[0].ailment_days.insert("fatal".into(), 1);
            game.progress_ailments(&mut Vec::new());
            assert!(!game.party[0].alive);
        }
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

    #[test]
    fn hunt_session_gates_commands_and_consumes_exact_bullets() {
        let mut game = minigame_game(18, "farmer");
        let day = game.day;
        game.apply(Command::BeginHunt);
        let session = game.active_minigame.clone().unwrap();
        assert_eq!(session.kind, MinigameKind::Hunt);
        assert_eq!(session.ammo_available, 40);
        assert_rejected_without_mutation(&mut game, Command::SetPace(Pace::Grueling));
        game.apply(Command::HuntResult { food_lbs: 50, shots: 3 });
        assert!(game.active_minigame.is_none());
        assert_eq!(game.inventory.get("ammunition"), 1);
        assert_eq!(game.loose_bullets, 17);
        assert_eq!(game.day, day + 1);
        assert_eq!(game.inventory.get("food"), 135);
        assert_eq!(game.weight(), 137);
        assert_rejected_without_mutation(&mut game, Command::HuntResult { food_lbs: 0, shots: 0 });
    }

    #[test]
    fn hunt_result_limits_are_atomic_and_hunter_has_larger_bag() {
        let mut game = minigame_game(19, "farmer");
        game.apply(Command::BeginHunt);
        assert_rejected_without_mutation(
            &mut game,
            Command::HuntResult { food_lbs: 151, shots: 1 },
        );
        assert_rejected_without_mutation(&mut game, Command::HuntResult { food_lbs: 1, shots: 21 });
        assert_rejected_without_mutation(&mut game, Command::HuntResult { food_lbs: 1, shots: 0 });
        let mut hunter = minigame_game(19, "hunter");
        hunter.apply(Command::BeginHunt);
        assert_rejected_without_mutation(
            &mut hunter,
            Command::HuntResult { food_lbs: 101, shots: 1 },
        );
        hunter.apply(Command::HuntResult { food_lbs: 200, shots: 2 });
        assert!(hunter.inventory.get("food") >= 285);
    }

    #[test]
    fn rafting_abort_and_completion_are_one_time_and_apply_route_distance() {
        let mut game = minigame_game(20, "farmer");
        game.current_node_id = Some("the_dalles".into());
        game.status = RunStatus::AwaitingFork("the_dalles".into());
        game.miles = 1813;
        game.inventory.quantities.insert("food".into(), 5);
        let day = game.day;
        game.apply(Command::ChooseRoute { route_id: "columbia".into() });
        assert!(matches!(
            game.active_minigame,
            Some(MinigameSession { kind: MinigameKind::Raft, .. })
        ));
        assert_rejected_without_mutation(&mut game, Command::Continue);
        game.apply(Command::RaftResult { cargo_lost_lbs: 120, casualties: 1, completed: false });
        assert!(game.active_minigame.is_none());
        assert_eq!(game.status, RunStatus::AwaitingFork("the_dalles".into()));
        assert_eq!(game.day, day + 1);
        assert_eq!(game.inventory.get("food"), 0);
        assert_eq!(
            game.party.iter().filter(|member| !member.alive && member.health == 0).count(),
            1
        );
        game.apply(Command::ChooseRoute { route_id: "columbia".into() });
        game.apply(Command::RaftResult { cargo_lost_lbs: 0, casualties: 0, completed: true });
        assert_eq!(game.status, RunStatus::Arrived);
        assert_eq!(game.current_node_id.as_deref(), Some("willamette"));
        assert_eq!(game.miles, 1885);
        assert_rejected_without_mutation(
            &mut game,
            Command::RaftResult { cargo_lost_lbs: 0, casualties: 0, completed: true },
        );
    }

    #[test]
    fn rafting_caps_and_barlow_rules_preserve_rejected_state() {
        let mut game = minigame_game(21, "farmer");
        game.current_node_id = Some("the_dalles".into());
        game.status = RunStatus::AwaitingFork("the_dalles".into());
        game.apply(Command::ChooseRoute { route_id: "columbia".into() });
        assert_rejected_without_mutation(
            &mut game,
            Command::RaftResult { cargo_lost_lbs: 121, casualties: 0, completed: false },
        );
        assert_rejected_without_mutation(
            &mut game,
            Command::RaftResult { cargo_lost_lbs: 0, casualties: 5, completed: false },
        );
        game.active_minigame = None;
        game.cash_cents = 500;
        game.era_id = Some("1843".into());
        game.content.eras.push(EraDefinition {
            id: "1843".into(),
            name: "1843".into(),
            year: 1843,
        });
        assert_rejected_without_mutation(
            &mut game,
            Command::ChooseRoute { route_id: "barlow".into() },
        );
        game.era_id = Some("1848".into());
        game.apply(Command::ChooseRoute { route_id: "barlow".into() });
        assert_eq!(game.cash_cents, 0);
        assert_eq!(game.status, RunStatus::Travelling);
    }

    #[test]
    fn minigame_save_validation_and_terrain_mapping_reject_impossible_sessions() {
        let mut game = minigame_game(22, "farmer");
        game.apply(Command::BeginHunt);
        game.validate().unwrap();
        game.loose_bullets = 20;
        assert_eq!(game.validate(), Err(CommandError::InvalidSetup));
        game.loose_bullets = 0;
        game.active_minigame.as_mut().unwrap().ammo_available = 1;
        assert_eq!(game.validate(), Err(CommandError::InvalidSetup));
        game.current_node_id = Some("sierra".into());
        game.target_node_id = Some("sierra".into());
        assert_eq!(game.terrain(), Terrain::Mountains);
        game.current_node_id = Some("salt_desert".into());
        game.target_node_id = Some("salt_desert".into());
        assert_eq!(game.terrain(), Terrain::Desert);
        game.current_node_id = Some("willamette".into());
        game.target_node_id = Some("willamette".into());
        assert_eq!(game.terrain(), Terrain::Forest);
    }

    #[test]
    fn occupation_modifiers_change_real_prices_setup_and_treatment() {
        let mut banker = modifier_game(23, "banker", "1848");
        assert_eq!(banker.price_cents("food"), Some(18));
        assert_eq!(banker.sell_price_cents("food"), Some(9));
        assert!(matches!(
            banker.apply(Command::Buy { item_id: "food".into(), quantity: 1 }).as_slice(),
            [Outcome::Purchased { cost_cents: 18, .. }]
        ));
        let carpenter = modifier_game(23, "carpenter", "1848");
        assert_eq!(carpenter.price_cents("wheel"), Some(800));
        let soldier = modifier_game(23, "soldier", "1866");
        assert_eq!(soldier.price_cents("ammunition"), Some(160));
        let merchant = modifier_game(23, "merchant", "1848");
        assert_eq!(merchant.inventory.get("trade_goods"), 2);
        let preacher = modifier_game(23, "preacher", "1848");
        assert_eq!(preacher.reputation, 10);
        let mut doctor = modifier_game(23, "doctor", "1848");
        doctor.status = RunStatus::Travelling;
        doctor.party[0].ailments.push("fever".into());
        doctor.party[0].ailment_days.insert("fever".into(), 2);
        let medicine_skill = doctor.party[0].skills.medicine;
        doctor.apply(Command::Treat { member_index: 0, ailment_id: "fever".into() });
        assert_eq!(doctor.inventory.get("medicine"), 0);
        assert_eq!(doctor.party[0].skills.medicine, medicine_skill + 1);
        assert!(!doctor.party[0].ailment_days.contains_key("fever"));
    }

    #[test]
    fn restricted_occupations_and_eras_reject_without_mutation() {
        let mut soldier = modifier_game(24, "farmer", "1848");
        soldier.status = RunStatus::Setup;
        assert_rejected_without_mutation(
            &mut soldier,
            Command::Configure {
                trail_id: "oregon".into(),
                era_id: "1848".into(),
                occupation_id: "soldier".into(),
                party: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
                departure_month: 4,
            },
        );
        let mut mormon = modifier_game(24, "farmer", "1848");
        mormon.status = RunStatus::Setup;
        mormon.content.trails[0].id = "mormon".into();
        assert_rejected_without_mutation(
            &mut mormon,
            Command::Configure {
                trail_id: "mormon".into(),
                era_id: "1843".into(),
                occupation_id: "farmer".into(),
                party: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
                departure_month: 4,
            },
        );
    }

    #[test]
    fn camp_weather_matches_travel_and_river_helpers_include_conditions() {
        let mut camp = run(25);
        let mut travel = camp.clone();
        camp.apply(Command::Rest { days: 1 });
        travel.apply(Command::TravelDay);
        assert_eq!(camp.weather_state, travel.weather_state);
        let mut river = GameState::with_content(26, branch_content());
        river.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
            departure_month: 4,
        });
        river.current_node_id = Some("river".into());
        river.status = RunStatus::AwaitingRiver("river".into());
        river.weather_state.river_depth_bonus = 2;
        assert_eq!(river.effective_depth(), Some(4));
        assert!(river.crossing_risk(CrossMethod::Ford).unwrap() > 0);
    }

    #[test]
    fn era_rules_change_purchase_ferry_and_service_availability_and_survive_save() {
        let mut content = branch_content();
        content.eras.extend([
            EraDefinition { id: "1843".into(), name: "1843".into(), year: 1843 },
            EraDefinition { id: "1852".into(), name: "1852".into(), year: 1852 },
        ]);
        content.era_rules = BTreeMap::from([
            (
                "1843".into(),
                EraRules {
                    price_percent: 100,
                    ferry_fee_percent: 100,
                    ferries_available: false,
                    guide_cost_clothing: 4,
                    unavailable_stores: vec!["independence".into()],
                    event_weight_percent: BTreeMap::new(),
                },
            ),
            (
                "1852".into(),
                EraRules {
                    price_percent: 110,
                    ferry_fee_percent: 110,
                    ferries_available: true,
                    guide_cost_clothing: 2,
                    unavailable_stores: vec![],
                    event_weight_percent: BTreeMap::new(),
                },
            ),
        ]);
        content.regions = BTreeMap::from([
            ("independence".into(), Region::plains()),
            ("river".into(), Region::river_valley()),
            ("fork".into(), Region::plains()),
            ("detour".into(), Region::hills()),
            ("willamette".into(), Region::forest()),
        ]);

        let names = vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()];
        let mut demand = GameState::with_content(27, content.clone());
        demand.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1852".into(),
            occupation_id: "farmer".into(),
            party: names.clone(),
            departure_month: 4,
        });
        assert_eq!(demand.price_cents("food"), Some(22));
        assert!(matches!(
            demand.apply(Command::Buy { item_id: "food".into(), quantity: 1 }).as_slice(),
            [Outcome::Purchased { cost_cents: 22, .. }]
        ));
        demand.current_node_id = Some("river".into());
        demand.status = RunStatus::AwaitingRiver("river".into());
        assert_eq!(demand.ferry_cost(), Some(110));
        assert_eq!(demand.guide_cost(), 2);
        let cash = demand.cash_cents;
        demand.apply(Command::CrossRiver { method: CrossMethod::Ferry });
        assert_eq!(demand.cash_cents, cash - 110);
        let restored: GameState =
            serde_json::from_str(&serde_json::to_string(&demand).unwrap()).unwrap();
        assert_eq!(restored.content.era_rules, demand.content.era_rules);

        let mut limited = GameState::with_content(28, content);
        limited.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1843".into(),
            occupation_id: "farmer".into(),
            party: names,
            departure_month: 4,
        });
        assert!(!limited.can_shop());
        assert_rejected_without_mutation(
            &mut limited,
            Command::Buy { item_id: "food".into(), quantity: 1 },
        );
        limited.current_node_id = Some("river".into());
        limited.status = RunStatus::AwaitingRiver("river".into());
        assert_eq!(limited.ferry_cost(), None);
        assert_rejected_without_mutation(
            &mut limited,
            Command::CrossRiver { method: CrossMethod::Ferry },
        );
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
