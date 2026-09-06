use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Market {
    pub stock: BTreeMap<String, u32>,
    pub reputation: i16,
    pub last_restock_day: u32,
}
impl Market {
    pub fn price(&self, base: i64, season_markup: i64) -> i64 {
        (base
            * (100
                + season_markup
                + i64::from(
                    100u32.saturating_sub(*self.stock.values().next().unwrap_or(&100)).min(50),
                ))
            / 100)
            .max(1)
    }
    pub fn replenish(&mut self, day: u32) {
        if day / 30 > self.last_restock_day / 30 {
            for amount in self.stock.values_mut() {
                *amount = amount.saturating_add(10);
            }
            self.last_restock_day = day;
        }
    }
}
