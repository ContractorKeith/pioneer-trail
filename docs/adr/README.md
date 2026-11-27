# Architecture decision records

Code in this repo carries no comments (enforced by `comment-ratchet`). A non-obvious decision
that a comment used to explain lives here instead. Add a new numbered file rather than
rewriting an accepted one; mark a replaced record `Superseded by NNNN`.

| ADR | Decision |
|---|---|
| [0001](0001-deterministic-pure-simulation.md) | The simulation is pure, deterministic and the only rules authority |
| [0002](0002-saves-and-compatibility.md) | Saves embed their content and old saves stay playable without rewriting |
| [0003](0003-no-farmable-rewards.md) | Repeatable interactions cannot farm rewards |
| [0004](0004-grounded-presentation.md) | Everything the player is told is grounded in real state |
| [0005](0005-terminal-and-art-pipeline.md) | Terminal lifecycle, fixed-step minigames and the strict `.px` pipeline |
| [0006](0006-crates-io-publishing.md) | crates.io publishing is a manual workflow |
| [0007](0007-no-code-comments.md) | Code carries no comments; decisions live in ADRs |
