# Skippable travel moments and visible weather

## What to build

Skippable travel moments and visible weather for the next playable Pioneer Trail update.

## Acceptance criteria

- [ ] Show weather matching simulation and occasional original wildlife/terrain travel vignettes.
- [ ] Some moments are cosmetic, others reuse appropriate existing event decisions; do not manufacture extra hazards just to animate.
- [ ] Space out interruptions, support immediate skip and persisted reduced-motion preference; auto travel must respect crisis/event stops.
- [ ] Rendering and animation never consume simulation RNG or advance extra days; test skip, state preservation, small/large and no-art/mono.

## Blocked by

None. Implement as a complete isolated slice; integration order is gathering, campsite, conversations, travel moments, visual map, journal, bonus. Shared-file conflicts are coordinator-owned.

## Verification

Simulation command/state behavior, save round trips and legacy loads, TUI input/render tests, workspace tests, fmt, clippy and content lint. A green worker report still requires full-diff review before merge.
