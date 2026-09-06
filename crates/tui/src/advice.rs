use pioneer_sim::{GameState, Pace, RationLevel, RunStatus, Terrain};

/// A short, state-specific prompt that fits the 80-column trail panel.
pub(crate) fn trail_advice(game: &GameState) -> String {
    let alive = game.party.iter().filter(|member| member.alive).count();
    let food = game.inventory.get("food");
    let daily_food = game.daily_food_lbs();
    let sick = game.party.iter().any(|member| member.alive && !member.ailments.is_empty());
    let critical = game.party.iter().any(|member| member.alive && member.health < 40);

    if matches!(game.status, RunStatus::Failed) {
        return if food == 0 {
            fit("Food gone. No single cause recorded. Buy food before extra ammo.")
        } else {
            fit("No single cause is recorded. Next run: check food, health, and pace often.")
        };
    }
    if matches!(game.status, RunStatus::Arrived) {
        return fit(format!(
            "You arrived with {alive} survivor(s). Supplies left over add to your score."
        ));
    }
    if food < daily_food.saturating_mul(3) {
        return fit(food_recovery_tip(game));
    }
    if sick {
        return if game.inventory.get("medicine") > 0
            || game.party.iter().any(|member| member.alive && member.skills.medicine >= 4)
        {
            fit("Someone is ill. Press I to treat an ailment; X opens the rest menu.")
        } else {
            fit("Illness, but no kit or trained medic. X rests with food; seek medicine.")
        };
    }
    if critical {
        return fit("Health is low. X opens rest; keep enough food for those camp days.");
    }

    match game.pace {
        Pace::Grueling => {
            fit("Grueling covers ground, but it does not restore health. Use it sparingly.")
        }
        Pace::Strenuous => fit("Strenuous gains miles without Steady's daily health recovery."),
        Pace::Steady if game.rations != RationLevel::Filling => {
            fit("Steady restores health when the day's chosen ration is met.")
        }
        Pace::Steady => fit(healthy_tip(game, food, daily_food, alive)),
    }
}

pub(crate) fn supplies_advice(game: &GameState) -> String {
    let daily_food = game.daily_food_lbs();
    let food = game.inventory.get("food");
    if food == 0 {
        fit("No food remains. Esc returns to camp for hunting, foraging, or buying.")
    } else {
        fit(format!(
            "{food} lb: up to {} ration days, less after spoilage or losses.",
            food / daily_food.max(1)
        ))
    }
}

pub(crate) fn screen_advice(game: &GameState, screen: crate::screens::Screen) -> Option<String> {
    match screen {
        crate::screens::Screen::Supplies => Some(supplies_advice(game)),
        crate::screens::Screen::Pace => {
            Some(fit("Steady restores 1 health when the day's chosen ration is fully met."))
        }
        crate::screens::Screen::Rations => {
            Some(fit("Filling: 3 lb/person/day and morale; meager: 2; bare bones: 1."))
        }
        crate::screens::Screen::Rest => {
            Some(fit("Rest uses rations and eases ox fatigue; with food, it can aid recovery."))
        }
        crate::screens::Screen::Score | crate::screens::Screen::Journey => Some(trail_advice(game)),
        _ => None,
    }
}

fn food_recovery_tip(game: &GameState) -> &'static str {
    let food_in_stock = game
        .current_node_id
        .as_ref()
        .and_then(|id| game.markets.get(id))
        .and_then(|market| market.stock.get("food"))
        .is_none_or(|stock| *stock > 0);
    if game.can_shop()
        && food_in_stock
        && game.price_cents("food").is_some_and(|price| game.cash_cents >= price)
    {
        "Food is short. Press 9 to buy food here before travelling another day."
    } else if matches!(game.terrain(), Terrain::RiverValley) {
        "Food is short. Press G to fish, or F to forage before travelling."
    } else if game.inventory.get("ammunition") > 0 || game.loose_bullets > 0 {
        "Food is short. Press 7 to hunt or F to forage before travelling."
    } else {
        "Food is short. Press F to forage before travelling."
    }
}

fn healthy_tip(game: &GameState, food: u32, daily_food: u32, alive: usize) -> String {
    match game.day % 4 {
        0 => format!(
            "{food} lb: up to {} ration days for {alive}, less after spoilage or losses.",
            food / daily_food.max(1)
        ),
        1 => "One ammunition box holds 20 shots. Hunt when food needs justify it.".into(),
        2 => "Rest uses rations, but eases ox fatigue and can help the party recover.".into(),
        _ => "At rivers, compare crossing risks before choosing a method.".into(),
    }
}

fn fit(text: impl AsRef<str>) -> String {
    let text = text.as_ref();
    if text.chars().count() <= 76 {
        text.into()
    } else {
        format!("{}…", text.chars().take(75).collect::<String>())
    }
}
