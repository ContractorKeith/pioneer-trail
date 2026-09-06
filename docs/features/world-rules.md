# World rules

`era_rules` and `regions` are content maps keyed by era and landmark ID. They keep
world variation in RON rather than in landmark-name checks or era-specific code.

## Era rules

Each era can adjust store prices, ferry fees and availability, Snake River guide
clothing cost, store availability, and individual event weights. The current values
are **game-balance abstractions**, not assertions about historical prices, fort
operations, ferry schedules, or guide services. Historical availability changes need
primary-source verification before being described as history.

The current play distinctions are deliberate:

- 1843 has limited services: Fort Kearney's store and ferries are unavailable.
- 1848 is the baseline.
- 1852 raises supply prices and increases the camp-pox event weight.
- 1866 lowers supply prices, reduces guide clothing cost, and reduces that disease weight.

Validation rejects unknown era, store, and event IDs, plus out-of-range percentages
and guide costs.

## Regions and weather

Every loaded landmark has a `Region { terrain, climate }`. During travel the target
landmark selects terrain and climate; at camp the current landmark does. Missing
regions fall back to temperate plains only for starter/test content, while release
content validation requires complete coverage.

`WeatherState::advance_in` preserves each selected weather state for one to five
days. Mountain regions retain colder seasonal outcomes, arid regions retain warmer
ones, and Pacific regions are wetter. `advance` remains the temperate compatibility
wrapper for callers and older tests.
