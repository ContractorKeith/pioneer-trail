# Playing Pioneer Trail

## Start and resume

Run `cargo run --release -p pioneer-trail` from the repository root. The game requires a terminal at
least 80 by 24 characters. At the title screen, begin a new journey or continue the latest one.
`cargo run --release -p pioneer-trail -- --continue` opens that save directly.

Choose an occupation, month, trail, and era, name your party, then buy supplies before leaving
Independence. A seed makes the same setup and command sequence repeatable. Pass a number with
`--seed`, or copy the displayed `pt-` seed code.

For a forgiving first trip, choose Banker and March. Start with three oxen, 1,500 pounds of food,
five clothing sets, ten rounds of ammunition, one wheel, axle, and tongue, two medicine kits, and
one tool set. Keep supply bags below 2,400 pounds.

## Keyboard controls

The game is keyboard-only.

| Where | Keys |
|---|---|
| Menus | Arrow keys or `h` `j` `k` `l` to move, Enter or Space to choose, number keys for visible choices |
| General | Esc goes back, Ctrl-Q quits |
| Trail setup | Up and Down choose a trail, Left and Right choose an era, `d` cycles difficulty, Enter continues |
| On the trail | Enter or `1` advances one day; `a` toggles automatic travel |
| On the trail | `s` supplies, `m` map, `p` pace, `r` rations, `x` rest, `7` hunt |
| On the trail | `i` treatment, `u` trade, `v` party, `f` forage, `g` fish, `t` talk |
| Store | Up and Down choose an item, Left and Right set quantity, Enter buys, `s` sells |
| Events | `r` repairs when a repair action is offered |
| Settings | `c` cycles color, `b` toggles the bell, `a` toggles text-only art, `s` cycles speed |
| Hunt | Arrow keys or `h` `j` `k` `l` aim, Space shoots, Esc finishes |
| Raft | Left and Right, or `h` and `l`, steer; Esc aborts the crossing |

Some trail commands are available only when their situation permits them. For example, buying
requires an open store, and river choices appear when the party reaches a crossing.

## Making the trip

Travel advances the calendar and consumes food. Pace, rations, weather, terrain, health, and
supplies affect progress and risk. Check the map before committing to a route, and treat sick party
members before travel makes a manageable problem expensive.

Forts provide supplies and sometimes services. Prices depend on the location, era, occupation, and
your reputation. Events and crossings can cost food, cash, clothing, medicine, ammunition, time,
or wagon parts. Read the conditions on a choice before selecting it.

## Hunting and rafting

Hunting uses a moving aim and consumes ammunition. Carry enough ammo and leave with the meat you
can transport. Rafting asks you to steer around rocks; an aborted raft run returns you to the
crossing choice.

`--no-art` turns both minigames into text-only, turn-based play:

| Game | Keys |
|---|---|
| Hunt | `1` through `5` shoot a listed living target, Space waits one second, Esc finishes |
| Raft | `1` left, `2` center, `3` right, Space holds course for one second, Esc aborts |

Text mode prints the available targets or nearby rocks, along with ammunition, time, cargo, and
other relevant state. It does not require a real-time timer.

## Saves and restart behavior

The game autosaves the active journey, settings, and local hall of fame in its application-data
directory. Use `--save-dir PATH` to choose a separate directory for a run or test. Save data remains
local.

Resuming preserves the journey, including pending events and crossings. If a save is made while a
hunt or raft minigame is active, resuming restarts that minigame from its seeded initial state. It
does not preserve an in-progress aim, timer position, or raft steering position.

## Terminal options

| Option | Purpose |
|---|---|
| `--seed VALUE` | Use a numeric or `pt-` journey seed |
| `--occupation NAME` | Choose the setup occupation |
| `--month MONTH` | Choose March through July, or `3` through `7` |
| `--trail NAME` | Choose a route |
| `--era YEAR` | Choose an available era |
| `--continue` | Resume the latest save |
| `--save-dir PATH` | Store saves in a chosen directory |
| `--no-art` | Use text-only scenes and turn-based minigames |
| `--mono` | Disable color |
| `--headless-sim N` | Run N simulated journeys without the TUI |
| `--verbose` | Print daily details during a headless simulation |

Set `NO_COLOR` to any nonempty value to disable color. Run
`cargo run --release -p pioneer-trail -- --help` for command-line help from your current build.
