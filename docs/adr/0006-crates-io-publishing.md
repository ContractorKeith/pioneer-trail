# 0006: crates.io publishing is a manual workflow

Status: Accepted (2026-11-07, recorded here 2026-11-27 from workflow comments)

## Decision
- `.github/workflows/crates.yml` runs on `workflow_dispatch` only, after the matching vX.Y.Z GitHub
  release. It needs the `CARGO_REGISTRY_TOKEN` secret with publish-new and publish-update scopes.
- `cargo publish --workspace` publishes pioneer-sim, pioneer-data and pioneer-trail in dependency
  order and waits for each to index.
- `pioneer-tools` is repo-only (`publish = false`).
