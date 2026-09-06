# Pioneer Trail

A terminal (TUI) trail-survival game in the spirit of the 1985 Apple II Oregon Trail, built in
Rust with ratatui. Faithful to the original's structure (occupations, store, pace/rations,
landmarks, rivers, ailments, hunting, rafting, score), then expanded with weather/terrain,
party morale and relationships, trading/economy, and alternate trails/eras. "Done" is a
polished, keyboard-only game at 80×24 published on crates.io and a Homebrew tap.
**Full design and phase plan: `docs/PLAN.md`. Visual system (palette, `.px` format, layouts): `docs/DESIGN.md`. Read both before starting any phase.**

## Status
active integration — core journey, terminal play, scenes, encounters, saves, and accessibility work
are implemented; phases 6–9 are being integrated and verified. See `docs/BUILD_STATUS.md` for the
player-visible mapping. Do not treat this as a publication claim.

## Commands
```bash
cargo run -p pioneer-trail -- --headless-sim 3     # headless journeys (Phase 1)
cargo run -p pioneer-trail                          # TUI (Phase 2+)
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check
cargo run -p pioneer-tools --bin event-lint         # validates embedded content
```

## Architecture
Cargo workspace, four crates:
- `crates/sim` (`pioneer-sim`): pure game logic. No I/O, no terminal deps (CI enforces).
  Single mutation entry point `GameState::apply(Command) -> Vec<Outcome>`. Seeded ChaCha RNG
  with named sub-streams (`rng.stream("events")`) so seeds stay stable as systems grow.
- `crates/data` (`pioneer-data`): all content as RON + `.px` sprites, embedded via `include_dir`.
  New events/quotes/art must not require Rust changes unless a new condition/effect is needed.
- `crates/tui` (`pioneer-trail` binary): ratatui screens, minigames, persistence. Never computes
  game rules; renders `Outcome`s and submits `Command`s.
- `crates/tools` (`pioneer-tools`): `event-lint`, later `px-preview` and `balance-report`.

## Conventions & Gotchas
- Rust project. Ignore Python defaults from global rules.
- Determinism is a hard rule: the same seed + same commands must produce the same state.
  Never use `thread_rng()` in `sim`. Minigame outcomes enter the sim as a `Command` payload.
- Balance changes ship with a before/after `balance-report` in the commit message (from Phase 1).
- Content voice rules live in `docs/CONTENT_STYLE.md`. No copied Oregon Trail text or art.
- Art palette is the six Apple II hi-res colors (`docs/DESIGN.md` §1); fallbacks to 256/16/mono are the renderer's job.
- The logo/title art is generated: edit sprites in `assets/gen_logo.py`, then run it. Never hand-edit `title.px`, `logo.svg`, `logo.png`.
- Snapshot tests (`insta`) at 80×24 and 120×40 for every screen from Phase 2.
- Commit format `<type>: <description>`; author @ContractorKeith only.
- Checkpoint via the `log-work` skill at the end of each session.

## Out of Scope
- Mouse input, GUI, web build.
- Online leaderboards or accounts (hall of fame is local; opt-in upload is a maybe, post-v1).
- Any reuse of trademarked "Oregon Trail" name, text, or assets.
