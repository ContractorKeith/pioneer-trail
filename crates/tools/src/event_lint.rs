//! Validates every embedded content file. Runs in CI.

fn main() -> anyhow::Result<()> {
    let content = pioneer_data::load()?;
    println!(
        "event-lint: {} trails, {} eras, {} occupations, {} items, {} ailments, {} events, {} quotes ok",
        content.trails.len(),
        content.eras.len(),
        content.occupations.len(),
        content.items.len(),
        content.ailments.len(),
        content.events.len(),
        content.quotes.len(),
    );
    Ok(())
}
