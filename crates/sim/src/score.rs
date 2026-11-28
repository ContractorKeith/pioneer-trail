use crate::content::{ItemDefinition, ItemScore, ScoreCategory};
use crate::health::PartyMember;

const BULLETS_PER_BOX: u64 = 20;

pub fn legacy_item_score(id: &str) -> Option<ItemScore> {
    let (points, per_units, category) = match id {
        "ammunition" => (20, 1, ScoreCategory::Ammunition),
        "oxen" => (8, 1, ScoreCategory::Goods),
        "food" => (1, 25, ScoreCategory::Goods),
        "clothing" | "wheel" | "axle" | "tongue" => (2, 1, ScoreCategory::Goods),
        _ => return None,
    };
    Some(ItemScore { points, per_units, category })
}

fn item_score(items: &[ItemDefinition], id: &str) -> Option<ItemScore> {
    items
        .iter()
        .find(|item| item.id == id)
        .and_then(|item| item.score)
        .or_else(|| legacy_item_score(id))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Breakdown {
    pub base: u32,
    pub early_arrival: u32,
    pub no_deaths: u32,
    pub no_ferry: u32,
    pub independence_rock: u32,
    pub total: u32,
}

pub fn breakdown(game: &crate::GameState) -> Breakdown {
    let multiplier = game
        .content
        .occupations
        .iter()
        .find(|job| Some(&job.id) == game.occupation_id.as_ref())
        .map_or(1., |job| job.score_multiplier);
    let mut inventory =
        game.inventory.quantities.iter().map(|(id, qty)| (id.clone(), *qty)).collect::<Vec<_>>();
    inventory.push(("loose_bullets".into(), u32::from(game.loose_bullets)));
    let base = calculate(&game.party, game.cash_cents, &inventory, &game.content.items, multiplier);
    let arrived = game.status == crate::RunStatus::Arrived;
    let (year, month, _) = game.date();
    let departure_year = game
        .content
        .eras
        .iter()
        .find(|era| Some(&era.id) == game.era_id.as_ref())
        .map_or(year, |era| era.year);
    let early_arrival = u32::from(arrived && year == departure_year && month < 10) * 200;
    let no_deaths = u32::from(
        arrived && !game.party.is_empty() && game.party.iter().all(|person| person.alive),
    ) * 100;
    let no_ferry = u32::from(arrived && !game.flags.contains("ferry_used")) * 100;
    let independence_rock =
        u32::from(arrived && game.flags.contains("independence_rock_early")) * 100;
    let total = base
        .saturating_add(early_arrival)
        .saturating_add(no_deaths)
        .saturating_add(no_ferry)
        .saturating_add(independence_rock);
    Breakdown { base, early_arrival, no_deaths, no_ferry, independence_rock, total }
}

pub fn calculate(
    members: &[PartyMember],
    cash_cents: i64,
    inventory: &[(String, u32)],
    items: &[ItemDefinition],
    multiplier: f32,
) -> u32 {
    let people: u32 = members
        .iter()
        .filter(|m| m.alive)
        .map(|m| match m.health {
            76.. => 500,
            51..=75 => 400,
            26..=50 => 300,
            _ => 200,
        })
        .sum();
    let mut bullets = 0u64;
    let mut goods = 0u64;
    for (id, qty) in inventory {
        let qty = u64::from(*qty);
        if id == "loose_bullets" {
            if let Some(score) = item_score(items, "ammunition").filter(|s| s.per_units > 0) {
                bullets +=
                    qty * u64::from(score.points) / (u64::from(score.per_units) * BULLETS_PER_BOX);
            }
            continue;
        }
        let Some(score) = item_score(items, id).filter(|score| score.per_units > 0) else {
            continue;
        };
        let points = qty * u64::from(score.points) / u64::from(score.per_units);
        match score.category {
            ScoreCategory::Ammunition => bullets += points,
            ScoreCategory::Goods => goods += points,
        }
    }
    let points = u64::from(people) + 50 + goods + bullets / 50 + cash_cents.max(0) as u64 / 500;
    (points as f64 * f64::from(multiplier)).round().clamp(0.0, u32::MAX as f64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_yokes_bullets_and_only_scoring_goods() {
        let people = vec![PartyMember::new("Ada".into())];
        let goods = [
            ("oxen", 3),
            ("ammunition", 5),
            ("food", 100),
            ("wheel", 2),
            ("clothing", 5),
            ("medicine", 5),
        ]
        .map(|(id, qty)| (id.to_owned(), qty));
        assert_eq!(
            calculate(&people, 1000, &goods, &[], 2.),
            (500 + 50 + 24 + 2 + 4 + 4 + 10 + 2) * 2
        );
    }
    #[test]
    fn old_save_content_without_item_scores_matches_current_content() {
        let mut current = crate::GameContent::starter();
        current.items.push(ItemDefinition {
            id: "ammunition".into(),
            name: "Ammunition".into(),
            unit: "box".into(),
            price_cents: 200,
            weight_lbs: 1,
            limit: 99,
            score: legacy_item_score("ammunition"),
        });
        let mut old = current.clone();
        for item in &mut old.items {
            item.score = None;
        }
        let people = vec![PartyMember::new("Ada".into())];
        let inventory = [
            ("oxen", 3),
            ("ammunition", 5),
            ("loose_bullets", 13),
            ("food", 117),
            ("clothing", 4),
            ("medicine", 2),
        ]
        .map(|(id, qty)| (id.to_owned(), qty));
        let expected = (500 + 50 + 24 + 4 + 8 + (100 + 13) / 50 + 2) * 2;
        assert_eq!(calculate(&people, 1000, &inventory, &current.items, 2.), expected);
        assert_eq!(calculate(&people, 1000, &inventory, &old.items, 2.), expected);
    }
    #[test]
    fn content_defined_weights_change_the_score() {
        let mut content = crate::GameContent::starter();
        let people = vec![PartyMember::new("Ada".into())];
        let inventory = [("oxen".to_owned(), 3), ("medicine".to_owned(), 2)];
        let before = calculate(&people, 0, &inventory, &content.items, 1.);
        content.items.iter_mut().find(|item| item.id == "oxen").unwrap().score =
            Some(ItemScore { points: 10, per_units: 1, category: ScoreCategory::Goods });
        content.items.iter_mut().find(|item| item.id == "medicine").unwrap().score =
            Some(ItemScore { points: 7, per_units: 2, category: ScoreCategory::Goods });
        assert_eq!(calculate(&people, 0, &inventory, &content.items, 1.), before - 24 + 30 + 7);
    }
    #[test]
    fn dead_members_do_not_score_and_large_cash_does_not_wrap() {
        let mut member = PartyMember::new("Ada".into());
        member.alive = false;
        assert_eq!(calculate(&[member], 0, &[], &[], 1.), 50);
        assert_eq!(calculate(&[], i64::MAX, &[], &[], 3.), u32::MAX);
    }
    #[test]
    fn loose_bullets_cross_box_scoring_boundaries() {
        assert_eq!(
            calculate(&[], 0, &[("ammunition".into(), 2), ("loose_bullets".into(), 10)], &[], 1.),
            51
        );
    }
    #[test]
    fn achievements_require_arrival_and_the_original_departure_year() {
        let mut game = crate::GameState::new(1);
        game.era_id = Some("1848".into());
        game.party = vec![PartyMember::new("Ada".into())];
        game.flags.insert("independence_rock_early".into());
        assert_eq!(breakdown(&game).total, breakdown(&game).base);
        game.status = crate::RunStatus::Arrived;
        assert_eq!(breakdown(&game).total, breakdown(&game).base + 500);
        game.day = 366;
        game.flags.insert("ferry_used".into());
        assert_eq!(breakdown(&game).early_arrival, 0);
        assert_eq!(breakdown(&game).no_ferry, 0);
    }
}
