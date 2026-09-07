# Interactive illustrated campsite

## What to build

Interactive illustrated campsite for the next playable Pioneer Trail update.

## Acceptance criteria

- [ ] Enter a campsite from Journey with a fire, parked wagon, oxen and visible wagon condition; use the existing six-color style without party portraits.
- [ ] Provide reachable rest, treatment, supplies, gathering, hunting and party actions with clear return navigation.
- [ ] Camping itself costs nothing; chosen actions retain real simulation costs and phase transitions.
- [ ] Reduced motion and no-art preserve all information; test navigation, action costs and both terminal sizes.

## Blocked by

None. Implement as a complete isolated slice; integration order is gathering, campsite, conversations, travel moments, visual map, journal, bonus. Shared-file conflicts are coordinator-owned.

## Verification

Simulation command/state behavior, save round trips and legacy loads, TUI input/render tests, workspace tests, fmt, clippy and content lint. A green worker report still requires full-diff review before merge.
