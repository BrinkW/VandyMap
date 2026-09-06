# VandyMap — Roadmap

Phased so that v1 stays small and shippable, and every later phase has a clear trigger rather than being assumed. See [docs/SPEC.md](docs/SPEC.md) for what's in/out of scope for v1, and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the migration path each stretch phase relies on.

## Design gate: mockups before frontend design

The site's visual design is **not** being improvised as implementation goes. Mockups will be provided before the frontend is designed for real; until then, UI built during Phase 0/1 should stay deliberately plain and functional, and no visual identity gets invented on the maintainer's behalf. Once mockups exist, they become the reference and this gate lifts.

## Phase 0 — Scaffolding

- Cargo workspace set up (`app`, `campus_data`, `xtask`) per [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
- Dioxus web app skeleton that builds and deploys to Cloudflare Pages (even with a blank page — prove the pipeline first).
- **MapLibre interop proof of concept, including the JS → Rust direction.** A pannable/zoomable map is the easy half; this phase is not done until a click on a map feature is provably received back in Rust state. This is the project's biggest technical unknown — if `document::eval` proves unworkable, decide on the `wasm-bindgen` fallback here rather than in Phase 1.
- CI: build + clippy + test + data validation wired up, even with zero data files.

## Phase 1 — MVP

- **Compile the in-scope building list** from public sources (main campus core, Peabody, athletics, Highland Village, Blakemore, the 19th/17th Ave buildings; VUMC excluded) and review it before bulk data entry.
- **Build `xtask import-osm`** to seed building files with names, centroids, and footprints from OpenStreetMap, rather than tracing several hundred polygons by hand.
- Building footprints/pins rendered on the map for every in-scope building.
- Click a building → info panel with the v1 fields from [docs/SPEC.md](docs/SPEC.md).
- Hand data-entry pass over the OSM-seeded files: categories and addresses at minimum, hours/amenities where known.
- Attribution in the UI for OpenStreetMap and the tile provider.

## Phase 2 — Filtering, search, and links

- Filter bar: categories (multi-valued), amenities (multi-select), open-now.
- "Open now" evaluated in `America/Chicago`, with buildings of unknown hours excluded and that exclusion made legible.
- Search box with fly-to-and-highlight, matching names/aliases/departments.
- Shareable deep links (`?building=<id>`) — both directions: selection updates the URL, and a link opens on that building.
- Geolocation "you are here" marker (opt-in; a denied permission is a normal state, not an error).
- Empty/error states from [docs/SPEC.md](docs/SPEC.md) implemented rather than left as defaults.
- Data entry pass expanded to fill in amenities and hours broadly, since filtering is only as useful as the data behind it.

## Phase 3 — Mobile web polish

v1 is deliberately desktop-first, but a campus map is most useful on a phone while walking. This phase pays that back:

- Touch-friendly targets and gestures.
- Info panel as a bottom sheet rather than a sidebar at narrow widths.
- Thumb-reachable filter and search controls.

## Phase 4 — Floorplans

- **Research spike first**: determine whether Vanderbilt publishes any usable floorplan source material (ADA accessibility maps, fire evacuation maps, official campus floorplans) before committing to how this phase gets built. This phase does not start with an implementation task — it starts with an answer to "do we have source material."
- If viable: digitize floorplans, populate the `Floor`/`FloorFacility` structures already reserved in [docs/DATA_MODEL.md](docs/DATA_MODEL.md), and build a per-floor interactive view showing facility markers (vending, restrooms, etc.) within a building.

## Phase 5 — Routing / walking directions

- Model a walkable path graph across campus (distinct effort from building footprints).
- Turn-by-turn or simple polyline walking directions between two selected points.

## Phase 6 — Stretch

Each of these is the trigger described in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)'s migration path — none of them is assumed to happen, and any one of them is what justifies introducing a real backend:

- Accounts, favorites, crowdsourced/user-submitted facility data (with moderation).
- Real-time data (shuttle tracking, live occupancy, event feeds).
- Building photos, once sourcing is solved (self-taken or openly licensed — not VU's copyrighted imagery).
- Richer hours modeling: term/break/summer schedules, holiday closures, split hours.
- Mobile app via Dioxus's mobile target, reusing `app` and `campus_data`.
