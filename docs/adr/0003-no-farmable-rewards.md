# 0003: Repeatable interactions cannot farm rewards

Status: Accepted (recorded 2026-11-27 from code and content comments)

## Context
Talking, trading and errands can be repeated at will. Any reward attached to them becomes a
faucet unless it is gated on something the player cannot repeat in place.

## Decision
- A speaker's favor is granted only on a recognized return visit on a later day, never for a
  second question in the same sitting, and only once (`ConversationMemory::favor_received`).
- Wagon-train speakers recognize the party only after a real trade, not prior talk.
- The favor is capacity-checked against wagon capacity and the item limit. If there is no room,
  nothing is claimed and the one-shot flag stays unset, so it can be offered once there is space.
- An NPC train improves reputation at most once per day (`last_reputation_day`).
- Sealed-letter courier rewards are a modest fixed bonus (about one food load, $15.00) and each
  origin offers a letter only once.
- A person can die only once per day, even with several ailments.

## Consequences
New rewards state their one-shot or per-day gate in content or sim state and get a test that
repeats the action to prove it.
