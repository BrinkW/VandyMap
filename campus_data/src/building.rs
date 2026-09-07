//! The `Building` schema, as specified in `docs/DATA_MODEL.md`.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::hours::Hours;

/// One campus building. Backed by a single hand-edited TOML file under
/// `data/buildings/`, named by [`Building::id`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Building {
    /// Stable slug; matches the filename. Never reused if a building is removed.
    pub id: String,
    /// Full official name.
    pub name: String,
    /// Abbreviations and nicknames, used by search (e.g. `["FEAS"]`).
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Multi-valued: Rand is genuinely both Dining and Administrative.
    pub categories: Vec<Category>,
    pub address: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub hours: Hours,
    #[serde(default)]
    pub amenities: Vec<Amenity>,
    /// Department/office names housed here, if applicable.
    #[serde(default)]
    pub departments: Vec<String>,
    #[serde(default)]
    pub external_link: Option<String>,
    /// Reserved; stays empty in v1 (see `docs/SPEC.md`).
    #[serde(default)]
    pub photos: Vec<String>,
    pub location: Location,
    /// When this record was last checked against reality. Surfaced in the UI so
    /// stale hand-maintained data is visibly stale rather than silently wrong.
    #[serde(default)]
    pub last_verified: Option<NaiveDate>,
    /// Reserved for the floorplan phase. Empty in v1.
    #[serde(default)]
    pub floors: Vec<Floor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Amenity {
    Restroom,
    Vending,
    Atm,
    Printing,
    StudySpace,
    WaterFountain,
    Outlets,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// Used for pins and search fly-to.
    pub centroid: LatLon,
    /// Polygon outline. Empty means "fall back to a pin at the centroid".
    #[serde(default)]
    pub footprint: Vec<LatLon>,
    /// Provenance, e.g. `"osm"`. Tracks which records carry OpenStreetMap-derived
    /// geometry, which is ODbL-licensed (see `docs/DATA_MODEL.md`).
    #[serde(default)]
    pub source: Option<String>,
}

/// A named coordinate pair.
///
/// Deliberately a struct rather than a `(f64, f64)` tuple: transposing latitude
/// and longitude is the single easiest mistake to make in this dataset, and
/// naming the fields makes it unrepresentable rather than something validation
/// has to catch after the fact.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

impl LatLon {
    pub const fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

/// Reserved for the floorplan phase (see `ROADMAP.md` Phase 4). Not populated in v1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Floor {
    pub level: i32,
    /// e.g. "1st Floor", "Basement".
    pub label: String,
    #[serde(default)]
    pub plan_image: Option<String>,
    #[serde(default)]
    pub facilities: Vec<FloorFacility>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloorFacility {
    pub kind: Amenity,
    /// Coordinates within the floorplan image, not latitude/longitude.
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub note: Option<String>,
}

impl Building {
    /// Parse a building from the contents of one TOML file.
    pub fn from_toml(contents: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(contents)
    }

    /// True if any of `wanted` is among this building's categories.
    ///
    /// Category filtering is OR within the facet: selecting Dining and Athletic
    /// shows buildings that are either.
    pub fn has_any_category(&self, wanted: &[Category]) -> bool {
        wanted.is_empty() || wanted.iter().any(|c| self.categories.contains(c))
    }

    /// True if this building has *every* amenity in `wanted`.
    ///
    /// Amenity filtering is AND within the facet: asking for restrooms and
    /// vending means both, not either.
    pub fn has_all_amenities(&self, wanted: &[Amenity]) -> bool {
        wanted.iter().all(|a| self.amenities.contains(a))
    }
}
