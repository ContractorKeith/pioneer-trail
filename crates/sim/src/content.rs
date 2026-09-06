//! Serializable content contract shared with `pioneer-data`.
//!
//! The simulation deliberately owns these types so data remains a one-way
//! dependency: data loads content, while rules never perform file I/O.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GameContent {
    pub eras: Vec<EraDefinition>,
    pub trails: Vec<TrailDefinition>,
    pub occupations: Vec<OccupationDefinition>,
    pub items: Vec<ItemDefinition>,
    pub ailments: Vec<AilmentDefinition>,
    pub events: Vec<EventDefinition>,
    pub quotes: Vec<QuoteDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EraDefinition {
    pub id: String,
    pub name: String,
    pub year: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OccupationDefinition {
    pub id: String,
    pub name: String,
    pub starting_cash_cents: i64,
    pub score_multiplier: f32,
    pub perk: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemDefinition {
    pub id: String,
    pub name: String,
    pub unit: String,
    pub price_cents: i64,
    pub weight_lbs: u32,
    pub limit: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AilmentDefinition {
    pub id: String,
    pub name: String,
    pub severity: u8,
    pub daily_damage: u8,
    pub mortality_per_mille: u16,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuoteDefinition {
    pub id: String,
    pub text: String,
    pub landmark_id: Option<String>,
    pub seasons: Vec<Season>,
    pub state_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrailDefinition {
    pub id: String,
    pub name: String,
    pub start_node_id: String,
    pub goal_node_id: String,
    pub nodes: Vec<LandmarkDefinition>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LandmarkDefinition {
    pub id: String,
    pub name: String,
    pub mile: u32,
    pub kind: LandmarkKind,
    pub routes: Vec<RouteDefinition>,
    pub river: Option<RiverDefinition>,
    pub store: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteDefinition {
    pub id: String,
    pub label: String,
    pub target_id: String,
    pub distance_miles: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiverDefinition {
    pub width_feet: u32,
    pub depth_feet: u32,
    pub ferry_cost_cents: Option<i64>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LandmarkKind {
    Town,
    Fort,
    River,
    Landmark,
    Fork,
    Finale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventDefinition {
    pub id: String,
    pub text: String,
    pub weight: u32,
    pub conditions: Vec<Condition>,
    pub effects: Vec<Effect>,
    pub choices: Vec<EventChoice>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventChoice {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    pub effects: Vec<Effect>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Condition {
    Always,
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
    MilesAtLeast(u32),
    FoodBelow(u32),
    DayAtLeast(u32),
    AtLandmark(String),
    HasAilment(String),
    Flag(String),
    Trail(String),
    Era(String),
    Occupation(String),
    Season(Season),
    Weather(WeatherKind),
    InventoryAtLeast { item_id: String, quantity: u32 },
    CashAtLeast(i64),
    MoraleBelow(i16),
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Effect {
    Message(String),
    AdjustFood(i32),
    AdjustItem { item_id: String, quantity: i32 },
    AdjustCash(i64),
    AdjustMorale(i16),
    InflictAilment(String),
    HealAilment(String),
    LoseDays(u32),
    SetFlag(String),
    ClearFlag(String),
    Schedule { event_id: String, days: u32 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WeatherKind {
    Clear,
    Warm,
    Hot,
    Rain,
    Storm,
    Snow,
    Cold,
}

impl GameContent {
    /// A complete small journey used by compatibility construction and tests.
    pub fn starter() -> Self {
        Self {
            eras: vec![EraDefinition { id: "1848".into(), name: "1848".into(), year: 1848 }],
            occupations: vec![OccupationDefinition {
                id: "farmer".into(),
                name: "Farmer".into(),
                starting_cash_cents: 40_000,
                score_multiplier: 3.0,
                perk: "Hardy oxen".into(),
            }],
            items: vec![
                ItemDefinition {
                    id: "oxen".into(),
                    name: "Oxen".into(),
                    unit: "yoke".into(),
                    price_cents: 4_000,
                    weight_lbs: 0,
                    limit: 9,
                },
                ItemDefinition {
                    id: "food".into(),
                    name: "Food".into(),
                    unit: "lb".into(),
                    price_cents: 20,
                    weight_lbs: 1,
                    limit: 2_000,
                },
                ItemDefinition {
                    id: "clothing".into(),
                    name: "Clothing".into(),
                    unit: "set".into(),
                    price_cents: 1_000,
                    weight_lbs: 2,
                    limit: 99,
                },
                ItemDefinition {
                    id: "medicine".into(),
                    name: "Medicine".into(),
                    unit: "kit".into(),
                    price_cents: 1_500,
                    weight_lbs: 2,
                    limit: 5,
                },
            ],
            trails: vec![TrailDefinition {
                id: "oregon".into(),
                name: "Oregon Trail".into(),
                start_node_id: "independence".into(),
                goal_node_id: "willamette".into(),
                nodes: vec![
                    LandmarkDefinition {
                        id: "independence".into(),
                        name: "Independence".into(),
                        mile: 0,
                        kind: LandmarkKind::Town,
                        routes: vec![RouteDefinition {
                            id: "main".into(),
                            label: "Main trail".into(),
                            target_id: "willamette".into(),
                            distance_miles: 2_040,
                        }],
                        river: None,
                        store: true,
                    },
                    LandmarkDefinition {
                        id: "willamette".into(),
                        name: "Willamette Valley".into(),
                        mile: 2_040,
                        kind: LandmarkKind::Finale,
                        routes: vec![],
                        river: None,
                        store: false,
                    },
                ],
            }],
            ..Self::default()
        }
    }
}
