# 0001: The simulation is pure, deterministic and the only rules authority

Status: Accepted (recorded 2026-11-27 from code comments)

## Context
The same seed and the same commands must always produce the same journey: share codes,
balance reports, replayed saves and snapshot tests all depend on it.

## Decision
- `pioneer-sim` does no I/O and has no terminal dependency. The UI changes state only through
  `GameState::apply(Command)`; each command handler (including `Converse`) is a private method
  behind that boundary.
- `pioneer-sim` owns the content types (`content.rs`). `pioneer-data` depends on the sim to load
  content, never the reverse, so rules never perform file I/O.
- RNG is a seeded ChaCha master with named sub-streams (`rng.stream("events")`); each stream's seed
  is `hash(master seed, name)`, so adding random calls in one system never reshuffles another's.
- Presentation never consumes simulation RNG or advances state. Travel moments are chosen by
  arithmetic over already-resolved state, not randomness, so opening or skipping one cannot change
  the journey. The weather overlay is derived only from the current simulated weather.
- Minigames: the sim decides when one may start and accepts exactly one result as a `Command`
  payload; the renderer runs the interactive part from the seed the sim supplies, on a fixed step.
- UI helpers that need a rule call the sim for it (for example `daily_food_lbs` feeds the food-day
  estimate). The headless reference player buys through the shop command so prices and stock stay
  simulation rules; a rejected purchase leaves state untouched (tested in sim).
- The one-day `Forage` and `Fish` commands stay alongside the multi-day `Gather`; each gathering day
  runs the same camp-day rules, so food, illness and a terminal party state resolve in the sim.

## Consequences
New randomness needs a new named stream. A UI feature that seems to need a rule needs a sim method.
