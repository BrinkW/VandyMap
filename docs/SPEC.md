# VandyMap — Product Spec

## Vision

VandyMap is an interactive, overhead map of Vanderbilt University's campus, in the spirit of Google Maps: pan and zoom over real campus geography, click a building to see what it is and what's inside it, and filter the whole campus down to just the buildings that match what you're looking for (dining, printing, a specific department, whatever). It starts as a public, read-only web app. Floorplan-level detail (individual vending machines, bathrooms, and other in-building facilities) and a native mobile app are explicit future phases, not v1 commitments.

The project is solo-maintained. Scope is deliberately staged so v1 is small, real, and shippable, with later phases layered on rather than assumed.

## User Stories

- As a prospective or current student, I want to see the campus laid out on a real map so I can orient myself before I've memorized it.
- As a student looking for a place to eat, print a document, or refill a water bottle, I want to filter buildings by amenity so I don't have to know the campus already to find one.
- As anyone searching for a specific building or department, I want to type a name and have the map fly to and highlight it, rather than hunting visually.
- As a student deciding where to go right now, I want to filter to buildings that are currently open, not just buildings that are open *sometime*.
- As a visitor, I want to click any building and get a quick, readable summary of what it is, without digging through Vanderbilt's own site.

## Scope

### In scope for v1

- Real overhead map of Vanderbilt's campus (pan/zoom/click), rendered over real georeferenced imagery/tiles.
- Every major campus building represented with an accurate footprint or pin.
- Click-to-open info panel per building (see **Building Info Panel** below).
- Advanced filtering by:
  - Building category/type
  - Amenities present
  - Hours / "open now"
- Search bar over buildings (and, where meaningful, named rooms/departments) with fly-to-and-highlight behavior.
- Fully public, read-only — no accounts, no login, no user-submitted content.

### Explicitly out of scope for v1

These are real goals for the project, just not v1. Calling them out here is intentional — the point is to prevent scope creep into v1, not to forget them:

- **Floorplans** with facility-level markers (vending machines, bathrooms, etc.) — needs floorplan source material that doesn't currently exist; see [ROADMAP.md](../ROADMAP.md) Phase 3.
- **Walking directions / routing** between two points on campus — needs a walkable path graph, a separable data-modeling effort; see Phase 4.
- **Accounts, favorites, or crowdsourced/user-submitted data** (e.g. "this bathroom was clean") — deferred; would also require introducing a backend and moderation; see Phase 5.
- **Real-time data** (shuttle tracking, live occupancy, event feeds) — deferred; see Phase 5.
- **Mobile app** — aspirational; the tech stack is chosen to keep this door open (see [ARCHITECTURE.md](ARCHITECTURE.md)), but it is not a committed v1/v2 deliverable.

## Building Info Panel (v1)

Clicking a building opens a panel with:

| Field | Notes |
|---|---|
| Name | Full official name |
| Category | e.g. Academic, Residential, Dining, Athletic, Administrative, Library |
| Short description | 1–3 sentences, human-written |
| Hours | Structured per-day hours where meaningful (some buildings are effectively always open — modeled explicitly, not left blank) |
| Amenities | Tag list: restrooms, vending, ATM, printing/copying, study space, water fountain, outlets/charging, etc. |
| Address | Street address |
| External link | Link out to Vanderbilt's own directory/page for the building, where one exists |
| Photo(s) | Placeholder for v1 — at least one representative photo where available |

See [docs/DATA_MODEL.md](DATA_MODEL.md) for the exact schema backing this panel.

## Filtering UX

- Filters are facets, not a single search box: category, amenities, and open-now are independently selectable and combine with AND semantics (a building must match the selected category *and* have all selected amenities *and*, if "open now" is toggled, currently be open).
- Amenities support multi-select (e.g. "restrooms" + "vending" together).
- "Open now" is a *derived*, live filter — it's computed client-side against the current time and each building's structured hours, not a stored flag, so it stays correct without manual updates.
- Filtering narrows what's shown on the map (dims or hides non-matching buildings) rather than navigating away from the map view.

## Search UX

- A single search box matches against building names (and known aliases/abbreviations — e.g. "Featheringill" vs "FEAS") and, where data exists, department names housed in a building.
- Selecting a result flies the map to that building and highlights it, then opens its info panel — the same end state as clicking the building directly.
- Search and the filter facets are independent: search does not clear active filters, and vice versa.

## Success Criteria for v1

- Every major Vanderbilt campus building appears on the map with at least name, category, and address filled in.
- Filtering and search both work entirely client-side with no perceptible lag for a campus-sized dataset (order of a few hundred buildings at most).
- The site is fully usable with zero backend infrastructure running — a static deploy is sufficient (see [ARCHITECTURE.md](ARCHITECTURE.md)).
