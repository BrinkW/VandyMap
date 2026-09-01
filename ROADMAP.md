# VandyMap — Roadmap

Phased so that v1 stays small and shippable, and every later phase has a clear trigger rather than being assumed. See [docs/SPEC.md](docs/SPEC.md) for what's in/out of scope for v1, and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the migration path each stretch phase relies on.

## Phase 0 — Scaffolding

- Cargo workspace set up (`app`, `campus_data`, `xtask`) per [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
- Dioxus web app skeleton that builds and deploys to Cloudflare Pages (even with a blank page — prove the pipeline first).
- MapLibre interop proof of concept: a pannable/zoomable map of Vanderbilt's campus on screen, driven from Rust, no building data yet.
- CI: build + clippy + test + data validation wired up, even with zero data files.

## Phase 1 — MVP

- Building footprints/pins rendered on the map for every major campus building.
- Click a building → info panel with the v1 fields from [docs/SPEC.md](docs/SPEC.md).
- Initial data entry pass — doesn't have to be exhaustive on day one, but every building that's on the map has at least name/category/address filled in.

## Phase 2 — Filtering & search

- Filter bar: category, amenities (multi-select), open-now.
- Search box with fly-to-and-highlight, matching names/aliases/departments.
- Data entry pass expanded to fill in amenities and hours for as many buildings as possible, since filtering is only as useful as the data behind it.

## Phase 3 — Floorplans

- **Research spike first**: determine whether Vanderbilt publishes any usable floorplan source material (ADA accessibility maps, fire evacuation maps, official campus floorplans) before committing to how this phase gets built. This phase does not start with an implementation task — it starts with an answer to "do we have source material."
- If viable: digitize floorplans, populate the `Floor`/`FloorFacility` structures already reserved in [docs/DATA_MODEL.md](docs/DATA_MODEL.md), and build a per-floor interactive view showing facility markers (vending, restrooms, etc.) within a building.

## Phase 4 — Routing / walking directions

- Model a walkable path graph across campus (distinct effort from building footprints).
- Turn-by-turn or simple polyline walking directions between two selected points.

## Phase 5 — Stretch

Each of these is the trigger described in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)'s migration path — none of them is assumed to happen, and any one of them is what justifies introducing a real backend:

- Accounts, favorites, crowdsourced/user-submitted facility data (with moderation).
- Real-time data (shuttle tracking, live occupancy, event feeds).
- Mobile app via Dioxus's mobile target, reusing `app` and `campus_data`.
