<p align="center"><img src="assets/logo.png" alt="Pioneer Trail" width="640"></p>

# Pioneer Trail

A keyboard-only terminal journey game. Outfit a wagon in Independence, travel west with a party,
and manage supplies, weather, illness, rivers, events, and hard choices.

![A Pioneer Trail journey in a terminal](assets/pioneer-trail.gif)

The project is playable from source. Public release binaries, crates.io publication, and Homebrew
distribution are not yet verified. Until then, use the source commands below.

## Play from source

Install a current Rust toolchain, clone this repository, then run:

```bash
cargo run -p pioneer-trail
```

To install the local binary into your Cargo bin directory:

```bash
cargo install --path crates/tui
pioneer-trail
```

The title screen starts a new journey or resumes a save. Use `--continue` to open the most recent
save directly. See [Playing](docs/PLAYING.md) for the full controls and save behavior.

## Useful commands

```bash
# Repeat a journey from a known seed.
cargo run -p pioneer-trail -- --seed 42

# Start with a particular setup.
cargo run -p pioneer-trail -- --occupation banker --month march --trail oregon --era 1848

# Keep saves separate while testing.
cargo run -p pioneer-trail -- --save-dir ./pioneer-save-test

# Use text-only scenes and turn-based minigames.
cargo run -p pioneer-trail -- --no-art

# Run simulated journeys without opening the terminal interface.
cargo run -p pioneer-trail -- --headless-sim 100 --seed 42
```

`--month` accepts `march` through `july` or `3` through `7`. `--no-art`, `--mono`, and a nonempty
`NO_COLOR` environment variable support terminals where pixel scenes or color are not useful.

## What is in the game

- Character setup, wagon shopping, pace and ration choices, travel, rest, and treatment.
- Weather, terrain, river crossings, forts, prices, and three routes across several eras.
- Data-driven trail events, ailments, morale, party relationships, and a local hall of fame.
- Hunting and rafting minigames, with text controls available through `--no-art`.
- Seeded runs, autosaves, local settings, and resumable journeys.

Read [the player guide](docs/PLAYING.md) before your first trip. [Build status](docs/BUILD_STATUS.md)
maps the current implementation to the original plan and records deliberate limits. The original
[design and phase plan](docs/PLAN.md) remains the project roadmap.

Future verified releases will appear on [GitHub Releases](https://github.com/ContractorKeith/pioneer-trail/releases/latest).
