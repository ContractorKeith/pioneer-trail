# Build status

This page maps the original [phase plan](PLAN.md) to the current source tree. It is a development
record. Version 0.1.1 is released on GitHub and Homebrew; see the [README](../README.md) to play
and [release acceptance](RELEASE_ACCEPTANCE.md) for verification.

[First-journey improvements](features/first-journey/README.md) records the v0.1.1 buying advice,
survival tips, automatic-travel safeguards, illustrated screens, and verification.

| Phase | Player-visible result | Status |
|---|---|---|
| 1. Core simulation | Calendar, travel, supplies, health, seeded state, and headless runs | Implemented |
| 2. Journey interface | Keyboard setup, store, events, river decisions, ailments, score, and title flow | Implemented |
| 3. Terminal scenes | Pixel scenes, wagon animation, map, and responsive terminal layouts | Implemented |
| 4. Active encounters | Hunting, rafting, talk, and route decisions | Implemented |
| 5. Persistence | Autosave, resume, seed codes, local hall of fame, and tombstones | Implemented |
| 6. Deeper journey state | Regional weather and terrain, expanded events, party morale, traits, relationships, and family milestones | Implemented and tested |
| 7. Economy and people | Fort stock and prices, reputation, trade, scheduled emigrant trains, and repairs | Implemented and tested |
| 8. Routes and eras | Oregon, California, Mormon trails; four eras and nine occupations | Implemented and tested |
| 9. Access and polish | Monochrome and text-only play, turn-based text minigames, keyboard journeys, wide-screen checks, and distribution | Released and verified; crates.io credentials pending |

## Deliberate limits

- Crates.io publication awaits registry credentials, tracked in [issue 11](https://github.com/ContractorKeith/pioneer-trail/issues/11).
  GitHub binaries and the Homebrew source formula are published and verified independently.
- Saves are local. There are no accounts, networked leaderboards, cloud sync, or shared worlds.
- A hunt or raft saved in progress restarts from its seeded beginning when resumed. The surrounding
  journey and its saved choices remain intact.
- Era, price, weather, and terrain rules are game balance. They are not presented as historical price
  tables or a complete historical simulation.
- Mod support, a GUI, mouse input, and web play remain outside the v1 scope.

Implementation notes for completed and active systems live in [the feature notes](features/). The
original [plan](PLAN.md) remains the design reference; it is not rewritten as status history.
