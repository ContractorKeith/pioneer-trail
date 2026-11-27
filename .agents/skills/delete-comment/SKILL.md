---
name: delete-comment
description: Triage and remove code comments in pioneer-trail. Each comment is a tautology, stale, an excuse for a hack, or a decision record; delete it, file an issue, or move it into docs/adr/, then lock in the lower count with comment-ratchet. Use for '/delete-comment', 'clean up comments', 'the comment ratchet failed', or before a PR that leaves comments behind.
---

# delete-comment

Code here carries no comments (`docs/adr/0007-no-code-comments.md`). This skill removes the
ones still listed in `comment-baseline.txt`, or the ones a branch added.

## 1. Inventory
```bash
cargo run -q -p pioneer-tools --bin comment-ratchet -- --list > /tmp/comments.txt
cargo run -q -p pioneer-tools --bin comment-ratchet          # shows files over/under baseline
```
Scope to an argument if given (a path, crate or file); otherwise take every listed comment.
Group consecutive lines into one comment. Read the code around each one before judging it.

## 2. Categorise each comment

| Category | Test | Action |
|---|---|---|
| Tautology | The code, a name or a type already says it | Delete |
| Stale | Mentions code, a bug or behavior that no longer exists ("used to leak…") | Delete |
| Excuse | Explains why a hack, magic number, hard-coded value, duplicated path or unreadable sequence is OK | Delete and file an issue to fix the code |
| Decision record | A non-obvious *why* someone would otherwise undo: a determinism, save-compatibility, anti-farming, honesty or safety rule | Move into an ADR, then delete |

Notes:
- Clap `///` on a `Parser` field is help text, not a comment. Convert it to
  `#[arg(..., help = "...")]` and drop the trailing period (clap strips it from doc comments,
  so `--help` output stays identical).
- Prefer fixing an excuse in the same change when it is a one-liner (rename, named constant).
  Otherwise file the issue and delete the comment anyway; the issue holds the context.
- An excuse whose comment cannot go until the fix lands (deleting it would leave unreadable
  code) may stay; it is already in the baseline, and the issue must say to delete it.
- Test-fixture rationale is usually a tautology of the test name. Rename the test if it isn't.

## 3. Decision records
- Add to the ADR in `docs/adr/` whose theme fits (see its README index); start a new numbered
  ADR only for a new theme. Write the rule, not the history of the comment.
- Keep `docs/adr/README.md`'s table current.

## 4. Excuse issues
One issue per hack, via `gh issue create --repo ContractorKeith/pioneer-trail`, label
`enhancement` (see `docs/agents/issue-tracker.md`). Body: the file:line, the removed comment
quoted, what is hacky, and the fix. Write the body to a temp file and pass `--body-file`.

## 5. Delete and verify
Delete in place (no `sed` across files without checking each hit; `//` in strings and URLs is
not a comment). Then:
```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p pioneer-tools --bin event-lint
cargo run -p pioneer-tools --bin comment-ratchet -- --update   # lock in the lower counts
cargo run -p pioneer-tools --bin comment-ratchet
```
If you touched `assets/gen_*.py`, run them and confirm the generated `.px`/`.svg` are unchanged.
Commit `comment-baseline.txt` with the deletions (`chore: delete comments in <scope>`).

## 6. Report
Counts per category, ADRs touched, issue numbers, lines removed, and the new baseline total.
