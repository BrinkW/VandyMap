//! Proves the TOML shape documented in `docs/DATA_MODEL.md` actually parses into
//! the schema. If these fail, the doc and the code have drifted apart.

use std::path::Path;

use campus_data::hours::Hours;
use campus_data::{validate_all, Amenity, Building, Category};
use chrono::{NaiveDate, TimeZone};
use chrono_tz::America::Chicago;

fn fixture_dir() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))
}

#[test]
fn the_documented_example_parses() {
    let raw =
        std::fs::read_to_string(fixture_dir().join("featheringill-hall.toml")).expect("fixture");
    let building = Building::from_toml(&raw).expect("fixture should parse");

    assert_eq!(building.id, "featheringill-hall");
    assert_eq!(building.name, "Featheringill Hall");
    assert_eq!(building.categories, vec![Category::Academic]);
    assert!(building.aliases.contains(&"FEAS".to_string()));
    assert!(building.amenities.contains(&Amenity::Printing));
    assert_eq!(building.location.centroid.lat, 36.1447);
    assert_eq!(building.location.centroid.lon, -86.8027);
    assert_eq!(building.location.source.as_deref(), Some("osm"));
    assert_eq!(building.last_verified, NaiveDate::from_ymd_opt(2026, 9, 2));
    // Reserved for the floorplan phase, empty in v1.
    assert!(building.floors.is_empty());

    // 2026-09-09 is a Wednesday; the fixture is open 07:00-22:00.
    let wednesday_noon = Chicago.with_ymd_and_hms(2026, 9, 9, 12, 0, 0).unwrap();
    assert_eq!(building.hours.is_open_at(wednesday_noon), Some(true));
    let wednesday_late = Chicago.with_ymd_and_hms(2026, 9, 9, 23, 0, 0).unwrap();
    assert_eq!(building.hours.is_open_at(wednesday_late), Some(false));
}

#[test]
fn the_documented_unknown_hours_shape_parses() {
    let raw = r#"
        id = "somewhere"
        name = "Somewhere"
        categories = ["Other"]
        address = "Nashville, TN"

        [hours]
        kind = "Unknown"

        [location]
        centroid = { lat = 36.145, lon = -86.80 }
    "#;

    let building = Building::from_toml(raw).expect("minimal building should parse");
    assert_eq!(building.hours, Hours::Unknown);
    assert!(!building.hours.is_known());
}

#[test]
fn loading_a_directory_validates_clean() {
    let buildings = campus_data::load::load_dir(fixture_dir()).expect("fixtures should load");
    assert_eq!(buildings.len(), 1);

    let today = NaiveDate::from_ymd_opt(2026, 9, 7).unwrap();
    let findings = validate_all(&buildings, today);
    assert_eq!(findings, vec![], "fixtures should be valid");
}
