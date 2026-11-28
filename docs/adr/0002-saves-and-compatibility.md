# 0002: Saves embed their content and old saves stay playable without rewriting

Status: Accepted (recorded 2026-11-27 from code comments)

## Context
A save resumed after an update must replay deterministically, and losing a player's journey is
the worst bug the game can have.

## Decision
- A save carries the content it was started with. Resuming never swaps in newer content, so an
  older save simply has no newer features (for example no sealed-letter offers). New fields are
  `#[serde(default)]` and default to "this did not exist yet": an empty route record, no
  conversation memory, `None` for an original party member's train id.
- Route records are validated without requiring an old save to begin at the trailhead.
- `GameState::validate` rejects a corrupted save before the UI navigates its content ids.
- Storage is versioned. The envelope's `schema_version` is checked before deserializing state, so a
  save from a future schema is refused rather than misread. A failed read never authorizes
  replacing a save.
- A run id is created once at new-game time and survives resume. A completed journey keeps its
  record after `save.json` is replaced.
- A wagon speaker whose id is missing from the save's embedded `npc_trains` (any save made before NPC trains moved into content) gets the original greetings, `"<name>: Well met on the road."` and `"<name>: Back again, are you?"`, so old saves read the same.
- An item definition without `score` (content embedded in an older save) is scored by
  `legacy_item_score(id)`, which holds the original weights, so old saves score exactly as before.
  New content must define `score`; `event-lint` rejects shipped items in the legacy table that lack it.
- A due pregnancy is consumed even when the mother has died or the wagon is full, so an old save
  cannot retry a birth on every load.

## Consequences
Every new persisted field needs a default that means "absent in older saves".
