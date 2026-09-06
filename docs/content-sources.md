# Content sources and route-mileage policy

Trail data is a game-scale route model, not a survey or a claim that every
party used one fixed road. The National Park Service describes the Oregon,
California, and Mormon Pioneer trails as corridors with changing alignments.
Our landmarks and route choices represent well-attested waypoints, while the
integer miles are deliberately rounded cumulative gameplay distances.

## Primary and agency sources consulted

- [National Park Service, Oregon Trail route map](https://www.nps.gov/oreg/planyourvisit/upload/National-Park-Service-Oregon-Trail-Map-508.pdf)
  establishes the corridor and major Oregon Trail landmarks.
- [National Park Service, Oregon Trail ruts](https://home.nps.gov/thingstodo/go-see-oregon-trail-ruts.htm)
  identifies Independence Rock and South Pass as major emigrant landmarks;
  it describes South Pass as a broad, gently sloped gap.
- [National Park Service, The 1847 Trek](https://home.nps.gov/mopi/learn/historyculture/the-1847-trek.htm)
  documents the north-bank Mormon route, the crossing near Fort Laramie,
  Independence Rock, South Pass, Fort Bridger, and the 1847 arrival in Salt
  Lake Valley.
- [National Park Service, Hastings Cutoff](https://www.nps.gov/places/hastings-cutoff.htm)
  establishes Fort Bridger, Salt Lake Valley, and Humboldt River as key
  points and warns that the purported shortcut proved longer and punishing.
- [Oregon Secretary of State chronology](https://sos.oregon.gov/blue-book/explore/Pages/chronology-1543-1850.aspx)
  dates the opening of the Barlow Road toll route to 1846.

## Deliberate approximations

- Oregon is modeled as roughly 2,040 miles from Independence to Willamette
  Valley. Branch totals are intentionally shorter where a route choice skips a
  represented stop; they are not measurements of a historic wagon wheel path.
- California is modeled as roughly 2,000 miles from Independence to Sutter's
  Fort. The Fort Bridger–Humboldt–Sierra sequence compresses a corridor with
  several historic alternatives.
- Mormon is modeled as roughly 1,415 miles from Nauvoo to Salt Lake Valley,
  matching the NPS comprehensive-management-plan state-mile total. Its
  Winter Quarters and north-Platte choices abstract seasonal camps and river
  crossings into playable landmarks.
- River widths and depths are gameplay estimates, not historic hydrology.
  They are kept in ranges suitable for the crossing simulation and should not
  be cited as survey data.

## Availability rules for the simulation/UI

The current shared trail schema has no era-eligibility field. Until that
compatible extension lands, the setup UI must offer the Mormon trail only for
1848, 1852, and 1866, and must hide the Barlow Road choice before 1846. The
Oregon route remains selectable in 1843, but its Columbia finale is the only
historically available option in that era. The Dalles is modeled as a river
crossing node (rather than a terminal node) because it offers those two
different continuation choices.

Source review: 2026-09-06. Content prose is original fiction informed by these
historical routes; it does not reproduce text from a trail game or source.

## Content inventory status

The first playable catalogue intentionally contains 25 fully authored events
and 25 named talk entries. The remaining 125 events and 175 quotes are tracked
as future authored batches for issue #2; this baseline does not claim that the
full inventory is complete.
