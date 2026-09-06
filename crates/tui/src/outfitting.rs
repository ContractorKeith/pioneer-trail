//! Read-only, budget-aware guidance for the first store visit.

use pioneer_sim::GameState;

const TARGET_FOOD_LBS: u32 = 1200;
const FOOD_STEP_LBS: u32 = 100;
const TARGET_OXEN: u32 = 3;
const TARGET_AMMUNITION: u32 = 10;
const CASH_RESERVE_CENTS: i64 = 5_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutfittingAdvice {
    pub(crate) food_per_day: u32,
    pub(crate) food_days: u32,
    pub(crate) target_food_lbs: u32,
    pub(crate) target_food_days: u32,
    pub(crate) completion_cost_cents: i64,
    pub(crate) cash_after_cents: i64,
    pub(crate) recommended: Vec<(String, u32)>,
    pub(crate) warnings: Vec<String>,
}

impl OutfittingAdvice {
    pub(crate) fn for_game(game: &GameState) -> Self {
        let party_size = game.party.iter().filter(|member| member.alive).count() as u32;
        let food_per_day = game.daily_food_lbs().max(1);
        let required = [
            ("oxen", TARGET_OXEN),
            ("clothing", party_size),
            ("ammunition", TARGET_AMMUNITION),
            ("wheel", 1),
            ("axle", 1),
            ("tongue", 1),
            ("medicine", 1),
        ];
        let non_food_cost = required
            .iter()
            .fold(0_i64, |total, (id, target)| total + missing_cost(game, id, *target));
        let food_price = game.price_cents("food").unwrap_or(0).max(1);
        let affordable_food_addition =
            ((game.cash_cents - non_food_cost - CASH_RESERVE_CENTS).max(0) / food_price) as u32;
        let food_on_hand = game.inventory.get("food");
        let food_item = game.content.items.iter().find(|item| item.id == "food");
        let required_weight = required.iter().fold(0_u32, |total, (id, target)| {
            let missing = target.saturating_sub(game.inventory.get(id));
            let weight = game
                .content
                .items
                .iter()
                .find(|item| item.id == *id)
                .map_or(0, |item| item.weight_lbs.saturating_mul(missing));
            total.saturating_add(weight)
        });
        let capacity_for_food =
            2_400_u32.saturating_sub(game.weight().saturating_add(required_weight));
        let food_limit = food_item.map_or(food_on_hand, |item| item.limit);
        let food_stock = game
            .current_node_id
            .as_ref()
            .and_then(|node| game.markets.get(node))
            .and_then(|market| market.stock.get("food").copied())
            .unwrap_or_else(|| food_limit.saturating_sub(food_on_hand));
        let desired_food_addition = TARGET_FOOD_LBS.saturating_sub(food_on_hand);
        let target_food_lbs = food_on_hand.saturating_add(
            desired_food_addition
                .min(affordable_food_addition)
                .min(capacity_for_food)
                .min(food_limit.saturating_sub(food_on_hand))
                .min(food_stock)
                / FOOD_STEP_LBS
                * FOOD_STEP_LBS,
        );
        let mut recommended =
            required.iter().map(|(id, target)| ((*id).to_owned(), *target)).collect::<Vec<_>>();
        recommended.insert(1, ("food".into(), target_food_lbs));
        let completion_cost_cents = recommended
            .iter()
            .fold(0_i64, |total, (id, target)| total + missing_cost(game, id, *target));
        let food_days = game.inventory.get("food") / food_per_day;
        let target_food_days = target_food_lbs / food_per_day;
        let mut warnings = Vec::new();
        if game.inventory.get("oxen") < TARGET_OXEN {
            warnings.push(format!(
                "{} yoke on hand; three yoke is a safer start.",
                game.inventory.get("oxen")
            ));
        }
        if food_days < 30 {
            warnings.push(format!(
                "Food lasts about {food_days} days before spoilage; thirty is a thin reserve."
            ));
        }
        if game.inventory.get("clothing") < party_size {
            warnings.push(format!(
                "{} clothing sets for {party_size} travelers.",
                game.inventory.get("clothing")
            ));
        }
        if game.inventory.get("medicine") == 0 {
            warnings.push("No medicine kit on hand.".into());
        }
        if game.inventory.get("ammunition") > TARGET_AMMUNITION * 2 && food_days < 30 {
            warnings.push("Ammunition is plentiful; food is the urgent purchase.".into());
        }
        if completion_cost_cents > game.cash_cents {
            warnings
                .push("This target exceeds your cash; buy food after oxen and clothing.".into());
        }
        Self {
            food_per_day,
            food_days,
            target_food_lbs,
            target_food_days,
            completion_cost_cents,
            cash_after_cents: game.cash_cents.saturating_sub(completion_cost_cents),
            recommended,
            warnings,
        }
    }

    pub(crate) fn selected_item_line(&self, game: &GameState, item_id: &str) -> Option<String> {
        let item = game.content.items.iter().find(|item| item.id == item_id)?;
        let price = game.price_cents(item_id)?;
        Some(format!(
            "{}: ${:.2} per {} · {} on hand",
            item.name,
            price as f64 / 100.0,
            item.unit,
            game.inventory.get(item_id)
        ))
    }

    pub(crate) fn target_line(&self, game: &GameState) -> String {
        let items =
            self.recommended
                .iter()
                .filter_map(|(id, target)| {
                    game.content.items.iter().find(|item| item.id == *id).map(|item| {
                        format!("{} {} {}", target, item.unit, item.name.to_lowercase())
                    })
                })
                .collect::<Vec<_>>();
        format!("Starter target: {}", items.join(" · "))
    }
}

fn missing_cost(game: &GameState, item_id: &str, target: u32) -> i64 {
    let missing = target.saturating_sub(game.inventory.get(item_id));
    game.price_cents(item_id).unwrap_or(0).saturating_mul(i64::from(missing))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pioneer_sim::Command;

    fn configured(occupation: &str, era: &str) -> GameState {
        let mut game = GameState::with_content(7, pioneer_data::load().unwrap());
        game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: era.into(),
            occupation_id: occupation.into(),
            party: ["Ada", "Ben", "Clara", "David", "Eve"].map(str::to_owned).to_vec(),
            departure_month: 4,
        });
        game
    }

    #[test]
    fn carpenter_target_fits_cash_and_weight() {
        let game = configured("carpenter", "1848");
        let advice = OutfittingAdvice::for_game(&game);
        assert!(advice.completion_cost_cents <= game.cash_cents);
        assert_eq!(advice.target_food_lbs, 1_200);
        assert!(advice.target_food_days >= 80);
        assert_target_is_buyable(game, advice);
    }

    #[test]
    fn every_available_occupation_and_era_gets_an_affordable_target() {
        for era in ["1843", "1848", "1852", "1866"] {
            for occupation in [
                "banker",
                "merchant",
                "doctor",
                "blacksmith",
                "carpenter",
                "hunter",
                "preacher",
                "farmer",
            ] {
                let game = configured(occupation, era);
                let advice = OutfittingAdvice::for_game(&game);
                assert!(
                    advice.completion_cost_cents <= game.cash_cents,
                    "{occupation} in {era} costs {} but has {}",
                    advice.completion_cost_cents,
                    game.cash_cents
                );
                assert_target_is_buyable(game, advice);
            }
        }
        let game = configured("soldier", "1866");
        let advice = OutfittingAdvice::for_game(&game);
        assert!(advice.completion_cost_cents <= game.cash_cents);
        assert_target_is_buyable(game, advice);
    }

    #[test]
    fn item_units_and_food_warning_reflect_the_live_inventory() {
        let mut game = configured("carpenter", "1848");
        game.inventory.quantities.insert("ammunition".into(), 49);
        game.inventory.quantities.insert("clothing".into(), 10);
        game.inventory.quantities.insert("medicine".into(), 5);
        let advice = OutfittingAdvice::for_game(&game);
        assert_eq!(advice.food_per_day, 15);
        assert_eq!(advice.food_days, 0);
        assert!(advice.warnings.iter().any(|warning| warning.contains("Food lasts")));
        assert!(advice.warnings.iter().any(|warning| warning.contains("Ammunition")));
        assert_eq!(
            advice.selected_item_line(&game, "ammunition").unwrap(),
            "Ammunition: $2.00 per box of 20 · 49 on hand"
        );
        assert_target_is_buyable(game, advice);
    }

    fn assert_target_is_buyable(mut game: GameState, advice: OutfittingAdvice) {
        for (item_id, target) in advice.recommended {
            let quantity = target.saturating_sub(game.inventory.get(&item_id));
            if quantity > 0 {
                assert!(
                    !matches!(
                        game.apply(Command::Buy { item_id, quantity }).as_slice(),
                        [pioneer_sim::Outcome::Rejected(_)]
                    ),
                    "advice must not recommend an unbuyable target"
                );
            }
        }
        assert!(game.weight() <= 2_400);
    }
}
