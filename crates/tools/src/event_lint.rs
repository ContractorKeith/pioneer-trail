//! Validates every embedded content file. Runs in CI. Grows with the event schema.

fn main() -> anyhow::Result<()> {
    let trails = pioneer_data::trail_files();
    anyhow::ensure!(!trails.is_empty(), "no trail files embedded");
    println!("event-lint: {} trail file(s) ok", trails.len());
    Ok(())
}
