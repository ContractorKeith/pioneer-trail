use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Market {
    pub stock: BTreeMap<String, u32>,
    #[serde(default)]
    pub normal_stock: BTreeMap<String, u32>,
    pub reputation: i16,
    pub last_restock_day: u32,
}
impl Market {
    pub fn price_for(&self, item_id: &str, base: i64, season_markup: i64) -> i64 {
        let normal = self.normal_stock.get(item_id).copied().unwrap_or(100).max(1);
        let stock = self.stock.get(item_id).copied().unwrap_or(normal);
        let scarcity = u64::from(normal.saturating_sub(stock)) * 50 / u64::from(normal);
        let multiplier = (100 + season_markup + scarcity as i64).max(1);
        ((i128::from(base) * i128::from(multiplier) / 100).clamp(1, i128::from(i64::MAX))) as i64
    }
    pub fn replenish(&mut self, day: u32) {
        if day / 30 > self.last_restock_day / 30 {
            for (id, amount) in &mut self.stock {
                let normal = self.normal_stock.get(id).copied().unwrap_or(100);
                *amount = amount.saturating_add((normal / 4).max(1)).min(normal);
            }
            self.last_restock_day = day;
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NpcTrain {
    pub id: String,
    pub name: String,
    pub reputation: i16,
    pub inventory: BTreeMap<String, u32>,
    pub recurring: bool,
    /// The day on which this train last improved the party's reputation.
    /// This prevents repeating the same interaction from becoming a reputation faucet.
    #[serde(default)]
    pub last_reputation_day: Option<u32>,
    #[serde(default)]
    pub first_mile: u32,
    #[serde(default)]
    pub last_mile: u32,
    #[serde(default)]
    pub period_days: u32,
    #[serde(default)]
    pub day_window: u32,
}
impl NpcTrain {
    pub fn accepts(
        &self,
        offered_value_cents: u64,
        wanted_value_cents: u64,
        reputation: i16,
    ) -> bool {
        let goodwill_discount = u64::try_from(reputation.max(0)).unwrap_or(0).min(25);
        u128::from(offered_value_cents) * u128::from(100 + goodwill_discount)
            >= u128::from(wanted_value_cents) * 100
    }
}

/// A trade the NPC has priced and left open for explicit confirmation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Counteroffer {
    pub npc_id: String,
    pub offered_item: String,
    pub offered_quantity: u32,
    pub wanted_item: String,
    pub wanted_quantity: u32,
    pub offered_value_cents: u64,
    pub wanted_value_cents: u64,
    pub quoted_day: u32,
}
