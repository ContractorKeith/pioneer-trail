# Camp and encounters

Status: implementation planned. Baseline: 4e85af0, v0.1.1.

Seven independently reviewable slices are specified in issues/. GitHub issues are the task tracker. The bonus is a small optional sealed-letter delivery system.

Integration order: gathering, campsite, conversations, travel moments, visual map, journal, bonus. Independent work uses isolated worktrees. The coordinator owns integration and reviews complete branch diffs before merging. Tests target the existing public simulation command/state boundary, persistence save/load, and TUI input/render behavior.

Constraints: deterministic named RNG streams, serde defaults for older saves, six-color original pixel art, no portraits, keyboard-only at 80x24 and 120x40, mono/no-art equivalence, no new dependencies without justification. Cosmetic viewing and skipping must never alter simulation outcomes.

Final gates: workspace tests, clippy -D warnings, fmt, event-lint, balance evidence for changed costs/rewards, full journey scenarios, visual inspection, CI and release verification. Keep user saves untouched.
