//! Deterministic Monte Carlo summaries for the reference headless player.

use std::collections::BTreeMap;

use anyhow::{ensure, Result};
use clap::Parser;
use pioneer_sim::{GameContent, GameState, Outcome, RunStatus};
use pioneer_trail::headless::{self, BalanceSummary, RunConfig, RunSummary};

#[derive(Parser, Debug)]
#[command(name = "balance-report", about = "Run deterministic Pioneer Trail balance scenarios")]
struct Args {
    /// Journeys per occupation/month scenario.
    #[arg(long, default_value_t = 10_000)]
    runs: u32,
    /// First deterministic journey seed.
    #[arg(long, default_value_t = 0)]
    seed: u64,
    /// One occupation ID; omit to report every occupation available in the era.
    #[arg(long)]
    occupation: Option<String>,
    /// One departure month (3 through 7); omit to report March through July.
    #[arg(long)]
    month: Option<u8>,
    #[arg(long, default_value = "oregon")]
    trail: String,
    #[arg(long, default_value = "1848")]
    era: String,
}

#[derive(Debug, Default)]
struct Observation {
    death_causes: BTreeMap<String, u32>,
    arrival_month: Option<u8>,
}

impl Observation {
    fn observe(&mut self, game: &GameState, outcomes: &[Outcome]) {
        for outcome in outcomes {
            if let Outcome::MemberDied { name } = outcome {
                *self.death_causes.entry(death_cause(game, name)).or_default() += 1;
            }
        }
        if game.status == RunStatus::Arrived
            && outcomes.iter().any(|outcome| matches!(outcome, Outcome::ArrivedAt { .. }))
        {
            self.arrival_month = Some(game.date().1);
        }
    }
}

/// An ailment is named only when it is the sole observable cause at death.
/// Starvation, crossings, and multi-ailment cases deliberately remain unknown.
fn death_cause(game: &GameState, name: &str) -> String {
    game.party
        .iter()
        .find(|member| member.name == name && member.health == 0 && member.ailments.len() == 1)
        .map(|member| member.ailments[0].clone())
        .unwrap_or_else(|| "unknown".into())
}

#[derive(Debug, Default)]
struct Report {
    summary: BalanceSummary,
    total_score: u64,
    death_causes: BTreeMap<String, u32>,
    arrival_months: BTreeMap<u8, u32>,
}

impl Report {
    fn record(&mut self, run: &RunSummary, observation: Observation) {
        self.summary.record(run);
        self.total_score += u64::from(run.score);
        for (cause, count) in observation.death_causes {
            *self.death_causes.entry(cause).or_default() += count;
        }
        if let Some(month) = observation.arrival_month {
            *self.arrival_months.entry(month).or_default() += 1;
        }
    }

    fn csv_row(&self, occupation: &str, month: u8) -> String {
        let runs = f64::from(self.summary.runs.max(1));
        format!(
            "{occupation},{month},{},{},{},{:.2},{:.2}",
            self.summary.runs,
            self.summary.arrivals,
            self.summary.arrivals_with_three,
            self.summary.total_days as f64 / runs,
            self.total_score as f64 / runs,
        )
    }
}

fn run_batch(content: &GameContent, seed: u64, runs: u32, config: &RunConfig) -> Result<Report> {
    let mut report = Report::default();
    for index in 0..runs {
        let mut observation = Observation::default();
        let run = headless::journey(
            content,
            seed.wrapping_add(u64::from(index)),
            config,
            |game, outcomes| observation.observe(game, outcomes),
        )?;
        report.record(&run, observation);
    }
    Ok(report)
}

fn available_occupations(
    content: &GameContent,
    era: &str,
    requested: Option<&str>,
) -> Result<Vec<String>> {
    if let Some(id) = requested {
        ensure!(
            content.occupations.iter().any(|occupation| occupation.id == id),
            "unknown occupation {id}"
        );
        ensure!(occupation_available(id, era), "occupation {id} is unavailable in era {era}");
        return Ok(vec![id.into()]);
    }
    Ok(content
        .occupations
        .iter()
        .filter(|occupation| occupation_available(&occupation.id, era))
        .map(|occupation| occupation.id.clone())
        .collect())
}

fn occupation_available(occupation: &str, era: &str) -> bool {
    occupation != "soldier" || era == "1866"
}

fn validate_args(content: &GameContent, args: &Args) -> Result<(Vec<String>, Vec<u8>)> {
    ensure!(args.runs > 0, "--runs must be greater than zero");
    ensure!(
        content.trails.iter().any(|trail| trail.id == args.trail),
        "unknown trail {}",
        args.trail
    );
    ensure!(content.eras.iter().any(|era| era.id == args.era), "unknown era {}", args.era);
    let months = match args.month {
        Some(month) => {
            ensure!((3..=7).contains(&month), "--month must be from 3 through 7");
            vec![month]
        }
        None => (3..=7).collect(),
    };
    Ok((available_occupations(content, &args.era, args.occupation.as_deref())?, months))
}

fn render_csv(content: &GameContent, args: &Args) -> Result<String> {
    let (occupations, months) = validate_args(content, args)?;
    let mut output =
        String::from("occupation,month,runs,arrivals,three_survivors,mean_days,mean_score\n");
    let mut death_causes: BTreeMap<String, u32> = BTreeMap::new();
    let mut arrival_months: BTreeMap<u8, u32> = BTreeMap::new();
    for occupation in occupations {
        for &month in &months {
            let config = RunConfig {
                trail: args.trail.clone(),
                era: args.era.clone(),
                occupation: occupation.clone(),
                month,
            };
            let report = run_batch(content, args.seed, args.runs, &config)?;
            output.push_str(&report.csv_row(&occupation, month));
            output.push('\n');
            for (cause, count) in report.death_causes {
                *death_causes.entry(cause).or_default() += count;
            }
            for (arrival_month, count) in report.arrival_months {
                *arrival_months.entry(arrival_month).or_default() += count;
            }
        }
    }
    output.push_str("\ncause_of_death,count\n");
    for (cause, count) in death_causes {
        output.push_str(&format!("{cause},{count}\n"));
    }
    output.push_str("\narrival_month,count\n");
    for (month, count) in arrival_months {
        output.push_str(&format!("{month},{count}\n"));
    }
    Ok(output)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let content = pioneer_data::load()?;
    print!("{}", render_csv(&content, &args)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report_for(seed: u64, runs: u32) -> String {
        let content = pioneer_data::load().unwrap();
        render_csv(
            &content,
            &Args {
                runs,
                seed,
                occupation: Some("banker".into()),
                month: Some(3),
                trail: "oregon".into(),
                era: "1848".into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn aggregation_is_deterministic_for_a_small_fixed_batch() {
        assert_eq!(report_for(0, 4), report_for(0, 4));
    }

    #[test]
    fn unavailable_soldier_is_skipped_before_1866() {
        let content = pioneer_data::load().unwrap();
        assert!(!available_occupations(&content, "1848", None)
            .unwrap()
            .contains(&"soldier".into()));
        assert!(available_occupations(&content, "1866", None).unwrap().contains(&"soldier".into()));
    }
}
