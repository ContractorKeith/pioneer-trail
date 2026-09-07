# Illustrated conversations with recurring travelers

## What to build

Illustrated conversations with recurring travelers for the next playable Pioneer Trail update.

## Acceptance criteria

- [ ] Show setting-specific fort, trading-post or neighboring-wagon scenes with named speakers.
- [ ] Offer meaningful questions about route, supplies and local news based on actual game state; preserve existing quote content.
- [ ] Recognize previous meetings/trades/favors using persisted state; avoid unlimited reward farming and invented forecasts.
- [ ] Data-driven original dialogue, valid NPC availability, save compatibility, keyboard/no-art and both sizes covered by tests.

## Blocked by

None. Implement as a complete isolated slice; integration order is gathering, campsite, conversations, travel moments, visual map, journal, bonus. Shared-file conflicts are coordinator-owned.

## Verification

Simulation command/state behavior, save round trips and legacy loads, TUI input/render tests, workspace tests, fmt, clippy and content lint. A green worker report still requires full-diff review before merge.
