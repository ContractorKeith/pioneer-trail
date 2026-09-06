use clap::Parser;
use pioneer_trail::{
    app::{self, App},
    headless::{self, BalanceSummary, RunConfig},
    persist::Storage,
    seed::WorldSeed,
};

#[derive(Parser, Debug)]
#[command(name = "pioneer-trail", version, about = "A keyboard-only trail survival game")]
struct Cli {
    /// Numeric world seed or a shared pt- code.
    #[arg(long)]
    seed: Option<String>,
    /// Simulate N journeys with the reference player.
    #[arg(long, value_name = "N")]
    headless_sim: Option<u32>,
    #[arg(long, default_value = "banker")]
    occupation: String,
    #[arg(long, default_value = "march", value_parser = parse_month)]
    month: u8,
    #[arg(long, default_value = "oregon")]
    trail: String,
    #[arg(long, default_value = "1848")]
    era: String,
    /// Print each advancing day during headless journeys.
    #[arg(long)]
    verbose: bool,
    /// Resume the most recent local journey.
    #[arg(long = "continue")]
    resume: bool,
    /// Text-only presentation without animated pixel scenes.
    #[arg(long)]
    no_art: bool,
    /// Render using monochrome glyphs.
    #[arg(long)]
    mono: bool,
}

fn parse_month(value: &str) -> Result<u8, String> {
    match value.to_ascii_lowercase().as_str() {
        "march" | "3" => Ok(3),
        "april" | "4" => Ok(4),
        "may" | "5" => Ok(5),
        "june" | "6" => Ok(6),
        "july" | "7" => Ok(7),
        _ => Err("departure month must be march, april, may, june or july".into()),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let content = pioneer_data::load()?;
    let mut config =
        RunConfig { occupation: cli.occupation, month: cli.month, trail: cli.trail, era: cli.era };
    let seed = match cli.seed {
        Some(code) if code.to_ascii_lowercase().starts_with("pt-") => {
            let world = WorldSeed::parse(&code)?;
            config.trail = world.trail;
            config.era = world.era;
            world.seed
        }
        Some(seed) => seed.parse::<u64>()?,
        None => {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos() as u64
        }
    };
    if let Some(count) = cli.headless_sim {
        anyhow::ensure!(count > 0, "--headless-sim must be greater than zero");
        let mut summary = BalanceSummary::default();
        for index in 0..count {
            let mut last_day = 0;
            let run = headless::journey(
                &content,
                seed.wrapping_add(u64::from(index)),
                &config,
                |game, _| {
                    if cli.verbose && game.day != last_day {
                        let (year, month, day) = game.date();
                        println!(
                            "{year}-{month:02}-{day:02}: {} miles, {} lb food, {} alive",
                            game.miles,
                            game.inventory.get("food"),
                            game.party.iter().filter(|member| member.alive).count()
                        );
                        last_day = game.day;
                    }
                },
            )?;
            if count <= 20 || cli.verbose {
                println!(
                    "seed {}: {} in {} days, {} miles, {} survivors, score {}",
                    run.seed,
                    if run.arrived { "arrived" } else { "failed" },
                    run.days,
                    run.miles,
                    run.survivors,
                    run.score
                );
            }
            summary.record(&run);
        }
        println!(
            "{} runs: {:.1}% arrived; {} arrived with at least three survivors; mean {:.1} days",
            summary.runs,
            summary.arrival_percent(),
            summary.arrivals_with_three,
            summary.total_days as f64 / f64::from(summary.runs)
        );
        return Ok(());
    }
    use std::io::IsTerminal;
    anyhow::ensure!(
        std::io::stdin().is_terminal() && std::io::stdout().is_terminal(),
        "Pioneer Trail needs an interactive terminal. Use --headless-sim N for batch runs."
    );
    let storage = Storage::platform()?;
    let mut settings = storage.load_settings()?;
    settings.no_art |= cli.no_art;
    if cli.mono {
        settings.color = pioneer_trail::persist::ColorMode::Mono;
    }
    let mut game = App::new(content, seed, settings).with_defaults(&config).with_storage(storage);
    if cli.resume {
        game.resume();
    }
    app::run(game)
}
