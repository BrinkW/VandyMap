# VandyMap — Architecture

## Stack summary

| Layer | Choice | Notes |
|---|---|---|
| Frontend framework | [Dioxus](https://dioxuslabs.com/) (Rust → WASM) | Chosen over Leptos specifically because Dioxus has a real, shared-codebase path to desktop and mobile targets later — keeps the "maybe an app" goal open without a rewrite. |
| Map rendering | [MapLibre GL JS](https://maplibre.org/) | The one piece of the stack that is JS, not Rust — see **Why Rust, and where it stops** below. |
| Tiles | [OpenFreeMap](https://openfreemap.org/) (preferred) or MapTiler free tier (fallback) | Free, no API key/billing to manage. Mapbox/Google avoided specifically to sidestep usage costs for a solo project. Pick one and pin it in code; don't dual-support both. |
| Data storage | Versioned JSON/TOML files in-repo (`data/buildings/*.toml`) | No database. See **Why no backend in v1**. |
| Backend | None in v1 | See **Migration path** for when this changes. |
| Hosting | [Cloudflare Pages](https://pages.cloudflare.com/) (recommended); GitHub Pages as fallback | Static hosting only — the build output is WASM + JS glue + static assets, nothing needs a server process. |

## Why Rust, and where it stops

The goal is Rust wherever it's actually the right tool. Concretely:

- **Rust owns**: application state, the building/filtering/search data model and logic, all UI composition (Dioxus components), and the info panel rendering.
- **JS is used for exactly one thing**: driving MapLibre GL JS — creating the map, adding/updating layers and markers, handling pan/zoom/click events on the map surface itself. There is no mature native-Rust equivalent to MapLibre/Mapbox GL JS for browser map rendering; reimplementing that in Rust/WebGL would be a large, lower-polish undertaking for no real benefit here.
- **The interop mechanism**: Dioxus's JS-eval interop (`document::eval` and friends) is used to call into a small, hand-written JS shim around the MapLibre API. This is not "app logic in JS" — it's a thin adapter layer. All decisions about *what* to show on the map (which buildings, which are highlighted/dimmed by the active filters) are computed in Rust and passed across that boundary as data.

This tradeoff was discussed and accepted explicitly during spec-writing: 100%-Rust map rendering was considered and rejected as disproportionately expensive for the polish it would sacrifice.

## Why no backend in v1

v1's data is manually curated by a single maintainer, there are no accounts, and the dataset size is small (on the order of a few hundred buildings at most). That combination means:

- There's no write path that needs a server (no user submissions, no auth).
- The read path (filtering/search) is cheap enough to do entirely client-side in Rust/WASM against data loaded once at startup.
- A database would add real operational cost (something to host, migrate, and back up) for zero functional benefit at this stage.

So v1 ships as a static site: build-time/load-time, the app reads structured data files, deserializes them into typed Rust structs, and does everything else in memory in the browser.

## Project structure

A Cargo workspace, so the data model can be shared and validated independently of the UI:

```
vandymap/
├── Cargo.toml                  # workspace root
├── app/                        # Dioxus web app
│   ├── src/
│   │   ├── main.rs
│   │   ├── components/         # map view, info panel, filter bar, search
│   │   ├── state.rs            # app state: loaded buildings, active filters/search
│   │   └── maplibre_interop.rs # JS-eval bindings into the MapLibre shim
│   └── assets/
│       └── maplibre_shim.js    # the thin JS adapter described above
├── campus_data/                # shared crate: schema + loading + validation
│   └── src/
│       ├── lib.rs
│       ├── building.rs         # Building struct, Category/Amenity enums, Hours
│       └── validate.rs         # schema/consistency checks over data/buildings/*.toml
├── xtask/                      # `cargo run -p xtask -- validate-data` etc.
├── data/
│   └── buildings/
│       └── *.toml              # one file per building — see docs/DATA_MODEL.md
├── docs/
│   ├── SPEC.md
│   ├── ARCHITECTURE.md
│   └── DATA_MODEL.md
└── ROADMAP.md
```

`campus_data` is a plain Rust crate with no Dioxus/WASM dependency, so it can be:
- pulled into `app` for use in the browser, and
- pulled into `xtask` to validate every file under `data/buildings/` in CI, without needing a browser or WASM target to do it.

## Data flow

1. At app startup, `campus_data` loads and deserializes every file in `data/buildings/` into `Vec<Building>` (bundled as static assets shipped alongside the WASM bundle, fetched once).
2. This becomes the single in-memory source of truth in `app::state`.
3. The filter bar and search box mutate a small `FilterState` (active category, active amenities, open-now toggle, search query) — pure Rust, no network calls.
4. Derived "visible/highlighted building IDs" are recomputed from `Vec<Building>` + `FilterState` on every change.
5. That derived set is passed across the MapLibre interop boundary to update which buildings are shown/dimmed/highlighted on the map; clicking a building (a MapLibre click event, surfaced back into Rust via the same interop) opens the info panel with data pulled straight from the already-loaded `Building` struct — no additional fetch.

## Migration path (when v1's model stops being enough)

This is deliberately not built now, but the shape of the change is worth documenting so it's not a surprise later:

- **Trigger: accounts / crowdsourced data / moderation** → introduce an [Axum](https://github.com/tokio-rs/axum) backend + a real database (start with SQLite if load stays low, Postgres if not), with `campus_data`'s types reused as the backend's data model (it's already plain Rust/serde, not WASM-specific). Static files under `data/buildings/` become the seed/migration data rather than the runtime source of truth.
- **Trigger: real-time data (shuttle tracking, live occupancy)** → same Axum backend, plus a websocket or polling endpoint; doesn't require accounts to exist first.
- **Trigger: mobile app** → Dioxus's mobile target reuses `app`'s components and all of `campus_data`; the MapLibre interop layer is the one piece that would need a mobile-appropriate replacement (e.g. MapLibre Native bindings) since it's currently browser-JS-specific.

## Development practices

- **Formatting/linting**: `rustfmt` and `clippy` enforced (`cargo clippy -- -D warnings`) in CI; no merging with lint failures.
- **Testing**: `cargo test` covers `campus_data` — deserialization of building files, filter-matching logic, search-matching logic, and "open now" time-derivation logic (with fixed/injected times, not real wall-clock, so tests are deterministic).
- **CI**: GitHub Actions running build + `clippy` + `cargo test` + `cargo run -p xtask -- validate-data` (schema/consistency check over every file in `data/buildings/`) on every push.
- **Error handling**: no `unwrap`/`expect` outside tests or genuinely-infallible constructs (e.g. a `const` regex known to compile). Data loading failures should produce a clear error rather than panic, since bad data is the most likely failure mode in a solo, manually-curated project.
- **Commits**: small, scoped commits; commit messages describe *why* a change was made when it isn't obvious from the diff alone.
