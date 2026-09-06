# Pioneer Trail — Design & Build Plan

A terminal (TUI) trail-survival game in the spirit of the 1985 MECC Apple II classic.
Rust + ratatui. Faithful to the original's structure, then deeper in every system.

Decisions locked 2026-09-06:

| Area | Decision |
|---|---|
| Stack | Rust, ratatui + crossterm |
| Fidelity | Faithful+ (original structure, expanded systems) |
| Visuals | Unicode half-block pixel scenes, Apple II-style 6-color palette, text panels |
| Minigames | Real-time (hunting, rafting) |
| Expansions | Trading/economy, party morale & relationships, weather/terrain, alternate trails/eras |
| Saves | Autosave + seeded runs + hall of fame |
| Repo | `projects/pioneer-trail`, distribute via crates.io + ContractorKeith brew tap |
| Content | Data files (RON) embedded at build |

Legal note: "The Oregon Trail" is a trademark (Gameloft/HMH). We use our own name, our own prose,
and our own art. Mechanics and public-domain history (landmarks, distances, diseases) are fair game.
Do not paste original text, epitaphs, or sprites.

---

## 1. Player-facing overview

### 1.1 The loop

1. **Setup** — choose trail/era, occupation, name five party members, pick departure month, outfit at the store.
2. **Journey** — day ticks advance you along the trail. Each day: pace, rations, weather, terrain, health,
   food consumption, random events. Stop at landmarks (forts, rivers, mountains) to act.
3. **Decisions** — pace, rations, rest, hunt, trade, buy, talk, map, river-crossing method, route forks.
4. **Crises** — illness, injury, breakdowns, theft, lost trail, bad water, weather, morale collapse, deaths.
5. **Finale** — Columbia River: raft (real-time) or Barlow toll road. Arrive in the Willamette Valley.
6. **Score & legacy** — points by survivors, health, cash, gear, occupation multiplier. Hall of fame entry.
   Dead parties leave tombstones on the trail that future runs (and shared seeds) will pass.

### 1.2 What "Faithful+" means concretely

Everything in the left column ships in v1 as the original had it. Right column is the "+" layer.

| Original system | Pioneer Trail expansion |
|---|---|
| 3 occupations (banker/carpenter/farmer) | 8+ occupations with active perks, not just cash and multiplier |
| Leader + 4 unnamed-role members | Named members with age, traits, skills, relationships, morale |
| Pace: steady/strenuous/grueling | Same, plus terrain/weather/oxen-condition modifiers and wagon weight |
| Rations: filling/meager/bare bones | Same, plus food spoilage, variety (morale), foraging, fishing |
| ~20 random events | 150+ data-driven events with preconditions, weights, and chained follow-ups |
| 8 ailments | 20+ ailments with progression stages, contagion, treatment options, doctor/medicine |
| 5 forts, fixed price ramp | Forts + trading posts + emigrant camps; prices drift with season, supply, and your reputation |
| River: ford/caulk/ferry/wait/guide | Same, plus river depth driven by weather sim, wagon weight, oxen count, and time of year |
| Hunting: 1 screen, 5 animals | Biome-specific animals, ammo weight, meat spoilage, hunting skill progression, injuries |
| Map: static | Map with weather overlay, tombstones, party position, route forks |
| Talk to people: static quotes | NPC emigrants who travel near you, recur, trade, and can join or leave your party |
| Score screen | Score + hall of fame + run summary + shareable seed |
| One trail (Independence → Willamette) | Oregon, California, Mormon trails; 1843/1848/1852/1866 eras |

---

## 2. Content inventory (v1 target)

### 2.1 Trail: Oregon (default), Independence MO → Willamette Valley OR (~2,040 mi)

Approximate mileages below match the commonly cited 1985 version. **Verify against a primary source
before locking data** (task in Phase 2).

| # | Landmark | Miles | Type | Notes |
|---|---|---|---|---|
| 0 | Independence, Missouri | 0 | town | Start. Store. Departure Mar–Jul. |
| 1 | Kansas River Crossing | 102 | river | Ferry available ($5). Width ~620 ft, depth ~4 ft. |
| 2 | Big Blue River Crossing | 185 | river | No ferry. |
| 3 | Fort Kearney | 304 | fort | Store, talk, rest. |
| 4 | Chimney Rock | 554 | landmark | Scene, talk. |
| 5 | Fort Laramie | 640 | fort | Store. |
| 6 | Independence Rock | 830 | landmark | Carve name (morale bonus if by July 4). |
| 7 | South Pass | 932 | fork | Fort Bridger (long, store) or Green River shortcut (short, river). |
| 7a | Green River Crossing | 989 | river | Shortcut branch. |
| 7b | Fort Bridger | 1151 | fort | Long branch. Store. |
| 8 | Soda Springs | 1295 / 1143 | landmark | Mileage depends on branch. |
| 9 | Fort Hall | 1352 / 1200 | fort | Store. |
| 10 | Snake River Crossing | 1534 / 1380 | river | Shoshone guide option (costs clothing). |
| 11 | Fort Boise | 1648 / 1493 | fort | Store. |
| 12 | Blue Mountains | 1808 / 1653 | fork | Fort Walla Walla (long, store) or The Dalles shortcut. |
| 12a | Fort Walla Walla | 1863 / 1708 | fort | Long branch. |
| 12b | The Dalles | 1968 / 1813 | finale | Raft the Columbia or Barlow Toll Road. |
| 13 | Willamette Valley | 2040 / 1885 | end | Score. |

### 2.2 Occupations

| Occupation | Cash | Score × | Perk |
|---|---|---|---|
| Banker | $1600 | 1 | Better fort prices (haggling) |
| Merchant | $1400 | 1 | Trading yields more, starts with trade goods |
| Doctor | $1200 | 1.5 | Treat illness, reduced death chance |
| Blacksmith | $900 | 2 | Repair wagon parts without spares (chance) |
| Carpenter | $800 | 2 | Faster repairs, spare parts cheaper |
| Hunter / Trapper | $600 | 2.5 | Hunting accuracy, more meat carried |
| Preacher | $500 | 2.5 | Morale recovery, NPC goodwill |
| Farmer | $400 | 3 | Oxen health, foraging bonus |
| Soldier (1866 era) | $700 | 2 | Less theft, ammo cheaper |

### 2.3 Store (Independence baseline prices)

| Item | Unit | Price | Limit | Weight |
|---|---|---|---|---|
| Oxen | yoke (2) | $40 | 1–9 yoke | — |
| Food | lb | $0.20 | 2000 lb | 1 |
| Clothing | set | $10 | 99 | 2 |
| Ammunition | box of 20 | $2 | 99 boxes | 1 |
| Wagon wheel | each | $10 | 3 | 30 |
| Wagon axle | each | $10 | 3 | 40 |
| Wagon tongue | each | $10 | 3 | 25 |
| Medicine (+) | kit | $15 | 5 | 2 |
| Tools (+) | set | $25 | 1 | 15 |
| Trade goods (+) | lot | $20 | 10 | 10 |

Fort price multiplier: base × (1 + 0.25 × fort_index) in original; we replace with a
season/supply model in Phase 6 but keep this as the fallback.

Wagon capacity: ~2,400 lb dry. Overweight slows pace and raises breakdown chance.

### 2.4 Ailments (v1 = original 8, Phase 6 grows the list)

Original: typhoid, cholera, measles, dysentery, exhaustion, fever, snakebite, broken arm, broken leg.
Expansion: mountain fever, scurvy, frostbite, heat stroke, pneumonia, infection, food poisoning,
childbirth complications, depression (morale-linked), tooth abscess, gunshot (hunting accident).

Each ailment: onset conditions, daily progression roll, stages, treatment options, recovery time,
contagion flag, mortality curve.

### 2.5 Events (150+ target, all in RON)

Categories with example entries:

- **Weather**: hail, thunderstorm, blizzard, heavy fog, heat wave, flash flood, dust storm
- **Wagon**: wheel/axle/tongue break, fire, tipped in mud, lost in river
- **Animals**: ox injured, ox died, ox wandered off, snakebite, bear at camp, buffalo stampede
- **People**: member wanders off, member found, thief in the night, stranger joins, argument,
  wedding, birth, member leaves party at fort
- **Trail**: lost trail (days), wrong trail, impassable section, rough trail, inadequate grass
- **Water/food**: bad water, no water, wild fruit found, abandoned wagon supplies, fish in river
- **Encounters**: Native trader offers guide/food, emigrant train ahead asks for help,
  preacher's sermon, army patrol, gold-rush rumor (1848 era)
- **Chained**: e.g., "stranger joins" → later "stranger steals" or "stranger saves member"

### 2.6 Talk-to-people

Pool of 200+ short quotes, tagged by landmark, season, and party state (starving, sick, rich, late).
NPC emigrant trains carry persistent names and personalities across a run.

### 2.7 Tombstones

Dead parties leave `{leader_name, date, cause, epitaph, mile}`. Stored locally in the hall-of-fame
DB and surfaced on future runs at that mile. Seeded runs also embed a hashed tombstone list so
shared seeds show the same graves.

### 2.8 Scoring (original formula, then bonuses)

```
points  = Σ survivors: 500 (good) / 400 (fair) / 300 (poor) / 200 (very poor)
        + wagon 50 + oxen 4 each + spare parts 2 each + clothing 2 each
        + ammo 1 per 50 bullets + food 1 per 25 lb + cash 1 per $5
score   = points × occupation multiplier
bonuses = arrive before Oct 1, zero deaths, no ferry used, Independence Rock by July 4
```

---

## 3. Screens & UI

Target terminal: 80×24 minimum, scales to 120×40. Scenes are drawn in a fixed 80×16 pixel-art region
above a text/menu panel. If terminal < 80×24, show a "resize" screen.

| Screen | Contents |
|---|---|
| Title | Pixel-art wagon, menu: New Journey / Continue / Hall of Fame / Seed / Settings / Quit |
| Setup: Trail & Era | Choose trail (Oregon default), era, difficulty |
| Setup: Occupation | List with cash, multiplier, perk |
| Setup: Party | Name 5 members; Phase 6 adds age/traits |
| Setup: Departure | Month picker with advice |
| Store | Line-item purchase with running total, weight bar, shopkeeper advice |
| Journey | Top: scrolling terrain with wagon animation. Bottom: date, weather, health, food, miles, next landmark. Menu on Enter. |
| Journey Menu | Continue / Check supplies / Map / Pace / Rations / Rest / Trade / Talk / Hunt / Buy (at forts) |
| Landmark | Scene art + name + options |
| River crossing | River scene, width/depth, options, animated result |
| Hunting | Real-time: crosshair, animals spawn by biome, ammo counter, timer, bag weight |
| Rafting | Real-time: raft on Columbia, rocks, steering, cargo/party loss on hit |
| Event popup | Modal with art (optional), message, choices if any |
| Illness / Death | Modal with treatment options; death screen with epitaph entry |
| Map | Trail line with landmarks, current position, weather icons, tombstones |
| Score | Breakdown, hall-of-fame rank, seed, share string |
| Hall of Fame | Top 20 runs with seed, occupation, date, survivors |
| Settings | Color mode (6-color / 16 / truecolor / mono), speed, sound bell on/off, keybinds |

Input: arrow keys + Enter + Esc + number hotkeys. Vim keys as alt. All screens keyboard-only.

### 3.1 Pixel-art pipeline

- Sprite format: plain text files in `data/art/*.px`, one char per pixel, palette letters
  (`.` transparent, `K` black, `W` white, `G` green, `V` violet, `O` orange, `B` blue — the Apple II hi-res six).
- Renderer converts 2 vertical pixels → one `▀`/`▄`/`█` cell with fg/bg colors. 80×32 pixels → 80×16 cells.
- Animation: frame list files (`wagon_0.px … wagon_3.px`) + a tick rate. Parallax layers for terrain.
- Color fallback: truecolor → 256 → 16 → mono (dithered with `░▒▓`).
- Tooling: `cargo run --bin px-preview data/art/chimney_rock.px` to check art fast.

---

## 4. Architecture

Cargo workspace. The simulation never depends on ratatui. Everything deterministic given a seed.

```
pioneer-trail/
├── Cargo.toml                 # workspace
├── CLAUDE.md
├── crates/
│   ├── sim/                   # pure game logic, no I/O, no terminal
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── state.rs       # GameState, Party, Member, Wagon, Inventory
│   │   │   ├── rng.rs         # ChaCha20 seeded, sub-streams per system
│   │   │   ├── calendar.rs    # dates, seasons, daylight
│   │   │   ├── trail.rs       # Trail, Segment, Landmark, forks
│   │   │   ├── travel.rs      # daily tick: pace, terrain, weather → miles
│   │   │   ├── health.rs      # ailments, progression, treatment, death
│   │   │   ├── supplies.rs    # consumption, spoilage, weight
│   │   │   ├── weather.rs     # regional/seasonal weather sim
│   │   │   ├── morale.rs      # member morale, relationships
│   │   │   ├── economy.rs     # prices, trading, fort supply
│   │   │   ├── events/        # event engine + condition DSL
│   │   │   ├── river.rs       # crossing outcomes
│   │   │   ├── hunting.rs     # hunting sim (world model; the minigame drives it)
│   │   │   ├── rafting.rs     # rafting sim
│   │   │   ├── score.rs
│   │   │   └── command.rs     # Command enum: every player action
│   │   └── tests/             # property + scenario tests, Monte Carlo balance
│   ├── data/                  # RON content + loader + schema validation
│   │   ├── src/lib.rs         # include_dir! embed, parse, validate at test time
│   │   ├── trails/oregon.ron
│   │   ├── trails/california.ron
│   │   ├── events/*.ron
│   │   ├── ailments.ron
│   │   ├── occupations.ron
│   │   ├── items.ron
│   │   ├── quotes.ron
│   │   └── art/*.px
│   ├── tui/                   # the binary
│   │   ├── src/
│   │   │   ├── main.rs        # clap: --seed, --continue, --headless-sim N
│   │   │   ├── app.rs         # App state machine, screen stack
│   │   │   ├── input.rs       # key mapping
│   │   │   ├── screens/       # one module per screen
│   │   │   ├── widgets/       # px canvas, status bar, menu, modal, weight bar
│   │   │   ├── art.rs         # .px loader + half-block renderer + palette fallback
│   │   │   ├── minigame/      # hunting.rs, rafting.rs (real-time loops)
│   │   │   └── persist.rs     # saves, hall of fame, settings (JSON via serde)
│   │   └── tests/             # TestBackend snapshot tests (insta)
│   └── tools/                 # px-preview, balance-report, event-lint
└── docs/                      # this plan, design notes, content style guide
```

### 4.1 Core types (sketch)

```rust
pub struct GameState {
    pub seed: u64,
    pub rng: SimRng,               // ChaCha20 with named sub-streams
    pub trail: TrailId,
    pub era: Era,
    pub date: Date,
    pub miles: u32,
    pub segment: SegmentId,
    pub party: Party,              // Vec<Member>, leader idx
    pub wagon: Wagon,              // oxen, parts, condition, weight
    pub inventory: Inventory,
    pub cash: Cents,
    pub pace: Pace,
    pub rations: Rations,
    pub weather: WeatherState,
    pub economy: EconomyState,
    pub flags: FlagSet,            // event memory, e.g. "stranger_joined"
    pub log: Vec<LogEntry>,        // journal for score screen
    pub phase: Phase,              // Traveling, AtLandmark, Crossing, Event(..), Dead, Arrived
}

pub enum Command {
    Continue, SetPace(Pace), SetRations(Rations), Rest(days),
    Buy(Item, qty), Sell(Item, qty), Trade(Offer), Talk,
    Cross(CrossMethod), ChooseRoute(Fork), Hunt(HuntResult), Raft(RaftResult),
    Treat(MemberId, Treatment), Respond(EventChoice), ...
}

impl GameState {
    pub fn apply(&mut self, cmd: Command) -> Vec<Outcome>;   // the only mutation entry point
    pub fn tick_day(&mut self) -> Vec<Outcome>;              // called by Continue
}
```

`Outcome` is what the UI renders (messages, screen transitions, art cues). The UI never
computes game logic. Minigames run their own real-time loop in `tui`, then submit a
`HuntResult { shots, kills: Vec<Animal>, meat_lb }` back to the sim.

### 4.2 RNG discipline

- One master seed. Each system pulls from a named sub-stream (`rng.stream("events")`,
  `rng.stream("weather")`) so adding a new random call in weather does not reshuffle events.
- The hunting minigame's animal spawns come from a sub-stream too, so a seeded run is replayable;
  only the player's aim differs.

### 4.3 Event engine

RON example:

```ron
Event(
    id: "ox_wandered_off",
    weight: 3.0,
    when: All([ Traveling, OxenAtLeast(2), Not(Flag("ox_lost_recently")) ]),
    text: "One of your oxen wandered off during the night.",
    choices: [
        Choice(label: "Search for it (lose a day)", effects: [ LoseDays(1), Roll(0.7, [OxenFound], [OxenLost(1)]) ]),
        Choice(label: "Leave it",                   effects: [ OxenLost(1), Morale(-5) ]),
    ],
    sets: [ Flag("ox_lost_recently", days: 10) ],
)
```

Conditions: trail region, season, date range, weather, pace, rations, health states, inventory
thresholds, flags, occupation, era, morale. Effects: inventory, health, days, miles, cash, flags,
morale, member add/remove, chained event scheduling (`Schedule("stranger_steals", days: 3..8)`).

`event-lint` tool validates every RON file (unknown ids, unreachable events, missing art) and runs in CI.

### 4.4 Persistence

- `~/Library/Application Support/pioneer-trail/` (via `directories` crate): `save.json`,
  `hall_of_fame.json`, `tombstones.json`, `settings.toml`.
- Autosave after every `apply()` that changes the day or phase. Atomic write (tmp + rename).
- Save = serialized `GameState` (rng state included), versioned with a `schema_version` and migrations.
- Seeds: `pt-<base32 of seed+trail+era>`. Entering a seed on the title screen replays world randomness.

### 4.5 Crates

| Crate | Why |
|---|---|
| ratatui, crossterm | TUI + terminal backend |
| serde, ron, serde_json, toml | content, saves, settings |
| rand, rand_chacha | seeded deterministic RNG |
| include_dir | embed data/ at build |
| clap | CLI flags |
| directories | platform data dirs |
| thiserror, anyhow | errors |
| insta | snapshot tests of rendered screens |
| proptest | property tests on sim invariants |
| tracing | debug log to file (never to terminal) |

No async runtime. Real-time minigames use a blocking loop with `crossterm::event::poll(16ms)`.

### 4.6 Testing strategy

- **Sim unit tests**: every module. Invariants: food never negative, miles monotonic, dead members stay dead.
- **Scenario tests**: "banker leaving in March with 2000 lb food on steady/filling arrives with ≥3 alive in ≥90% of 1000 seeds."
- **Monte Carlo balance tool**: `cargo run --bin balance-report` runs 10k headless journeys per occupation/month and prints survival, arrival date, cause-of-death histograms. Run before every balance change.
- **Snapshot tests**: each screen rendered via `TestBackend` at 80×24 and 120×40.
- **Content lint**: RON parse + reference check in CI.
- **Playtest checklist** in `docs/playtest.md`, updated per phase.

---

## 5. Expansion systems (design summaries)

### 5.1 Weather & terrain
- Trail split into segments with `terrain: Plains | Hills | Mountains | Desert | RiverValley | Forest`
  and a `climate_zone`.
- Weather state machine per zone with monthly transition tables (clear, rain, storm, snow, heat, fog).
  Persistence: 1–5 days. Extremes trigger events (blizzard, flash flood).
- Effects: pace multiplier, oxen fatigue, illness onset (cold/heat), river depth (+rain), grass
  availability (season), wagon damage (mud, rocks).
- Map overlay shows current weather per zone.

### 5.2 Party morale & relationships
- Members: name, age, sex, traits (2 of ~20: Hardy, Sickly, Cheerful, Grumbler, Sharpshooter,
  Herbalist, Devout, Restless…), skills (hunting, medicine, repair, animals), morale 0–100.
- Morale drivers: rations, deaths, weather streaks, rest, milestones, events, relationships.
- Low morale: slower pace, arguments (events), member may leave at a fort, refuses to work.
- Relationships: pairwise affinity. Enables wedding, feuds, "stays behind to care for" choices.
- Births: pregnancy flag at setup or via event, due date, risk, new member.

### 5.3 Trading & economy
- Each fort/post has stock levels per item, restocked seasonally. Buying moves the price.
- Trade screen: propose item-for-item with NPC trains or Native traders; acceptance uses a
  value table + reputation + occupation perk. Counter-offers.
- Trade goods item class exists only for trading.
- Reputation: helping NPC trains raises it; refusing, stealing (event choices) lowers it.

### 5.4 Alternate trails & eras
- Trails share the Independence → Fort Hall spine; California branches at Fort Hall to Sutter's Fort;
  Mormon starts at Nauvoo/Council Bluffs and ends at Salt Lake.
- Eras change: prices, ferry availability, fort presence, event weights (cholera 1849–52,
  gold-rush 1848+, army patrols 1866), Native relations tone, guide availability.
- Data-only: a trail is one RON file; an era is a modifier RON file.

---

## 6. Build phases (weeks, in order)

Each phase ends playable and committed. Estimates assume evening/weekend pacing with an agent.

| Phase | Deliverable | Est. |
|---|---|---|
| 0 | Repo scaffold, workspace, CI (fmt/clippy/test), CLAUDE.md, this plan in docs/, content style guide | 1 session |
| 1 | `sim` core: state, calendar, trail data (Oregon), travel tick, supplies, pace/rations, health basics, death, arrival. Headless CLI journey printing daily lines. Monte Carlo tool. | 1 week |
| 2 | Setup flow, store, landmarks, forts (buy), river crossings, original event set, original ailments, score. Text-only TUI: title → setup → journey → score. **First real game.** | 1–2 weeks |
| 3 | Pixel-art pipeline: `.px` format, renderer, palette fallback, px-preview tool. Art for wagon animation, 14 landmarks, rivers, title. Map screen. | 1–2 weeks |
| 4 | Real-time hunting (biome animals, ammo, bag limit) and Columbia rafting. Barlow road. Talk-to-people. | 1 week |
| 5 | Autosave, resume, seeds, hall of fame, tombstones with epitaph entry, settings. Snapshot tests. | 1 week |
| 6 | Weather/terrain sim, expanded ailments (20+), morale & relationships, event set to 150+. Balance pass. | 2–3 weeks |
| 7 | Trading & economy: fort stock, price drift, NPC trains, trade screen, reputation. | 1–2 weeks |
| 8 | California + Mormon trails, 4 eras, expanded occupations. | 1–2 weeks |
| 9 | Polish: sound bell cues, accessibility (mono mode, screen reader text mode), 120×40 layout, README with gifs (vhs). crates.io publish, brew tap, GitHub release with binaries. | 1 week |
| 10 | Mod support: `--data-dir` override, docs for writing events/art. Optional: daily-seed challenge. | later |

### 6.1 Phase 1 definition of done
- `cargo run -p tui -- --headless-sim 1000 --occupation farmer --month april` prints survival stats.
- Property tests pass. Journey from mile 0 to 2040 completes with correct dates.

### 6.2 Phase 2 definition of done
- Can play a full game keyboard-only at 80×24. Deaths, ailments, rivers, forts, score all work.
- 25 original-style events in RON. `event-lint` in CI.

---

## 7. Repo conventions (goes into CLAUDE.md)

- Sim crate has zero terminal dependencies. Enforced by CI (`cargo tree -p sim` must not include ratatui).
- All content in `crates/data`. New events/quotes/art never require Rust changes unless a new
  condition or effect is needed.
- Every balance change comes with a `balance-report` before/after in the commit message.
- Art files are ASCII-safe `.px`; palettes only from `palette.ron`.
- Commit format `<type>: <description>`. Commits by @ContractorKeith only.
- Tests: `cargo test --workspace` must pass; `cargo clippy -- -D warnings`; `cargo fmt --check`.
- Checkpoint via KödMem `log-work` at end of each session.

---

## 8. Open items to verify or decide during build

1. Confirm 1985 mileage table against a primary source (MECC manual or disassembly notes) — Phase 2.
2. Six-color Apple II palette vs. adding a 7th/8th color for readability — decide after first art batch.
3. Hunting minigame layout: original-style top-down field vs. side-scrolling. Prototype both in Phase 4.
4. Hall of fame: local only, or optional opt-in upload later. Local only for v1.
5. Difficulty tiers: original had none beyond occupation. Add Easy/Normal/Hard as multipliers in Phase 6.
6. Whether NPC trains persist across runs (a "world" file). Default no.

---

## 9. Immediate next steps

1. Scaffold `projects/pioneer-trail` with the `new-project` skill (Cargo workspace, CLAUDE.md from §7).
2. Copy this plan to `docs/PLAN.md`; create `docs/CONTENT_STYLE.md` (voice: plain 1840s frontier prose,
   no anachronisms, no copied MECC text).
3. Start Phase 1: `crates/sim` with `state.rs`, `calendar.rs`, `trail.rs`, `travel.rs`, plus the
   headless runner. Commit when the first mile-0-to-arrival journey prints.
