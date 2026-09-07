# Illustrated foraging and fishing encounters

## What to build

Illustrated foraging and fishing encounters for the next playable Pioneer Trail update.

## Acceptance criteria

- [ ] Open a distinct original woodland or riverbank scene before gathering, with clear cost, availability and controls.
- [ ] Offer the existing one-day action and an explained longer search option governed by deterministic simulation; show the actual haul and elapsed days after resolution.
- [ ] Cancel without cost; quick results remain available; no repeated reward on dismissal or reload.
- [ ] Keyboard, 80x24/120x40, mono and no-art work; tests cover availability, costs, interruptions, results, save compatibility and rendering.

## Blocked by

None. Implement as a complete isolated slice; integration order is gathering, campsite, conversations, travel moments, visual map, journal, bonus. Shared-file conflicts are coordinator-owned.

## Verification

Simulation command/state behavior, save round trips and legacy loads, TUI input/render tests, workspace tests, fmt, clippy and content lint. A green worker report still requires full-diff review before merge.
