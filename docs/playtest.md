# Manual playtest checklist

Use this checklist for a fresh hands-on playthrough. These unchecked boxes are not the current
release status. Verified build, scenario, and distribution evidence is in
[release acceptance](RELEASE_ACCEPTANCE.md). The first player feedback and improvement scope are
in [first-journey improvements](features/first-journey/README.md).

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

## First-journey checks

- [ ] Choose Carpenter and read the occupation and departure advice.
- [ ] Open store advice with `?`; inspect item units, food-days, and the affordable target.
- [ ] Confirm that advice does not buy, sell, spend time, or change the world.
- [ ] Attempt a risky departure, then choose to repack or deliberately proceed.
- [ ] Let food run low; check that automatic travel pauses and suggests an available action.
- [ ] Read pace, rations, and rest explanations; verify rest is not presented as free food.
- [ ] View the illustrated failure and arrival screens without losing the seed or exit controls.
