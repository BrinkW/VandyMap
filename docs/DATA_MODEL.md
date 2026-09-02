# VandyMap — Data Model

## Format and location

One TOML file per building, under `data/buildings/`, named by the building's `id` (e.g. `data/buildings/featheringill-hall.toml`). TOML over JSON for this data specifically because it's hand-edited by a single maintainer and TOML is more forgiving to read/diff/write by hand; `campus_data` deserializes it into the same Rust structs the app uses either way.

## `Building` schema

```rust
struct Building {
    id: String,                // stable slug, matches filename, never reused if a building is removed
    name: String,              // full official name
    aliases: Vec<String>,      // abbreviations/nicknames used in search, e.g. ["FEAS"]
    categories: Vec<Category>, // multi-valued: Rand is both Dining and Administrative
    address: String,
    description: String,       // 1-3 sentences, human-written
    hours: Hours,
    amenities: Vec<Amenity>,
    departments: Vec<String>,  // department/office names housed here, if applicable
    external_link: Option<String>, // link to Vanderbilt's own page for this building
    photos: Vec<String>,       // reserved; stays empty in v1 (see SPEC.md)
    location: Location,
    last_verified: Option<Date>, // when this record was last checked against reality

    // Reserved for a later phase (floorplans) - present in the schema now so
    // adding real data later isn't a breaking change. Empty/omitted in v1.
    floors: Vec<Floor>,
}

enum Category {
    Academic,
    Residential,
    Dining,
    Athletic,
    Administrative,
    Library,
    Parking,
    Chapel,
    Health,
    Greek,
    StudentServices,
    Other,
}

enum Amenity {
    Restroom,
    Vending,
    Atm,
    Printing,
    StudySpace,
    WaterFountain,
    Outlets,
}
```

### Hours

Modeled as an enum rather than a bool plus an optional schedule, so that "unknown" is a real, representable state. This matters: most buildings start life with unknown hours during data entry, and an unknown building must never be silently treated as closed.

```rust
enum Hours {
    Unknown,                 // no data yet - excluded from "open now" results
    AlwaysOpen,              // 24/7 or effectively always accessible
    Weekly(WeeklySchedule),
}

struct WeeklySchedule {
    monday: Option<TimeRange>,    // None = closed that day
    tuesday: Option<TimeRange>,
    wednesday: Option<TimeRange>,
    thursday: Option<TimeRange>,
    friday: Option<TimeRange>,
    saturday: Option<TimeRange>,
    sunday: Option<TimeRange>,
}

struct TimeRange {
    open: Time,   // parsed and validated, not a raw String
    close: Time,
}
```

**v1 deliberately does not model** term/break/summer schedules, holiday closures, or split hours (open, closed midday, reopen). Those are real — dining especially — but each multiplies the hand-maintenance burden. v1 accepts that finals-week and summer hours will be wrong, which is why `last_verified` is surfaced in the UI.

**"Open now" is always evaluated in `America/Chicago`**, never the viewer's local timezone. A viewer in another timezone must see what's actually open on campus right now. This is a correctness requirement, not a detail — the naive implementation (browser-local time) is wrong.

### Location

```rust
struct Location {
    centroid: LatLon,          // used for pins and search fly-to
    footprint: Vec<LatLon>,    // polygon outline; empty = fall back to a pin at centroid
    source: Option<String>,    // provenance, e.g. "osm" - see Attribution below
}

// A newtype, not a bare (f64, f64) tuple: prevents lat/lon transposition
// at the type level rather than catching it after the fact in validation.
struct LatLon {
    lat: f64,
    lon: f64,
}
```

### Reserved for the floorplan phase

Not populated in v1.

```rust
struct Floor {
    level: i32,
    label: String,             // e.g. "1st Floor", "Basement"
    plan_image: Option<String>,
    facilities: Vec<FloorFacility>,
}

struct FloorFacility {
    kind: Amenity,
    position: (f64, f64),      // coordinates within the floorplan image, not lat/lon
    note: Option<String>,
}
```

## Example: `data/buildings/featheringill-hall.toml`

```toml
id = "featheringill-hall"
name = "Featheringill Hall"
aliases = ["FEAS", "Featheringill"]
categories = ["Academic"]
address = "400 24th Ave S, Nashville, TN 37212"
description = "Home to the Vanderbilt School of Engineering, housing classrooms, labs, and engineering department offices."
departments = ["School of Engineering"]
external_link = "https://engineering.vanderbilt.edu"
amenities = ["Restroom", "Vending", "Printing", "StudySpace", "Outlets"]
photos = []
last_verified = "2026-09-02"

[hours]
kind = "Weekly"

[hours.schedule]
monday = { open = "07:00", close = "22:00" }
tuesday = { open = "07:00", close = "22:00" }
wednesday = { open = "07:00", close = "22:00" }
thursday = { open = "07:00", close = "22:00" }
friday = { open = "07:00", close = "20:00" }
saturday = { open = "09:00", close = "18:00" }
sunday = { open = "09:00", close = "18:00" }

[location]
centroid = { lat = 36.1447, lon = -86.8027 }
footprint = []
source = "osm"

floors = []
```

A building with no hours data yet is simply:

```toml
[hours]
kind = "Unknown"
```

Note: the coordinates above are approximate/illustrative. Real footprint/centroid data comes from the OpenStreetMap import (see [ARCHITECTURE.md](ARCHITECTURE.md)), hand-corrected where OSM is wrong or missing — not treated as verified survey data.

## Licensing and attribution

**Settled: the data is ODbL, the code is MIT.** This is a deliberate choice, not an unresolved question.

Importing building names and geometry from OpenStreetMap makes `data/buildings/` a **Derivative Database** under the [ODbL](https://opendatacommons.org/licenses/odbl/1-0/), so ODbL's share-alike terms apply to it. That is accepted: this project is intended to be open source, and the hand-curated fields (hours, amenities, descriptions) being reusable by others is fine.

What this obligates, all of it already handled or cheap:

- **`data/LICENSE`** declares the data ODbL. The application code stays MIT under the repo-root `LICENSE` — ODbL applies to databases and does not reach the Rust source.
- **Attribution in the map UI**: "© OpenStreetMap contributors" linking to https://www.openstreetmap.org/copyright, alongside whatever the tile provider separately requires.
- **Availability** (ODbL §4.6): the derivative database must be obtainable by anyone using the site. The public GitHub repo satisfies this — the TOML files are already there.
- **`location.source`** records which entries are OSM-derived.

Liability is covered on both sides: MIT and ODbL each carry warranty disclaimers and limitation-of-liability terms, which apply to data accuracy (wrong hours, stale amenities) as much as to the code.

The rendered map itself is a **Produced Work** under ODbL, not a derivative database — it requires attribution only, and imposes nothing on the app.

## Validation

A checker (`xtask validate-data`, see [ARCHITECTURE.md](ARCHITECTURE.md)) runs over every file in `data/buildings/` in CI and enforces:

- File parses as valid TOML matching the `Building` schema (via `serde`).
- `id` matches the filename (minus extension), and is unique across all files.
- `categories` is non-empty.
- `centroid` is within a plausible bounding box around Vanderbilt's campus (catches obvious lat/lon mistakes).
- Times parse as `HH:MM` in 24-hour form, and `open` is earlier than `close` within a day. (Past-midnight closing times are not representable in v1 — flagged as an error, not silently accepted.)
- `footprint`, if non-empty, has at least 3 points and is closed or closable (first/last point handling defined in `campus_data`).
- `last_verified`, if present, parses as a date and is not in the future.

This keeps bad data from silently shipping, which matters more than usual here since there's no second reviewer on a solo project.
