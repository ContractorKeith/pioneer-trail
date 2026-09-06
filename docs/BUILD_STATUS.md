# Build status

This page maps the original [phase plan](PLAN.md) to the current source tree. It is a development
record, not a release announcement. Play from source using the commands in the [README](../README.md).

| Phase | Player-visible result | Status |
|---|---|---|
| 1. Core simulation | Calendar, travel, supplies, health, seeded state, and headless runs | Implemented |
| 2. Journey interface | Keyboard setup, store, events, river decisions, ailments, score, and title flow | Implemented |
| 3. Terminal scenes | Pixel scenes, wagon animation, map, and responsive terminal layouts | Implemented |
| 4. Active encounters | Hunting, rafting, talk, and route decisions | Implemented |
| 5. Persistence | Autosave, resume, seed codes, local hall of fame, and tombstones | Implemented |
| 6. Deeper journey state | Regional weather and terrain, expanded events, party morale, traits, and relationships | Active integration |
| 7. Economy and people | Fort stock and prices, reputation, trade, and scheduled emigrant trains | Active integration |
| 8. Routes and eras | Alternate trails, occupations, era rules, and location-specific world behavior | Active integration |
| 9. Access and polish | Monochrome and text-only play, turn-based text minigames, keyboard journeys, and wide-screen checks | Active integration |

## Deliberate limits

- The game is currently distributed from source only. No public crate, Homebrew formula, or release
  binary is claimed here. A release workflow and packaging work do not establish publication.
- Saves are local. There are no accounts, networked leaderboards, cloud sync, or shared worlds.
- A hunt or raft saved in progress restarts from its seeded beginning when resumed. The surrounding
  journey and its saved choices remain intact.
- Era, price, weather, and terrain rules are game balance. They are not presented as historical price
  tables or a complete historical simulation.
- Mod support, a GUI, mouse input, and web play remain outside the v1 scope.

Implementation notes for completed and active systems live in [the feature notes](features/). The
original [plan](PLAN.md) remains the design reference; it is not rewritten as status history.
