# 0005: Terminal lifecycle, fixed-step minigames and the strict `.px` pipeline

Status: Accepted (recorded 2026-11-27 from code comments)

## Decision
- The terminal is owned by a guard (in the game and in `px-preview`) so a panic or error restores it.
- Minigames advance on a fixed 30 Hz step, paused below the minimum terminal size. After a stalled
  terminal, catch-up is bounded to three steps; a whole hunt is never fast-forwarded.
- `.px` sprites use a tiny, strict grammar so a malformed asset fails while it is being made, not
  as a broken scene. `#` rows are comments; blank rows are invalid. Images are capped at 256×256,
  far below `u16::MAX`, so the renderer never allocates an unbounded canvas from a file. Embedded
  assets are parsed once; a malformed one fails the content test gate.
- Rendering packs two vertical pixels into one half-block cell. Transparent pixels keep the cell
  underneath, so layers compose. Images clip to the area and the buffer. Short vignettes render a
  vertical slice so they show ground-level action, not sky.
- Generated art (`assets/gen_scenes.py`, `assets/gen_logo.py`) is the source; outputs are never
  hand-edited. Animation frames keep the canonical silhouette and footprint and change only a few
  pixels (wagon rim and canopy rib, ox hooves).
- `pioneer-data`'s build script watches content directories, so new RON and `.px` files rebuild too.
