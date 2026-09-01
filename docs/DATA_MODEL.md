# VandyMap — Data Model

## Format and location

One TOML file per building, under `data/buildings/`, named by the building's `id` (e.g. `data/buildings/featheringill-hall.toml`). TOML over JSON for this data specifically because it's hand-edited by a single maintainer and TOML is more forgiving to read/diff/write by hand; `campus_data` deserializes it into the same Rust structs the app uses either way.

## `Building` schema

```rust
struct Building {
    id: String,               // stable slug, matches filename, never reused if a building is removed
    name: String,              // full official name
    aliases: Vec<String>,      // abbreviations/nicknames used in search, e.g. ["FEAS"]
    category: Category,
    address: String,
    description: String,       // 1-3 sentences, human-written
    hours: Hours,
    amenities: Vec<Amenity>,
    departments: Vec<String>,  // department/office names housed here, if applicable
    external_link: Option<String>, // link to Vanderbilt's own page for this building
    photos: Vec<String>,       // relative paths/URLs; empty is valid for v1
    location: Location,

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

struct Hours {
    // One entry per day; a building open 24/7 or effectively always-open
    // is modeled explicitly (`always_open = true`), not left blank -
    // blank must always mean "unknown", never "closed" or "always open".
    always_open: bool,
    schedule: Option<WeeklySchedule>, // present when always_open is false
}

struct WeeklySchedule {
    monday: Option<(String, String)>,    // ("07:00", "22:00"); None = closed that day
    tuesday: Option<(String, String)>,
    wednesday: Option<(String, String)>,
    thursday: Option<(String, String)>,
    friday: Option<(String, String)>,
    saturday: Option<(String, String)>,
    sunday: Option<(String, String)>,
}

struct Location {
    centroid: (f64, f64),      // (latitude, longitude), used for pins/search fly-to
    footprint: Vec<(f64, f64)>, // polygon outline, empty = fall back to a pin at centroid
}

// Reserved for the floorplan phase - not populated in v1.
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
category = "Academic"
address = "400 24th Ave S, Nashville, TN 37212"
description = "Home to the Vanderbilt School of Engineering, housing classrooms, labs, and engineering department offices."
departments = ["School of Engineering"]
external_link = "https://engineering.vanderbilt.edu"
photos = []

[hours]
always_open = false

[hours.schedule]
monday = ["07:00", "22:00"]
tuesday = ["07:00", "22:00"]
wednesday = ["07:00", "22:00"]
thursday = ["07:00", "22:00"]
friday = ["07:00", "20:00"]
saturday = ["09:00", "18:00"]
sunday = ["09:00", "18:00"]

amenities = ["Restroom", "Vending", "Printing", "StudySpace", "Outlets"]

[location]
centroid = [36.1447, -86.8027]
footprint = []

floors = []
```

Note: the coordinates above are approximate/illustrative. Real footprint/centroid data should be traced from an authoritative source (e.g. by eye against the tile imagery in MapLibre during data entry, or from OpenStreetMap's existing Vanderbilt building outlines where present) as buildings are actually entered — not treated as verified survey data.

## Validation

A checker (`xtask validate-data`, see [ARCHITECTURE.md](ARCHITECTURE.md)) runs over every file in `data/buildings/` in CI and enforces:

- File parses as valid TOML matching the `Building` schema (via `serde`).
- `id` matches the filename (minus extension).
- `id` is unique across all files.
- `centroid` is within a plausible bounding box around Vanderbilt's campus (catches obvious lat/lon mistakes, e.g. swapped coordinates).
- If `always_open = false`, `schedule` is present; if `always_open = true`, `schedule` is absent (the two are mutually exclusive, not just "schedule ignored when always_open").
- `footprint`, if non-empty, has at least 3 points and is closed or closable (first/last point handling defined in `campus_data`).

This keeps bad data from silently shipping, which matters more than usual here since there's no second reviewer on a solo project.
