use crate::weather::{ClimateZone, Terrain};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GameContent {
    pub eras: Vec<EraDefinition>,
    #[serde(default)]
    pub era_rules: BTreeMap<String, EraRules>,
    #[serde(default)]
    pub regions: BTreeMap<String, Region>,
    pub trails: Vec<TrailDefinition>,
    pub occupations: Vec<OccupationDefinition>,
    pub items: Vec<ItemDefinition>,
    pub ailments: Vec<AilmentDefinition>,
    pub events: Vec<EventDefinition>,
    pub quotes: Vec<QuoteDefinition>,
    #[serde(default)]
    pub letters: Vec<LetterDefinition>,
    #[serde(default)]
    pub speakers: Vec<SpeakerDefinition>,
    #[serde(default)]
    pub npc_trains: Vec<NpcTrainDefinition>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LetterDefinition {
    pub id: String,
    pub trail_id: String,
    pub origin_id: String,
    pub destination_id: String,
    pub recipient: String,
    pub text: String,
    pub reward_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EraDefinition {
    pub id: String,
    pub name: String,
    pub year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EraRules {
    pub price_percent: u16,
    pub ferry_fee_percent: u16,
    pub ferries_available: bool,
    pub guide_cost_clothing: u32,
    pub unavailable_stores: Vec<String>,
    pub event_weight_percent: BTreeMap<String, u16>,
}

impl Default for EraRules {
    fn default() -> Self {
        Self {
            price_percent: 100,
            ferry_fee_percent: 100,
            ferries_available: true,
            guide_cost_clothing: 3,
            unavailable_stores: Vec::new(),
            event_weight_percent: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Region {
    pub terrain: Terrain,
    pub climate: ClimateZone,
}

impl Region {
    pub const fn plains() -> Self {
        Self { terrain: Terrain::Plains, climate: ClimateZone::Temperate }
    }
    pub const fn hills() -> Self {
        Self { terrain: Terrain::Hills, climate: ClimateZone::Temperate }
    }
    pub const fn river_valley() -> Self {
        Self { terrain: Terrain::RiverValley, climate: ClimateZone::Temperate }
    }
    pub const fn forest() -> Self {
        Self { terrain: Terrain::Forest, climate: ClimateZone::Pacific }
    }
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
    #[serde(default)]
    pub score: Option<ItemScore>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScoreCategory {
    Ammunition,
    Goods,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemScore {
    pub points: u32,
    pub per_units: u32,
    pub category: ScoreCategory,
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
pub struct SpeakerDefinition {
    pub id: String,
    pub name: String,
    pub landmark_id: String,
    pub greeting: String,
    pub returning_greeting: String,
    pub favor_text: String,
    #[serde(default = "default_favor_food_lbs")]
    pub favor_food_lbs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NpcTrainDefinition {
    pub id: String,
    pub name: String,
    pub reputation: i16,
    pub inventory: BTreeMap<String, u32>,
    pub recurring: bool,
    pub first_mile: u32,
    pub last_mile: u32,
    pub period_days: u32,
    pub day_window: u32,
    pub greeting: String,
    pub returning_greeting: String,
}

fn default_favor_food_lbs() -> u32 {
    15
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
    RelationshipAtLeast(i16),
    PartySizeBelow(u8),
    HasAdultPair,
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
    AdjustRelationship(i16),
    CelebrateWedding,
    MemberLeaves,
    AddMember { name: String, age: u8 },
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
    #[cfg(any(test, feature = "test-support"))]
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
                    score: Some(ItemScore {
                        points: 8,
                        per_units: 1,
                        category: ScoreCategory::Goods,
                    }),
                },
                ItemDefinition {
                    id: "food".into(),
                    name: "Food".into(),
                    unit: "lb".into(),
                    price_cents: 20,
                    weight_lbs: 1,
                    limit: 2_000,
                    score: Some(ItemScore {
                        points: 1,
                        per_units: 25,
                        category: ScoreCategory::Goods,
                    }),
                },
                ItemDefinition {
                    id: "clothing".into(),
                    name: "Clothing".into(),
                    unit: "set".into(),
                    price_cents: 1_000,
                    weight_lbs: 2,
                    limit: 99,
                    score: Some(ItemScore {
                        points: 2,
                        per_units: 1,
                        category: ScoreCategory::Goods,
                    }),
                },
                ItemDefinition {
                    id: "medicine".into(),
                    name: "Medicine".into(),
                    unit: "kit".into(),
                    price_cents: 1_500,
                    weight_lbs: 2,
                    limit: 5,
                    score: None,
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
            npc_trains: vec![
                NpcTrainDefinition {
                    id: "emigrant_train".into(),
                    name: "Ruth Holloway".into(),
                    reputation: 0,
                    inventory: BTreeMap::from([("food".into(), 100)]),
                    recurring: true,
                    first_mile: 0,
                    last_mile: 700,
                    period_days: 1,
                    day_window: 1,
                    greeting: "Ruth Holloway: Well met on the road.".into(),
                    returning_greeting: "Ruth Holloway: Back again, are you?".into(),
                },
                NpcTrainDefinition {
                    id: "mercer_wagon".into(),
                    name: "Elias Mercer".into(),
                    reputation: 0,
                    inventory: BTreeMap::from([("clothing".into(), 2), ("food".into(), 60)]),
                    recurring: true,
                    first_mile: 500,
                    last_mile: 1350,
                    period_days: 3,
                    day_window: 2,
                    greeting: "Elias Mercer: Well met on the road.".into(),
                    returning_greeting: "Elias Mercer: Back again, are you?".into(),
                },
                NpcTrainDefinition {
                    id: "bird_traders".into(),
                    name: "Nora Bird".into(),
                    reputation: 0,
                    inventory: BTreeMap::from([("medicine".into(), 1), ("food".into(), 40)]),
                    recurring: true,
                    first_mile: 1200,
                    last_mile: 1850,
                    period_days: 4,
                    day_window: 2,
                    greeting: "Nora Bird: Well met on the road.".into(),
                    returning_greeting: "Nora Bird: Back again, are you?".into(),
                },
            ],
            ..Self::default()
        }
    }
}
