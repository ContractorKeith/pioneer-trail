# First-journey improvements

## Playtest evidence

The first player report ended after 23 days and 120 miles as a Carpenter on Oregon/1848.
The saved terminal state had no food, $303.70, 49 ammunition boxes, ten clothing sets,
and five medicine kits. The save does not retain a full command or death-cause history.
These are observations, not a reconstruction of every purchase or cause of death.

The interface did not explain units, food consumption, or purchase priorities adequately.
The completion screen also had no illustration or useful advice for another attempt.

## Scope and integration order

1. [GH #12](https://github.com/ContractorKeith/pioneer-trail/issues/12): budget-aware outfitting,
   item units, occupation guidance, and departure warnings.
2. [GH #13](https://github.com/ContractorKeith/pioneer-trail/issues/13): contextual trail advice,
   action explanations, automatic-travel safety pauses, and an honest end-of-journey debrief.
3. [GH #14](https://github.com/ContractorKeith/pioneer-trail/issues/14): illustrated decision and
   outcome screens, readable layouts, and art/text/monochrome verification.

Workers use isolated branches. The coordinator reviews each full diff, requests corrections,
integrates in this order, and runs the final workspace checks. Existing player saves must not
be modified by testing. Guidance must follow actual rules and prices rather than promise survival.

Starting cash, mortality, random events, and the existing seed rules are not changed in this pass.
Better information should make the first attempt understandable without removing hard choices.

## Acceptance scenarios

- Carpenter, Oregon/1848: affordable recommended supplies, intelligible quantities and cash reserve.
- Every supported occupation/era: advice remains affordable and within cargo capacity.
- Under-equipped departure: an explicit warning and a deliberate way to proceed.
- Low food with ammunition, no ammunition, an open store, or a river: relevant available action.
- Illness or critical health: treatment/rest explanations and no unattended automatic travel.
- Failure and arrival: different illustrations, truthful debriefs, seed and navigation retained.
- 80x24 and 120x40, art, monochrome, and text modes: no hidden essential controls.
- Existing save load, keyboard journeys, simulation tests, content lint, formatting, and clippy.

## Status

Implemented and integrated for v0.1.1. Reviewed fixes include mutation-proof advice input,
visible departure confirmation, accurate ration/treatment explanations, and room for both
trail tips and the full status bar. The first 23 days were sampled with the recommended
Carpenter/Oregon/1848/March outfit across seeds 0–99: 100 runs retained at least three survivors,
with no food shortage observed during that period. This is not a full-route win-rate claim.

The original 1,000-run Carpenter reference baseline was 158 arrivals, 60 with at least three
survivors, and mean 231.4 days. It uses a different outfit and fixed policy from the new advice.
The only simulation change extracts the existing ration arithmetic into `daily_food_lbs()`;
no starting cash, consumption rate, mortality, or random sequence changed.

## Verification

- 158 tests pass, including five independent cross-screen acceptance tests and the 100-seed
  Carpenter early-trip sample. Workspace formatting, clippy with warnings denied, and content lint pass.
- Snapshots cover 80x24 and 120x40. The long-name failure keeps its seed, epitaph, and exit controls
  in color, monochrome, and text modes. Reading advice does not spend money or advance the world.
- The unchanged reference policy reproduces the pre-change 1,000-run Carpenter result exactly:
  158 arrivals, 60 with at least three survivors, mean 231.4 days.
- A separate copy of the player's v0.1.0 save loads in the updated binary. The original save and
  hall of fame are not used as recording or test destinations.

- Release source: `18383a2`, annotated tag `v0.1.1`. The
  [five-platform release workflow](https://github.com/ContractorKeith/pioneer-trail/actions/runs/34040453131)
  passed and published five binary archives plus `SHA256SUMS`.
- A fresh unauthenticated Apple Silicon public download passed its checksum, version 0.1.1,
  and a three-journey smoke test. The real terminal demo was regenerated and inspected, including
  the advice view, hunting, and the journey's full status, tips, and wagon. The isolated failed-save
  recording was also inspected with the new grounded tombstone and complete exit controls.
- Homebrew tap commit `0cde698` passed a source upgrade, `brew test`, strict audit, and style checks.
  `/opt/homebrew/bin/pioneer-trail` resolves to 0.1.1 and reproduced the 1,000-run Carpenter baseline.
  A real PTY opened the installed game with the isolated save; Ctrl-Q restored terminal settings.
  Homebrew replaced the old installation and removed its old cache; the v0.1.0 public release remains available.
- Original save and hall-of-fame SHA256 checksums were unchanged after verification. All three
  issues are closed, the team branches are merged, and their worktrees are removed.

KödMem checkpointing remains unavailable because this workspace is not registered. Crates.io
credentials remain a separate follow-up in GH #11; neither blocks GitHub or Homebrew play.
