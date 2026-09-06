//! A command-only reference player for reproducible journeys and balance reports.

use anyhow::{bail, Result};
use pioneer_sim::{Command, CrossMethod, GameContent, GameState, Outcome, RunStatus};

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub trail: String,
    pub era: String,
    pub occupation: String,
    pub month: u8,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self { trail: "oregon".into(), era: "1848".into(), occupation: "banker".into(), month: 3 }
    }
}

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub seed: u64,
    pub arrived: bool,
    pub survivors: usize,
    pub days: u32,
    pub miles: u32,
    pub score: u32,
}

pub fn journey(
    content: &GameContent,
    seed: u64,
    config: &RunConfig,
    mut observe: impl FnMut(&GameState, &[Outcome]),
) -> Result<RunSummary> {
    let mut game = GameState::with_content(seed, content.clone());
    submit(
        &mut game,
        Command::Configure {
            trail_id: config.trail.clone(),
            era_id: config.era.clone(),
            occupation_id: config.occupation.clone(),
            party: ["Ada", "James", "Ruth", "Thomas", "Clara"].map(str::to_owned).to_vec(),
            departure_month: config.month,
        },
    )?;
    for (item, quantity) in [
        ("oxen", 3),
        ("clothing", 5),
        ("ammunition", 10),
        ("medicine", 2),
        ("wheel", 1),
        ("axle", 1),
        ("tongue", 1),
        ("food", 2000),
    ] {
        buy_up_to(&mut game, item, quantity);
    }
    submit(&mut game, Command::Depart)?;
    let mut last_shop = None;
    for _ in 0..5000 {
        if matches!(game.status, RunStatus::Arrived | RunStatus::Failed) {
            return Ok(RunSummary {
                seed,
                arrived: game.status == RunStatus::Arrived,
                survivors: game.party.iter().filter(|member| member.alive).count(),
                days: game.day,
                miles: game.miles,
                score: game.score(),
            });
        }
        let command = if let Some(session) = &game.active_minigame {
            // The reference pilot holds the center lane for the entire real course.
            // Use the same collision world as interactive play, not a fabricated success.
            let mut raft = crate::minigame::rafting::RaftingGame::new(session.seed);
            while !raft.is_finished() {
                raft.tick();
            }
            let result = raft.result();
            Command::RaftResult {
                cargo_lost_lbs: result.cargo_lost_lbs.into(),
                casualties: result.casualties,
                completed: result.completed,
            }
        } else if let Some(id) = &game.pending_event {
            let event = game
                .content
                .events
                .iter()
                .find(|event| &event.id == id)
                .ok_or_else(|| anyhow::anyhow!("Pending event {id} is missing"))?;
            let choice = event
                .choices
                .iter()
                .find(|choice| game.choice_available(choice))
                .ok_or_else(|| anyhow::anyhow!("Pending event {id} has no choices"))?;
            Command::Respond { event_id: id.clone(), choice_id: choice.id.clone() }
        } else {
            match &game.status {
                RunStatus::AwaitingRiver(_) => {
                    let method = if game.ferry_cost().is_some_and(|cost| cost <= game.cash_cents) {
                        CrossMethod::Ferry
                    } else {
                        CrossMethod::Caulk
                    };
                    Command::CrossRiver { method }
                }
                RunStatus::AwaitingFork(id) => {
                    let route = node(&game, id)?
                        .routes
                        .iter()
                        .find(|route| {
                            route.id != "barlow" || (config.era != "1843" && game.cash_cents >= 500)
                        })
                        .ok_or_else(|| anyhow::anyhow!("Fork {id} has no routes"))?;
                    Command::ChooseRoute { route_id: route.id.clone() }
                }
                RunStatus::AtLandmark(id) => {
                    if last_shop.as_ref() != Some(id) && game.can_shop() {
                        last_shop = Some(id.clone());
                        let food = 1500u32.saturating_sub(game.inventory.get("food"));
                        buy_up_to(&mut game, "food", food);
                        let medicine = 3u32.saturating_sub(game.inventory.get("medicine"));
                        buy_up_to(&mut game, "medicine", medicine);
                    }
                    Command::Continue
                }
                RunStatus::Travelling => {
                    let sick = game
                        .party
                        .iter()
                        .enumerate()
                        .find(|(_, member)| member.alive && !member.ailments.is_empty());
                    if let Some((index, member)) =
                        sick.filter(|_| game.inventory.get("medicine") > 0)
                    {
                        Command::Treat {
                            member_index: index,
                            ailment_id: member.ailments[0].clone(),
                        }
                    } else {
                        Command::Continue
                    }
                }
                _ => bail!("Reference player cannot advance phase {:?}", game.status),
            }
        };
        let outcomes = submit(&mut game, command)?;
        observe(&game, &outcomes);
    }
    bail!(
        "Seed {seed} exceeded 5000 commands at day {}, mile {}, phase {:?}",
        game.day,
        game.miles,
        game.status
    )
}

fn node<'a>(game: &'a GameState, id: &str) -> Result<&'a pioneer_sim::LandmarkDefinition> {
    game.content
        .trails
        .iter()
        .find(|trail| Some(&trail.id) == game.trail_id.as_ref())
        .and_then(|trail| trail.nodes.iter().find(|node| node.id == id))
        .ok_or_else(|| anyhow::anyhow!("Missing landmark {id}"))
}

fn submit(game: &mut GameState, command: Command) -> Result<Vec<Outcome>> {
    let outcomes = game.apply(command.clone());
    if let Some(error) = outcomes.iter().find_map(|outcome| match outcome {
        Outcome::Rejected(error) => Some(error),
        _ => None,
    }) {
        bail!("Reference command {command:?} rejected at day {}: {error}", game.day);
    }
    Ok(outcomes)
}

fn buy_up_to(game: &mut GameState, item: &str, maximum: u32) {
    // Quantity search uses the shop command so prices and stock stay simulation rules.
    // Rejected purchases must leave state untouched, an invariant tested in sim.
    for quantity in (1..=maximum).rev() {
        let outcomes = game.apply(Command::Buy { item_id: item.into(), quantity });
        if !outcomes.iter().any(|outcome| matches!(outcome, Outcome::Rejected(_))) {
            break;
        }
    }
}

#[derive(Debug, Default)]
pub struct BalanceSummary {
    pub runs: u32,
    pub arrivals: u32,
    pub arrivals_with_three: u32,
    pub total_days: u64,
    pub total_survivors: u64,
}

impl BalanceSummary {
    pub fn record(&mut self, run: &RunSummary) {
        self.runs += 1;
        self.arrivals += u32::from(run.arrived);
        self.arrivals_with_three += u32::from(run.arrived && run.survivors >= 3);
        self.total_days += u64::from(run.days);
        self.total_survivors += run.survivors as u64;
    }

    pub fn arrival_percent(&self) -> f64 {
        100.0 * f64::from(self.arrivals) / f64::from(self.runs.max(1))
    }
}
