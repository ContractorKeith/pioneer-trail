# Release acceptance — v0.1.0

Release source: `a89abb8bcc2ea7ccd44a6223e8611cbb314036c6`, annotated tag `v0.1.0`.

## Code and playable journeys

- `cargo fmt --all --check`, locked workspace tests, and all-target clippy with warnings denied pass.
- 138 tests: 60 simulation, 14 embedded-data, 53 TUI, 9 keyboard journeys, and 2 balance-tool tests.
- Keyboard journeys cover title/setup/outfitting, event and river refusal/resume, trade/invite/dismiss,
  repair, a saved hunt, fatal events, the full 1848 Barlow route, and the full 1843 Columbia raft finale.
- Screens are snapshotted at 80×24 and 120×40. The color recording in `assets/pioneer-trail.gif`
  was inspected as actual terminal frames. A real PTY run confirmed Ctrl-Q restores terminal settings.
- Content validation passes for 3 trails, 4 eras, 9 occupations, 10 items, 22 ailments, 154 events,
  200 contextual quotes, and the embedded sprite/animation manifest.
- All 11 supported trail/era combinations completed 100-run smoke batches. Mormon/1843 is rejected.
- The shopping-only baseline contains 450,000 completed seeded journeys. Banker/March reached
  arrival with at least three survivors in 9,997 of 10,000 cases. See
  [the baseline](features/balance-shopping-baseline.md) and [reference policy](features/balance.md)
  before interpreting those rates as player difficulty.

## Distribution checks

The [tagged release workflow](https://github.com/ContractorKeith/pioneer-trail/actions/runs/34036491816)
gates publication on native Linux x86_64/arm64, macOS Intel/Apple Silicon, and Windows x86_64 builds.
Each target runs formatting, clippy, tests, content validation, release compilation, and a headless smoke.
Public artifact and Homebrew checks are recorded here after they complete.

Crates.io publication requires credentials not available in this session; it is tracked separately
in [issue 11](https://github.com/ContractorKeith/pioneer-trail/issues/11). The simulation package
passes Cargo package verification. Data and TUI package inventories include licenses and all
embedded assets; publishing them requires their predecessor crates to be indexed first.

## Deliberate limits

Minigames resume from their saved seeded start, not from a partially elapsed animation frame.
Text mode uses the same minigame worlds with action-driven time. Era prices and terrain/climate
weights are game abstractions. Phase 10 mod support is not included.
