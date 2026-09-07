# Bonus feature: sealed letters carried between trail stops

## What to build

Bonus feature: sealed letters carried between trail stops for the next playable Pioneer Trail update.

## Acceptance criteria

- [ ] At an eligible supply stop, optionally accept a sealed letter for a named recipient at a reachable later stop.
- [ ] Show destination and modest reward before acceptance; deliver through a clear keyboard action, only once, with no penalty for declining.
- [ ] Persist delivery state, handle route branches and all three trails, and record acceptance/delivery in the journal when available.
- [ ] Original data-driven prose; no network, new dependency or save break; deterministic rewards, duplicate protection and UI/accessibility tests.

## Blocked by

None. Implement as a complete isolated slice; integration order is gathering, campsite, conversations, travel moments, visual map, journal, bonus. Shared-file conflicts are coordinator-owned.

## Verification

Simulation command/state behavior, save round trips and legacy loads, TUI input/render tests, workspace tests, fmt, clippy and content lint. A green worker report still requires full-diff review before merge.
