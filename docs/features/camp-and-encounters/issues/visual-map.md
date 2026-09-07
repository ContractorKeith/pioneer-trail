# Visual route record and landmark stamps

## What to build

Visual route record and landmark stamps for the next playable Pioneer Trail update.

## Acceptance criteria

- [ ] Display a readable route with current position, visited landmark stamps, next supply stop and encountered graves.
- [ ] Support Oregon, California and Mormon trails, route forks and actual chosen route rather than assuming all nodes visited.
- [ ] Preserve existing grave inspection and useful textual distances/weather without implying unknown forecasts.
- [ ] Persist visited-route evidence with legacy-save defaults; test forks, mid-segment positions, keyboard scrolling, both sizes and no-art.

## Blocked by

None. Implement as a complete isolated slice; integration order is gathering, campsite, conversations, travel moments, visual map, journal, bonus. Shared-file conflicts are coordinator-owned.

## Verification

Simulation command/state behavior, save round trips and legacy loads, TUI input/render tests, workspace tests, fmt, clippy and content lint. A green worker report still requires full-diff review before merge.
