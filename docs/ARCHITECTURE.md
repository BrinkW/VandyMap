# VandyMap — Architecture

## Stack summary

| Layer | Choice | Notes |
|---|---|---|
| Frontend framework | [Dioxus](https://dioxuslabs.com/) (Rust → WASM) | Chosen over Leptos specifically because Dioxus has a real, shared-codebase path to desktop and mobile targets later — keeps the "maybe an app" goal open without a rewrite. |
| Map rendering | [MapLibre GL JS](https://maplibre.org/) | The one piece of the stack that is JS, not Rust — see **Why Rust, and where it stops** below. |
| Tiles | [OpenFreeMap](https://openfreemap.org/) (preferred) or MapTiler free tier (fallback) | Free, no API key/billing to manage. Mapbox/Google avoided specifically to sidestep usage costs for a solo project. Pick one and pin it in code; don't dual-support both. |
| Styling | Plain CSS (hand-written) | No Tailwind/npm toolchain — a JS build pipeline would undercut the point of a Rust-first stack for marginal benefit at this size. |
| Data storage | TOML files in-repo (`data/buildings/*.toml`), embedded at build time | No database. See **Data loading** and **Why no backend in v1**. |
| Backend | None in v1 | See **Migration path** for when this changes. |
| Hosting | [Cloudflare Pages](https://pages.cloudflare.com/) (recommended); GitHub Pages as fallback | Static hosting only — the build output is WASM + JS glue + static assets, nothing needs a server process. |

## Why Rust, and where it stops

The goal is Rust wherever it's actually the right tool. Concretely:

- **Rust owns**: application state, the building/filtering/search data model and logic, all UI composition (Dioxus components), and the info panel rendering.
- **JS is used for exactly one thing**: driving MapLibre GL JS — creating the map, adding/updating layers and markers, handling pan/zoom/click events on the map surface itself. There is no mature native-Rust equivalent to MapLibre/Mapbox GL JS for browser map rendering; reimplementing that in Rust/WebGL would be a large, lower-polish undertaking for no real benefit here.
- **The interop mechanism**: Dioxus's JS-eval interop (`document::eval` and friends) is used to call into a small, hand-written JS shim around the MapLibre API. This is not "app logic in JS" — it's a thin adapter layer. All decisions about *what* to show on the map (which buildings, which are highlighted/dimmed by the active filters) are computed in Rust and passed across that boundary as data.

This tradeoff was discussed and accepted explicitly during spec-writing: 100%-Rust map rendering was considered and rejected as disproportionately expensive for the polish it would sacrifice.

### The riskiest part of this design

Rust → JS (telling the map what to draw) is the easy direction. **JS → Rust — getting map click events back into Rust state — is the hard one**, because `document::eval` is a string-passing channel rather than a typed binding. This is the single largest technical unknown in the project, so **Phase 0's proof of concept must specifically prove the event-callback direction works**, not merely that a map renders. If that path turns out to be unworkable or unbearably awkward, the fallback is `wasm-bindgen` `extern "C"` bindings against the MapLibre API directly — more boilerplate, properly typed — and that decision should be made in Phase 0, not discovered in Phase 1.

## Data loading

The app is static, so there is no directory to enumerate at runtime — a browser cannot list `data/buildings/`. The chosen approach:

**Building data is aggregated and embedded at build time.** A build step (in `campus_data`'s `build.rs`, or an `xtask` that produces a generated source file) walks `data/buildings/*.toml`, validates them, and emits a single aggregated blob compiled into the WASM binary via `include_str!`/`include_bytes!`.

Why this over fetching data files at runtime:
- One artifact, zero extra network round-trips — no N-requests-for-N-buildings, and no manifest file to keep in sync.
- Malformed data fails the **build**, not the user's page load.
- The dataset is small enough (a few hundred buildings) that bundle-size cost is negligible.

The tradeoff is that a data-only change requires a rebuild and redeploy. For a hand-maintained dataset with a single maintainer, that's fine — it's a git push either way.

## Why no backend in v1

v1's data is manually curated by a single maintainer, there are no accounts, and the dataset size is small. That combination means:

- There's no write path that needs a server (no user submissions, no auth).
- The read path (filtering/search) is cheap enough to do entirely client-side in Rust/WASM against data loaded once at startup.
- A database would add real operational cost (something to host, migrate, and back up) for zero functional benefit at this stage.

## Project structure

A Cargo workspace, so the data model can be shared and validated independently of the UI:

Items marked *(planned)* do not exist yet; everything else is current as of Phase 0 Chunk 2.

```
vandymap/
├── Cargo.toml                  # workspace root; default-members excludes `app`
├── Dioxus.toml                 # app name, page title, web CDN resources
├── rust-toolchain.toml         # pinned toolchain + wasm32 target + components
├── rustfmt.toml
├── LICENSE                     # MIT (code)
├── .github/workflows/
│   └── ci.yml                  # fmt, clippy (host + wasm32), test, validate-data
├── app/                        # Dioxus web app (wasm32)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── components/         # (planned) map view, info panel, filter bar, search
│       ├── state.rs            # (planned) loaded buildings, active filters/search
│       ├── url_state.rs        # (planned) deep-link parsing (?building=...)
│       └── maplibre_interop.rs # (planned) JS-eval bindings into the MapLibre shim
│   └── assets/                 # (planned) maplibre_shim.js, styles.css
├── campus_data/                # shared crate: schema + loading + validation
│   ├── src/
│   │   ├── lib.rs
│   │   ├── building.rs         # Building, Category/Amenity, LatLon, Location
│   │   ├── hours.rs            # Hours enum + "open now" in America/Chicago
│   │   ├── load.rs             # reading data/buildings/*.toml (native tooling)
│   │   └── validate.rs         # schema/consistency checks
│   └── tests/
│       ├── parsing.rs          # proves DATA_MODEL.md's example actually parses
│       └── fixtures/*.toml
├── xtask/                      # dev tooling: validate-data; import-osm (planned)
├── data/
│   ├── LICENSE                 # ODbL (data)
│   └── buildings/
│       └── *.toml              # one file per building — see docs/DATA_MODEL.md
├── docs/
│   ├── SPEC.md
│   ├── ARCHITECTURE.md
│   ├── DATA_MODEL.md
│   ├── BUILDING_LIST.md
│   └── BUILDING_LIST_DEFERRED.md
└── ROADMAP.md
```

**`app` is excluded from workspace `default-members`.** The Dioxus web renderer targets wasm32, so building it for the host during a plain `cargo test`/`cargo clippy` is slow and beside the point. Those commands stay scoped to the portable crates, and CI checks the app against wasm32 explicitly. The side effect worth knowing: `dx` commands must pass `-p app`, or dx picks `xtask` (the only remaining default binary) and bundles the wrong crate.

> Keep this tree current — update it in the same change that adds, removes, or relocates a crate, module, or top-level directory.

`campus_data` is a plain Rust crate with no Dioxus/WASM dependency, so it can be:
- pulled into `app` for use in the browser, and
- pulled into `xtask` to validate every file under `data/buildings/` in CI, without needing a browser or WASM target to do it.

## Data entry tooling: OSM import

Hand-tracing polygon footprints for a few hundred buildings is prohibitively slow, and OpenStreetMap already has most Vanderbilt building outlines and names. `xtask import-osm` queries the Overpass API for buildings within the in-scope campus bounds (see [SPEC.md](SPEC.md)) and emits seed TOML files with `name`, `location.centroid`, `location.footprint`, and `location.source = "osm"` populated.

Everything else (categories, amenities, hours, description) is filled in by hand afterward. The importer seeds; it does not own the files. Re-running it must not clobber hand-entered fields — treat it as generate-if-missing, or emit to a staging directory for manual merge.

**Licensing**: OSM data is ODbL. See the Attribution section of [DATA_MODEL.md](DATA_MODEL.md) — this obligates crediting OSM in the UI and requires confirming share-alike implications for `data/buildings/` before the site goes public.

## Data flow

1. At build time, `data/buildings/*.toml` is validated and aggregated into a blob embedded in the binary.
2. At startup, `campus_data` deserializes that blob into `Vec<Building>` — the single in-memory source of truth in `app::state`.
3. The filter bar and search box mutate a small `FilterState` (active categories, active amenities, open-now toggle, search query) — pure Rust, no network calls.
4. Derived "visible/highlighted building IDs" are recomputed from `Vec<Building>` + `FilterState` on every change.
5. That derived set is passed across the MapLibre interop boundary to update which buildings are shown/dimmed/highlighted; clicking a building (a MapLibre click event, surfaced back into Rust) opens the info panel with data pulled straight from the already-loaded `Building` struct — no additional fetch.
6. Selecting a building also updates the URL query string (`?building=<id>`), and the reverse holds: a deep link is parsed on load and resolves to the same selected state.

## Migration path (when v1's model stops being enough)

This is deliberately not built now, but the shape of the change is worth documenting so it's not a surprise later:

- **Trigger: accounts / crowdsourced data / moderation** → introduce an [Axum](https://github.com/tokio-rs/axum) backend + a real database (start with SQLite if load stays low, Postgres if not), with `campus_data`'s types reused as the backend's data model (it's already plain Rust/serde, not WASM-specific). Static files under `data/buildings/` become the seed/migration data rather than the runtime source of truth.
- **Trigger: real-time data (shuttle tracking, live occupancy)** → same Axum backend, plus a websocket or polling endpoint; doesn't require accounts to exist first.
- **Trigger: mobile app** → Dioxus's mobile target reuses `app`'s components and all of `campus_data`; the MapLibre interop layer is the one piece that would need a mobile-appropriate replacement (e.g. MapLibre Native bindings) since it's currently browser-JS-specific.

## Development practices

- **Formatting/linting**: `rustfmt` and `clippy` enforced (`cargo clippy -- -D warnings`) in CI; no merging with lint failures.
- **Testing**: `cargo test` covers `campus_data` — deserialization of building files, filter-matching logic, search-matching logic, and "open now" evaluation. Open-now tests must inject a fixed timestamp rather than reading the wall clock, and must include a case proving a non-Central viewer timezone still yields campus-local results.
- **What tests won't cover**: the MapLibre interop layer is not unit-testable in any cheap way. Changes there need manual verification in a browser; no automated browser testing is planned for v1.
- **CI**: GitHub Actions running build + `clippy` + `cargo test` + `cargo run -p xtask -- validate-data` on every push.
- **Error handling**: no `unwrap`/`expect` outside tests or genuinely-infallible constructs. Data loading failures should produce a clear error rather than panic, since bad data is the most likely failure mode in a solo, manually-curated project.
- **Commits**: small, scoped commits; commit messages describe *why* a change was made when it isn't obvious from the diff alone.
