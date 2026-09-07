//! Shared building/facility data for VandyMap.
//!
//! This crate holds the schema, parsing, filtering predicates, and validation
//! rules. It has no Dioxus or WASM dependency, so the same types back both the
//! browser app and the `xtask` validator that runs in CI.
//!
//! See `docs/DATA_MODEL.md` for the schema this mirrors.

pub mod building;
pub mod hours;
pub mod load;
pub mod validate;

pub use building::{Amenity, Building, Category, Floor, FloorFacility, LatLon, Location};
pub use hours::{Hours, Time, TimeRange, WeeklySchedule, CAMPUS_TZ};
pub use validate::{validate_all, validate_building, Finding, CAMPUS_BOUNDS};
