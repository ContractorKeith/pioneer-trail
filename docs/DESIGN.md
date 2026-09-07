# Pioneer Trail — Visual Design System

The look is the Apple II hi-res screen: six hard colors, chunky pixels, black background,
white text. Nothing anti-aliased, nothing gradient. If it could not be drawn on a 280×192
screen in 1985, it does not belong here.

Logo source of truth: `assets/gen_logo.py` (generates `crates/data/art/title.px`,
`assets/logo.svg`, `assets/logo.png`). Edit the sprites in the script, never the outputs.

![Pioneer Trail logo](../assets/logo.png)

---

## 1. Palette

Six colors, named by one letter. That letter is the pixel character in `.px` files.

| Letter | Name | Truecolor | 256-color | 16-color | Use |
|---|---|---|---|---|---|
| `K` | Black | `#000000` | 16 | black | background, outlines, wheel spokes, hooves |
| `W` | White | `#FFFFFF` | 231 | white | canvas, text, bone, snow, horns, rims |
| `G` | Green | `#1BCB01` | 40 | green | grass, trees, health "good", success |
| `V` | Violet | `#E434FE` | 165 | magenta | dusk sky, sickness, night events, water accents |
| `O` | Orange | `#F26A00` | 202 | red/yellow* | wood, wagon, oxen, dirt, title text, warnings |
| `B` | Blue | `#1B9AFE` | 33 | blue | rivers, day sky, cold, ice |
| `.` | Transparent | — | — | — | shows whatever is behind (usually black) |

\* 16-color terminals have no orange; render `O` as bright yellow (11) for sprites and red (9)
for warning text. Mono terminals render by luminance with `░▒▓█`.

Rules:
- Black background everywhere. Never a colored panel background.
- No color is ever used at reduced opacity. Dithering (checkerboard of two palette colors)
  is the only way to make a "new" shade, and it should be rare.
- Semantic colors in UI text: `G` good/success, `O` caution/attention, `V` illness/danger,
  `B` water/cold/info, `W` default.

---

## 2. Pixel grid and canvas

- Scene canvas: **80 × 32 pixels**, rendered into **80 × 16 terminal cells** using half-blocks
  (`▀` with fg = top pixel, bg = bottom pixel). One `.px` file = one 80×32 scene or a smaller sprite.
- Pixels are 1 wide × 1 tall in the source. On screen they are roughly 1:1 in most terminal fonts
  (a cell is ~1:2, halved vertically). Do not compensate; the slight squash is the look.
- Sprites snap to whole pixels. No sub-pixel positioning, no smoothing when scrolling.
- Activity screens may crop an 80×32 source into a shorter vignette to keep all actions visible at 80×24. Campsite uses a 12-cell crop; decision and ending screens use smaller crops. Preserve the ground line and put controls outside the art.
- Terminal minimum 80×24: 16 rows scene, 1 row status, 7 rows text/menu. At 120×40 the scene stays
  80×16 centered; extra width goes to margins and a wider text panel; extra height to the log.

### `.px` file format

```
# optional comment lines start with #
..........WWWWWWWWWWWWWWWWWWWW..........
.......WWWWWWWWWWWWWWWWWWWWWWWWWW.......
```
- One character per pixel, rows of equal length, `.` transparent.
- Animation = numbered files: `wagon_0.px … wagon_3.px`. Frame rate declared in `art/manifest.ron`.
- Scenes may reference a `layers` list in the manifest for parallax (sky, hills, ground, actors).

---

## 3. Pixel art style rules

Learned from the logo pass; apply to every sprite.

1. **Silhouette first.** At this resolution a thing is recognized by its outline against black.
   Draw the filled shape, then add at most two interior details (an eye, a horn, ribs).
2. **One color per object, plus black and white.** The wagon is orange + white canvas. An ox is
   orange + white horns + black hooves and eye. Trees are green + black trunk. Never three hues on one object.
3. **Black outlines are invisible on black.** Don't rely on `K` outlines for shape; rely on the
   fill. Use `K` only for interior separation (wheel spokes, canvas ribs, hooves, eye).
4. **Ground line.** Every travel scene has a 2-row ground: solid `G` then a `G`/`K` broken row.
   Actors' feet sit on the top ground row, never floating, never sinking.
5. **Wheels are rings.** `W` rim, `K` spokes/hub. Two frames of rotation is enough (spokes at
   + and ×).
6. **Sky is black by default.** Time of day is shown by a thin band: `B` band at dawn/day,
   `V` band at dusk, none at night. Weather adds sprites (rain = `B` diagonal ticks, snow = `W` dots).
7. **Text in scenes** uses the 5×7 pixel font in `gen_logo.py` (`FONT`). Titles only; body text
   is real terminal text.
8. **Faces are forbidden.** Party members are never drawn; the wagon and oxen stand for them.
   Deaths are a tombstone sprite (`W` slab, `K` cross), never a body.
9. **Scale reference.** Wagon = 40 px wide. Ox = 20. Human (rare, e.g. at forts) = 6 wide × 12 tall.
   Fort = up to 60 wide. Landmark rocks/mountains fill the 80 width.

---

## 4. Typography and text UI

Everything outside the scene is terminal text.

- **Case.** Headings in ALL CAPS, as the original did. Body sentence case. Menus Title Case.
- **Box drawing.** Single-line `┌─┐│└┘` for panels. Double-line `╔═╗` only for modals that
  demand attention (death, river result). Never rounded corners.
- **Menus.** Numbered `1.` through `9.`; the cursor is `►` on the active row, two spaces
  otherwise. Enter selects, number keys jump, Esc backs out.
- **Prompts.** Trailing `?` question, then `[Y/N]` or a numbered list. No "Press any key",
  use `Press SPACE BAR to continue` on informational screens, as a nod to the original.
- **Numbers.** Right-aligned in tables. Money as `$1,234.50`. Distances as `102 miles`. Dates
  as `March 14, 1848`.
- **Emphasis.** Bold for the current value of a setting. Color per §1 semantics. No underline,
  no italics, no blink.
- **Status bar** (one row, between scene and panel):

```
 Date: April 22, 1848   Weather: Warm   Health: Good   Food: 1,240 lbs   Next: Fort Kearney 61 mi
```

---

## 5. Layout templates (80×24)

### Journey
```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          [ 80×16 scene: scrolling terrain + wagon ]          │
│                                                                              │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│ Date: April 22, 1848  Weather: Warm  Health: Good  Food: 1,240 lbs  Pace: Steady│
├──────────────────────────────────────────────────────────────────────────────┤
│ Fort Kearney is 61 miles ahead.                                              │
│                                                                              │
│ ► 1. Continue on trail        4. Change pace        7. Hunt for food         │
│   2. Check supplies           5. Change rations     8. Talk to people        │
│   3. Look at map              6. Stop to rest       9. Buy supplies          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Event modal (over the journey screen)
```
                ╔════════════════════════════════════════════════╗
                ║  A wheel splinters on the rocks.               ║
                ║                                                ║
                ║  ► 1. Replace it with a spare                  ║
                ║    2. Try to repair it (lose a day)            ║
                ╚════════════════════════════════════════════════╝
```

### Store
```
┌ MATT'S GENERAL STORE ───────────────────────────── Independence, Missouri ───┐
│  Item                  Price       Qty       Cost     │  Wagon weight        │
│  1. Oxen (yoke)        $40.00        3    $120.00     │  ██████░░░░ 1,410 lb │
│  2. Food (lb)           $0.20      800    $160.00     │                      │
│  3. Clothing (set)     $10.00        6     $60.00     │  "You'll want at     │
│  4. Ammunition (box)    $2.00       10     $20.00     │   least three yoke   │
│  5. Wagon wheel        $10.00        2     $20.00     │   of oxen, and 200   │
│  6. Wagon axle         $10.00        1     $10.00     │   pounds of food     │
│  7. Wagon tongue       $10.00        1     $10.00     │   per person."       │
│                                   Total   $400.00     │                      │
│  Cash remaining: $1,200.00                            │                      │
└──────────────────────────────────────────────────────────────────────────────┘
```

Hunting and rafting take the full 80×24 as a 80×48 pixel field with a one-row HUD at the bottom.

---

## 6. Motion and timing

- Travel animation: 4 wagon frames at 4 fps on steady, 6 fps strenuous, 8 fps grueling.
  Terrain scrolls 1 px per frame; far hills every other frame.
- Day tick: scene runs ~0.6 s per day at default speed; settings offer 0.3 s / 1.2 s / instant.
- Modals appear instantly, no slide. A single terminal bell on death and on arrival (settings toggle).
- Real-time minigames run at 30 fps with `poll(16ms)`. Input is polled, never buffered across frames.
- River crossing result: 3-frame animation (wagon enters, wagon mid-river, outcome) at 1 fps.

---

## 7. Screens: art inventory for Phase 3

| Scene file | Size | Notes |
|---|---|---|
| `title.px` | 80×32 | generated by `gen_logo.py` |
| `wagon_0..3.px` | 40×20 | wheel spokes alternate + and ×; oxen legs alternate |
| `ox_0..1.px` | 20×10 | leg positions |
| `terrain_plains.px`, `_hills`, `_mountains`, `_desert`, `_forest`, `_river_valley` | 80×32 | looping backgrounds, tileable at x=0/80 |
| `kansas_river.px`, `big_blue.px`, `green_river.px`, `snake_river.px`, `columbia.px` | 80×32 | `B` water, `O` banks |
| `fort_kearney.px`, `fort_laramie.px`, `fort_bridger.px`, `fort_hall.px`, `fort_boise.px`, `fort_walla_walla.px` | 80×32 | palisade in `O`, flag in `W`/`B` |
| `chimney_rock.px`, `independence_rock.px`, `south_pass.px`, `soda_springs.px`, `blue_mountains.px`, `the_dalles.px`, `willamette.px` | 80×32 | landmark portraits |
| `tombstone.px` | 8×10 | overlaid on terrain at the grave's mile |
| `map_oregon.px` | 80×32 | trail line `W`, landmarks `O` dots, rivers `B`, party marker `G` |
| hunting: `buffalo`, `bear`, `deer`, `rabbit`, `squirrel`, `crosshair` | various | each with 2 walk frames |
| rafting: `raft.px`, `rock_0..2.px` | 12×6, 6×4 | |

---

## 8. Accessibility

- Mono mode: full game with no color; semantics carried by words (`[GOOD]`, `[SICK]`).
- Text mode: `--no-art` replaces scenes with a one-line description ("Chimney Rock rises ahead.").
  All information in the art must also exist in text somewhere.
- Every action reachable by number key. Every screen exits with Esc.
- Minimum contrast: only `W`, `G`, `O`, `B` on black for text. `V` text only for short labels.

---

## 9. Voice of the visuals

Sparse. Confident. A little grim. The original's charm was that a wheel breaking was a
one-line sentence and a beat of silence. Keep white space. Never fill a panel because it
is empty.
