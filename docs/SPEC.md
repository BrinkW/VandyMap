# VandyMap — Product Spec

## Vision

VandyMap is an interactive, overhead map of Vanderbilt University's campus, in the spirit of Google Maps: pan and zoom over real campus geography, click a building to see what it is and what's inside it, and filter the whole campus down to just the buildings that match what you're looking for (dining, printing, a specific department, whatever). It starts as a public, read-only web app. Floorplan-level detail (individual vending machines, bathrooms, and other in-building facilities) and a native mobile app are explicit future phases, not v1 commitments.

The project is solo-maintained. Scope is deliberately staged so v1 is small, real, and shippable, with later phases layered on rather than assumed.

## Campus scope

Which properties get data entered is a deliberate boundary, not "everything Vanderbilt owns":

**In scope:**
- Main campus core (central academic and residential buildings)
- Peabody College
- Athletics facilities, including those away from the core campus
- Highland Village housing
- Blakemore
- The subsidiary/newer buildings along 19th Ave and 17th Ave

**Out of scope:**
- **VUMC / Vanderbilt University Medical Center** — a legally separate entity from the university, with a large and complex footprint of its own.

The concrete building list to be entered is compiled from public sources and reviewed before data entry begins — see [ROADMAP.md](../ROADMAP.md) Phase 1.

## User Stories

- As a prospective or current student, I want to see the campus laid out on a real map so I can orient myself before I've memorized it.
- As a student looking for a place to eat, print a document, or refill a water bottle, I want to filter buildings by amenity so I don't have to know the campus already to find one.
- As anyone searching for a specific building or department, I want to type a name and have the map fly to and highlight it, rather than hunting visually.
- As a student deciding where to go right now, I want to filter to buildings that are currently open, not just buildings that are open *sometime*.
- As a visitor, I want to click any building and get a quick, readable summary of what it is, without digging through Vanderbilt's own site.
- As someone telling a friend where to meet, I want to send them a link that opens the map on that exact building.

## Scope

### In scope for v1

- Real overhead map of the in-scope campus (pan/zoom/click), rendered over real georeferenced imagery/tiles.
- Every in-scope building represented with an accurate footprint or pin.
- Click-to-open info panel per building (see **Building Info Panel** below).
- Advanced filtering by:
  - Building category/type
  - Amenities present
  - Hours / "open now"
- Search bar over buildings (and, where meaningful, named rooms/departments) with fly-to-and-highlight behavior.
- **Shareable deep links** — a URL that opens the map focused on a specific building (e.g. `/?building=featheringill-hall`).
- **Geolocation "you are here"** — an opt-in position marker for viewers who grant permission.
- **"Last verified" dates** surfaced per building, so hand-maintained data that has gone stale is visibly stale rather than silently wrong.
- Fully public, read-only — no accounts, no login, no user-submitted content.

### Explicitly out of scope for v1

These are real goals for the project, just not v1. Calling them out here is intentional — the point is to prevent scope creep into v1, not to forget them:

- **Floorplans** with facility-level markers (vending machines, bathrooms, etc.) — needs floorplan source material that doesn't currently exist; see [ROADMAP.md](../ROADMAP.md) Phase 4.
- **Walking directions / routing** between two points on campus — needs a walkable path graph, a separable data-modeling effort; see Phase 5.
- **Accounts, favorites, or crowdsourced/user-submitted data** (e.g. "this bathroom was clean") — deferred; would also require introducing a backend and moderation; see Phase 6.
- **Real-time data** (shuttle tracking, live occupancy, event feeds) — deferred; see Phase 6.
- **Building photos** — the schema keeps a `photos` field, but it stays empty in v1. Vanderbilt's own photography is VU's copyright and not ours to redistribute; sourcing openly-licensed or self-taken images is deferred rather than rushed.
- **Mobile app** — aspirational; the tech stack is chosen to keep this door open (see [ARCHITECTURE.md](ARCHITECTURE.md)), but it is not a committed v1/v2 deliverable.

## Platform priority

**v1 is desktop-first.** Layout, interaction, and polish target desktop browsers; phones are expected to work but are not what the design optimizes for.

This is a deliberate decision, not an oversight. The tradeoff is understood: a campus map is most useful on a phone while walking, so **mobile-web polish (touch targets, bottom-sheet info panel, thumb-reachable filters) is a named later phase** — see [ROADMAP.md](../ROADMAP.md) — rather than something to be quietly skipped. Nothing in v1 should be built in a way that makes that later pass harder than it needs to be.

Browser support: current versions of Chrome, Firefox, Safari, and Edge. WASM is required; there is no non-WASM fallback.

## Building Info Panel (v1)

Clicking a building opens a panel with:

| Field | Notes |
|---|---|
| Name | Full official name |
| Categories | One or more of: Academic, Residential, Dining, Athletic, Administrative, Library, Parking, Chapel, Health, Greek, StudentServices, Other |
| Short description | 1–3 sentences, human-written |
| Hours | Weekly schedule, "always open," or explicitly "unknown" — never a blank that could be misread as closed |
| Amenities | Tag list: restrooms, vending, ATM, printing/copying, study space, water fountain, outlets/charging, etc. |
| Address | Street address |
| External link | Link out to Vanderbilt's own directory/page for the building, where one exists |
| Last verified | Date the building's data was last checked against reality |
| Photo(s) | Field reserved; empty in v1 (see out-of-scope above) |

See [docs/DATA_MODEL.md](DATA_MODEL.md) for the exact schema backing this panel.

## Filtering UX

- Filters are facets, not a single search box: category, amenities, and open-now are independently selectable and combine with AND semantics across facets.
- **Categories are multi-valued per building** (Rand is genuinely both Dining and Administrative). A building matches the category facet if *any* of its categories is selected.
- Amenities support multi-select, and a building must have *all* selected amenities to match.
- **"Open now" is a derived, live filter** — computed client-side from each building's structured hours. It is evaluated in **campus-local time (`America/Chicago`), never the viewer's local timezone**, so someone browsing from another timezone sees what's actually open on campus right now.
- Buildings with `unknown` hours are excluded from "open now" results rather than assumed open or closed, and the UI should make that exclusion legible rather than silent.
- Filtering narrows what's shown on the map (dims or hides non-matching buildings) rather than navigating away from the map view.

## Search UX

- A single search box matches against building names (and known aliases/abbreviations — e.g. "Featheringill" vs "FEAS") and, where data exists, department names housed in a building.
- Selecting a result flies the map to that building and highlights it, then opens its info panel — the same end state as clicking the building directly.
- Search and the filter facets are independent: search does not clear active filters, and vice versa.

## Empty and error states

These need real designs, not defaults — for a map-centric app the failure modes are visible and confusing if unhandled:

- **No filter matches** — an explicit "no buildings match these filters" message with a one-click way to clear filters, not just an empty map.
- **Building data fails to load** — a clear error rather than a silently empty map; the map itself should still render.
- **Map tiles fail to load** — the app should stay usable (search, filtering, info panels) rather than appearing broken.
- **Geolocation denied or unavailable** — treated as normal, not an error; the map simply has no position marker.
- **Unknown data** — an absent value renders as "unknown" or is omitted, never as a confident-looking blank.

## Success Criteria for v1

- Every in-scope Vanderbilt campus building appears on the map with at least name, categories, and address filled in.
- Filtering and search both work entirely client-side with no perceptible lag for a campus-sized dataset (order of a few hundred buildings at most).
- A deep link to any building opens directly on that building.
- The site is fully usable with zero backend infrastructure running — a static deploy is sufficient (see [ARCHITECTURE.md](ARCHITECTURE.md)).
