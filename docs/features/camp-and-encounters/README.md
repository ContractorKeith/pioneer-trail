# Camp and encounters

Status: all seven features reviewed, merged and pushed; v0.2.0 release checks underway.
Baseline: 4e85af0, v0.1.1. Full feature-review base: bef7f0c.

Seven independently reviewable slices are specified in issues/. GitHub issues are the task tracker. The bonus is a small optional sealed-letter delivery system.

Integration order: gathering, campsite, conversations, travel moments, visual map, journal, bonus. Independent work uses isolated worktrees. The coordinator owns integration and reviews complete branch diffs before merging. Tests target the existing public simulation command/state boundary, persistence save/load, and TUI input/render behavior.

Constraints: deterministic named RNG streams, serde defaults for older saves, six-color original pixel art, no portraits, keyboard-only at 80x24 and 120x40, mono/no-art equivalence, no new dependencies without justification. Cosmetic viewing and skipping must never alter simulation outcomes.

Final gates: workspace tests, clippy -D warnings, fmt, event-lint, balance evidence for changed costs/rewards, full journey scenarios, visual inspection, CI and release verification. Keep user saves untouched.

## Implementation

| Issue | Feature | State |
|---|---|---|
| [15](https://github.com/ContractorKeith/pioneer-trail/issues/15) | Illustrated gathering previews and results | Merged |
| [16](https://github.com/ContractorKeith/pioneer-trail/issues/16) | No-cost campsite with actionable supplies and party controls | Merged |
| [17](https://github.com/ContractorKeith/pioneer-trail/issues/17) | Named speakers, full replies, contextual topics and repeat visits | Merged |
| [18](https://github.com/ContractorKeith/pioneer-trail/issues/18) | Skippable travel moments, weather, wildlife and reduced motion | Merged |
| [19](https://github.com/ContractorKeith/pioneer-trail/issues/19) | Actual route history, wagon progress and next supply stop | Merged |
| [20](https://github.com/ContractorKeith/pioneer-trail/issues/20) | Persistent journals, factual memorials and personal epilogues | Merged |
| [21](https://github.com/ContractorKeith/pioneer-trail/issues/21) | Optional sealed-letter deliveries | Merged |

## Deliberate decisions

- Old saves keep their embedded content. New fields default safely, but new fort speakers and
  letter definitions require a new journey. Missing earlier route and journal history is not invented.
- Journal retention targets 200 entries, trimming routine entries first. Critical deaths and
  milestones are never discarded; a journal containing only critical entries can exceed that target.
- Letter destinations must be reachable and unavoidable after the offer, so a valid route choice
  cannot strand an accepted errand. Delivery pays $15 once; declining has no penalty.
- Cosmetic scenes use existing simulation state and do not draw simulation randomness. Opening
  camp, viewing a map or journal, and skipping a travel moment do not spend a day.
- Mandatory events still require a decision. Reduced motion suppresses optional travel pauses
  and freezes environmental and camp animation without bypassing game decisions.

## Verification

Each feature received a complete branch-diff review and rework before integration. Integrated
verification covers real route commands on both Oregon forks, letter journeys on all three trails,
legacy and malformed saves, source-specific death records, and art/text layouts at 80x24 and 120x40.
All seven issues are closed. Feature integration finished at c4d3acf; the release addendum retains
visible auto-travel and trade shortcuts and adds the mandatory-event transition guard.

- All 244 tests pass: 17 data, 89 simulation, 2 tools, 87 TUI unit, and 49 integration tests.
- Locked workspace tests, strict all-target Clippy, formatting, and embedded-content lint pass.
- A real-terminal recording was inspected in color and monochrome: camp, gathering preview/result,
  full conversation replies, route map, journal and letter offer/acceptance. TestBackend snapshots
  separately cover exact 80x24 and 120x40 layouts, including text-only mode.
- A real PTY run confirmed Ctrl-Q exits successfully and restores the exact prior terminal settings.
- All 11 supported trail/era combinations completed three seeded headless journeys (42–44).
  Mormon/1843 is rejected as intended. Both full Oregon endings also pass keyboard journey tests.
- Before/after carpenter/March balance, seeds from the reference 100-run report, is unchanged:
  12 arrivals, 5 with at least three survivors, mean 234.87 days and score 353.36. The reference
  policy does not claim optional conversation or letter rewards; dedicated tests bound those rewards.

### Standards review

No unresolved hard violations. Removed a single-use formatting module and shared duplicated
destination-name lookup. Simulation changes remain behind commands; presentation adds no RNG draws
or dependencies. Full branch diffs, not only final commits, were reviewed.

### Spec review

No unresolved requirements. Rework preserved full conversation replies, accurate death causes,
Camp return navigation and truthful control labels. A real travel command now tests the existing
mandatory weathered-event path, including render/Escape state invariance and response resolution.

Distribution checks and public release evidence are pending. The KödMem checkpoint connection
returns `workspace_not_registered`; no substitute project or retired worklog was used.
