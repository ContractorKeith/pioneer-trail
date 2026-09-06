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
    pub(crate) affordable: bool,
    pub(crate) shortfall_cents: i64,
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
        let affordable = completion_cost_cents <= game.cash_cents;
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
        if !affordable {
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
            affordable,
            shortfall_cents: completion_cost_cents.saturating_sub(game.cash_cents).max(0),
            recommended,
            warnings,
        }
    }

    pub(crate) fn selected_item_line(
        &self,
        game: &GameState,
        item_id: &str,
        quantity: u32,
    ) -> Option<String> {
        let item = game.content.items.iter().find(|item| item.id == item_id)?;
        let price = game.price_cents(item_id)?;
        Some(format!(
            "{}: ${:.2}/{} x{} = ${:.2}; own {}",
            item.name,
            price as f64 / 100.0,
            item.unit,
            quantity,
            price.saturating_mul(i64::from(quantity)) as f64 / 100.0,
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
    use pioneer_sim::{Command, CrossMethod, RunStatus};

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
        let mut game = configured("carpenter", "1848");
        let advice = OutfittingAdvice::for_game(&game);
        assert!(advice.completion_cost_cents <= game.cash_cents);
        assert_eq!(advice.target_food_lbs, 1_200);
        assert!(advice.target_food_days >= 80);
        assert_target_is_buyable(&mut game, advice);
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
                let mut game = configured(occupation, era);
                let advice = OutfittingAdvice::for_game(&game);
                assert!(
                    advice.completion_cost_cents <= game.cash_cents,
                    "{occupation} in {era} costs {} but has {}",
                    advice.completion_cost_cents,
                    game.cash_cents
                );
                assert_target_is_buyable(&mut game, advice);
            }
        }
        let mut game = configured("soldier", "1866");
        let advice = OutfittingAdvice::for_game(&game);
        assert!(advice.completion_cost_cents <= game.cash_cents);
        assert_target_is_buyable(&mut game, advice);
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
            advice.selected_item_line(&game, "ammunition", 10).unwrap(),
            "Ammunition: $2.00/box of 20 x10 = $20.00; own 49"
        );
        assert_target_is_buyable(&mut game, advice);
    }

    #[test]
    fn partial_shopping_keeps_owned_food_in_the_target() {
        let mut game = configured("carpenter", "1848");
        buy(&mut game, "oxen", 3);
        buy(&mut game, "food", 400);
        let advice = OutfittingAdvice::for_game(&game);
        assert!(advice.target_food_lbs >= 400);
        assert!(advice.affordable);
        assert_target_is_buyable(&mut game, advice);
    }

    #[test]
    fn low_cash_target_is_a_priority_list_not_an_affordable_claim() {
        let mut content = pioneer_data::load().unwrap();
        content
            .occupations
            .iter_mut()
            .find(|occupation| occupation.id == "farmer")
            .unwrap()
            .starting_cash_cents = 10_000;
        let mut game = GameState::with_content(7, content);
        game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: ["Ada", "Ben", "Clara", "David", "Eve"].map(str::to_owned).to_vec(),
            departure_month: 4,
        });
        let advice = OutfittingAdvice::for_game(&game);
        assert!(!advice.affordable);
        assert!(advice.shortfall_cents > 0);
        assert!(advice.warnings.iter().any(|warning| warning.contains("exceeds your cash")));
    }

    #[test]
    fn recommended_carpenter_load_covers_the_first_23_days() {
        let mut reached_day_23 = 0;
        let mut three_or_more_alive = 0;
        let mut food_shortages_before_fort = 0;
        for seed in 0..100 {
            let mut game = GameState::with_content(seed, pioneer_data::load().unwrap());
            game.apply(Command::Configure {
                trail_id: "oregon".into(),
                era_id: "1848".into(),
                occupation_id: "carpenter".into(),
                party: ["Ada", "Ben", "Clara", "David", "Eve"].map(str::to_owned).to_vec(),
                departure_month: 3,
            });
            let advice = OutfittingAdvice::for_game(&game);
            assert_target_is_buyable(&mut game, advice);
            assert!(!matches!(
                game.apply(Command::Depart).as_slice(),
                [pioneer_sim::Outcome::Rejected(_)]
            ));

            let mut steps = 0;
            while game.day < 23 && !matches!(game.status, RunStatus::Arrived | RunStatus::Failed) {
                steps += 1;
                assert!(steps < 200, "seed {seed} did not make progress");
                assert!(
                    game.active_minigame.is_none(),
                    "the early-trip driver never starts minigames"
                );
                let command = if let Some(event_id) = game.pending_event.clone() {
                    let choice_id = game
                        .content
                        .events
                        .iter()
                        .find(|event| event.id == event_id)
                        .and_then(|event| {
                            event.choices.iter().find(|choice| game.choice_available(choice))
                        })
                        .map(|choice| choice.id.clone())
                        .expect("an early event should offer a legal choice");
                    Command::Respond { event_id, choice_id }
                } else {
                    match game.status {
                        RunStatus::AwaitingRiver(_) => Command::CrossRiver {
                            method: if game.ferry_cost().is_some_and(|cost| cost <= game.cash_cents)
                            {
                                CrossMethod::Ferry
                            } else {
                                CrossMethod::Ford
                            },
                        },
                        RunStatus::AwaitingFork(_) => {
                            let route_id = game.current_landmark().unwrap().routes[0].id.clone();
                            Command::ChooseRoute { route_id }
                        }
                        RunStatus::AtLandmark(_) | RunStatus::Travelling => Command::Continue,
                        _ => unreachable!("configured run should be underway"),
                    }
                };
                game.apply(command);
                if game.miles < 304 && game.inventory.get("food") < game.daily_food_lbs() {
                    food_shortages_before_fort += 1;
                }
            }
            if game.day >= 23 {
                reached_day_23 += 1;
                if game.party.iter().filter(|member| member.alive).count() >= 3 {
                    three_or_more_alive += 1;
                }
            }
        }
        eprintln!(
            "recommended Carpenter March early trip: {three_or_more_alive}/{reached_day_23} runs had >=3 alive on day 23; {food_shortages_before_fort} food shortages during the first 23 days before Fort Kearney"
        );
        assert_eq!(reached_day_23, 100, "the test is a 100-seed early-trip sample");
        assert!(three_or_more_alive >= 90, "the recommended early outfit must remain viable");
        assert_eq!(
            food_shortages_before_fort, 0,
            "the recommended reserve should cover these first 23 days"
        );
    }

    fn buy(game: &mut GameState, item_id: &str, quantity: u32) {
        assert!(!matches!(
            game.apply(Command::Buy { item_id: item_id.into(), quantity }).as_slice(),
            [pioneer_sim::Outcome::Rejected(_)]
        ));
    }

    fn assert_target_is_buyable(game: &mut GameState, advice: OutfittingAdvice) {
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
