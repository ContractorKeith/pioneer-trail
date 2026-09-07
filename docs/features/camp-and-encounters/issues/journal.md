# Persistent party journal and personal epilogues

## What to build

Persistent party journal and personal epilogues for the next playable Pioneer Trail update.

## Acceptance criteria

- [ ] Persist dated, mile-marked notable milestones, recoveries, relationships and losses, captured when they actually happen.
- [ ] Provide a scrollable journal during a journey and a personal ending recap alongside score; memorialize individuals respectfully.
- [ ] Capture trustworthy cause information at the source; never infer a death cause from end-of-run inventory; legacy history is explicitly unknown.
- [ ] Bound growth sensibly without deleting critical deaths/milestones; cover arrivals, failures, departure versus death, old saves, no-art and both sizes.

## Blocked by

None. Implement as a complete isolated slice; integration order is gathering, campsite, conversations, travel moments, visual map, journal, bonus. Shared-file conflicts are coordinator-owned.

## Verification

Simulation command/state behavior, save round trips and legacy loads, TUI input/render tests, workspace tests, fmt, clippy and content lint. A green worker report still requires full-diff review before merge.
