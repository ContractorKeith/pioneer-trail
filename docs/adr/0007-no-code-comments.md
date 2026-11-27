# 0007: Code carries no comments; decisions live in ADRs

Status: Accepted (2026-11-27)

## Context
Comments drift from the code, restate it, or excuse a hack that then gets copied. Agents read
them as instructions.

## Decision
- `.rs` and `.py` files carry no comments, doc comments included. `comment-ratchet`
  (`crates/tools`) enforces it in CI against `comment-baseline.txt`: a file's count may only go
  down, unlisted files must have none, and a reduction must be locked in with `--update`.
- A non-obvious decision goes in an ADR here. A workaround gets a GitHub issue, not a comment.
- Clap help text is an attribute (`#[arg(help = "...")]`), not a doc comment.
- The `/delete-comment` skill (`.agents/skills/delete-comment/`) repeats the triage.

## Consequences
docs.rs pages for the published crates have no prose; `README.md` and `docs/` are the docs.
