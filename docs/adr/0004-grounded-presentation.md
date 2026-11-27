# 0004: Everything the player is told is grounded in real state

Status: Accepted (recorded 2026-11-27 from code and content comments)

## Context
The game makes historical-flavored claims and gives survival advice. Invented facts mislead
players and drift from the rules.

## Decision
- Conversation answers come from live `GameState`: the party's progress toward its chosen target,
  current supplies, and real river and store data ahead. Never an invented forecast. Mid-route, the
  chosen target and remaining miles win over routes still listed on the last landmark;
  `current_node_id` names the last stop, so news never claims the party is still standing there.
  Landmark lookups stay on the party's own trail so a shared id on another trail cannot leak.
- The route target comes from the selected route (`target_node_id`), never inferred from miles.
- Era modifiers (ferry fees, guide cost, prices) are game-balance knobs, not historical claims.
- The camp load marker depicts actual cargo weight rather than an invented wagon-health stat. The
  route map is a schematic of the trail graph, not geographic coordinates.
- The balance report treats an ailment at death as context, not proof of cause.
- Trail graves are placed from the trail's fixed total length, never the party's mileage, so they
  stay put while travelling. If a saved trail graph is malformed or cyclic, placement falls back to
  the furthest landmark mile so it stays bounded.
- Text modes for minigames drive the same accounting as the pixel modes (ammo, corpses, bag cap,
  hits), and the hunting crosshair marks exactly the cell used for hit testing.

## Consequences
New UI copy that states a fact needs a source in state or content.
