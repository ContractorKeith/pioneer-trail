//! Pioneer Trail binary. Phase 1 uses `--headless-sim`; the TUI arrives in Phase 2.

use clap::Parser;
use pioneer_sim::{Command, GameState};

#[derive(Parser, Debug)]
#[command(name = "pioneer-trail", version, about)]
struct Cli {
    /// World seed. Random if omitted.
    #[arg(long)]
    seed: Option<u64>,

    /// Run N headless journeys and print a summary instead of launching the TUI.
    #[arg(long, value_name = "N")]
    headless_sim: Option<u32>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let seed = cli.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    });

    if let Some(n) = cli.headless_sim {
        for i in 0..n {
            let mut g = GameState::new(seed.wrapping_add(i as u64));
            while g.miles < 2040 {
                g.apply(Command::Continue);
            }
            println!("run {i}: arrived day {}", g.day);
        }
        return Ok(());
    }

    println!("Pioneer Trail (seed {seed}). The TUI lands in Phase 2; try --headless-sim 3.");
    Ok(())
}
