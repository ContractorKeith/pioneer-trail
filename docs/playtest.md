# Release acceptance

This checklist is the acceptance record for GitHub issue #8. Unchecked items
remain unverified. Automated coverage and hands-on play complement each other.

## Journey

- [ ] Complete setup using only the keyboard, including editing all five names.
- [ ] Outfitting displays cost, cash, quantities and wagon weight accurately.
- [ ] Play an Oregon journey from departure through arrival and score.
- [ ] Complete California and Mormon journeys with their own destinations.
- [ ] Test all four eras and occupation restrictions/perks.
- [ ] Verify both branches at forks and all available river methods.
- [ ] Reach the Columbia finale, complete rafting, and separately take Barlow Road.
- [ ] Die during a journey and enter an epitaph; find the grave on a later run.
- [ ] Treat illness, recover, and verify dead members cannot recover.
- [ ] Exercise event choices and a scheduled follow-up event.
- [ ] Hunt with keyboard controls; verify ammunition, time and meat accounting.
- [ ] Trade with a recurring NPC and buy from a fort with limited stock.

## Terminal and persistence

- [ ] Review every screen at 80×24 and 120×40, including long names/messages.
- [ ] Resize below minimum dimensions and back during play and a minigame.
- [ ] Verify numbered keys, arrows, Vim keys, Enter and Escape consistently.
- [ ] Test truecolor, 256-color, 16-color, mono and text-only modes.
- [ ] Test each speed setting and optional arrival/death bell.
- [ ] Quit and resume with identical state and subsequent random outcomes.
- [ ] Handle a corrupt or unsupported save without silently overwriting it.
- [ ] Enter a shared seed and verify trail/era/world reproducibility.
- [ ] Verify a completed run is recorded once in the hall of fame.
- [ ] Restore the terminal after normal exit, input error and panic.

## Build and distribution

- [ ] Workspace tests, formatting, Clippy and content lint pass.
- [ ] Screen snapshots cover 80×24 and 120×40.
- [ ] Simulation dependency tree contains no terminal libraries.
- [ ] Record balance results by occupation/month/difficulty and arrival/death causes.
- [ ] Banker March steady/filling benchmark meets the documented survival target.
- [ ] CI passes for the exact release commit.
- [ ] Package and install the published crates in dependency order.
- [ ] Download a fresh GitHub release binary and play it.
- [ ] Install through Homebrew and verify the resolved binary/version.
- [ ] README commands and demonstration match the shipped behavior.

## Evidence

The initial scaffold passed `cargo test --workspace` on September 6, 2026.
Those three tests do not establish gameplay acceptance. Add verified results here
as implementation is integrated, including commands, seed and tested dimensions.
